import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import {
  BridgeTransaction,
  Escrow,
  Reward,
  RewardPool,
  DashboardSnapshot,
  AlertRule,
  AlertLog,
} from '@database/entities';
import { PerformanceModule } from '../performance/performance.module';
import { AppConfigModule } from '../config/config.module';
import { ConfigManager } from '../config/config.manager';
import { DashboardService } from './dashboard.service';
import { ReportExportService } from './report-export.service';
import { ReportSchedulerService } from './report-scheduler.service';
import { AlertService } from './alert.service';
import { ReportingController } from './reporting.controller';

@Module({
  imports: [
    PerformanceModule,
    AppConfigModule,
    TypeOrmModule.forFeature([
      BridgeTransaction,
      Escrow,
      Reward,
      RewardPool,
      DashboardSnapshot,
      AlertRule,
      AlertLog,
    ]),
  ],
  controllers: [ReportingController],
  providers: [DashboardService, ReportExportService, ReportSchedulerService, AlertService, ConfigManager],
  exports: [DashboardService, ReportExportService, AlertService],
})
export class ReportingModule {}
