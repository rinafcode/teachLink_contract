import { Module } from '@nestjs/common';
import { EventProcessorService } from './event-processor.service';
import { DatabaseModule } from '@database/database.module';

@Module({
  imports: [DatabaseModule],
  providers: [EventProcessorService],
  exports: [EventProcessorService],
})
export class EventsModule {}
