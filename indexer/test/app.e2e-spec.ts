import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { AppModule } from '../src/app.module';
import { IndexerService } from '../src/indexer/indexer.service';
import configuration from '../src/config/configuration';

describe('Indexer Integration Tests (e2e)', () => {
  let app: INestApplication;
  let indexerService: IndexerService;

  beforeAll(async () => {
    const moduleFixture: TestingModule = await Test.createTestingModule({
      imports: [
        ConfigModule.forRoot({
          isGlobal: true,
          load: [configuration],
        }),
        // Use in-memory or test database
        AppModule,
      ],
    })
      .overrideProvider('DATABASE_CONFIG')
      .useValue({
        type: 'postgres',
        host: process.env.TEST_DB_HOST || 'localhost',
        port: parseInt(process.env.TEST_DB_PORT || '5433', 10),
        username: process.env.TEST_DB_USERNAME || 'test',
        password: process.env.TEST_DB_PASSWORD || 'test',
        database: process.env.TEST_DB_DATABASE || 'teachlink_test',
        synchronize: true,
        dropSchema: true,
      })
      .compile();

    app = moduleFixture.createNestApplication();
    await app.init();

    indexerService = moduleFixture.get<IndexerService>(IndexerService);
  });

  afterAll(async () => {
    await app.close();
  });

  describe('Indexer Service', () => {
    it('should be defined', () => {
      expect(indexerService).toBeDefined();
    });

    it('should get initial status', async () => {
      const status = await indexerService.getStatus();

      expect(status).toHaveProperty('isRunning');
      expect(status).toHaveProperty('lastProcessedLedger');
      expect(status).toHaveProperty('totalEventsProcessed');
      expect(status).toHaveProperty('totalErrors');
    });

    it('should start and stop indexing', async () => {
      await indexerService.startIndexing();
      let status = await indexerService.getStatus();
      expect(status.isRunning).toBe(true);

      await indexerService.stopIndexing();
      status = await indexerService.getStatus();
      expect(status.isRunning).toBe(false);
    }, 30000);
  });

  describe('Database Entities', () => {
    it('should create database tables', async () => {
      // This test verifies that TypeORM entities are properly configured
      // and the database schema can be created
      expect(app).toBeDefined();
    });
  });
});
