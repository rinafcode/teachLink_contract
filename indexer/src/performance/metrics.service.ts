import { Injectable } from '@nestjs/common';
import { collectDefaultMetrics, Counter, Gauge, Histogram, Registry } from 'prom-client';

type HealthDependency = 'database' | 'horizon' | 'indexer_state';

@Injectable()
export class MetricsService {
  private readonly registry = new Registry();
  private readonly httpRequestsTotal: Counter<string>;
  private readonly httpRequestDurationSeconds: Histogram<string>;
  private readonly dashboardCacheRequestsTotal: Counter<string>;
  private readonly dashboardGenerationDurationSeconds: Histogram<string>;
  private readonly indexerEventsProcessedTotal: Gauge<string>;
  private readonly indexerErrorsTotal: Gauge<string>;
  private readonly indexerLastProcessedLedger: Gauge<string>;
  private readonly indexerLastProcessedTimestampSeconds: Gauge<string>;
  private readonly indexerRunning: Gauge<string>;
  private readonly indexerLedgerLagSeconds: Gauge<string>;
  private readonly dependencyUp: Gauge<string>;

  constructor() {
    collectDefaultMetrics({
      prefix: 'teachlink_indexer_',
      register: this.registry,
    });

    this.httpRequestsTotal = new Counter({
      name: 'teachlink_indexer_http_requests_total',
      help: 'Total HTTP requests served by the indexer',
      labelNames: ['method', 'route', 'status_code'],
      registers: [this.registry],
    });

    this.httpRequestDurationSeconds = new Histogram({
      name: 'teachlink_indexer_http_request_duration_seconds',
      help: 'HTTP request latency in seconds',
      labelNames: ['method', 'route', 'status_code'],
      buckets: [0.01, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10],
      registers: [this.registry],
    });

    this.dashboardCacheRequestsTotal = new Counter({
      name: 'teachlink_indexer_dashboard_cache_requests_total',
      help: 'Dashboard cache lookups by result',
      labelNames: ['result'],
      registers: [this.registry],
    });

    this.dashboardGenerationDurationSeconds = new Histogram({
      name: 'teachlink_indexer_dashboard_generation_duration_seconds',
      help: 'Dashboard aggregation latency in seconds',
      buckets: [0.01, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5],
      registers: [this.registry],
    });

    this.indexerEventsProcessedTotal = new Gauge({
      name: 'teachlink_indexer_events_processed_total',
      help: 'Latest persisted total event count from indexer state',
      registers: [this.registry],
    });

    this.indexerErrorsTotal = new Gauge({
      name: 'teachlink_indexer_errors_total',
      help: 'Latest persisted total error count from indexer state',
      registers: [this.registry],
    });

    this.indexerLastProcessedLedger = new Gauge({
      name: 'teachlink_indexer_last_processed_ledger',
      help: 'Last ledger successfully processed by the indexer',
      registers: [this.registry],
    });

    this.indexerLastProcessedTimestampSeconds = new Gauge({
      name: 'teachlink_indexer_last_processed_timestamp_seconds',
      help: 'Unix timestamp of the last processed event',
      registers: [this.registry],
    });

    this.indexerRunning = new Gauge({
      name: 'teachlink_indexer_running',
      help: 'Whether the indexer loop is currently running',
      registers: [this.registry],
    });

    this.indexerLedgerLagSeconds = new Gauge({
      name: 'teachlink_indexer_ledger_lag_seconds',
      help: 'Seconds since the most recent processed event',
      registers: [this.registry],
    });

    this.dependencyUp = new Gauge({
      name: 'teachlink_indexer_dependency_up',
      help: 'Dependency health status where 1 is healthy and 0 is unhealthy',
      labelNames: ['dependency'],
      registers: [this.registry],
    });
  }

  recordHttpRequest(
    method: string,
    route: string,
    statusCode: number,
    durationSeconds: number,
  ): void {
    const labels = {
      method: method.toUpperCase(),
      route,
      status_code: statusCode.toString(),
    };

    this.httpRequestsTotal.inc(labels);
    this.httpRequestDurationSeconds.observe(labels, durationSeconds);
  }

  recordCacheHit(): void {
    this.dashboardCacheRequestsTotal.inc({ result: 'hit' });
  }

  recordCacheMiss(): void {
    this.dashboardCacheRequestsTotal.inc({ result: 'miss' });
  }

  recordDashboardLatency(ms: number): void {
    this.dashboardGenerationDurationSeconds.observe(ms / 1000);
  }

  updateIndexerState(snapshot: {
    isRunning: boolean;
    lastProcessedLedger: string;
    totalEventsProcessed: number;
    totalErrors: number;
    lastProcessedTimestamp?: string;
  }): void {
    this.indexerRunning.set(snapshot.isRunning ? 1 : 0);
    this.indexerEventsProcessedTotal.set(snapshot.totalEventsProcessed);
    this.indexerErrorsTotal.set(snapshot.totalErrors);
    this.indexerLastProcessedLedger.set(Number(snapshot.lastProcessedLedger || '0'));

    const eventTimestamp = Number(snapshot.lastProcessedTimestamp || '0');
    if (eventTimestamp > 0) {
      this.indexerLastProcessedTimestampSeconds.set(eventTimestamp);
      this.indexerLedgerLagSeconds.set(Math.max(0, Math.floor(Date.now() / 1000) - eventTimestamp));
    } else {
      this.indexerLastProcessedTimestampSeconds.set(0);
      this.indexerLedgerLagSeconds.set(0);
    }
  }

  updateDependencyHealth(dependency: HealthDependency, isUp: boolean): void {
    this.dependencyUp.set({ dependency }, isUp ? 1 : 0);
  }

  async getPrometheusMetrics(): Promise<string> {
    return this.registry.metrics();
  }

  getContentType(): string {
    return this.registry.contentType;
  }
}
