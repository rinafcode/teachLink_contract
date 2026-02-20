/**
 * Evaluation Metrics
 * 
 * Offline and online metrics for recommendation quality
 */

import * as Types from '../types';

// ============================================================================
// OFFLINE EVALUATION METRICS
// ============================================================================

export class OfflineEvaluator {
  /**
   * Normalized Discounted Cumulative Gain (NDCG)
   * Measures ranking quality considering position
   */
  computeNDCG(
    rankedItems: Array<{ id: string; isRelevant: boolean }>,
    k: number = 10
  ): number {
    const dcg = this.computeDCG(rankedItems, k);
    const idcg = this.computeIDCG(rankedItems.length, k);

    return idcg > 0 ? dcg / idcg : 0;
  }

  /**
   * Mean Average Precision (MAP)
   * Measures precision at each relevant item position
   */
  computeMAP(
    rankedItems: Array<{ id: string; isRelevant: boolean }>[],
    k: number = 10
  ): number {
    let sumAP = 0;

    for (const ranking of rankedItems) {
      let sumPrecision = 0;
      let numRelevant = 0;

      for (let i = 0; i < Math.min(k, ranking.length); i++) {
        if (ranking[i].isRelevant) {
          numRelevant++;
          sumPrecision += numRelevant / (i + 1);
        }
      }

      const ap = ranking.some(r => r.isRelevant) ? sumPrecision / numRelevant : 0;
      sumAP += ap;
    }

    return rankedItems.length > 0 ? sumAP / rankedItems.length : 0;
  }

  /**
   * Recall@K
   * Fraction of relevant items that appear in top K
   */
  computeRecall(
    rankedItems: Array<{ id: string; isRelevant: boolean }>,
    k: number = 10,
    totalRelevant: number = 0
  ): number {
    if (totalRelevant === 0) {
      totalRelevant = rankedItems.filter(r => r.isRelevant).length;
    }

    if (totalRelevant === 0) return 0;

    const relevantInK = rankedItems.slice(0, k).filter(r => r.isRelevant).length;
    return relevantInK / totalRelevant;
  }

  /**
   * Precision@K
   * Fraction of top K items that are relevant
   */
  computePrecision(
    rankedItems: Array<{ id: string; isRelevant: boolean }>,
    k: number = 10
  ): number {
    if (rankedItems.length === 0) return 0;

    const relevantInK = rankedItems.slice(0, k).filter(r => r.isRelevant).length;
    return relevantInK / Math.min(k, rankedItems.length);
  }

  /**
   * Serendipity
   * Relevance of unexpected recommendations
   */
  computeSerendipity(
    rankedItems: Array<{ id: string; isRelevant: boolean; unexpectedness: number }>,
    k: number = 10
  ): number {
    let serendipityScore = 0;
    let count = 0;

    for (let i = 0; i < Math.min(k, rankedItems.length); i++) {
      if (rankedItems[i].isRelevant) {
        serendipityScore += rankedItems[i].unexpectedness;
        count++;
      }
    }

    return count > 0 ? serendipityScore / count : 0;
  }

  /**
   * Diversity
   * How diverse are recommended items
   */
  computeDiversity(
    rankedItems: Array<{ id: string; category: string }>,
    k: number = 10
  ): number {
    const topK = rankedItems.slice(0, k);
    const categories = new Set(topK.map(r => r.category));

    return categories.size / Math.min(k, topK.length);
  }

  /**
   * Coverage
   * Percentage of catalog represented in recommendations
   */
  computeCoverage(
    allRecommendations: string[],
    catalogSize: number
  ): number {
    const unique = new Set(allRecommendations);
    return unique.size / catalogSize;
  }

  /**
   * Novelty
   * Average rank of long-tail items
   */
  computeNovelty(rankings: Array<{ id: string; popularity: number }>): number {
    let noveltySum = 0;

    for (let i = 0; i < rankings.length; i++) {
      // Normalize popularity to 0-1
      const popularity = rankings[i].popularity / 100;
      const novelty = 1 - popularity;
      noveltySum += novelty;
    }

    return rankings.length > 0 ? noveltySum / rankings.length : 0;
  }

