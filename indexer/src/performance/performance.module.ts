import { Module } from '@nestjs/common';
import { APP_INTERCEPTOR } from '@nestjs/core';
import { TypeOrmModule } from '@nestjs/typeorm';
import { IndexerState } from '@database/entities';
import { MetricsService } from './metrics.service';
import { HealthService } from './health.service';
import { MetricsInterceptor } from './metrics.interceptor';
import { PerformanceController } from './performance.controller';

@Module({
  imports: [TypeOrmModule.forFeature([IndexerState])],
  controllers: [PerformanceController],
  providers: [
    MetricsService,
    HealthService,
    {
      provide: APP_INTERCEPTOR,
      useClass: MetricsInterceptor,
    },
  ],
  exports: [MetricsService, HealthService],
})
export class PerformanceModule {}
