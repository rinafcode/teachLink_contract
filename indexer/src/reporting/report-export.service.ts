import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { DashboardSnapshot } from '@database/entities';
import { ReportType } from '@database/entities/dashboard-snapshot.entity';
import { DashboardService, DashboardAnalyticsDto } from './dashboard.service';

export type ExportFormat = 'json' | 'csv';

@Injectable()
export class ReportExportService {
  constructor(
    @InjectRepository(DashboardSnapshot)
    private snapshotRepo: Repository<DashboardSnapshot>,
    private dashboardService: DashboardService,
  ) {}

  /**
   * Export current dashboard analytics as JSON or CSV.
   */
  async exportCurrent(format: ExportFormat): Promise<string> {
    const data = await this.dashboardService.getCurrentAnalytics();
    return format === 'csv' ? this.toCsvSingle(data) : JSON.stringify(data, null, 2);
  }

  /**
   * Export snapshot history for a period as JSON or CSV.
   */
  async exportSnapshots(
    periodStart: string,
    periodEnd: string,
    format: ExportFormat,
    limit = 500,
  ): Promise<string> {
    const snapshots = await this.snapshotRepo.find({
      take: limit,
      order: { generatedAt: 'DESC' },
    });
    const filtered = snapshots.filter(
      (s) => s.periodStart >= periodStart && s.periodEnd <= periodEnd,
    );
    return format === 'csv' ? this.snapshotsToCsv(filtered) : JSON.stringify(filtered, null, 2);
  }

  private toCsvSingle(d: DashboardAnalyticsDto): string {
    const headers = [
      'bridgeHealthScore',
      'bridgeTotalVolume',
      'bridgeTotalTransactions',
      'bridgeSuccessRate',
      'escrowTotalCount',
      'escrowTotalVolume',
      'escrowDisputeCount',
      'escrowAvgResolutionTime',
      'totalRewardsIssued',
      'rewardClaimCount',
      'complianceReportCount',
      'auditRecordCount',
      'generatedAt',
    ];
    const row = headers.map((h) => (d as Record<string, unknown>)[h]);
    return [headers.join(','), row.join(',')].join('\n');
  }

  private snapshotsToCsv(snapshots: DashboardSnapshot[]): string {
    if (snapshots.length === 0) {
      return 'reportType,periodStart,periodEnd,generatedAt,bridgeHealthScore,bridgeTotalVolume,bridgeTotalTransactions\n';
    }
    const headers = [
      'reportType',
      'periodStart',
      'periodEnd',
      'generatedAt',
      'bridgeHealthScore',
      'bridgeTotalVolume',
      'bridgeTotalTransactions',
      'escrowTotalCount',
      'escrowTotalVolume',
      'escrowDisputeCount',
    ];
    const rows = snapshots.map((s) =>
      [
        s.reportType,
        s.periodStart,
        s.periodEnd,
        s.generatedAt,
        s.bridgeHealthScore,
        s.bridgeTotalVolume,
        s.bridgeTotalTransactions,
        s.escrowTotalCount,
        s.escrowTotalVolume,
        s.escrowDisputeCount,
      ].join(','),
    );
    return [headers.join(','), ...rows].join('\n');
  }
}
