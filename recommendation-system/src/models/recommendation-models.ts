/**
 * Machine Learning Models
 * 
 * Core recommender models:
 * 1. Collaborative Filtering (ALS)
 * 2. Content-Based (Similarity)
 * 3. Learning Path Optimizer (Graph-based)
 * 4. Learning-to-Rank (Neural)
 */

import * as Types from '../types';

// ============================================================================
// COLLABORATIVE FILTERING MODEL (ALS)
// ============================================================================

export class CollaborativeFilteringModel {
  private model: Types.CollaborativeFilteringModel | null = null;
  private regularization: number = 0.01;
  private iterations: number = 10;
  private alpha: number = 40; // implicit feedback weight

  private minRating: number = 0;
  private maxRating: number = 1;

  /**
   * Train collaborative filtering model using ALS
   * Implicit feedback: engagement score (higher = better)
   */
  async train(
    interactions: Types.UserContentInteraction[],
    factorDimension: number = 100
  ): Promise<Types.CollaborativeFilteringModel> {
    console.log(`[CF] Training ALS with ${interactions.length} interactions`);

    const userIds = Array.from(new Set(interactions.map(i => i.userId)));
    const contentIds = Array.from(new Set(interactions.map(i => i.contentId)));

    // Initialize random factors
    const userFactors = new Map<string, number[]>();
    const contentFactors = new Map<string, number[]>();

    for (const userId of userIds) {
      userFactors.set(userId, this.randomVector(factorDimension));
    }
    for (const contentId of contentIds) {
      contentFactors.set(contentId, this.randomVector(factorDimension));
    }

    // ALS iterations
    for (let iter = 0; iter < this.iterations; iter++) {
      console.log(`[CF] ALS iteration ${iter + 1}/${this.iterations}`);

      // Fix content factors, solve for user factors
      for (const userId of userIds) {
        const userInteractions = interactions.filter(i => i.userId === userId);
        const X = userInteractions.map(i => contentFactors.get(i.contentId)!);
        const R = userInteractions.map(i => i.implicitFeedback);

        const userFactor = this.solveALS(X, R, factorDimension);
        userFactors.set(userId, userFactor);
      }

      // Fix user factors, solve for content factors
      for (const contentId of contentIds) {
        const contentInteractions = interactions.filter(i => i.contentId === contentId);
        const X = contentInteractions.map(i => userFactors.get(i.userId)!);
        const R = contentInteractions.map(i => i.implicitFeedback);

        const contentFactor = this.solveALS(X, R, factorDimension);
        contentFactors.set(contentId, contentFactor);
      }
    }

    this.model = {
      modelId: `cf_${Date.now()}`,
      modelType: Types.ModelType.COLLABORATIVE_FILTERING,
      userLatentFactors: userFactors,
      contentLatentFactors: contentFactors,
      factorDimension,
      trainedAt: new Date(),
    };

    console.log('[CF] Model training completed');
    return this.model;
  }

  /**
   * Predict rating for user-content pair
   */
  predict(userId: string, contentId: string): number {
    if (!this.model) return 0.5;
    
    const userFactor = this.model.userLatentFactors.get(userId);
    const contentFactor = this.model.contentLatentFactors.get(contentId);

    if (!userFactor || !contentFactor) return 0.5;

    // Dot product
    let score = 0;
    for (let i = 0; i < userFactor.length; i++) {
      score += userFactor[i] * contentFactor[i];
    }

    // Normalize to 0-1
    return Math.max(this.minRating, Math.min(this.maxRating, score / this.model.factorDimension));
  }

  /**
   * Score multiple items for a user
   */
  scoreMany(userId: string, contentIds: string[]): Map<string, number> {
    const scores = new Map<string, number>();
    for (const contentId of contentIds) {
      scores.set(contentId, this.predict(userId, contentId));
    }
    return scores;
  }

  // ========================================================================
  // PRIVATE HELPERS
  // ========================================================================

  private solveALS(X: number[][], R: number[], factorDimension: number): number[] {
    /**
     * Solve: (X^T X + λI) w = X^T R
     * Using regularized least squares
     */
    const factor = new Array(factorDimension).fill(0);

    if (X.length === 0) {
      return this.randomVector(factorDimension);
    }

    // Build X^T X (Gram matrix)
    const XTX: number[][] = Array(factorDimension)
      .fill(null)
      .map(() => Array(factorDimension).fill(0));

    for (let d1 = 0; d1 < factorDimension; d1++) {
      for (let d2 = 0; d2 < factorDimension; d2++) {
        for (let i = 0; i < X.length; i++) {
          XTX[d1][d2] += X[i][d1] * X[i][d2];
        }
        if (d1 === d2) {
          XTX[d1][d2] += this.regularization;
        }
      }
    }

    // Build X^T R
    const XTR: number[] = new Array(factorDimension).fill(0);
    for (let d = 0; d < factorDimension; d++) {
      for (let i = 0; i < X.length; i++) {
        XTR[d] += X[i][d] * R[i];
      }
    }

    // Simple iterative solver (simplified, non-matrix inverse approach)
    const solution = this.gaussianElimination(XTX, XTR);
    return solution || this.randomVector(factorDimension);
  }

