import {
  Entity,
  Column,
  PrimaryGeneratedColumn,
  Index,
  CreateDateColumn,
} from 'typeorm';

export enum ReportType {
  BRIDGE_HEALTH = 'bridge_health',
  ESCROW_SUMMARY = 'escrow_summary',
  COMPLIANCE_AUDIT = 'compliance_audit',
  REWARDS_SUMMARY = 'rewards_summary',
  TOKENIZATION_SUMMARY = 'tokenization_summary',
  CUSTOM = 'custom',
}

/**
 * Snapshot of dashboard/aggregate analytics at a point in time.
 * Used for report history, export, and visualization time series.
 */
@Entity('dashboard_snapshots')
@Index(['generatedAt'])
@Index(['reportType'])
@Index(['periodStart', 'periodEnd'])
export class DashboardSnapshot {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({
    type: 'enum',
    enum: ReportType,
  })
  reportType: ReportType;

  @Column({ type: 'bigint' })
  periodStart: string;

  @Column({ type: 'bigint' })
  periodEnd: string;

  @Column({ type: 'bigint' })
  generatedAt: string;

  @Column({ nullable: true })
  generatedBy: string;

  /** Bridge metrics */
  @Column({ type: 'int', default: 0 })
  bridgeHealthScore: number;

  @Column({ type: 'decimal', precision: 30, scale: 0, default: 0 })
  bridgeTotalVolume: string;

  @Column({ type: 'bigint', default: 0 })
  bridgeTotalTransactions: string;

  @Column({ type: 'int', default: 0 })
  bridgeSuccessRate: number;

  /** Escrow metrics */
  @Column({ type: 'bigint', default: 0 })
  escrowTotalCount: string;

  @Column({ type: 'decimal', precision: 30, scale: 0, default: 0 })
  escrowTotalVolume: string;

  @Column({ type: 'bigint', default: 0 })
  escrowDisputeCount: string;

  @Column({ type: 'bigint', default: 0 })
  escrowAvgResolutionTime: string;

  /** Rewards summary */
  @Column({ type: 'decimal', precision: 30, scale: 0, default: 0 })
  totalRewardsIssued: string;

  @Column({ type: 'bigint', default: 0 })
  rewardClaimCount: string;

  /** Audit/compliance */
  @Column({ type: 'int', default: 0 })
  complianceReportCount: number;

  @Column({ type: 'bigint', default: 0 })
  auditRecordCount: string;

  @CreateDateColumn()
  indexedAt: Date;
}
