import {
  Entity,
  Column,
  PrimaryGeneratedColumn,
  Index,
  CreateDateColumn,
} from 'typeorm';

/**
 * Backup verification record for integrity monitoring and compliance reporting.
 */
@Entity('backup_verifications')
@Index(['backupId'])
@Index(['verifiedAt'])
@Index(['valid'])
export class BackupVerificationRecord {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'bigint' })
  backupId: string;

  @Column({ type: 'bigint' })
  verifiedAt: string;

  @Column()
  verifiedBy: string;

  @Column({ type: 'boolean' })
  valid: boolean;

  @Column({ type: 'bigint' })
  ledger: string;

  @Column()
  txHash: string;

  @CreateDateColumn()
  indexedAt: Date;
}
