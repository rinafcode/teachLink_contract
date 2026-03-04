import { Injectable } from '@nestjs/common';

/**
 * In-memory performance metrics for monitoring and alerting.
 * Use for cache hit rate, request counts, and latency tracking.
 */
@Injectable()
export class MetricsService {
  private requestCount = 0;
  private cacheHits = 0;
  private cacheMisses = 0;
  private lastDashboardMs = 0;
  private dashboardCallCount = 0;

  recordRequest(): void {
    this.requestCount += 1;
  }

  recordCacheHit(): void {
    this.cacheHits += 1;
  }

  recordCacheMiss(): void {
    this.cacheMisses += 1;
  }

  recordDashboardLatency(ms: number): void {
    this.lastDashboardMs = ms;
    this.dashboardCallCount += 1;
  }

  getSnapshot(): {
    requestCount: number;
    cacheHits: number;
    cacheMisses: number;
    cacheHitRate: number;
    lastDashboardMs: number;
    dashboardCallCount: number;
    uptimeSeconds: number;
  } {
    const total = this.cacheHits + this.cacheMisses;
    return {
      requestCount: this.requestCount,
      cacheHits: this.cacheHits,
      cacheMisses: this.cacheMisses,
      cacheHitRate: total > 0 ? this.cacheHits / total : 0,
      lastDashboardMs: this.lastDashboardMs,
      dashboardCallCount: this.dashboardCallCount,
      uptimeSeconds: process.uptime(),
    };
  }

  reset(): void {
    this.requestCount = 0;
    this.cacheHits = 0;
    this.cacheMisses = 0;
    this.lastDashboardMs = 0;
    this.dashboardCallCount = 0;
  }
}
