/**
 * Explainability Layer
 * 
 * Generates human-understandable explanations for recommendations
 * using multiple methods:
 * - Feature attribution
 * - Similarity traces
 * - Rule-based explanations
 */

import * as Types from '../types';

// ============================================================================
// EXPLANATION GENERATOR
// ============================================================================

export class ExplanationGenerator {
  /**
   * Generate explanation for a recommendation
   */
  generateExplanation(
    recommendation: Types.Recommendation,
    userProfile: Types.UserProfile,
    rankingSignal: {
      collaborativeSignal: number;
      contentSignal: number;
      learningPathSignal: number;
      qualitySignal: number;
    },
    similarContent?: Array<[string, number]>,
    similarUsers?: string[]
  ): Types.RecommendationExplanation {
    let primaryReason = '';
    const supportingSignals: string[] = [];
    const featureAttribution: Array<{
      feature: string;
      importance: number;
      contribution: string;
    }> = [];

    // Determine dominant signal
    const signals = Object.entries(rankingSignal);
    const [dominantSignal] = signals.reduce((a, b) => (a[1] > b[1] ? a : b));

    // Generate primary reason and supporting signals
    if (dominantSignal === 'collaborativeSignal' && similarUsers && similarUsers.length > 0) {
      primaryReason = `Users like you enjoyed this content`;
      supportingSignals.push(`Liked by ${similarUsers.length} similar learners`);
      featureAttribution.push({
        feature: 'user_similarity',
        importance: rankingSignal.collaborativeSignal,
        contribution: `Based on similar learning patterns`,
      });
    } else if (dominantSignal === 'contentSignal') {
      primaryReason = `Matches your interests`;
      const topics = Array.from(userProfile.features.topicAffinities.keys()).slice(0, 2);
      supportingSignals.push(`Related to your interest in ${topics.join(' and ')}`);
      featureAttribution.push({
        feature: 'topic_match',
        importance: rankingSignal.contentSignal,
        contribution: `Content topic alignment with your profile`,
      });
    } else if (dominantSignal === 'learningPathSignal') {
      primaryReason = `Recommended based on your learning path`;
      supportingSignals.push(`Prerequisite for your next goal`);
      featureAttribution.push({
        feature: 'learning_path_fit',
        importance: rankingSignal.learningPathSignal,
        contribution: `Aligns with recommended progression`,
      });
    } else if (dominantSignal === 'qualitySignal') {
      primaryReason = `High-quality content`;
      supportingSignals.push(`Highly rated by other learners`);
      featureAttribution.push({
        feature: 'content_quality',
        importance: rankingSignal.qualitySignal,
        contribution: `Strong engagement and completion metrics`,
      });
    }

    // Add modality preference explanation
    if (userProfile.features.preferredModality === Types.ContentModality.VIDEO) {
      supportingSignals.push('Available as video (your preferred format)');
    }

    // Add similarity trace
    let similarityTrace: Types.RecommendationExplanation['similarityTrace'] | undefined;
    if (similarContent && similarContent.length > 0) {
      similarityTrace = {
        similarContentIds: similarContent.slice(0, 3).map(([id]) => id),
      };
    }

    return {
      primaryReason,
      supportingSignals,
      featureAttribution,
      similarityTrace,
      ruleBasedExplanation: this.generateRuleBasedExplanation(userProfile, supportingSignals),
      transparencyMetadata: {
        modelVersion: 'hybrid_v1',
        confidenceLevel: recommendation.confidence,
        explanationMethod: 'hybrid',
      },
    };
  }

  /**
   * Rule-based explanation generator
   */
  private generateRuleBasedExplanation(
    userProfile: Types.UserProfile,
    signals: string[]
  ): string {
    const rules: string[] = [];

    // Learning pattern rules
    if (userProfile.behavior.pattern === Types.UserBehaviorPattern.FAST_TRACK) {
      rules.push('You are a fast learner, so we prioritize advanced content');
    } else if (userProfile.behavior.pattern === Types.UserBehaviorPattern.STRUGGLING) {
      rules.push('We detected you need support in this area, recommending foundational content');
    }

    // Engagement rules
    if (userProfile.features.engagementScore > 0.8) {
      rules.push('Based on your high engagement history, we prioritize content like this');
    }

    // Dropout risk rules
    if (userProfile.behavior.dropoutRisk === Types.DropoutRisk.HIGH) {
      rules.push('We are recommending engaging content to keep you motivated');
    }

    // Diversity rules
    if (userProfile.features.topicAffinities.size > 5) {
      rules.push('You have diverse interests, so we cross-recommend across your topics');
    }

    return rules.join('. ') + '.';
  }
}

// ============================================================================
// FEATURE ATTRIBUTION (LIME-style)
// ============================================================================

