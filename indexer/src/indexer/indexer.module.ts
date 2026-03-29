import { Module } from '@nestjs/common';
import { IndexerService } from './indexer.service';
import { HorizonModule } from '@horizon/horizon.module';
import { EventsModule } from '@events/events.module';
import { DatabaseModule } from '@database/database.module';
import { PerformanceModule } from '../performance/performance.module';

@Module({
  imports: [HorizonModule, EventsModule, DatabaseModule, PerformanceModule],
  providers: [IndexerService],
  exports: [IndexerService],
})
export class IndexerModule {}
