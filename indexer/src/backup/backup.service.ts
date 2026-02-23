import { Injectable, Logger } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { BackupManifestRecord, RecoveryRecordEntity } from '@database/entities';
import { RtoTier } from '@database/entities/backup-manifest.entity';

/**
 * Backup and disaster recovery: audit trail, RTO reporting, and integrity monitoring.
 */
@Injectable()
export class BackupService {
  private readonly logger = new Logger(BackupService.name);

  constructor(
    @InjectRepository(BackupManifestRecord)
    private backupManifestRepo: Repository<BackupManifestRecord>,
    @InjectRepository(RecoveryRecordEntity)
    private recoveryRecordRepo: Repository<RecoveryRecordEntity>,
  ) {}

  async getBackupManifests(limit = 100, rtoTier?: RtoTier): Promise<BackupManifestRecord[]> {
    const qb = this.backupManifestRepo
      .createQueryBuilder('b')
      .orderBy('b.createdAt', 'DESC')
      .take(limit);
    if (rtoTier) qb.andWhere('b.rtoTier = :rtoTier', { rtoTier });
    return qb.getMany();
  }

  async getRecoveryRecords(limit = 100): Promise<RecoveryRecordEntity[]> {
    return this.recoveryRecordRepo.find({
      take: limit,
      order: { executedAt: 'DESC' },
    });
  }

  /** RTO metrics: average recovery duration and success rate */
  async getRtoMetrics(): Promise<{ avgDurationSecs: number; successCount: number; totalCount: number }> {
    const records = await this.recoveryRecordRepo.find({ take: 500 });
    const total = records.length;
    const successCount = records.filter((r) => r.success).length;
    const sumSecs = records.reduce((acc, r) => acc + Number(r.recoveryDurationSecs), 0);
    return {
      avgDurationSecs: total > 0 ? Math.round(sumSecs / total) : 0,
      successCount,
      totalCount: total,
    };
  }

  /** Compliance: backup and recovery audit trail for a period */
  async getBackupAuditTrail(since: string, limit = 200): Promise<{
    backups: BackupManifestRecord[];
    recoveries: RecoveryRecordEntity[];
  }> {
    const backups = await this.backupManifestRepo
      .createQueryBuilder('b')
      .where('b.createdAt >= :since', { since })
      .orderBy('b.createdAt', 'DESC')
      .take(limit)
      .getMany();
    const recoveries = await this.recoveryRecordRepo
      .createQueryBuilder('r')
      .where('r.executedAt >= :since', { since })
      .orderBy('r.executedAt', 'DESC')
      .take(limit)
      .getMany();
    return { backups, recoveries };
  }

  /** Automated backup check: run periodically; off-chain should call contract create_backup with integrity hash */
  @Cron(CronExpression.EVERY_HOUR)
  async runBackupCheck(): Promise<void> {
    this.logger.log('Backup check: consider triggering create_backup for any scheduled backups (off-chain).');
  }
}
