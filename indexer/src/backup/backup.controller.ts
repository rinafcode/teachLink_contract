import { Controller, Get, Query, ParseIntPipe, DefaultValuePipe } from '@nestjs/common';
import { BackupService } from './backup.service';
import { RtoTier } from '@database/entities/backup-manifest.entity';

/**
 * API for backup and disaster recovery: audit trail, RTO reporting, compliance.
 */
@Controller('backup')
export class BackupController {
  constructor(private backupService: BackupService) {}

  @Get('manifests')
  async getManifests(
    @Query('limit', new DefaultValuePipe(100), ParseIntPipe) limit?: number,
    @Query('rtoTier') rtoTier?: RtoTier,
  ) {
    return this.backupService.getBackupManifests(limit, rtoTier);
  }

  @Get('recoveries')
  async getRecoveries(@Query('limit', new DefaultValuePipe(100), ParseIntPipe) limit?: number) {
    return this.backupService.getRecoveryRecords(limit);
  }

  @Get('rto-metrics')
  async getRtoMetrics() {
    return this.backupService.getRtoMetrics();
  }

  @Get('audit-trail')
  async getAuditTrail(
    @Query('since') since: string,
    @Query('limit', new DefaultValuePipe(200), ParseIntPipe) limit?: number,
  ) {
    return this.backupService.getBackupAuditTrail(since || '0', limit);
  }
}