  private gaussianElimination(A: number[][], b: number[]): number[] {
    const n = b.length;
    const aug: number[][] = A.map((row, i) => [...row, b[i]]);

    // Forward elimination
    for (let i = 0; i < n; i++) {
      let maxRow = i;
      for (let k = i + 1; k < n; k++) {
        if (Math.abs(aug[k][i]) > Math.abs(aug[maxRow][i])) {
          maxRow = k;
        }
      }

      [aug[i], aug[maxRow]] = [aug[maxRow], aug[i]];

      if (Math.abs(aug[i][i]) < 1e-10) continue;

      for (let k = i + 1; k < n; k++) {
        const factor = aug[k][i] / aug[i][i];
        for (let j = i; j <= n; j++) {
          aug[k][j] -= factor * aug[i][j];
        }
      }
    }

    // Back substitution
    const x = new Array(n).fill(0);
    for (let i = n - 1; i >= 0; i--) {
      x[i] = aug[i][n];
      for (let j = i + 1; j < n; j++) {
        x[i] -= aug[i][j] * x[j];
      }
      if (Math.abs(aug[i][i]) > 1e-10) {
        x[i] /= aug[i][i];
      }
    }

    return x;
  }

  private randomVector(dimension: number): number[] {
    return Array(dimension)
      .fill(null)
      .map(() => (Math.random() - 0.5) * 0.01);
  }
}

// ============================================================================
// CONTENT-BASED MODEL (Semantic Similarity)
// ============================================================================

export class ContentBasedModel {
  private model: Types.ContentBasedModel | null = null;

  /**
   * Build content-based model from embeddings
   */
  async buildFromEmbeddings(
    contentEmbeddings: Map<string, number[]>
  ): Promise<Types.ContentBasedModel> {
    console.log(`[CB] Building content-based model from ${contentEmbeddings.size} items`);

    if (contentEmbeddings.size === 0) {
      throw new Error('No embeddings provided');
    }

    const firstEmbedding = contentEmbeddings.values().next().value as number[] | undefined;
    if (!firstEmbedding || firstEmbedding.length === 0) {
      throw new Error('First embedding is invalid');
    }
    const dimension = firstEmbedding.length;

    this.model = {
      modelId: `cb_${Date.now()}`,
      modelType: Types.ModelType.CONTENT_BASED,
      contentEmbeddings,
      embeddingDimension: dimension,
      trainedAt: new Date(),
    };

    console.log(`[CB] Model built with dimension ${dimension}`);
    return this.model;
  }

  /**
   * Find similar content based on embeddings
   */
  getSimilarContent(contentId: string, k: number = 10): Array<[string, number]> {
    if (!this.model) return [];
    
    const embedding = this.model.contentEmbeddings.get(contentId);
    if (!embedding) return [];

    const similarities: Array<[string, number]> = [];

    for (const [otherContentId, otherEmbedding] of this.model.contentEmbeddings) {
      if (otherContentId === contentId) continue;

      const similarity = this.cosineSimilarity(embedding, otherEmbedding);
      similarities.push([otherContentId, similarity]);
    }

    return similarities.sort((a, b) => b[1] - a[1]).slice(0, k);
  }

  /**
   * Get content similarity for user preference vector
   */
  scoreContent(userEmbedding: number[], contentIds: string[]): Map<string, number> {
    const scores = new Map<string, number>();

    if (!this.model) return scores;

    for (const contentId of contentIds) {
      const contentEmbedding = this.model.contentEmbeddings.get(contentId);
      if (!contentEmbedding) continue;

      const similarity = this.cosineSimilarity(userEmbedding, contentEmbedding);
      scores.set(contentId, similarity);
    }

    return scores;
  }

  // ========================================================================
  // PRIVATE HELPERS
  // ========================================================================

