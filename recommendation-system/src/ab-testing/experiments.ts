/**
 * A/B Testing Framework
 * 
 * Manages:
 * - Experiment design and assignment
 * - Variant ranking pipelines
 * - Metrics collection
 * - Statistical analysis
 */

import * as Types from '../types';

// ============================================================================
// EXPERIMENT MANAGER
// ============================================================================

export class ExperimentManager {
  private experiments: Map<string, Types.ExperimentConfig> = new Map();
  private assignments: Map<string, Types.ExperimentAssignment> = new Map();

  /**
   * Create a new experiment
   */
  createExperiment(config: Types.ExperimentConfig): void {
    if (config.status === 'running' && !config.startDate) {
      config.startDate = new Date();
    }

    this.experiments.set(config.experimentId, config);
    console.log(`[Experiment] Created experiment: ${config.experimentId}`);
  }

  /**
   * Assign user to variant (deterministic based on hash)
   */
  assignUserToVariant(userId: string, experimentId: string): Types.ExperimentVariant {
    const experiment = this.experiments.get(experimentId);
    if (!experiment || experiment.status !== 'running') {
      return Types.ExperimentVariant.CONTROL;
    }

    // Deterministic assignment using hash
    const hash = this.simpleHash(userId + experimentId);
    const variantIndex = hash % experiment.variants.length;
    const variant = experiment.variants[variantIndex];

    const assignment: Types.ExperimentAssignment = {
      userId,
      experimentId,
      variant,
      assignedAt: new Date(),
      cohortId: `cohort_${Math.floor(hash / experiment.variants.length)}`,
    };

    const assignmentKey = `${userId}:${experimentId}`;
    this.assignments.set(assignmentKey, assignment);

    return variant;
  }

  /**
   * Get assignment for user in experiment
   */
  getAssignment(userId: string, experimentId: string): Types.ExperimentAssignment | null {
    const key = `${userId}:${experimentId}`;
    return this.assignments.get(key) || null;
  }

  /**
   * Get experiment configuration
   */
  getExperiment(experimentId: string): Types.ExperimentConfig | null {
    return this.experiments.get(experimentId) || null;
  }

  /**
   * End experiment
   */
  endExperiment(experimentId: string): void {
    const experiment = this.experiments.get(experimentId);
    if (experiment) {
      experiment.status = 'completed';
      experiment.endDate = new Date();
      console.log(`[Experiment] Ended experiment: ${experimentId}`);
    }
  }

  /**
   * Get all active experiments
   */
  getActiveExperiments(): Types.ExperimentConfig[] {
    return Array.from(this.experiments.values()).filter(e => e.status === 'running');
  }

  // ========================================================================
  // PRIVATE HELPERS
  // ========================================================================

  private simpleHash(str: string): number {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash; // Convert to 32bit integer
    }
    return Math.abs(hash);
  }
}

// ============================================================================
// VARIANT RANKING ENGINE
// ============================================================================

export class VariantRankingEngine {
  /**
   * Get ranking weights for variant
   */
  getRankingWeights(variant: Types.ExperimentVariant): Types.RankingWeights {
    const weightsMap: Record<Types.ExperimentVariant, Types.RankingWeights> = {
      [Types.ExperimentVariant.CONTROL]: {
        collaborativeWeight: 0.35,
        contentBasedWeight: 0.35,
        learningPathWeight: 0.2,
        qualityPriorWeight: 0.1,
      },
      [Types.ExperimentVariant.VARIANT_A]: {
        collaborativeWeight: 0.2, // Content-heavy
        contentBasedWeight: 0.6,
        learningPathWeight: 0.15,
        qualityPriorWeight: 0.05,
      },
      [Types.ExperimentVariant.VARIANT_B]: {
        collaborativeWeight: 0.6, // Collaborative-heavy
        contentBasedWeight: 0.2,
        learningPathWeight: 0.1,
        qualityPriorWeight: 0.1,
      },
      [Types.ExperimentVariant.VARIANT_C]: {
        collaborativeWeight: 0.3,
        contentBasedWeight: 0.3,
        learningPathWeight: 0.3, // Path-heavy
        qualityPriorWeight: 0.1,
        ltrBlendAlpha: 0.3, // Use LTR re-ranking
      },
      [Types.ExperimentVariant.VARIANT_D]: {
        collaborativeWeight: 0.3,
        contentBasedWeight: 0.3,
        learningPathWeight: 0.25,
        qualityPriorWeight: 0.15, // Quality-heavy
      },
    };

    return weightsMap[variant] || weightsMap[Types.ExperimentVariant.CONTROL];
  }

