import { Injectable } from '@nestjs/common';
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
   */
  async getCurrentAnalytics(): Promise<DashboardAnalyticsDto> {
    const now = Math.floor(Date.now() / 1000).toString();
    const dayAgo = (Math.floor(Date.now() / 1000) - 86400).toString();

    const [bridgeTxs, completedBridge, escrows, disputedEscrows, releasedEscrows, rewards, claimedRewards, pools] =
      await Promise.all([
        this.bridgeRepo.count(),
        this.bridgeRepo.count({ where: { status: BridgeStatus.COMPLETED } }),
        this.escrowRepo.find(),
        this.escrowRepo.count({ where: { status: EscrowStatus.DISPUTED } }),
        this.escrowRepo.find({ where: { status: EscrowStatus.RELEASED } }),
        this.rewardRepo.find(),
        this.rewardRepo.count({ where: { status: RewardStatus.CLAIMED } }),
        this.rewardPoolRepo.find(),
      ]);

    let totalBridgeVolume = '0';
    if (bridgeTxs > 0) {
      const result = await this.bridgeRepo
        .createQueryBuilder('b')
        .select('COALESCE(SUM(b.amount), 0)', 'sum')
        .getRawOne<{ sum: string }>();
      totalBridgeVolume = result?.sum ?? '0';
    }
    const totalEscrowVolume = escrows.length
      ? escrows.reduce((acc, e) => acc + BigInt(e.amount), BigInt(0)).toString()
      : '0';
    const totalRewardsIssued = rewards.length
      ? rewards.reduce((acc, r) => acc + BigInt(r.amount), BigInt(0)).toString()
      : '0';

    const successRate = bridgeTxs > 0 ? Math.round((completedBridge / bridgeTxs) * 10000) : 10000;
    const healthScore = Math.min(100, Math.round(successRate / 100) + 80);

    let avgResolutionTime = 0;
    if (releasedEscrows.length > 0) {
      const withTimes = releasedEscrows.filter((e) => e.completedAtLedger && e.createdAtLedger);
      if (withTimes.length > 0) {
        const sum = withTimes.reduce(
          (acc, e) => acc + (Number(e.completedAtLedger) - Number(e.createdAtLedger)),
          0,
        );
        avgResolutionTime = Math.round(sum / withTimes.length);
      }
    }

    return {
      bridgeHealthScore: healthScore,
      bridgeTotalVolume: totalBridgeVolume?.toString?.() ?? '0',
      bridgeTotalTransactions: bridgeTxs,
      bridgeSuccessRate: successRate,
      escrowTotalCount: escrows.length,
      escrowTotalVolume: totalEscrowVolume,
      escrowDisputeCount: disputedEscrows,
      escrowAvgResolutionTime: avgResolutionTime,
      totalRewardsIssued,
      rewardClaimCount: claimedRewards,
      complianceReportCount: 0,
      auditRecordCount: 0,
      generatedAt: now,
    };
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
      generatedBy: generatedBy ?? null,
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
    return this.snapshotRepo.save(snapshot);
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