export interface FeatureImportanceConfig {
  collaborativeWeights: Map<string, number>;
  contentWeights: Map<string, number>;
  learningPathWeights: Map<string, number>;
  qualityWeights: Map<string, number>;
}

export class FeatureAttributionExplainer {
  private importanceConfig: FeatureImportanceConfig;

  constructor(importanceConfig?: FeatureImportanceConfig) {
    this.importanceConfig = importanceConfig || this.getDefaultImportances();
  }

  /**
   * Compute LIME-style feature importance
   * Simplified version for interpretability
   */
  computeFeatureImportance(
    userFeatures: Types.UserFeatures,
    contentFeatures: Types.ContentFeatures,
    scores: Types.RankingScores
  ): Array<{
    feature: string;
    importance: number;
    direction: 'positive' | 'negative';
  }> {
    const importances: Array<{
      feature: string;
      importance: number;
      direction: 'positive' | 'negative';
    }> = [];

    // Collaborative filtering features
    const cfImp = Math.abs(scores.collaborativeScore - 0.5) * 0.4;
    importances.push({
      feature: 'Similar user preferences',
      importance: cfImp,
      direction: scores.collaborativeScore > 0.5 ? 'positive' : 'negative',
    });

    // Content-based features
    const cbImp = Math.abs(scores.contentBasedScore - 0.5) * 0.35;
    importances.push({
      feature: 'Topic match',
      importance: cbImp,
      direction: scores.contentBasedScore > 0.5 ? 'positive' : 'negative',
    });

    // Learning path features
    const lpImp = Math.abs(scores.learningPathScore - 0.5) * 0.15;
    importances.push({
      feature: 'Learning path fit',
      importance: lpImp,
      direction: scores.learningPathScore > 0.5 ? 'positive' : 'negative',
    });

    // Quality features
    const qImp = Math.abs(scores.qualityPriorScore - 0.5) * 0.1;
    importances.push({
      feature: 'Content quality',
      importance: qImp,
      direction: scores.qualityPriorScore > 0.5 ? 'positive' : 'negative',
    });

    // Additional insights
    if (userFeatures.completionRate > 0.8) {
      importances.push({
        feature: 'Your high completion rate',
        importance: 0.15,
        direction: 'positive',
      });
    }

    if (contentFeatures.difficultyLevel > userFeatures.learningVelocity) {
      importances.push({
        feature: 'Challenge level matching your pace',
        importance: 0.1,
        direction: 'positive',
      });
    }

    return importances.sort((a, b) => b.importance - a.importance);
  }

  /**
   * Generate counterfactual explanation
   * "If X were different, recommendation would change"
   */
  generateCounterfactual(
    userFeatures: Types.UserFeatures,
    contentFeatures: Types.ContentFeatures
  ): string[] {
    const counterfactuals: string[] = [];

    // Completion rate counterfactual
    if (userFeatures.completionRate < 0.5) {
      counterfactuals.push(
        'If you had a higher completion rate, this content would rank higher'
      );
    }

    // Topic affinity counterfactual
    const topTopics = Array.from(userFeatures.topicAffinities.entries())
      .sort((a, b) => b[1] - a[1])
      .map(([topic]) => topic);

    if (!contentFeatures.concepts.some(c => topTopics.includes(c.name))) {
      counterfactuals.push(
        `If you were more interested in ${contentFeatures.concepts[0]?.name}, this would rank much higher`
      );
    }

    // Difficulty counterfactual
    if (contentFeatures.difficultyLevel > 3 && userFeatures.learningVelocity < 5) {
      counterfactuals.push('If you were progressing faster, we would recommend more advanced content');
    }

    return counterfactuals;
  }

  // ========================================================================
  // PRIVATE HELPERS
  // ========================================================================

  private getDefaultImportances(): FeatureImportanceConfig {
    return {
      collaborativeWeights: new Map([
        ['user_similarity', 0.35],
        ['user_embedding', 0.25],
      ]),
      contentWeights: new Map([
        ['topic_match', 0.25],
        ['content_embedding', 0.2],
      ]),
      learningPathWeights: new Map([
        ['prerequisite_fit', 0.15],
        ['difficulty_progression', 0.1],
      ]),
      qualityWeights: new Map([
        ['content_quality', 0.08],
        ['engagement_score', 0.02],
      ]),
    };
  }
}

// ============================================================================
// TRANSPARENCY DASHBOARD
// ============================================================================

