import { Injectable, Logger } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import {
  BackupManifestRecord,
  BackupVerificationRecord,
  RecoveryRecordEntity,
} from '@database/entities';
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
    @InjectRepository(BackupVerificationRecord)
    private backupVerificationRepo: Repository<BackupVerificationRecord>,
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

  async getVerificationRecords(limit = 100, backupId?: string): Promise<BackupVerificationRecord[]> {
    const qb = this.backupVerificationRepo
      .createQueryBuilder('v')
      .orderBy('v.verifiedAt', 'DESC')
      .take(limit);
    if (backupId) qb.andWhere('v.backupId = :backupId', { backupId });
    return qb.getMany();
  }

  /** RTO metrics: average recovery duration and success rate */
  async getRtoMetrics(): Promise<{ avgDurationSecs: number; successCount: number; totalCount: number }> {
    const records = await this.recoveryRecordRepo.find({ take: 500 });
    const total = records.length;
    const successCount = records.filter((r: RecoveryRecordEntity) => r.success).length;
    const sumSecs = records.reduce(
      (acc: number, r: RecoveryRecordEntity) => acc + Number(r.recoveryDurationSecs),
      0,
    );
    return {
      avgDurationSecs: total > 0 ? Math.round(sumSecs / total) : 0,
      successCount,
      totalCount: total,
    };
  }

  /**
   * Integrity metrics for backup verification success monitoring.
   * Success rate is based on valid verifications in the selected time window.
   */
  async getIntegrityMetrics(windowHours = 24): Promise<{
    windowHours: number;
    totalBackups: number;
    totalVerifications: number;
    validVerifications: number;
    invalidVerifications: number;
    verificationSuccessRate: number;
    backupCoverageRate: number;
  }> {
    const nowSecs = Math.floor(Date.now() / 1000);
    const windowStart = String(nowSecs - (windowHours * 60 * 60));

    const backups = await this.backupManifestRepo
      .createQueryBuilder('b')
      .where('b.createdAt >= :windowStart', { windowStart })
      .getMany();

    const verifications = await this.backupVerificationRepo
      .createQueryBuilder('v')
      .where('v.verifiedAt >= :windowStart', { windowStart })
      .getMany();

    const totalBackups = backups.length;
    const totalVerifications = verifications.length;
    const validVerifications = verifications.filter((v: BackupVerificationRecord) => v.valid).length;
    const invalidVerifications = totalVerifications - validVerifications;

    const verifiedBackupIds = new Set(
      verifications.map((v: BackupVerificationRecord) => v.backupId),
    );
    const verifiedBackupsInWindow = backups.filter(
      (b: BackupManifestRecord) => verifiedBackupIds.has(b.backupId),
    ).length;

    const verificationSuccessRate = totalVerifications > 0
      ? Number(((validVerifications / totalVerifications) * 100).toFixed(2))
      : 0;
    const backupCoverageRate = totalBackups > 0
      ? Number(((verifiedBackupsInWindow / totalBackups) * 100).toFixed(2))
      : 0;

    return {
      windowHours,
      totalBackups,
      totalVerifications,
      validVerifications,
      invalidVerifications,
      verificationSuccessRate,
      backupCoverageRate,
    };
  }

  /** Compliance: backup and recovery audit trail for a period.
   *
   * Backups and recovery records are fetched in parallel since
   * both are independent read queries.
   */
  async getBackupAuditTrail(since: string, limit = 200): Promise<{
    backups: BackupManifestRecord[];
    recoveries: RecoveryRecordEntity[];
  }> {
    const [backups, recoveries] = await Promise.all([
      this.backupManifestRepo
        .createQueryBuilder('b')
        .where('b.createdAt >= :since', { since })
        .orderBy('b.createdAt', 'DESC')
        .take(limit)
        .getMany(),
      this.recoveryRecordRepo
        .createQueryBuilder('r')
        .where('r.executedAt >= :since', { since })
        .orderBy('r.executedAt', 'DESC')
        .take(limit)
        .getMany(),
    ]);
    return { backups, recoveries };
  }

  /** Automated backup check: run periodically; off-chain should call contract create_backup with integrity hash */
  @Cron(CronExpression.EVERY_HOUR)
  async runBackupCheck(): Promise<void> {
    const metrics = await this.getIntegrityMetrics(24);
    this.logger.log(
      `Backup check 24h: backups=${metrics.totalBackups} verifications=${metrics.totalVerifications} valid=${metrics.validVerifications} invalid=${metrics.invalidVerifications} success_rate=${metrics.verificationSuccessRate}% coverage=${metrics.backupCoverageRate}%`,
    );
  }
}