  private cosineSimilarity(a: number[], b: number[]): number {
    let dotProduct = 0;
    let normA = 0;
    let normB = 0;

    for (let i = 0; i < a.length; i++) {
      dotProduct += a[i] * b[i];
      normA += a[i] * a[i];
      normB += b[i] * b[i];
    }

    normA = Math.sqrt(normA);
    normB = Math.sqrt(normB);

    if (normA === 0 || normB === 0) return 0;

    return dotProduct / (normA * normB);
  }
}

// ============================================================================
// LEARNING PATH OPTIMIZER (Graph-Based)
// ============================================================================

export class LearningPathOptimizer {
  private model: Types.LearningPathOptimizerModel | null = null;

  /**
   * Build optimizer from concept graph
   */
  buildFromConceptGraph(conceptGraph: Types.ConceptGraph): Types.LearningPathOptimizerModel {
    console.log(`[LPO] Building learning path optimizer with ${conceptGraph.nodes.length} concepts`);

    const difficultyProgression = new Map<number, Types.DifficultyLevel[]>();

    // Group concepts by difficulty
    for (const node of conceptGraph.nodes) {
      if (!difficultyProgression.has(node.difficulty)) {
        difficultyProgression.set(node.difficulty, []);
      }
      difficultyProgression.get(node.difficulty)!.push(node.difficulty);
    }

    this.model = {
      modelId: `lpo_${Date.now()}`,
      modelType: Types.ModelType.LEARNING_PATH_OPTIMIZER,
      conceptGraph,
      difficultyProgression,
      policyWeights: {
        prerequisiteImportance: 0.5,
        difficultyProgression: 0.3,
        performanceAdaptation: 0.2,
      },
      trainedAt: new Date(),
    };

    return this.model;
  }

  /**
   * Generate optimal learning path for user
   */
  generatePath(
    contentIds: string[],
    userCompletedIds: Set<string>,
    userPerformance: Map<string, number>
  ): string[] {
    const available = contentIds.filter(id => !userCompletedIds.has(id));

    // Topological sort based on prerequisites
    const path: string[] = [];
    const visited = new Set<string>();

    for (const contentId of available) {
      if (!visited.has(contentId)) {
        this.topologicalSort(contentId, available, path, visited);
      }
    }

    // Adaptive difficulty progression
    path.sort((a, b) => {
      const perfA = userPerformance.get(a) || 0;
      const perfB = userPerformance.get(b) || 0;

      // Recommend next difficulty based on performance
      if (perfA > 0.75 && perfB > 0.75) {
        // Both mastered, prefer harder content
        return 0;
      }
      if (perfA < 0.5) return 1; // Remedial first
      if (perfB < 0.5) return -1;

      return 0;
    });

    return path;
  }

  /**
   * Adapt path based on real-time performance
   */
  updatePathAdaptively(
    currentPath: Types.LearningPath,
    latestPerformance: Map<string, number>
  ): Types.LearningPath {
    const adapted = { ...currentPath };

    // Check if user is struggling
    const recentScores = Array.from(latestPerformance.values()).slice(-5);
    const avgScore = recentScores.reduce((a, b) => a + b, 0) / recentScores.length;

    if (avgScore < 0.5 && adapted.currentStep > 0) {
      // Provide remedial content
      console.log(`[LPO] User struggling (score: ${avgScore}), providing remedial path`);
      adapted.currentStep = Math.max(0, adapted.currentStep - 1);
    }

    if (avgScore > 0.85) {
      // User excelling, skip ahead
      console.log(`[LPO] User excelling (score: ${avgScore}), accelerating path`);
      adapted.currentStep = Math.min(adapted.contentSequence.length - 1, adapted.currentStep + 1);
    }

    adapted.updatedAt = new Date();
    return adapted;
  }

  // ========================================================================
  // PRIVATE HELPERS
  // ========================================================================

  private topologicalSort(
    node: string,
    available: string[],
    path: string[],
    visited: Set<string>
  ): void {
    visited.add(node);
    // In real implementation, follow prerequisite edges
    path.push(node);
  }
}

// ============================================================================
// LEARNING-TO-RANK MODEL
// ============================================================================

export class LTRRankingModel {
  private model: Types.LTRModel | null = null;
  private weights: number[] = [];

