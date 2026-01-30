import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { ScheduleModule } from '@nestjs/schedule';
import configuration from './config/configuration';
import { DatabaseModule } from '@database/database.module';
import { HorizonModule } from '@horizon/horizon.module';
import { EventsModule } from '@events/events.module';
import { IndexerModule } from '@indexer/indexer.module';

@Module({
  imports: [
    ConfigModule.forRoot({
      isGlobal: true,
      load: [configuration],
    }),
    ScheduleModule.forRoot(),
    DatabaseModule,
    HorizonModule,
    EventsModule,
    IndexerModule,
  ],
})
export class AppModule {}
