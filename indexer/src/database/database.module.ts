import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { ConfigModule, ConfigService } from '@nestjs/config';
import {
  BridgeTransaction,
  Reward,
  Escrow,
  ContentToken,
  ProvenanceRecord,
  CreditScore,
  CourseCompletion,
  Contribution,
  RewardPool,
  IndexerState,
} from './entities';

@Module({
  imports: [
    TypeOrmModule.forRootAsync({
      imports: [ConfigModule],
      useFactory: (configService: ConfigService) => ({
        type: 'postgres' as const,
        host: configService.get<string>('database.host'),
        port: configService.get<number>('database.port'),
        username: configService.get<string>('database.username'),
        password: configService.get<string>('database.password'),
        database: configService.get<string>('database.database'),
        entities: [
          BridgeTransaction,
          Reward,
          Escrow,
          ContentToken,
          ProvenanceRecord,
          CreditScore,
          CourseCompletion,
          Contribution,
          RewardPool,
          IndexerState,
        ],
        synchronize: configService.get<boolean>('database.synchronize'),
        logging: configService.get<boolean>('database.logging'),
      }),
      inject: [ConfigService],
    }),
    TypeOrmModule.forFeature([
      BridgeTransaction,
      Reward,
      Escrow,
      ContentToken,
      ProvenanceRecord,
      CreditScore,
      CourseCompletion,
      Contribution,
      RewardPool,
      IndexerState,
    ]),
  ],
  exports: [TypeOrmModule],
})
export class DatabaseModule {}
