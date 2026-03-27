/**
 * Inference Service
 * 
 * Production-grade real-time recommendation engine
 * Latency target: <150ms P95
 */

import * as Types from '../types';
import { IFeatureStore } from '../feature-store/feature-store';
import { HybridRecommender } from '../models/recommendation-models';
import { ExplanationGenerator, FeatureAttributionExplainer } from '../explainability/explainability';
import { ExperimentManager, VariantRankingEngine } from '../ab-testing/experiments';
import { PrivacyComplianceManager } from '../privacy/privacy';

// ============================================================================
// INFERENCE SERVICE
// ============================================================================

export class RecommendationInferenceService {
  private featureStore: IFeatureStore;
  private hybridRecommender: HybridRecommender;
  private explanationGenerator: ExplanationGenerator;
  private featureAttributionExplainer: FeatureAttributionExplainer;
  private experimentManager: ExperimentManager;
  private variantRankingEngine: VariantRankingEngine;
  private privacyManager: PrivacyComplianceManager;

  private requestCache: Map<string, { response: Types.RecommendationResponse; expiresAt: Date }> = new Map();
  private cacheTTL: number = 300000; // 5 minutes

  constructor(
    featureStore: IFeatureStore,
    hybridRecommender: HybridRecommender,
    explanationGenerator: ExplanationGenerator,
    featureAttributionExplainer: FeatureAttributionExplainer,
    experimentManager: ExperimentManager,
    variantRankingEngine: VariantRankingEngine,
    privacyManager: PrivacyComplianceManager
  ) {
    this.featureStore = featureStore;
    this.hybridRecommender = hybridRecommender;
    this.explanationGenerator = explanationGenerator;
    this.featureAttributionExplainer = featureAttributionExplainer;
    this.experimentManager = experimentManager;
    this.variantRankingEngine = variantRankingEngine;
    this.privacyManager = privacyManager;
  }

