import {
  Entity,
  Column,
  PrimaryGeneratedColumn,
  Index,
  CreateDateColumn,
} from 'typeorm';

/**
 * Recovery execution record for RTO tracking and disaster recovery audit trail.
 */
@Entity('recovery_records')
@Index(['recoveryId'])
@Index(['backupId'])
@Index(['executedAt'])
export class RecoveryRecordEntity {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'bigint' })
  recoveryId: string;

  @Column({ type: 'bigint' })
  backupId: string;

  @Column({ type: 'bigint' })
  executedAt: string;

  @Column()
  executedBy: string;

  @Column({ type: 'bigint' })
  recoveryDurationSecs: string;

  @Column({ type: 'boolean' })
  success: boolean;

  @Column({ type: 'bigint' })
  ledger: string;

  @Column()
  txHash: string;

  @CreateDateColumn()
  indexedAt: Date;
}
