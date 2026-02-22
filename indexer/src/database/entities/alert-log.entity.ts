import {
  Entity,
  Column,
  PrimaryGeneratedColumn,
  Index,
  CreateDateColumn,
} from 'typeorm';

/**
 * Log entry when an alert rule is triggered (real-time alerting).
 */
@Entity('alert_logs')
@Index(['ruleId'])
@Index(['triggeredAt'])
export class AlertLog {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'bigint' })
  ruleId: string;

  @Column()
  conditionType: string;

  @Column({ type: 'decimal', precision: 30, scale: 0 })
  currentValue: string;

  @Column({ type: 'decimal', precision: 30, scale: 0 })
  threshold: string;

  @Column({ type: 'bigint' })
  triggeredAt: string;

  @CreateDateColumn()
  indexedAt: Date;
}
