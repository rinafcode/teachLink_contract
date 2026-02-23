import { Module } from '@nestjs/common';
import { MetricsService } from './metrics.service';
import { PerformanceController } from './performance.controller';

@Module({
  controllers: [PerformanceController],
  providers: [MetricsService],
  exports: [MetricsService],
})
export class PerformanceModule {}