  /**
   * Train LTR model with pairwise ranking
   * Using simplified linear ranker (in production: XGBoost/Neural)
   */
  async train(
    trainingData: Array<{
      features: Record<string, number>;
      score: number;
      rank: number;
    }>
  ): Promise<Types.LTRModel> {
    console.log(`[LTR] Training ranker with ${trainingData.length} examples`);

    if (trainingData.length === 0) {
      throw new Error('No training data provided');
    }

    const featureNames = Object.keys(trainingData[0].features);
    const X = trainingData.map(d => Object.values(d.features));
    const y = trainingData.map(d => d.score);

    // Linear regression (simplified LTR)
    this.weights = this.trainLinearRanker(X, y, featureNames.length);

    // Calculate importance
    const importance = new Map<string, number>();
    featureNames.forEach((name, i) => {
      importance.set(name, Math.abs(this.weights[i]));
    });

    this.model = {
      modelId: `ltr_${Date.now()}`,
      modelType: Types.ModelType.LTR_RANKER,
      modelFormat: 'onnx',
      featureNames,
      trainedAt: new Date(),
      featureImportance: importance,
    };

    console.log('[LTR] Model training completed');
    return this.model;
  }

  /**
   * Predict ranking score for features
   */
  predict(features: Record<string, number>): number {
    if (!this.model) return 0.5;
    
    let score = 0;
    this.model.featureNames.forEach((name, i) => {
      score += this.weights[i] * (features[name] || 0);
    });
    return Math.max(0, Math.min(1, score)); // Normalize to 0-1
  }

  /**
   * Re-rank a list of items with their features
   */
  reRank(
    items: Array<{ id: string; features: Record<string, number>; currentScore: number }>
  ): Array<{ id: string; ltrScore: number }> {
    return items
      .map(item => ({
        id: item.id,
        ltrScore: this.predict(item.features),
      }))
      .sort((a, b) => b.ltrScore - a.ltrScore);
  }

  // ========================================================================
  // PRIVATE HELPERS
  // ========================================================================

  private trainLinearRanker(X: number[][], y: number[], numFeatures: number): number[] {
    /**
     * Simple linear regression: w = (X^T X)^-1 X^T y
     * In production: use XGBoost or neural network
     */
    const weights = new Array(numFeatures).fill(0);

    // Calculate means for normalization
    const xMeans = new Array(numFeatures).fill(0);
    const yMean = y.reduce((a, b) => a + b, 0) / y.length;

    for (let j = 0; j < numFeatures; j++) {
      for (let i = 0; i < X.length; i++) {
        xMeans[j] += X[i][j];
      }
      xMeans[j] /= X.length;
    }

    // Calculate weights using gradient descent
    const learningRate = 0.01;
    const iterations = 100;

    for (let iter = 0; iter < iterations; iter++) {
      let gradient = new Array(numFeatures).fill(0);

      for (let i = 0; i < X.length; i++) {
        let pred = 0;
        for (let j = 0; j < numFeatures; j++) {
          pred += weights[j] * X[i][j];
        }

        const error = pred - y[i];
        for (let j = 0; j < numFeatures; j++) {
          gradient[j] += error * X[i][j];
        }
      }

      for (let j = 0; j < numFeatures; j++) {
        weights[j] -= (learningRate * gradient[j]) / X.length;
      }
    }

    return weights;
  }
}

// ============================================================================
// MODEL ENSEMBLE
// ============================================================================

export class HybridRecommender {
  private cfModel?: CollaborativeFilteringModel;
  private cbModel?: ContentBasedModel;
  private lpoModel?: LearningPathOptimizer;
  private ltrModel?: LTRRankingModel;

  setModels(
    cf?: CollaborativeFilteringModel,
    cb?: ContentBasedModel,
    lpo?: LearningPathOptimizer,
    ltr?: LTRRankingModel
  ): void {
    this.cfModel = cf;
    this.cbModel = cb;
    this.lpoModel = lpo;
    this.ltrModel = ltr;
  }

  /**
   * Score content using all available models
   */
  scoreContent(
    userId: string,
    contentIds: string[],
    weights: Types.RankingWeights,
    userEmbedding?: number[]
  ): Map<string, Types.RankingScores> {
    const scores = new Map<string, Types.RankingScores>();

    for (const contentId of contentIds) {
      const cfScore = this.cfModel?.predict(userId, contentId) ?? 0.5;
      const cbScore = userEmbedding && this.cbModel ? this.cbModel.scoreContent(userEmbedding, [contentId]).get(contentId) ?? 0.5 : 0.5;
      const lpScore = 0.5; // Placeholder
      const qualityScore = 0.5; // Would come from content features

      const hybridScore =
        weights.collaborativeWeight * cfScore +
        weights.contentBasedWeight * cbScore +
        weights.learningPathWeight * lpScore +
        weights.qualityPriorWeight * qualityScore;

      scores.set(contentId, {
        contentId,
        collaborativeScore: cfScore,
        contentBasedScore: cbScore,
        learningPathScore: lpScore,
        qualityPriorScore: qualityScore,
        hybridScore,
        finalRankedScore: hybridScore,
      });
    }

    return scores;
  }
}
