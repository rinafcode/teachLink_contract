import { Injectable, Logger } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { DashboardService } from './dashboard.service';
import { ReportType } from '@database/entities/dashboard-snapshot.entity';

/**
 * Scheduled report generation. Runs periodically and persists dashboard snapshots
 * for report history and automated reporting.
 */
@Injectable()
export class ReportSchedulerService {
  private readonly logger = new Logger(ReportSchedulerService.name);

  constructor(private dashboardService: DashboardService) {}

  @Cron(CronExpression.EVERY_HOUR)
  async handleHourlyReport(): Promise<void> {
    try {
      const now = Math.floor(Date.now() / 1000);
      const periodStart = (now - 3600).toString();
      const periodEnd = now.toString();
      await this.dashboardService.saveSnapshot(
        ReportType.BRIDGE_HEALTH,
        periodStart,
        periodEnd,
        'scheduler',
      );
      this.logger.log(`Hourly report snapshot saved for period ${periodStart}-${periodEnd}`);
    } catch (err) {
      this.logger.error('Hourly report generation failed', err);
    }
  }

  @Cron(CronExpression.EVERY_DAY_AT_MIDNIGHT)
  async handleDailyReport(): Promise<void> {
    try {
      const now = Math.floor(Date.now() / 1000);
      const periodStart = (now - 86400).toString();
      const periodEnd = now.toString();
      await this.dashboardService.saveSnapshot(
        ReportType.COMPLIANCE_AUDIT,
        periodStart,
        periodEnd,
        'scheduler',
      );
      this.logger.log(`Daily compliance report snapshot saved for period ${periodStart}-${periodEnd}`);
    } catch (err) {
      this.logger.error('Daily report generation failed', err);
    }
  }
}
