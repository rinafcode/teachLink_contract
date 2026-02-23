import { Module } from '@nestjs/common';
import { CacheModule } from '@nestjs/cache-manager';
import { ConfigModule } from '@nestjs/config';
import { ScheduleModule } from '@nestjs/schedule';
import configuration from './config/configuration';
import { DatabaseModule } from '@database/database.module';
import { HorizonModule } from '@horizon/horizon.module';
import { EventsModule } from '@events/events.module';
import { IndexerModule } from '@indexer/indexer.module';
import { ReportingModule } from './reporting/reporting.module';
import { BackupModule } from './backup/backup.module';
import { PerformanceModule } from './performance/performance.module';

@Module({
  imports: [
    ConfigModule.forRoot({
      isGlobal: true,
      load: [configuration],
    }),
    CacheModule.register({
      ttl: 60 * 1000, // 60s for dashboard/analytics cache
      max: 500,
      isGlobal: true,
    }),
    ScheduleModule.forRoot(),
    DatabaseModule,
    HorizonModule,
    EventsModule,
    IndexerModule,
    ReportingModule,
    BackupModule,
    PerformanceModule,
  ],
})
export class AppModule {}