  /**
   * Apply variant-specific ranking adjustments
   */
  applyVariantRanking(
    baseScores: Map<string, number>,
    variant: Types.ExperimentVariant
  ): Map<string, number> {
    const adjustedScores = new Map(baseScores);

    switch (variant) {
      case Types.ExperimentVariant.VARIANT_A:
        // Boost content that matches user topics more
        for (const [contentId, score] of adjustedScores) {
          adjustedScores.set(contentId, score * 1.1);
        }
        break;

      case Types.ExperimentVariant.VARIANT_B:
        // Boost based on collaborative signals
        for (const [contentId, score] of adjustedScores) {
          adjustedScores.set(contentId, score * 0.95); // Keep as-is
        }
        break;

      case Types.ExperimentVariant.VARIANT_C:
        // Boost trending content (learning path aligned)
        for (const [contentId, score] of adjustedScores) {
          adjustedScores.set(contentId, Math.min(1, score * 1.15));
        }
        break;

      case Types.ExperimentVariant.VARIANT_D:
        // Boost high-quality content
        for (const [contentId, score] of adjustedScores) {
          adjustedScores.set(contentId, Math.min(1, score + 0.05));
        }
        break;
    }

    return adjustedScores;
  }
}

// ============================================================================
// METRICS COLLECTOR
// ============================================================================

export class ExperimentMetricsCollector {
  private metrics: Map<string, Types.ExperimentMetrics> = new Map();
  private events: Array<{
    userId: string;
    experimentId: string;
    variant: Types.ExperimentVariant;
    eventType: string;
    properties: Record<string, any>;
    timestamp: Date;
  }> = [];

  /**
   * Record user interaction event
   */
  recordEvent(
    userId: string,
    experimentId: string,
    variant: Types.ExperimentVariant,
    eventType: string,
    properties: Record<string, any> = {}
  ): void {
    this.events.push({
      userId,
      experimentId,
      variant,
      eventType,
      properties,
      timestamp: new Date(),
    });
  }

  /**
   * Calculate CTR (click-through rate)
   */
  calculateCTR(experimentId: string, variant: Types.ExperimentVariant): number {
    const variantEvents = this.events.filter(
      e => e.experimentId === experimentId && e.variant === variant
    );

    const impressions = variantEvents.filter(e => e.eventType === 'impression').length;
    const clicks = variantEvents.filter(e => e.eventType === 'click').length;

    return impressions > 0 ? clicks / impressions : 0;
  }

  /**
   * Calculate completion rate
   */
  calculateCompletionRate(experimentId: string, variant: Types.ExperimentVariant): number {
    const variantEvents = this.events.filter(
      e => e.experimentId === experimentId && e.variant === variant
    );

    const started = variantEvents.filter(e => e.eventType === 'content_started').length;
    const completed = variantEvents.filter(e => e.eventType === 'content_completed').length;

    return started > 0 ? completed / started : 0;
  }

  /**
   * Calculate average session length
   */
  calculateAvgSessionLength(experimentId: string, variant: Types.ExperimentVariant): number {
    const variantEvents = this.events.filter(
      e => e.experimentId === experimentId && e.variant === variant && e.eventType === 'session_end'
    );

    if (variantEvents.length === 0) return 0;

    const totalDuration = variantEvents.reduce(
      (sum, e) => sum + (e.properties.duration_seconds || 0),
      0
    );

    return totalDuration / variantEvents.length;
  }

