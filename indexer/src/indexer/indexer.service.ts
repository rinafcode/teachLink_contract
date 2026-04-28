import { Injectable, Logger, OnModuleInit, OnModuleDestroy } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { Cron, CronExpression } from '@nestjs/schedule';
import { HorizonService, ProcessedEvent } from '@horizon/horizon.service';
import { EventProcessorService } from '@events/event-processor.service';
import { IndexerState } from '@database/entities';
import { MetricsService } from '../performance/metrics.service';

@Injectable()
export class IndexerService implements OnModuleInit, OnModuleDestroy {
  private readonly logger = new Logger(IndexerService.name);
  private closeStreamHandler: (() => void) | null = null;
  private isRunning = false;
  private readonly stateKey = 'main_indexer';

  constructor(
    private horizonService: HorizonService,
    private eventProcessor: EventProcessorService,
    private configService: ConfigService,
    private metricsService: MetricsService,
    @InjectRepository(IndexerState)
    private indexerStateRepo: Repository<IndexerState>,
  ) {}

  async onModuleInit() {
    this.logger.log('Initializing TeachLink Indexer Service');
    await this.startIndexing();
  }

  async onModuleDestroy() {
    this.logger.log('Shutting down TeachLink Indexer Service');
    await this.stopIndexing();
  }

  async startIndexing(): Promise<void> {
    if (this.isRunning) {
      this.logger.warn('Indexer is already running');
      return;
    }

    try {
      this.isRunning = true;

      // Get or create indexer state
      let state = await this.indexerStateRepo.findOne({
        where: { key: this.stateKey },
      });

      let startLedger: string;

      if (!state) {
        // First run - determine starting point
        const configStartLedger = this.configService.get<string>('indexer.startLedger') || 'latest';

        if (configStartLedger === 'latest') {
          const latestLedger = await this.horizonService.getLatestLedger();
          startLedger = latestLedger.toString();
        } else {
          startLedger = configStartLedger;
        }

        state = this.indexerStateRepo.create({
          key: this.stateKey,
          lastProcessedLedger: startLedger,
          totalEventsProcessed: 0,
          totalErrors: 0,
        });

        await this.indexerStateRepo.save(state);
        this.logger.log(`Created new indexer state starting from ledger ${startLedger}`);
        this.publishStateMetrics(state);
      } else {
        startLedger = state.lastProcessedLedger;
        this.logger.log(`Resuming indexing from ledger ${startLedger}`);
        this.publishStateMetrics(state);
      }

      // Start streaming events
      this.closeStreamHandler = await this.horizonService.streamContractEvents(
        startLedger,
        this.handleEvent.bind(this),
        this.handleError.bind(this),
      );

      this.logger.log('Indexer started successfully');
      this.metricsService.updateIndexerState({
        isRunning: true,
        lastProcessedLedger: startLedger,
        totalEventsProcessed: state.totalEventsProcessed,
        totalErrors: state.totalErrors,
        lastProcessedTimestamp: state.lastProcessedTimestamp || '0',
      });
    } catch (error) {
      this.logger.error(`Failed to start indexer: ${error.message}`, error.stack);
      this.isRunning = false;
      this.metricsService.updateIndexerState({
        isRunning: false,
        lastProcessedLedger: '0',
        totalEventsProcessed: 0,
        totalErrors: 0,
        lastProcessedTimestamp: '0',
      });
      throw error;
    }
  }

  async stopIndexing(): Promise<void> {
    if (!this.isRunning) {
      return;
    }

    try {
      if (this.closeStreamHandler) {
        this.closeStreamHandler();
        this.closeStreamHandler = null;
      }

      this.isRunning = false;
      const status = await this.getStatus();
      this.metricsService.updateIndexerState({
        ...status,
        isRunning: false,
      });
      this.logger.log('Indexer stopped successfully');
    } catch (error) {
      this.logger.error(`Error stopping indexer: ${error.message}`, error.stack);
    }
  }

  private async handleEvent(event: ProcessedEvent): Promise<void> {
    try {
      this.logger.debug(`Processing event from ledger ${event.ledger}`);

      // Process the event
      await this.eventProcessor.processEvent(event);

      // Update indexer state
      const state = await this.indexerStateRepo.findOne({
        where: { key: this.stateKey },
      });

      if (state) {
        state.lastProcessedLedger = event.ledger;
        state.lastProcessedTxHash = event.txHash;
        state.lastProcessedTimestamp = event.timestamp;
        state.totalEventsProcessed += 1;
        await this.indexerStateRepo.save(state);
        this.publishStateMetrics(state);
      }
    } catch (error) {
      this.logger.error(`Error handling event: ${error.message}`, error.stack);
      await this.incrementErrorCount();
    }
  }