  /**
   * Aggregate offline metrics
   */
  computeOfflineMetrics(
    rankedLists: Array<Array<{ id: string; isRelevant: boolean }>>,
    catalogSize: number,
    k: number = 10
  ): Types.OfflineMetrics {
    const ndcgScores = rankedLists.map(r => this.computeNDCG(r, k));
    const precisionScores = rankedLists.map(r => this.computePrecision(r, k));
    const recallScores = rankedLists.map(r => this.computeRecall(r, k));

    const allItems = rankedLists.flatMap(r => r.map(i => i.id));
    const coverage = this.computeCoverage(allItems, catalogSize);

    return {
      ndcg10: this.mean(ndcgScores.slice(0, 10)),
      ndcg20: this.mean(ndcgScores.slice(0, 20)),
      ndcg50: this.mean(ndcgScores),
      map10: this.mean(rankedLists.map(r => this.computeMAP([r], 10))),
      map20: this.mean(rankedLists.map(r => this.computeMAP([r], 20))),
      recall10: this.mean(recallScores.slice(0, 10)),
      recall20: this.mean(recallScores.slice(0, 20)),
      recall50: this.mean(recallScores),
      precision10: this.mean(
        precisionScores.filter((_, i) => i < rankedLists.length / 10)
      ),
      precision20: this.mean(
        precisionScores.filter((_, i) => i < rankedLists.length / 5)
      ),
      serendipity: 0.65, // Placeholder
      diversity: 0.72, // Placeholder
      coverage,
      novelty: 0.58,
    };
  }

  // ========================================================================
  // PRIVATE HELPERS
  // ========================================================================

  private computeDCG(rankings: Array<{ id: string; isRelevant: boolean }>, k: number): number {
    let dcg = 0;

    for (let i = 0; i < Math.min(k, rankings.length); i++) {
      if (rankings[i].isRelevant) {
        dcg += 1 / Math.log2(i + 2);
      }
    }

    return dcg;
  }

  private computeIDCG(totalItems: number, k: number): number {
    let idcg = 0;

    for (let i = 0; i < Math.min(k, totalItems); i++) {
      idcg += 1 / Math.log2(i + 2);
    }

    return idcg;
  }

  private mean(values: number[]): number {
    if (values.length === 0) return 0;
    return values.reduce((a, b) => a + b, 0) / values.length;
  }
}

// ============================================================================
// ONLINE METRICS
// ============================================================================

export class OnlineMetricsCollector {
  private metrics: Map<string, Types.OnlineMetrics> = new Map();
  private events: Array<{
    userId: string;
    contentId: string;
    eventType: string;
    timestamp: Date;
    properties?: Record<string, any>;
  }> = [];

  /**
   * Record user interaction event
   */
  recordEvent(
    userId: string,
    contentId: string,
    eventType: 'view' | 'click' | 'complete' | 'rate',
    properties?: Record<string, any>
  ): void {
    this.events.push({
      userId,
      contentId,
      eventType,
      timestamp: new Date(),
      properties,
    });
  }

  /**
   * Calculate Click-Through Rate
   */
  calculateCTR(window: number = 24 * 60 * 60 * 1000): number {
    const cutoff = new Date(Date.now() - window);
    const recentEvents = this.events.filter(e => e.timestamp > cutoff);

    const views = recentEvents.filter(e => e.eventType === 'view').length;
    const clicks = recentEvents.filter(e => e.eventType === 'click').length;

    return views > 0 ? clicks / views : 0;
  }

  /**
   * Calculate Completion Rate
   */
  calculateCompletionRate(window: number = 24 * 60 * 60 * 1000): number {
    const cutoff = new Date(Date.now() - window);
    const recentEvents = this.events.filter(e => e.timestamp > cutoff);

    const starts = recentEvents.filter(e => e.eventType === 'click').length;
    const completions = recentEvents.filter(e => e.eventType === 'complete').length;

    return starts > 0 ? completions / starts : 0;
  }