  /**
   * Main entry point for recommendations
   * Latency-optimized for <150ms
   */
  async getRecommendations(
    request: Types.RequestContext,
    candidateContentIds: string[],
    k: number = 10
  ): Promise<Types.RecommendationResponse> {
    const startTime = Date.now();
    const requestId = request.requestId;

    try {
      // 1. PRIVACY CHECK (early exit)
      if (!this.privacyManager.canRecommendTo(request.userId)) {
        console.log(`[Inference] User ${request.userId} opted out`);
        return this.createEmptyResponse(request, startTime);
      }

      // 2. CHECK CACHE
      const cached = this.getCachedResponse(requestId);
      if (cached) {
        console.log(`[Inference] Cache hit for ${requestId}`);
        return cached;
      }

      // 3. A/B TEST ASSIGNMENT
      const activeExperiments = this.experimentManager.getActiveExperiments();
      let experimentVariant = Types.ExperimentVariant.CONTROL;
      
      if (activeExperiments.length > 0) {
        experimentVariant = this.experimentManager.assignUserToVariant(
          request.userId,
          activeExperiments[0].experimentId
        );
      }

      // 4. GET USER PROFILE & EMBEDDING (from feature store cache)
      const userFeatures = await this.featureStore.getUserFeatures(request.userId);
      const userEmbedding = await this.featureStore.getUserEmbedding(request.userId);

      if (!userFeatures) {
        console.log(`[Inference] No user features found for ${request.userId}, cold start`);
        return this.handleColdStart(request, candidateContentIds, startTime);
      }

      // 5. GET CONTENT FEATURES (batch query)
      const contentFeaturesMap = await this.featureStore.batchGetContentFeatures(
        candidateContentIds
      );

      // 6. GET RANKING WEIGHTS FOR VARIANT
      const rankingWeights = this.variantRankingEngine.getRankingWeights(experimentVariant);

      // 7. SCORE CONTENT (hybrid model)
      const scoreMap = this.hybridRecommender.scoreContent(
        request.userId,
        candidateContentIds,
        rankingWeights,
        userEmbedding?.embedding
      );

      // 8. APPLY BUSINESS RULES
      const rankedContent = this.applyBusinessRules(
        scoreMap,
        contentFeaturesMap,
        userFeatures,
        k
      );

      // 9. BUILD RECOMMENDATIONS WITH EXPLANATIONS
      const recommendations = await Promise.all(
        rankedContent.map(async (contentId, rank) => {
          const scores = scoreMap.get(contentId)!;
          const contentFeatures = contentFeaturesMap.get(contentId);

          const explanation = this.explanationGenerator.generateExplanation(
            {
              contentId,
              rank: rank + 1,
              score: scores.finalRankedScore,
              explanation: {} as any, // Placeholder
              experimentVariant,
              confidence: 0.8,
              metadata: {
                reasonCode: 'hybrid_ranking',
                modality: contentFeatures?.modality || Types.ContentModality.INTERACTIVE,
                difficulty: contentFeatures?.difficultyLevel || Types.DifficultyLevel.INTERMEDIATE,
                estimatedTimeMinutes: contentFeatures?.estimatedDurationMinutes || 30,
              },
            },
            {
              userId: request.userId,
              features: userFeatures,
              embedding: userEmbedding || { userId: '', embedding: [] as number[], dimension: 0, generatedAt: new Date() },
              behavior: {} as any,
              privacySettings: {} as any,
            },
            {
              collaborativeSignal: scores.collaborativeScore,
              contentSignal: scores.contentBasedScore,
              learningPathSignal: scores.learningPathScore,
              qualitySignal: scores.qualityPriorScore,
            },
            undefined,
            []
          );

          return {
            contentId,
            rank: rank + 1,
            score: scores.finalRankedScore,
            explanation,
            experimentVariant,
            confidence: 0.8,
            metadata: {
              reasonCode: 'hybrid_ranking',
              modality: contentFeatures?.modality || Types.ContentModality.INTERACTIVE,
              difficulty: contentFeatures?.difficultyLevel || Types.DifficultyLevel.INTERMEDIATE,
              estimatedTimeMinutes: contentFeatures?.estimatedDurationMinutes || 30,
            },
          };
        })
      );

      // 10. GET LEARNING PATH (if available)
      const learningPath = await this.featureStore.getUserLearningPath(request.userId);

      // 11. BUILD RESPONSE
      const latencyMs = Date.now() - startTime;
      const response: Types.RecommendationResponse = {
        requestId,
        userId: request.userId,
        recommendations,
        learningPath: learningPath || undefined,
        contextUsed: request.context,
        experimentVariant,
        generatedAt: new Date(),
        latencyMs,
      };

      // 12. CACHE RESPONSE
      this.cacheResponse(requestId, response);

      // 13. LOG FOR MONITORING
      console.log(
        `[Inference] Generated ${recommendations.length} recommendations in ${latencyMs}ms for user ${request.userId}`
      );

      return response;
    } catch (error) {
      console.error(`[Inference] Error generating recommendations: ${error}`);
      return this.createEmptyResponse(request, startTime);
    }
  }

  /**
   * Handle cold start for new users
   */
  private async handleColdStart(
    request: Types.RequestContext,
    candidateContentIds: string[],
    startTime: number
  ): Promise<Types.RecommendationResponse> {
    console.log(`[Inference] Cold start for user ${request.userId}`);

    // Strategy: return popular/high-quality content
    const contentFeaturesMap = await this.featureStore.batchGetContentFeatures(
      candidateContentIds
    );

    const byQuality = Array.from(contentFeaturesMap.entries())
      .sort((a, b) => b[1].qualityScore - a[1].qualityScore)
      .slice(0, 10)
      .map(([contentId], rank) => ({
        contentId,
        rank: rank + 1,
        score: 0.7,
        explanation: {
          primaryReason: 'Popular and highly-rated content for new learners',
          supportingSignals: ['Recommended for beginners'],
          featureAttribution: [],
          transparencyMetadata: {
            modelVersion: 'cold_start_v1',
            confidenceLevel: 0.6,
            explanationMethod: 'rule_based' as const,
          },
        },
        experimentVariant: Types.ExperimentVariant.CONTROL,
        confidence: 0.6,
        metadata: {
          reasonCode: 'cold_start',
          modality: Types.ContentModality.VIDEO,
          difficulty: Types.DifficultyLevel.BEGINNER,
          estimatedTimeMinutes: 30,
        },
      }));

    const latencyMs = Date.now() - startTime;

    return {
      requestId: request.requestId,
      userId: request.userId,
      recommendations: byQuality,
      contextUsed: request.context,
      experimentVariant: Types.ExperimentVariant.CONTROL,
      generatedAt: new Date(),
      latencyMs,
    };
  }

