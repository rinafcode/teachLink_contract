import { Injectable } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { InjectRepository } from '@nestjs/typeorm';
import { DataSource, Repository } from 'typeorm';
import { IndexerState } from '@database/entities';
import { MetricsService } from './metrics.service';

type ServiceStatus = 'ok' | 'degraded' | 'error';

@Injectable()
export class HealthService {
  private readonly stateKey = 'main_indexer';

  constructor(
    private readonly dataSource: DataSource,
    private readonly configService: ConfigService,
    private readonly metricsService: MetricsService,
    @InjectRepository(IndexerState)
    private readonly indexerStateRepo: Repository<IndexerState>,
  ) {}

  async getHealthStatus(): Promise<{
    status: ServiceStatus;
    timestamp: string;
    service: string;
    checks: {
      database: ServiceStatus;
      horizon: ServiceStatus;
      indexerState: ServiceStatus;
    };
    details: {
      lastProcessedLedger: string;
      totalEventsProcessed: number;
      totalErrors: number;
      ledgerLagSeconds: number;
      staleAfterSeconds: number;
    };
    durationMs: number;
  }> {
    const startMs = Date.now();
    const staleAfterSeconds = this.configService.get<number>('indexer.staleAfterSeconds') ?? 900;

    // Run all independent checks in parallel
    const [dbStatus, horizonStatus, state] = await Promise.all([
      this.checkDatabase(),
      this.checkHorizon(),
      this.indexerStateRepo.findOne({ where: { key: this.stateKey } }),
    ]);

    const lastProcessedTimestamp = Number(state?.lastProcessedTimestamp ?? '0');
    const ledgerLagSeconds =
      lastProcessedTimestamp > 0
        ? Math.max(0, Math.floor(Date.now() / 1000) - lastProcessedTimestamp)
        : staleAfterSeconds + 1;

    const indexerState: ServiceStatus =
      !state || ledgerLagSeconds > staleAfterSeconds ? 'degraded' : 'ok';

    this.metricsService.updateDependencyHealth('indexer_state', indexerState === 'ok');

    const checks = { database: dbStatus, horizon: horizonStatus, indexerState };
    const statuses = Object.values(checks);
    const overallStatus: ServiceStatus = statuses.includes('error')
      ? 'error'
      : statuses.includes('degraded')
        ? 'degraded'
        : 'ok';

    this.metricsService.updateIndexerState({
      isRunning: state !== null,
      lastProcessedLedger: state?.lastProcessedLedger ?? '0',
      totalEventsProcessed: state?.totalEventsProcessed ?? 0,
      totalErrors: state?.totalErrors ?? 0,
      lastProcessedTimestamp: state?.lastProcessedTimestamp ?? '0',
    });

    return {
      status: overallStatus,
      timestamp: new Date().toISOString(),
      service: 'teachlink-indexer',
      checks,
      details: {
        lastProcessedLedger: state?.lastProcessedLedger ?? '0',
        totalEventsProcessed: state?.totalEventsProcessed ?? 0,
        totalErrors: state?.totalErrors ?? 0,
        ledgerLagSeconds,
        staleAfterSeconds,
      },
      durationMs: Date.now() - startMs,
    };
  }

  private async checkDatabase(): Promise<ServiceStatus> {
    try {
      await this.dataSource.query('SELECT 1');
      this.metricsService.updateDependencyHealth('database', true);
      return 'ok';
    } catch {
      this.metricsService.updateDependencyHealth('database', false);
      return 'error';
    }
  }

  private async checkHorizon(): Promise<ServiceStatus> {
    const horizonUrl = this.configService.get<string>('stellar.horizonUrl');

    if (!horizonUrl) {
      this.metricsService.updateDependencyHealth('horizon', false);
      return 'error';
    }

    try {
      const response = await fetch(horizonUrl, {
        method: 'GET',
        signal: AbortSignal.timeout(5000),
      });
      const healthy = response.ok;
      this.metricsService.updateDependencyHealth('horizon', healthy);
      return healthy ? 'ok' : 'error';
    } catch {
      this.metricsService.updateDependencyHealth('horizon', false);
      return 'error';
    }
  }
}