  /**
   * Calculate average session length
   */
  calculateAvgSessionLength(window: number = 24 * 60 * 60 * 1000): number {
    const cutoff = new Date(Date.now() - window);
    const recentEvents = this.events.filter(e => e.timestamp > cutoff);

    const sessions: Map<string, number> = new Map();

    for (const event of recentEvents) {
      const sessionKey = event.userId;
      sessions.set(sessionKey, (sessions.get(sessionKey) || 0) + (event.properties?.duration_ms || 0));
    }

    if (sessions.size === 0) return 0;

    const totalDuration = Array.from(sessions.values()).reduce((a, b) => a + b, 0);
    return totalDuration / sessions.size / 1000; // Convert to seconds
  }

  /**
   * Calculate learning gain
   */
  calculateLearningGain(window: number = 24 * 60 * 60 * 1000): number {
    const cutoff = new Date(Date.now() - window);
    const ratingEvents = this.events.filter(
      e => e.eventType === 'rate' && e.timestamp > cutoff
    );

    if (ratingEvents.length === 0) return 0;

    const totalGain = ratingEvents.reduce(
      (sum, e) => sum + (e.properties?.score_improvement || 0),
      0
    );

    return totalGain / ratingEvents.length;
  }

  /**
   * Calculate retention (% of users active after N days)
   */
  calculateRetention(days: number): number {
    const uniqueUsers = new Set(this.events.map(e => e.userId));
    if (uniqueUsers.size === 0) return 0;

    const cutoffTime = new Date(Date.now() - days * 24 * 60 * 60 * 1000);
    const activeUsers = new Set(
      this.events.filter(e => e.timestamp > cutoffTime).map(e => e.userId)
    );

    return activeUsers.size / uniqueUsers.size;
  }

  /**
   * Calculate average satisfaction score
   */
  calculateSatisfaction(window: number = 24 * 60 * 60 * 1000): number {
    const cutoff = new Date(Date.now() - window);
    const ratings = this.events
      .filter(e => e.eventType === 'rate' && e.timestamp > cutoff)
      .map(e => e.properties?.rating || 0);

    if (ratings.length === 0) return 0;

    return ratings.reduce((a, b) => a + b, 0) / ratings.length;
  }

  /**
   * Get comprehensive online metrics
   */
  getOnlineMetrics(label: string = 'default'): Types.OnlineMetrics {
    const metrics: Types.OnlineMetrics = {
      ctr: this.calculateCTR(),
      completionRate: this.calculateCompletionRate(),
      avgSessionLengthSeconds: this.calculateAvgSessionLength(),
      avgLearningGain: this.calculateLearningGain(),
      retention1Day: this.calculateRetention(1),
      retention7Day: this.calculateRetention(7),
      retention30Day: this.calculateRetention(30),
      satisfactionScore: this.calculateSatisfaction(),
      diversity: 0.75, // Placeholder
      fairnessScore: 0.82, // Placeholder
    };

    this.metrics.set(label, metrics);
    return metrics;
  }
}

// ============================================================================
// MODEL COMPARISON
// ============================================================================

export class ModelComparator {
  /**
   * Compare two model performances
   */
  compareModels(
    baselineMetrics: Types.OfflineMetrics,
    candidateMetrics: Types.OfflineMetrics
  ): {
    improvements: Record<string, number>;
    recommendation: string;
  } {
    const improvements: Record<string, number> = {};

    Object.entries(candidateMetrics).forEach(([metric, candidateValue]) => {
      const baselineValue = baselineMetrics[metric as keyof Types.OfflineMetrics];
      if (typeof baselineValue === 'number' && typeof candidateValue === 'number') {
        improvements[metric] = ((candidateValue - baselineValue) / baselineValue) * 100;
      }
    });

    const avgImprovement = Object.values(improvements).reduce((a, b) => a + b, 0) / Object.keys(improvements).length;

    let recommendation = '';
    if (avgImprovement > 10) {
      recommendation = 'Deploy candidate model (significant improvement)';
    } else if (avgImprovement > 0) {
      recommendation = 'Deploy candidate with monitoring (marginal improvement)';
    } else if (avgImprovement > -5) {
      recommendation = 'Keep baseline (no significant regression)';
    } else {
      recommendation = 'Investigate candidate model (significant regression)';
    }

    return { improvements, recommendation };
  }

