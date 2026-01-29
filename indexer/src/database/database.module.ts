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
        host: configService.get('database.host'),
        port: configService.get('database.port'),
        username: configService.get('database.username'),
        password: configService.get('database.password'),
        database: configService.get('database.database'),
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
        synchronize: configService.get('database.synchronize'),
        logging: configService.get('database.logging'),
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
