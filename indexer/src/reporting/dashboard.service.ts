import { Injectable, Inject } from '@nestjs/common';
import { CACHE_MANAGER } from '@nestjs/cache-manager';
import { Cache } from 'cache-manager';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import {
  BridgeTransaction,
  BridgeStatus,
  Escrow,
  EscrowStatus,
  Reward,
  RewardStatus,
  RewardPool,
  DashboardSnapshot,
} from '@database/entities';
import { ReportType } from '@database/entities/dashboard-snapshot.entity';
import { MetricsService } from '../performance/metrics.service';

export const DASHBOARD_CACHE_KEY = 'dashboard:analytics';
export const DASHBOARD_CACHE_TTL_MS = 60_000; // 60s

export interface DashboardAnalyticsDto {
  bridgeHealthScore: number;
  bridgeTotalVolume: string;
  bridgeTotalTransactions: number;
  bridgeSuccessRate: number;
  escrowTotalCount: number;
  escrowTotalVolume: string;
  escrowDisputeCount: number;
  escrowAvgResolutionTime: number;
  totalRewardsIssued: string;
  rewardClaimCount: number;
  complianceReportCount: number;
  auditRecordCount: number;
  generatedAt: string;
}

@Injectable()
export class DashboardService {
  constructor(
    @Inject(CACHE_MANAGER) private cacheManager: Cache,
    private metricsService: MetricsService,
    @InjectRepository(BridgeTransaction)
    private bridgeRepo: Repository<BridgeTransaction>,
    @InjectRepository(Escrow)
    private escrowRepo: Repository<Escrow>,
    @InjectRepository(Reward)
    private rewardRepo: Repository<Reward>,
    @InjectRepository(RewardPool)
    private rewardPoolRepo: Repository<RewardPool>,
    @InjectRepository(DashboardSnapshot)
    private snapshotRepo: Repository<DashboardSnapshot>,
  ) {}

  /**
   * Aggregate current metrics from indexed data for dashboard visualization.
   * Uses in-memory cache (TTL 60s) and optimized SUM/count queries to avoid loading full tables.
   */
  async getCurrentAnalytics(): Promise<DashboardAnalyticsDto> {
    const cached = await this.cacheManager.get<DashboardAnalyticsDto>(DASHBOARD_CACHE_KEY);
    if (cached) {
      this.metricsService.recordCacheHit();
      return cached;
    }
    this.metricsService.recordCacheMiss();
    const start = Date.now();

    const now = Math.floor(Date.now() / 1000).toString();

    const [
      bridgeTxs,
      completedBridge,
      escrowCount,
      disputedEscrows,
      escrowVolumeResult,
      escrowAvgResult,
      rewardCount,
      claimedRewards,
      rewardVolumeResult,
    ] = await Promise.all([
      this.bridgeRepo.count(),
      this.bridgeRepo.count({ where: { status: BridgeStatus.COMPLETED } }),
      this.escrowRepo.count(),
      this.escrowRepo.count({ where: { status: EscrowStatus.DISPUTED } }),
      this.escrowRepo
        .createQueryBuilder('e')
        .select('COALESCE(SUM(CAST(e.amount AS DECIMAL)), 0)', 'sum')
        .getRawOne<{ sum: string }>(),
      this.escrowRepo
        .createQueryBuilder('e')
        .where('e.status = :status', { status: EscrowStatus.RELEASED })
        .andWhere('e.completedAtLedger IS NOT NULL')
        .andWhere('e.createdAtLedger IS NOT NULL')
        .select(
          'COALESCE(AVG(CAST(e.completedAtLedger AS BIGINT) - CAST(e.createdAtLedger AS BIGINT)), 0)',
          'avg',
        )
        .getRawOne<{ avg: string }>(),
      this.rewardRepo.count(),
      this.rewardRepo.count({ where: { status: RewardStatus.CLAIMED } }),
      this.rewardRepo
        .createQueryBuilder('r')
        .select('COALESCE(SUM(CAST(r.amount AS DECIMAL)), 0)', 'sum')
        .getRawOne<{ sum: string }>(),
    ]);

    let totalBridgeVolume = '0';
    if (bridgeTxs > 0) {
      const result = await this.bridgeRepo
        .createQueryBuilder('b')
        .select('COALESCE(SUM(b.amount), 0)', 'sum')
        .getRawOne<{ sum: string }>();
      totalBridgeVolume = result?.sum ?? '0';
    }

    const totalEscrowVolume = escrowVolumeResult?.sum ?? '0';
    const totalRewardsIssued = rewardVolumeResult?.sum ?? '0';
    const successRate = bridgeTxs > 0 ? Math.round((completedBridge / bridgeTxs) * 10000) : 10000;
    const healthScore = Math.min(100, Math.round(successRate / 100) + 80);
    const avgResolutionTime = Math.round(Number(escrowAvgResult?.avg ?? 0));

    const dto: DashboardAnalyticsDto = {
      bridgeHealthScore: healthScore,
      bridgeTotalVolume: totalBridgeVolume?.toString?.() ?? '0',
      bridgeTotalTransactions: bridgeTxs,
      bridgeSuccessRate: successRate,
      escrowTotalCount: escrowCount,
      escrowTotalVolume: totalEscrowVolume,
      escrowDisputeCount: disputedEscrows,
      escrowAvgResolutionTime: avgResolutionTime,
      totalRewardsIssued,
      rewardClaimCount: claimedRewards,
      complianceReportCount: 0,
      auditRecordCount: 0,
      generatedAt: now,
    };

    await this.cacheManager.set(DASHBOARD_CACHE_KEY, dto, DASHBOARD_CACHE_TTL_MS);
    this.metricsService.recordDashboardLatency(Date.now() - start);
    return dto;
  }

