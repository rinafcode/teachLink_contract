import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { EventProcessorService } from './event-processor.service';
import {
  BridgeTransaction,
  BridgeStatus,
  Reward,
  RewardStatus,
  Escrow,
  EscrowStatus,
  ContentToken,
  ProvenanceRecord,
  CreditScore,
  CourseCompletion,
  Contribution,
  RewardPool,
} from '@database/entities';
import { ProcessedEvent } from '@horizon/horizon.service';

describe('EventProcessorService', () => {
  let service: EventProcessorService;
  let bridgeTransactionRepo: Repository<BridgeTransaction>;
  let rewardRepo: Repository<Reward>;
  let escrowRepo: Repository<Escrow>;
  let contentTokenRepo: Repository<ContentToken>;
  let provenanceRepo: Repository<ProvenanceRecord>;
  let creditScoreRepo: Repository<CreditScore>;
  let courseCompletionRepo: Repository<CourseCompletion>;
  let contributionRepo: Repository<Contribution>;
  let rewardPoolRepo: Repository<RewardPool>;

  const createMockRepository = () => ({
    create: jest.fn((entity) => entity),
    save: jest.fn((entity) => Promise.resolve(entity)),
    find: jest.fn(() => Promise.resolve([])),
    findOne: jest.fn(() => Promise.resolve(null)),
  });

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        EventProcessorService,
        {
          provide: getRepositoryToken(BridgeTransaction),
          useValue: createMockRepository(),
        },
        {
          provide: getRepositoryToken(Reward),
          useValue: createMockRepository(),
        },
        {
          provide: getRepositoryToken(Escrow),
          useValue: createMockRepository(),
        },
        {
          provide: getRepositoryToken(ContentToken),
          useValue: createMockRepository(),
        },
        {
          provide: getRepositoryToken(ProvenanceRecord),
          useValue: createMockRepository(),
        },
        {
          provide: getRepositoryToken(CreditScore),
          useValue: createMockRepository(),
        },
        {
          provide: getRepositoryToken(CourseCompletion),
          useValue: createMockRepository(),
        },
        {
          provide: getRepositoryToken(Contribution),
          useValue: createMockRepository(),
        },
        {
          provide: getRepositoryToken(RewardPool),
          useValue: createMockRepository(),
        },
      ],
    }).compile();

    service = module.get<EventProcessorService>(EventProcessorService);
    bridgeTransactionRepo = module.get(getRepositoryToken(BridgeTransaction));
    rewardRepo = module.get(getRepositoryToken(Reward));
    escrowRepo = module.get(getRepositoryToken(Escrow));
    contentTokenRepo = module.get(getRepositoryToken(ContentToken));
    provenanceRepo = module.get(getRepositoryToken(ProvenanceRecord));
    creditScoreRepo = module.get(getRepositoryToken(CreditScore));
    courseCompletionRepo = module.get(getRepositoryToken(CourseCompletion));
    contributionRepo = module.get(getRepositoryToken(Contribution));
    rewardPoolRepo = module.get(getRepositoryToken(RewardPool));
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  describe('processEvent', () => {
    it('should process DepositEvent correctly', async () => {
      const event: ProcessedEvent = {
        type: 'DepositEvent',
        data: {
          nonce: '1',
          from: 'GADDRESS',
          amount: '1000',
          destination_chain: 'ethereum',
          destination_address: '0x123',
        },
        ledger: '100',
        txHash: 'txhash123',
        timestamp: '1234567890',
        contractId: 'CONTRACT_ID',
      };

      await service.processEvent(event);

      expect(bridgeTransactionRepo.create).toHaveBeenCalledWith({
        nonce: '1',
        from: 'GADDRESS',
        amount: '1000',
        destinationChain: 'ethereum',
        destinationAddress: '0x123',
        status: BridgeStatus.INITIATED,
        ledger: '100',
        txHash: 'txhash123',
        timestamp: '1234567890',
      });
      expect(bridgeTransactionRepo.save).toHaveBeenCalled();
    });

    it('should process RewardIssuedEvent correctly', async () => {
      const event: ProcessedEvent = {
        type: 'RewardIssuedEvent',
        data: {
          recipient: 'GADDRESS',
          amount: '500',
          reward_type: 'course_completion',
          timestamp: '1234567890',
        },
        ledger: '101',
        txHash: 'txhash456',
        timestamp: '1234567890',
        contractId: 'CONTRACT_ID',
      };

      await service.processEvent(event);

      expect(rewardRepo.create).toHaveBeenCalledWith({
        recipient: 'GADDRESS',
        amount: '500',
        rewardType: 'course_completion',
        status: RewardStatus.ISSUED,
        timestamp: '1234567890',
        ledger: '101',
        txHash: 'txhash456',
      });
      expect(rewardRepo.save).toHaveBeenCalled();
    });

    it('should process EscrowCreatedEvent correctly', async () => {
      const event: ProcessedEvent = {
        type: 'EscrowCreatedEvent',
        data: {
          escrow: {
            id: '1',
            depositor: 'GDEPOSITOR',
            beneficiary: 'GBENEFICIARY',
            amount: '1000',
            required_signers: ['GSIGNER1', 'GSIGNER2'],
            required_approvals: 2,
          },
        },
        ledger: '102',
        txHash: 'txhash789',
        timestamp: '1234567890',
        contractId: 'CONTRACT_ID',
      };

      await service.processEvent(event);

      expect(escrowRepo.create).toHaveBeenCalledWith(
        expect.objectContaining({
          escrowId: '1',
          depositor: 'GDEPOSITOR',
          beneficiary: 'GBENEFICIARY',
          amount: '1000',
          requiredSigners: ['GSIGNER1', 'GSIGNER2'],
          requiredApprovals: 2,
          status: EscrowStatus.ACTIVE,
        }),
      );
      expect(escrowRepo.save).toHaveBeenCalled();
    });

    it('should process ContentMintedEvent correctly', async () => {
      const event: ProcessedEvent = {
        type: 'ContentMintedEvent',
        data: {
          token_id: '1',
          creator: 'GCREATOR',
          metadata: {
            content_hash: 'hash123',
            metadata_uri: 'ipfs://metadata',
            transferable: true,
            royalty_percentage: 10,
          },
        },
        ledger: '103',
        txHash: 'txhash101112',
        timestamp: '1234567890',
        contractId: 'CONTRACT_ID',
      };

      await service.processEvent(event);

      expect(contentTokenRepo.create).toHaveBeenCalledWith(
        expect.objectContaining({
          tokenId: '1',
          creator: 'GCREATOR',
          currentOwner: 'GCREATOR',
          contentHash: 'hash123',
          metadataUri: 'ipfs://metadata',
          transferable: true,
          royaltyPercentage: 10,
        }),
      );
      expect(contentTokenRepo.save).toHaveBeenCalled();
      expect(provenanceRepo.save).toHaveBeenCalled();
    });

    it('should process CreditScoreUpdatedEvent correctly', async () => {
      const event: ProcessedEvent = {
        type: 'CreditScoreUpdatedEvent',
        data: {
          user: 'GUSER',
          new_score: '850',
        },
        ledger: '104',
        txHash: 'txhash131415',
        timestamp: '1234567890',
        contractId: 'CONTRACT_ID',
      };

      await service.processEvent(event);

      expect(creditScoreRepo.findOne).toHaveBeenCalledWith({
        where: { userAddress: 'GUSER' },
      });
      expect(creditScoreRepo.create).toHaveBeenCalled();
      expect(creditScoreRepo.save).toHaveBeenCalled();
    });

    it('should process CourseCompletedEvent correctly', async () => {
      const event: ProcessedEvent = {
        type: 'CourseCompletedEvent',
        data: {
          user: 'GUSER',
          course_id: '42',
          points: '100',
        },
        ledger: '105',
        txHash: 'txhash161718',
        timestamp: '1234567890',
        contractId: 'CONTRACT_ID',
      };

      await service.processEvent(event);

      expect(courseCompletionRepo.create).toHaveBeenCalledWith(
        expect.objectContaining({
          userAddress: 'GUSER',
          courseId: '42',
          pointsEarned: '100',
        }),
      );
      expect(courseCompletionRepo.save).toHaveBeenCalled();
    });

    it('should handle unknown event types gracefully', async () => {
      const event: ProcessedEvent = {
        type: 'UnknownEvent',
        data: {},
        ledger: '106',
        txHash: 'txhash192021',
        timestamp: '1234567890',
        contractId: 'CONTRACT_ID',
      };

      await expect(service.processEvent(event)).resolves.not.toThrow();
    });

    it('should propagate errors from repository operations', async () => {
      const event: ProcessedEvent = {
        type: 'DepositEvent',
        data: {
          nonce: '1',
          from: 'GADDRESS',
          amount: '1000',
          destination_chain: 'ethereum',
          destination_address: '0x123',
        },
        ledger: '100',
        txHash: 'txhash123',
        timestamp: '1234567890',
        contractId: 'CONTRACT_ID',
      };

      jest.spyOn(bridgeTransactionRepo, 'save').mockRejectedValue(new Error('Database error'));

      await expect(service.processEvent(event)).rejects.toThrow('Database error');
    });
  });
});