  /**
   * Calculate learning gain
   */
  calculateLearningGain(experimentId: string, variant: Types.ExperimentVariant): number {
    const variantEvents = this.events.filter(
      e =>
        e.experimentId === experimentId &&
        e.variant === variant &&
        e.eventType === 'assessment_completed'
    );

    if (variantEvents.length === 0) return 0;

    const totalGain = variantEvents.reduce(
      (sum, e) => sum + (e.properties.score_improvement || 0),
      0
    );

    return totalGain / variantEvents.length;
  }

  /**
   * Calculate 7-day retention
   */
  calculateRetention7Day(experimentId: string, variant: Types.ExperimentVariant): number {
    const variantUsers = new Set(
      this.events
        .filter(e => e.experimentId === experimentId && e.variant === variant)
        .map(e => e.userId)
    );

    const now = new Date();
    const sevenDaysAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);

    const recentUsers = new Set(
      this.events
        .filter(
          e =>
            e.experimentId === experimentId &&
            e.variant === variant &&
            e.timestamp > sevenDaysAgo
        )
        .map(e => e.userId)
    );

    return variantUsers.size > 0 ? recentUsers.size / variantUsers.size : 0;
  }

  /**
   * Get metrics for variant
   */
  getMetricsForVariant(
    experimentId: string,
    variant: Types.ExperimentVariant,
    sampleSize: number = 0
  ): Types.ExperimentMetrics {
    const uniqueUsers = new Set(
      this.events
        .filter(e => e.experimentId === experimentId && e.variant === variant)
        .map(e => e.userId)
    ).size;

    const metrics: Types.ExperimentMetrics = {
      experimentId,
      variant,
      metrics: {
        ctr: this.calculateCTR(experimentId, variant),
        completionRate: this.calculateCompletionRate(experimentId, variant),
        avgSessionLength: this.calculateAvgSessionLength(experimentId, variant),
        avgLearningGain: this.calculateLearningGain(experimentId, variant),
        retention7Day: this.calculateRetention7Day(experimentId, variant),
        diversity: 0.8, // Placeholder
      },
      sampleSize: uniqueUsers || sampleSize,
      confidenceInterval: {
        lower: 0.05,
        upper: 0.15,
        confidence: 0.95,
      },
    };

    return metrics;
  }

  /**
   * Get all events for export
   */
  getEvents(experimentId?: string, variant?: Types.ExperimentVariant) {
    let filtered = this.events;

    if (experimentId) {
      filtered = filtered.filter(e => e.experimentId === experimentId);
    }

    if (variant) {
      filtered = filtered.filter(e => e.variant === variant);
    }

    return filtered;
  }
}

// ============================================================================
// STATISTICAL ANALYSIS
// ============================================================================

export class StatisticalAnalyzer {
  /**
   * Perform t-test between two variants
   */
  performTTest(
    controlMetrics: number[],
    treatmentMetrics: number[],
    alpha: number = 0.05
  ): {
    tStatistic: number;
    pValue: number;
    significant: boolean;
    controlMean: number;
    treatmentMean: number;
  } {
    const controlMean = this.calculateMean(controlMetrics);
    const treatmentMean = this.calculateMean(treatmentMetrics);

    const controlVar = this.calculateVariance(controlMetrics, controlMean);
    const treatmentVar = this.calculateVariance(treatmentMetrics, treatmentMean);

    const pooledStdErr = Math.sqrt(
      (controlVar / controlMetrics.length + treatmentVar / treatmentMetrics.length)
    );

    const tStatistic = (treatmentMean - controlMean) / (pooledStdErr || 1);

    // Approximation of p-value using normal distribution
    const pValue = 2 * (1 - this.normalCDF(Math.abs(tStatistic)));

    return {
      tStatistic,
      pValue,
      significant: pValue < alpha,
      controlMean,
      treatmentMean,
    };
  }

