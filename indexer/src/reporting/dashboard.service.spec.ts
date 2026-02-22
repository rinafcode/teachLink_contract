import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import {
  BridgeTransaction,
  BridgeStatus,
  Escrow,
  EscrowStatus,
  Reward,
  RewardStatus,
  RewardPool,
  DashboardSnapshot,
} from '@database/entities';
import { ReportType } from '@database/entities/dashboard-snapshot.entity';
import { DashboardService } from './dashboard.service';

describe('DashboardService', () => {
  let service: DashboardService;
  let bridgeRepo: Repository<BridgeTransaction>;
  let escrowRepo: Repository<Escrow>;
  let rewardRepo: Repository<Reward>;
  let snapshotRepo: Repository<DashboardSnapshot>;

  const mockBridgeRepo = {
    count: jest.fn().mockResolvedValue(0),
    find: jest.fn().mockResolvedValue([]),
    createQueryBuilder: jest.fn().mockReturnValue({
      select: jest.fn().mockReturnThis(),
      getRawOne: jest.fn().mockResolvedValue({ sum: '0' }),
    }),
  };

  const mockEscrowRepo = {
    find: jest.fn().mockResolvedValue([]),
    count: jest.fn().mockResolvedValue(0),
  };

  const mockRewardRepo = {
    find: jest.fn().mockResolvedValue([]),
    count: jest.fn().mockResolvedValue(0),
  };

  const mockRewardPoolRepo = {
    find: jest.fn().mockResolvedValue([]),
  };

  const mockSnapshotRepo = {
    create: jest.fn((dto) => ({ ...dto, id: 'snap-1' })),
    save: jest.fn((entity) => Promise.resolve({ ...entity, id: 'snap-1' })),
    createQueryBuilder: jest.fn().mockReturnValue({
      where: jest.fn().mockReturnThis(),
      andWhere: jest.fn().mockReturnThis(),
      orderBy: jest.fn().mockReturnThis(),
      take: jest.fn().mockReturnThis(),
      getMany: jest.fn().mockResolvedValue([]),
    }),
  };

  beforeEach(async () => {
    jest.clearAllMocks();
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        DashboardService,
        { provide: getRepositoryToken(BridgeTransaction), useValue: mockBridgeRepo },
        { provide: getRepositoryToken(Escrow), useValue: mockEscrowRepo },
        { provide: getRepositoryToken(Reward), useValue: mockRewardRepo },
        { provide: getRepositoryToken(RewardPool), useValue: mockRewardPoolRepo },
        { provide: getRepositoryToken(DashboardSnapshot), useValue: mockSnapshotRepo },
      ],
    }).compile();

    service = module.get<DashboardService>(DashboardService);
    bridgeRepo = module.get(getRepositoryToken(BridgeTransaction));
    escrowRepo = module.get(getRepositoryToken(Escrow));
    rewardRepo = module.get(getRepositoryToken(Reward));
    snapshotRepo = module.get(getRepositoryToken(DashboardSnapshot));
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  describe('getCurrentAnalytics', () => {
    it('should return dashboard analytics with zeroed metrics when no data', async () => {
      const result = await service.getCurrentAnalytics();
      expect(result).toMatchObject({
        bridgeHealthScore: expect.any(Number),
        bridgeTotalVolume: '0',
        bridgeTotalTransactions: 0,
        escrowTotalCount: 0,
        escrowDisputeCount: 0,
        totalRewardsIssued: '0',
        rewardClaimCount: 0,
      });
      expect(Number(result.generatedAt)).toBeGreaterThan(0);
    });

    it('should include success rate and health score fields', async () => {
      const result = await service.getCurrentAnalytics();
      expect(typeof result.bridgeSuccessRate).toBe('number');
      expect(typeof result.bridgeHealthScore).toBe('number');
    });
  });

  describe('saveSnapshot', () => {
    it('should create and save a dashboard snapshot', async () => {
      const snapshot = await service.saveSnapshot(
        ReportType.BRIDGE_HEALTH,
        '1000',
        '2000',
        'owner-addr',
      );
      expect(mockSnapshotRepo.create).toHaveBeenCalled();
      expect(mockSnapshotRepo.save).toHaveBeenCalled();
      expect(snapshot).toHaveProperty('id', 'snap-1');
    });
  });

  describe('getSnapshots', () => {
    it('should return snapshots for period', async () => {
      const result = await service.getSnapshots('0', '9999', 50);
      expect(result).toEqual([]);
      expect(mockSnapshotRepo.createQueryBuilder).toHaveBeenCalled();
    });
  });
});