  private handleError(error: Error): void {
    this.logger.error(`Stream error: ${error.message}`, error.stack);
    this.incrementErrorCount();
  }

  private async incrementErrorCount(): Promise<void> {
    try {
      const state = await this.indexerStateRepo.findOne({
        where: { key: this.stateKey },
      });

      if (state) {
        state.totalErrors += 1;
        await this.indexerStateRepo.save(state);
        this.publishStateMetrics(state);
      }
    } catch (error) {
      this.logger.error(`Error updating error count: ${error.message}`);
    }
  }

  /**
   * Backfill historical data with parallel event processing.
   *
   * Events are fetched in parallel (handled by HorizonService), then
   * processed in parallel batches (concurrency=20) for throughput.
   * State is updated after each batch to provide progress tracking.
   */
  async backfillHistoricalData(startLedger: number, endLedger: number): Promise<void> {
    this.logger.log(`Starting parallel backfill from ledger ${startLedger} to ${endLedger}`);
    const backfillStart = Date.now();

    try {
      // Parallel ledger fetching (handled inside HorizonService)
      const events = await this.horizonService.fetchOperationsInRange(startLedger, endLedger);

      this.logger.log(`Found ${events.length} events to process in parallel`);

      const BATCH_SIZE = 20;
      let processedCount = 0;
      let errorCount = 0;

      for (let i = 0; i < events.length; i += BATCH_SIZE) {
        const batch = events.slice(i, i + BATCH_SIZE);

        const results = await Promise.allSettled(
          batch.map((event) => this.eventProcessor.processEvent(event)),
        );

        for (const result of results) {
          if (result.status === 'fulfilled') {
            processedCount++;
          } else {
            errorCount++;
            this.logger.warn(`Backfill event error: ${result.reason?.message}`);
          }
        }

        // Periodic state update after each batch
        const lastEvent = batch[batch.length - 1];
        const state = await this.indexerStateRepo.findOne({
          where: { key: this.stateKey },
        });

        if (state && lastEvent) {
          state.lastProcessedLedger = lastEvent.ledger;
          state.totalEventsProcessed += batch.length;
          state.totalErrors += results.filter((r) => r.status === 'rejected').length;
          await this.indexerStateRepo.save(state);
        }
      }

      const durationMs = Date.now() - backfillStart;
      const eventsPerSec = processedCount > 0 ? Math.round((processedCount / durationMs) * 1000) : 0;

      this.logger.log(
        `Backfill completed: ${processedCount} events processed, ${errorCount} errors, ` +
        `${durationMs}ms elapsed (${eventsPerSec} events/sec)`,
      );
    } catch (error) {
      this.logger.error(`Backfill failed: ${error.message}`, error.stack);
      throw error;
    }
  }

  /**
   * Get current indexer status
   */
  async getStatus(): Promise<{
    isRunning: boolean;
    lastProcessedLedger: string;
    totalEventsProcessed: number;
    totalErrors: number;
    lastProcessedTimestamp: string;
  }> {
    const state = await this.indexerStateRepo.findOne({
      where: { key: this.stateKey },
    });

    if (!state) {
      return {
        isRunning: this.isRunning,
        lastProcessedLedger: '0',
        totalEventsProcessed: 0,
        totalErrors: 0,
        lastProcessedTimestamp: '0',
      };
    }

    return {
      isRunning: this.isRunning,
      lastProcessedLedger: state.lastProcessedLedger,
      totalEventsProcessed: state.totalEventsProcessed,
      totalErrors: state.totalErrors,
      lastProcessedTimestamp: state.lastProcessedTimestamp || '0',
    };
  }

  /**
   * Health check - runs periodically to ensure indexer is healthy
   */
  @Cron(CronExpression.EVERY_5_MINUTES)
  async healthCheck(): Promise<void> {
    const status = await this.getStatus();

    this.logger.debug('Indexer health check', {
      isRunning: status.isRunning,
      lastProcessedLedger: status.lastProcessedLedger,
      totalEventsProcessed: status.totalEventsProcessed,
      totalErrors: status.totalErrors,
    });

    this.metricsService.updateIndexerState(status);

    // Restart indexer if it's not running
    if (!this.isRunning) {
      this.logger.warn('Indexer is not running - attempting to restart');
      await this.startIndexing();
    }
  }

  private publishStateMetrics(state: IndexerState): void {
    this.metricsService.updateIndexerState({
      isRunning: this.isRunning,
      lastProcessedLedger: state.lastProcessedLedger,
      totalEventsProcessed: state.totalEventsProcessed,
      totalErrors: state.totalErrors,
      lastProcessedTimestamp: state.lastProcessedTimestamp || '0',
    });
  }
}
