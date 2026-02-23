import {
  Entity,
  Column,
  PrimaryGeneratedColumn,
  Index,
  CreateDateColumn,
} from 'typeorm';

export enum RtoTier {
  CRITICAL = 'critical',
  HIGH = 'high',
  STANDARD = 'standard',
}

/**
 * Indexed backup manifest for disaster recovery audit and monitoring.
 */
@Entity('backup_manifests')
@Index(['backupId'])
@Index(['createdAt'])
@Index(['createdBy'])
export class BackupManifestRecord {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'bigint' })
  backupId: string;

  @Column({ type: 'bigint' })
  createdAt: string;

  @Column()
  createdBy: string;

  @Column({ type: 'text' })
  integrityHash: string;

  @Column({ type: 'enum', enum: RtoTier })
  rtoTier: RtoTier;

  @Column({ type: 'bigint', default: 0 })
  encryptionRef: string;

  @Column({ type: 'bigint' })
  ledger: string;

  @Column()
  txHash: string;

  @CreateDateColumn()
  indexedAt: Date;
}