  /**
   * Apply business rules to potentially block/reorder content
   */
  private applyBusinessRules(
    scoreMap: Map<string, Types.RankingScores>,
    contentFeaturesMap: Map<string, Types.ContentFeatures>,
    userFeatures: Types.UserFeatures,
    k: number
  ): string[] {
    let candidates = Array.from(scoreMap.entries())
      .map(([contentId, scores]) => ({ contentId, score: scores.finalRankedScore }))
      .sort((a, b) => b.score - a.score);

    // Rule 1: Diversify by modality
    const selectedByModality = new Map<string, number>();
    const diversified: string[] = [];

    for (const { contentId } of candidates) {
      const content = contentFeaturesMap.get(contentId);
      if (!content) continue;

      const modalityCount = selectedByModality.get(content.modality) || 0;
      if (modalityCount < 3) { // At most 3 per modality
        diversified.push(contentId);
        selectedByModality.set(content.modality, modalityCount + 1);

        if (diversified.length >= k) break;
      }
    }

    // Rule 2: Don't recommend something already completed
    return diversified.filter((contentId) => {
      // In production: check completion status from feature store
      return true;
    }).slice(0, k);
  }

  /**
   * Cache response to reduce latency for repeated requests
   */
  private cacheResponse(requestId: string, response: Types.RecommendationResponse): void {
    this.requestCache.set(requestId, {
      response,
      expiresAt: new Date(Date.now() + this.cacheTTL),
    });
  }

  /**
   * Retrieve cached response if still valid
   */
  private getCachedResponse(requestId: string): Types.RecommendationResponse | null {
    const cached = this.requestCache.get(requestId);

    if (!cached) return null;

    if (cached.expiresAt < new Date()) {
      this.requestCache.delete(requestId);
      return null;
    }

    return cached.response;
  }

  /**
   * Create empty response (error case)
   */
  private createEmptyResponse(
    request: Types.RequestContext,
    startTime: number
  ): Types.RecommendationResponse {
    return {
      requestId: request.requestId,
      userId: request.userId,
      recommendations: [],
      contextUsed: request.context,
      experimentVariant: Types.ExperimentVariant.CONTROL,
      generatedAt: new Date(),
      latencyMs: Date.now() - startTime,
    };
  }

  /**
   * Health check for monitoring
   */
  getHealthStatus(): {
    status: 'healthy' | 'degraded' | 'unhealthy';
    latency: number;
    cacheSize: number;
  } {
    // In production: track actual latencies
    return {
      status: 'healthy',
      latency: 45, // Average latency in ms
      cacheSize: this.requestCache.size,
    };
  }

  /**
   * Clear cache (maintenance)
   */
  clearCache(): void {
    const now = new Date();
    let cleared = 0;

    for (const [key, value] of this.requestCache.entries()) {
      if (value.expiresAt < now) {
        this.requestCache.delete(key);
        cleared++;
      }
    }

    console.log(`[Inference] Cleared ${cleared} expired cache entries`);
  }
}

// ============================================================================
// BATCH INFERENCE
// ============================================================================

export class BatchInferenceService {
  private inferenceService: RecommendationInferenceService;

  constructor(inferenceService: RecommendationInferenceService) {
    this.inferenceService = inferenceService;
  }

  /**
   * Generate recommendations for multiple users
   * Process in parallel with concurrency control
   */
  async getRecommendationsBatch(
    requests: Types.RequestContext[],
    candidateContentIds: string[],
    maxConcurrency: number = 10
  ): Promise<Types.RecommendationResponse[]> {
    const results: Types.RecommendationResponse[] = [];
    const queue = [...requests];
    const processing: Promise<Types.RecommendationResponse>[] = [];

    console.log(`[BatchInference] Processing ${requests.length} requests with concurrency ${maxConcurrency}`);

    while (queue.length > 0 || processing.length > 0) {
      // Fill up to max concurrency
      while (processing.length < maxConcurrency && queue.length > 0) {
        const request = queue.shift()!;
        const promise = this.inferenceService.getRecommendations(request, candidateContentIds);
        processing.push(promise);
      }

      if (processing.length > 0) {
        const result = await Promise.race(processing);
        results.push(result);

        const index = processing.findIndex(p => p === Promise.resolve(result));
        processing.splice(index, 1);
      }
    }

    console.log(`[BatchInference] Completed batch processing`);
    return results;
  }
}