  /** Invalidate dashboard cache (e.g. after bulk event processing). */
  async invalidateDashboardCache(): Promise<void> {
    await this.cacheManager.del(DASHBOARD_CACHE_KEY);
  }

  /**
   * Save a snapshot for report history and time-series visualization.
   */
  async saveSnapshot(
    reportType: ReportType,
    periodStart: string,
    periodEnd: string,
    generatedBy?: string,
  ): Promise<DashboardSnapshot> {
    const analytics = await this.getCurrentAnalytics();
    const snapshot = this.snapshotRepo.create({
      reportType,
      periodStart,
      periodEnd,
      generatedAt: analytics.generatedAt,
      generatedBy: generatedBy ?? undefined,
      bridgeHealthScore: analytics.bridgeHealthScore,
      bridgeTotalVolume: analytics.bridgeTotalVolume,
      bridgeTotalTransactions: analytics.bridgeTotalTransactions.toString(),
      bridgeSuccessRate: analytics.bridgeSuccessRate,
      escrowTotalCount: analytics.escrowTotalCount.toString(),
      escrowTotalVolume: analytics.escrowTotalVolume,
      escrowDisputeCount: analytics.escrowDisputeCount.toString(),
      escrowAvgResolutionTime: analytics.escrowAvgResolutionTime.toString(),
      totalRewardsIssued: analytics.totalRewardsIssued,
      rewardClaimCount: analytics.rewardClaimCount.toString(),
      complianceReportCount: analytics.complianceReportCount,
      auditRecordCount: analytics.auditRecordCount.toString(),
    });
    return this.snapshotRepo.save(snapshot) as Promise<DashboardSnapshot>;
  }

  /**
   * Get snapshots for a time range (for charts and export).
   */
  async getSnapshots(periodStart: string, periodEnd: string, limit = 100): Promise<DashboardSnapshot[]> {
    const qb = this.snapshotRepo
      .createQueryBuilder('s')
      .where('s.periodStart >= :periodStart', { periodStart })
      .andWhere('s.periodEnd <= :periodEnd', { periodEnd })
      .orderBy('s.generatedAt', 'DESC')
      .take(limit);
    return qb.getMany();
  }
}