  /**
   * Calculate sample size needed for experiment
   */
  calculateRequiredSampleSize(
    baselineRate: number,
    minDetectableEffect: number = 0.05,
    alpha: number = 0.05,
    beta: number = 0.2
  ): number {
    const z_alpha = this.inverseNormalCDF(1 - alpha / 2);
    const z_beta = this.inverseNormalCDF(1 - beta);

    const p1 = baselineRate;
    const p2 = baselineRate + minDetectableEffect;

    const numerator = (z_alpha + z_beta) ** 2 * (p1 * (1 - p1) + p2 * (1 - p2));
    const denominator = (p2 - p1) ** 2;

    return Math.ceil(numerator / denominator);
  }

  /**
   * Determine winner between variants
   */
  determineWinner(
    controlMetrics: Types.ExperimentMetrics,
    treatmentMetrics: Types.ExperimentMetrics,
    primaryMetric: 'ctr' | 'completionRate' | 'retention7Day' = 'ctr',
    minSampleSize: number = 1000
  ): {
    winner: Types.ExperimentVariant | 'inconclusive';
    confidence: number;
    recommendation: string;
  } {
    if (
      controlMetrics.sampleSize < minSampleSize ||
      treatmentMetrics.sampleSize < minSampleSize
    ) {
      return {
        winner: 'inconclusive',
        confidence: 0,
        recommendation: `Run experiment longer - need ${minSampleSize} users per variant`,
      };
    }

    const controlValue = controlMetrics.metrics[primaryMetric];
    const treatmentValue = treatmentMetrics.metrics[primaryMetric];

    const percentChange = ((treatmentValue - controlValue) / controlValue) * 100;

    if (Math.abs(percentChange) < 5) {
      return {
        winner: 'inconclusive',
        confidence: 0.5,
        recommendation: 'Difference is not practically significant (< 5%)',
      };
    }

    const winner = treatmentValue > controlValue ? treatmentMetrics.variant : controlMetrics.variant;
    const confidence = Math.min(0.95, Math.abs(percentChange) / 100);

    return {
      winner,
      confidence,
      recommendation: `Variant ${winner} performs ${percentChange.toFixed(1)}% ${treatmentValue > controlValue ? 'better' : 'worse'} on ${primaryMetric}`,
    };
  }

  // ========================================================================
  // PRIVATE HELPERS
  // ========================================================================

  private calculateMean(values: number[]): number {
    return values.reduce((a, b) => a + b, 0) / values.length;
  }

  private calculateVariance(values: number[], mean: number): number {
    const squaredDiffs = values.map(v => (v - mean) ** 2);
    return squaredDiffs.reduce((a, b) => a + b, 0) / (values.length - 1 || 1);
  }

  private normalCDF(z: number): number {
    // Approximation of standard normal CDF
    return 0.5 * (1 + this.erf(z / Math.sqrt(2)));
  }

  private erf(x: number): number {
    const a1 = 0.254829592;
    const a2 = -0.284496736;
    const a3 = 1.421413741;
    const a4 = -1.453152027;
    const a5 = 1.061405429;
    const p = 0.3275911;

    const sign = x < 0 ? -1 : 1;
    x = Math.abs(x);

    const t = 1 / (1 + p * x);
    const y = 1 - (((((a5 * t + a4) * t + a3) * t + a2) * t + a1) * t) * Math.exp(-x * x);

    return sign * y;
  }

  private inverseNormalCDF(p: number): number {
    // Approximation (Newton-Raphson would be more accurate)
    if (p === 0.5) return 0;
    if (p < 0.5) return -this.inverseNormalCDF(1 - p);

    const t = Math.sqrt(-2 * Math.log(1 - p));
    const c0 = 2.515517;
    const c1 = 0.802853;
    const c2 = 0.010328;
    const d1 = 1.432788;
    const d2 = 0.189269;
    const d3 = 0.001308;

    return (
      t -
      (c0 + c1 * t + c2 * t * t) / (1 + d1 * t + d2 * t * t + d3 * t * t * t)
    );
  }
}