export class TransparencyDashboard {
  /**
   * Generate personalized transparency report
   */
  generateTransparencyReport(
    userId: string,
    recommendations: Types.Recommendation[],
    userProfile: Types.UserProfile
  ): {
    userId: string;
    reportDate: Date;
    topReasons: Array<{ reason: string; frequency: number }>;
    featureContributions: Map<string, number>;
    modelExplainability: {
      explainableFeatures: string[];
      blackBoxFactors: string[];
    };
  } {
    // Aggregate explanation reasons
    const reasonCounts = new Map<string, number>();
    const featureContributions = new Map<string, number>();

    for (const rec of recommendations) {
      for (const signal of rec.explanation.supportingSignals) {
        reasonCounts.set(signal, (reasonCounts.get(signal) || 0) + 1);
      }

      for (const attr of rec.explanation.featureAttribution) {
        featureContributions.set(
          attr.feature,
          (featureContributions.get(attr.feature) || 0) + attr.importance
        );
      }
    }

    const topReasons = Array.from(reasonCounts.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, 5)
      .map(([reason, frequency]) => ({ reason, frequency }));

    return {
      userId,
      reportDate: new Date(),
      topReasons,
      featureContributions,
      modelExplainability: {
        explainableFeatures: [
          'User interest in topics',
          'Your learning velocity',
          'Content quality scores',
          'Prerequisite alignment',
        ],
        blackBoxFactors: [
          'Neural embedding similarity (ML-generated)',
          'Latent collaborative factors',
        ],
      },
    };
  }

  /**
   * Generate bias report
   */
  generateBiasReport(
    recommendations: Types.Recommendation[],
    userProfiles: Map<string, Types.UserProfile>
  ): {
    recommendationDiversity: number;
    contentTypeDistribution: Record<string, number>;
    difficultyDistribution: Record<string, number>;
    potentialBiases: string[];
  } {
    let videoCount = 0;
    let textCount = 0;
    let interactiveCount = 0;

    const difficultyDistribution: Record<string, number> = {
      beginner: 0,
      intermediate: 0,
      advanced: 0,
      expert: 0,
    };

    for (const rec of recommendations) {
      const modality = rec.metadata.modality;
      if (modality === Types.ContentModality.VIDEO) videoCount++;
      else if (modality === Types.ContentModality.TEXT) textCount++;
      else if (modality === Types.ContentModality.INTERACTIVE) interactiveCount++;

      const difficulty = rec.metadata.difficulty;
      const diffKey = Object.keys(Types.DifficultyLevel)[difficulty - 1]?.toLowerCase() || 'unknown';
      difficultyDistribution[diffKey]++;
    }

    const biases: string[] = [];

    if (videoCount > recommendations.length * 0.7) {
      biases.push('Overrepresentation of video content (possible modality bias)');
    }

    const avgDifficulty =
      Object.values(difficultyDistribution).reduce((a, b) => a + b, 0) / recommendations.length;
    if (avgDifficulty > 2.5) {
      biases.push('Content skewed toward advanced difficulty (possible learner perception bias)');
    }

    return {
      recommendationDiversity: 1 - this.calculateHerfindahlIndex([videoCount, textCount, interactiveCount]),
      contentTypeDistribution: {
        video: videoCount,
        text: textCount,
        interactive: interactiveCount,
      },
      difficultyDistribution,
      potentialBiases: biases,
    };
  }

  // ========================================================================
  // PRIVATE HELPERS
  // ========================================================================

  private calculateHerfindahlIndex(counts: number[]): number {
    const total = counts.reduce((a, b) => a + b, 0);
    const proportions = counts.map(c => (c / total) ** 2);
    return proportions.reduce((a, b) => a + b, 0);
  }
}

// ============================================================================
// MODEL EXPLANATION AGGREGATOR
// ============================================================================

export class ModelExplainabilityAggregator {
  /**
   * Combine multiple explanation methods
   */
  aggregateExplanations(
    explanations: Types.RecommendationExplanation[],
    weights: {
      ruleBasedWeight: number;
      attributionWeight: number;
      similarityWeight: number;
    } = {
      ruleBasedWeight: 0.2,
      attributionWeight: 0.5,
      similarityWeight: 0.3,
    }
  ): {
    consolidated_explanation: string;
    confidence_in_explanation: number;
  } {
    if (explanations.length === 0) {
      return {
        consolidated_explanation: 'Recommendation based on system analysis',
        confidence_in_explanation: 0.5,
      };
    }

    // Weighted combination
    const ruleBased = explanations
      .filter(e => e.ruleBasedExplanation)
      .map(e => e.ruleBasedExplanation)
      .join(' ');

    const topAttribution = explanations[0]?.featureAttribution?.[0];

    const consolidated =
      ruleBased || topAttribution?.contribution || explanations[0]?.primaryReason || '';

    const avgConfidence =
      explanations.reduce((sum, e) => sum + e.transparencyMetadata.confidenceLevel, 0) /
      explanations.length;

    return {
      consolidated_explanation: consolidated,
      confidence_in_explanation: avgConfidence,
    };
  }
}
