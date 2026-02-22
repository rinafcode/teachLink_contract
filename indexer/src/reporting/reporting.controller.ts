import {
  Controller,
  Get,
  Post,
  Query,
  Param,
  Body,
  ParseIntPipe,
  DefaultValuePipe,
} from '@nestjs/common';
import { DashboardService } from './dashboard.service';
import { ReportExportService, ExportFormat } from './report-export.service';
import { AlertService } from './alert.service';
import { ReportType } from '@database/entities/dashboard-snapshot.entity';

/**
 * API for advanced analytics and reporting dashboard:
 * - Interactive data visualization (dashboard metrics)
 * - Customizable reports (snapshots by type/period)
 * - Data export and sharing (JSON/CSV)
 * - Real-time reporting and alerting (evaluate alerts, get alert logs)
 * - Report usage and compliance (audit trail via snapshots)
 */
@Controller('analytics')
export class ReportingController {
  constructor(
    private dashboardService: DashboardService,
    private reportExportService: ReportExportService,
    private alertService: AlertService,
  ) {}

  /** Current aggregate metrics for dashboard visualization */
  @Get('dashboard')
  async getDashboard() {
    return this.dashboardService.getCurrentAnalytics();
  }

  /** Generate and persist a report snapshot (manual trigger) */
  @Post('reports/snapshots')
  async generateSnapshot(
    @Body('reportType') reportType: ReportType = ReportType.BRIDGE_HEALTH,
    @Body('periodStart') periodStart: string,
    @Body('periodEnd') periodEnd: string,
    @Body('generatedBy') generatedBy?: string,
  ) {
    return this.dashboardService.saveSnapshot(
      reportType,
      periodStart,
      periodEnd,
      generatedBy,
    );
  }

  /** List report snapshots for time range */
  @Get('reports/snapshots')
  async getSnapshots(
    @Query('periodStart') periodStart?: string,
    @Query('periodEnd') periodEnd?: string,
    @Query('limit', new DefaultValuePipe(100), ParseIntPipe) limit?: number,
  ) {
    return this.dashboardService.getSnapshots(
      periodStart ?? '0',
      periodEnd ?? String(Math.floor(Date.now() / 1000)),
      limit,
    );
  }

  /** Export current analytics or snapshot history (JSON or CSV) */
  @Get('reports/export')
  async export(
    @Query('format') format: ExportFormat = 'json',
    @Query('periodStart') periodStart?: string,
    @Query('periodEnd') periodEnd?: string,
    @Query('limit', new DefaultValuePipe(500), ParseIntPipe) limit?: number,
  ) {
    const periodStartStr = periodStart ?? '0';
    const periodEndStr = periodEnd ?? String(Math.floor(Date.now() / 1000));
    if (periodStart && periodEnd) {
      const data = await this.reportExportService.exportSnapshots(
        periodStartStr,
        periodEndStr,
        format,
        limit,
      );
      return format === 'json' ? JSON.parse(data) : { data, contentType: 'text/csv' };
    }
    const data = await this.reportExportService.exportCurrent(format);
    return format === 'json' ? JSON.parse(data) : { data, contentType: 'text/csv' };
  }

  /** Evaluate alert rules now (real-time alerting) */
  @Post('alerts/evaluate')
  async evaluateAlerts() {
    const triggered = await this.alertService.evaluateAlerts();
    return { triggered };
  }

  /** Get alert log entries */
  @Get('alerts/logs')
  async getAlertLogs(
    @Query('ruleId') ruleId?: string,
    @Query('since') since?: string,
  ) {
    return this.alertService.getAlertLogs(ruleId, since);
  }

  /** Compliance: list report snapshots as audit trail (by period) */
  @Get('compliance/audit-trail')
  async getComplianceAuditTrail(
    @Query('periodStart') periodStart: string,
    @Query('periodEnd') periodEnd: string,
    @Query('limit', new DefaultValuePipe(100), ParseIntPipe) limit?: number,
  ) {
    return this.dashboardService.getSnapshots(periodStart, periodEnd, limit);
  }
}
