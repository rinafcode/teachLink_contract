import { Test, TestingModule } from '@nestjs/testing';
import { ConfigService } from '@nestjs/config';
import { HorizonService } from './horizon.service';

describe('HorizonService', () => {
  let service: HorizonService;
  let configService: ConfigService;

  const mockConfigService = {
    get: jest.fn((key: string) => {
      const config = {
        'stellar.horizonUrl': 'https://horizon-testnet.stellar.org',
        'stellar.network': 'testnet',
        'contract.teachlinkContractId': 'CDUMMYCONTRACTID',
      };
      return config[key];
    }),
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        HorizonService,
        {
          provide: ConfigService,
          useValue: mockConfigService,
        },
      ],
    }).compile();

    service = module.get<HorizonService>(HorizonService);
    configService = module.get<ConfigService>(ConfigService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  it('should initialize with correct network configuration', async () => {
    await service.onModuleInit();
    expect(configService.get).toHaveBeenCalledWith('stellar.horizonUrl');
    expect(configService.get).toHaveBeenCalledWith('stellar.network');
    expect(configService.get).toHaveBeenCalledWith('contract.teachlinkContractId');
  });

  describe('getLatestLedger', () => {
    it('should return the latest ledger number', async () => {
      // This would need proper mocking of Stellar SDK
      // For now, this is a placeholder test structure
      expect(service.getLatestLedger).toBeDefined();
    });
  });

  describe('streamContractEvents', () => {
    it('should set up event streaming', async () => {
      const mockOnEvent = jest.fn();
      const mockOnError = jest.fn();

      // This would need proper mocking of Stellar SDK streaming
      expect(service.streamContractEvents).toBeDefined();
    });
  });

  describe('fetchOperationsInRange', () => {
    it('should fetch operations for a ledger range', async () => {
      const startLedger = 100;
      const endLedger = 105;

      // This would need proper mocking of Stellar SDK
      expect(service.fetchOperationsInRange).toBeDefined();
    });
  });
});
