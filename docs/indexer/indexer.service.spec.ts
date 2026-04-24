import { Test, TestingModule } from '@nestjs/testing';
import { ConfigService } from '@nestjs/config';
import { getRepositoryToken } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { IndexerService } from './indexer.service';
import { HorizonService } from '@horizon/horizon.service';
import { EventProcessorService } from '@events/event-processor.service';
import { IndexerState } from '@database/entities';

describe('IndexerService', () => {
  let service: IndexerService;
  let horizonService: HorizonService;
  let eventProcessor: EventProcessorService;
  let indexerStateRepo: Repository<IndexerState>;
  let configService: ConfigService;

  const mockConfigService = {
    get: jest.fn((key: string) => {
      const config: Record<string, any> = {
        'indexer.startLedger': 'latest',
        'indexer.pollInterval': 5000,
        'indexer.batchSize': 100,
      };
      return config[key];
    }),
  };

  const mockHorizonService = {
    getLatestLedger: jest.fn(),
    streamContractEvents: jest.fn(),
    fetchOperationsInRange: jest.fn(),
  };

  const mockEventProcessor = {
    processEvent: jest.fn(() => Promise.resolve()),
  };

  const mockIndexerStateRepo = {
    findOne: jest.fn(),
    create: jest.fn((entity) => entity),
    save: jest.fn((entity) => Promise.resolve(entity)),
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        IndexerService,
        {
          provide: ConfigService,
          useValue: mockConfigService,
        },
        {
          provide: HorizonService,
          useValue: mockHorizonService,
        },
        {
          provide: EventProcessorService,
          useValue: mockEventProcessor,
        },
        {
          provide: getRepositoryToken(IndexerState),
          useValue: mockIndexerStateRepo,
        },
      ],
    }).compile();

    service = module.get<IndexerService>(IndexerService);
    horizonService = module.get<HorizonService>(HorizonService);
    eventProcessor = module.get<EventProcessorService>(EventProcessorService);
    indexerStateRepo = module.get(getRepositoryToken(IndexerState));
    configService = module.get<ConfigService>(ConfigService);

    jest.clearAllMocks();

    // Set default return values
    mockHorizonService.getLatestLedger.mockResolvedValue(1000);
    mockHorizonService.streamContractEvents.mockResolvedValue(jest.fn());
    mockHorizonService.fetchOperationsInRange.mockResolvedValue([]);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  describe('startIndexing', () => {
    it('should create new state on first run', async () => {
      mockIndexerStateRepo.findOne.mockResolvedValue(null);

      await service.startIndexing();

      expect(mockIndexerStateRepo.findOne).toHaveBeenCalled();
      expect(horizonService.getLatestLedger).toHaveBeenCalled();
      expect(mockIndexerStateRepo.create).toHaveBeenCalledWith(
        expect.objectContaining({
          key: 'main_indexer',
          lastProcessedLedger: '1000',
          totalEventsProcessed: 0,
          totalErrors: 0,
        }),
      );
      expect(mockIndexerStateRepo.save).toHaveBeenCalled();
      expect(horizonService.streamContractEvents).toHaveBeenCalled();
    });

    it('should resume from existing state', async () => {
      const existingState = {
        key: 'main_indexer',
        lastProcessedLedger: '500',
        totalEventsProcessed: 100,
        totalErrors: 0,
      };

      mockIndexerStateRepo.findOne.mockResolvedValue(existingState);

      await service.startIndexing();

      expect(mockIndexerStateRepo.findOne).toHaveBeenCalled();
      expect(horizonService.streamContractEvents).toHaveBeenCalledWith(
        '500',
        expect.any(Function),
        expect.any(Function),
      );
    });

    it('should not start if already running', async () => {
      mockIndexerStateRepo.findOne.mockResolvedValue({
        key: 'main_indexer',
        lastProcessedLedger: '500',
      });

      await service.startIndexing();
      await service.startIndexing(); // Second call

      // streamContractEvents should only be called once
      expect(horizonService.streamContractEvents).toHaveBeenCalledTimes(1);
    });
  });

  describe('stopIndexing', () => {
    it('should stop the indexer gracefully', async () => {
      const mockCloseHandler = jest.fn();
      mockHorizonService.streamContractEvents.mockResolvedValue(mockCloseHandler);
      mockIndexerStateRepo.findOne.mockResolvedValue(null);

      await service.startIndexing();
      await service.stopIndexing();

      expect(mockCloseHandler).toHaveBeenCalled();
    });
  });

  describe('getStatus', () => {
    it('should return current indexer status', async () => {
      const mockState = {
        key: 'main_indexer',
        lastProcessedLedger: '1000',
        totalEventsProcessed: 250,
        totalErrors: 5,
        lastProcessedTimestamp: '1234567890',
      };

      mockIndexerStateRepo.findOne.mockResolvedValue(mockState);

      const status = await service.getStatus();

      expect(status).toEqual({
        isRunning: false,
        lastProcessedLedger: '1000',
        totalEventsProcessed: 250,
        totalErrors: 5,
        lastProcessedTimestamp: '1234567890',
      });
    });

    it('should return default status when no state exists', async () => {
      mockIndexerStateRepo.findOne.mockResolvedValue(null);

      const status = await service.getStatus();

      expect(status).toEqual({
        isRunning: false,
        lastProcessedLedger: '0',
        totalEventsProcessed: 0,
        totalErrors: 0,
        lastProcessedTimestamp: '0',
      });
    });
  });

  describe('backfillHistoricalData', () => {
    it('should backfill data for a given ledger range', async () => {
      const mockEvents = [
        {
          type: 'DepositEvent',
          data: { nonce: '1', from: 'GADDRESS', amount: '1000' },
          ledger: '100',
          txHash: 'hash1',
          timestamp: '1234567890',
          contractId: 'CONTRACT_ID',
        },
        {
          type: 'RewardIssuedEvent',
          data: { recipient: 'GADDRESS', amount: '500', reward_type: 'test' },
          ledger: '101',
          txHash: 'hash2',
          timestamp: '1234567891',
          contractId: 'CONTRACT_ID',
        },
      ];

      mockHorizonService.fetchOperationsInRange.mockResolvedValue(mockEvents);
      mockIndexerStateRepo.findOne.mockResolvedValue({
        key: 'main_indexer',
        lastProcessedLedger: '99',
        totalEventsProcessed: 0,
        totalErrors: 0,
      });

      await service.backfillHistoricalData(100, 105);

      expect(horizonService.fetchOperationsInRange).toHaveBeenCalledWith(100, 105);
      expect(eventProcessor.processEvent).toHaveBeenCalledTimes(2);
    });

    it('should handle errors during backfill', async () => {
      mockHorizonService.fetchOperationsInRange.mockRejectedValue(
        new Error('Backfill error'),
      );

      await expect(service.backfillHistoricalData(100, 105)).rejects.toThrow('Backfill error');
    });
  });
});