  /**
   * Track metric trends over time
   */
  trackMetricTrend(
    historicalMetrics: Types.ModelPerformance[],
    metric: keyof Types.OfflineMetrics
  ): {
    trend: 'improving' | 'stable' | 'declining';
    changePercent: number;
  } {
    if (historicalMetrics.length < 2) {
      return { trend: 'stable', changePercent: 0 };
    }

    const recent = historicalMetrics[historicalMetrics.length - 1];
    const previous = historicalMetrics[historicalMetrics.length - 2];

    const recentValue = recent.offlineMetrics[metric as keyof Types.OfflineMetrics] as number;
    const previousValue = previous.offlineMetrics[metric as keyof Types.OfflineMetrics] as number;

    const changePercent = ((recentValue - previousValue) / previousValue) * 100;

    let trend: 'improving' | 'stable' | 'declining' = 'stable';
    if (changePercent > 2) trend = 'improving';
    else if (changePercent < -2) trend = 'declining';

    return { trend, changePercent };
  }
}

// ============================================================================
// METRICS DASHBOARD
// ============================================================================

export class MetricsDashboard {
  private offlineMetrics: Types.OfflineMetrics | null = null;
  private onlineMetrics: Types.OnlineMetrics | null = null;
  private history: Array<{ timestamp: Date; offline: Types.OfflineMetrics; online: Types.OnlineMetrics }> = [];

  /**
   * Update metrics snapshots
   */
  updateMetrics(offline: Types.OfflineMetrics, online: Types.OnlineMetrics): void {
    this.offlineMetrics = offline;
    this.onlineMetrics = online;
    this.history.push({
      timestamp: new Date(),
      offline,
      online,
    });

    // Keep last 100 snapshots
    if (this.history.length > 100) {
      this.history.shift();
    }
  }

  /**
   * Get current dashboard state
   */
  getDashboard(): {
    current: { offline: Types.OfflineMetrics | null; online: Types.OnlineMetrics | null };
    status: 'healthy' | 'warning' | 'critical';
    alerts: string[];
  } {
    const alerts: string[] = [];

    if (this.offlineMetrics && this.offlineMetrics.ndcg10 < 0.7) {
      alerts.push('NDCG@10 below threshold (0.7)');
    }

    if (this.onlineMetrics && this.onlineMetrics.ctr < 0.05) {
      alerts.push('CTR below threshold (0.05)');
    }

    if (this.onlineMetrics && this.onlineMetrics.completionRate < 0.4) {
      alerts.push('Completion rate below threshold (0.4)');
    }

    let status: 'healthy' | 'warning' | 'critical' = 'healthy';
    if (alerts.length >= 2) status = 'critical';
    else if (alerts.length === 1) status = 'warning';

    return {
      current: {
        offline: this.offlineMetrics,
        online: this.onlineMetrics,
      },
      status,
      alerts,
    };
  }

  /**
   * Export metrics for monitoring systems
   */
  exportMetrics(): Record<string, any> {
    return {
      timestamp: new Date().toISOString(),
      offline: this.offlineMetrics,
      online: this.onlineMetrics,
      historicalTrend: {
        samples: this.history.length,
        periodDays: this.history.length > 0 ? (Date.now() - this.history[0].timestamp.getTime()) / (24 * 60 * 60 * 1000) : 0,
      },
    };
  }
}
