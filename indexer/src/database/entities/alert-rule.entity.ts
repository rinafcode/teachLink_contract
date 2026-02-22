import {
  Entity,
  Column,
  PrimaryGeneratedColumn,
  Index,
  CreateDateColumn,
} from 'typeorm';

export enum AlertConditionType {
  BRIDGE_HEALTH_BELOW = 'bridge_health_below',
  ESCROW_DISPUTE_RATE_ABOVE = 'escrow_dispute_rate_above',
  VOLUME_ABOVE = 'volume_above',
  VOLUME_BELOW = 'volume_below',
  TRANSACTION_COUNT_ABOVE = 'transaction_count_above',
}

/**
 * Alert rule for real-time reporting and alerting.
 * Indexer evaluates these against current metrics and creates AlertLog on breach.
 */
@Entity('alert_rules')
@Index(['owner'])
@Index(['enabled'])
export class AlertRule {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'bigint', unique: true, nullable: true })
  ruleId: string;

  @Column()
  name: string;

  @Column({
    type: 'enum',
    enum: AlertConditionType,
  })
  conditionType: AlertConditionType;

  @Column({ type: 'decimal', precision: 30, scale: 0 })
  threshold: string;

  @Column()
  owner: string;

  @Column({ type: 'boolean', default: true })
  enabled: boolean;

  @Column({ type: 'bigint' })
  createdAt: string;

  @CreateDateColumn()
  indexedAt: Date;
}
