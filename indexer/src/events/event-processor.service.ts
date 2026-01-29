import { Injectable, Logger } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import {
  BridgeTransaction,
  BridgeStatus,
  Reward,
  RewardStatus,
  Escrow,
  EscrowStatus,
  ContentToken,
  ProvenanceRecord,
  ProvenanceEventType,
  CreditScore,
  CourseCompletion,
  Contribution,
  RewardPool,
} from '@database/entities';
import { ProcessedEvent } from '@horizon/horizon.service';
import {
  BridgeEvent,
  RewardEvent,
  EscrowEvent,
  TokenizationEvent,
  ScoringEvent,
} from './event-types';

@Injectable()
export class EventProcessorService {
  private readonly logger = new Logger(EventProcessorService.name);

  constructor(
    @InjectRepository(BridgeTransaction)
    private bridgeTransactionRepo: Repository<BridgeTransaction>,
    @InjectRepository(Reward)
    private rewardRepo: Repository<Reward>,
    @InjectRepository(Escrow)
    private escrowRepo: Repository<Escrow>,
    @InjectRepository(ContentToken)
    private contentTokenRepo: Repository<ContentToken>,
    @InjectRepository(ProvenanceRecord)
    private provenanceRepo: Repository<ProvenanceRecord>,
    @InjectRepository(CreditScore)
    private creditScoreRepo: Repository<CreditScore>,
    @InjectRepository(CourseCompletion)
    private courseCompletionRepo: Repository<CourseCompletion>,
    @InjectRepository(Contribution)
    private contributionRepo: Repository<Contribution>,
    @InjectRepository(RewardPool)
    private rewardPoolRepo: Repository<RewardPool>,
  ) {}

  async processEvent(event: ProcessedEvent): Promise<void> {
    try {
      const eventType = event.type;
      this.logger.debug(`Processing event type: ${eventType}`);

      switch (eventType) {
        // Bridge Events
        case 'DepositEvent':
          await this.handleDepositEvent(event);
          break;
        case 'ReleaseEvent':
          await this.handleReleaseEvent(event);
          break;
        case 'BridgeInitiatedEvent':
          await this.handleBridgeInitiatedEvent(event);
          break;
        case 'BridgeCompletedEvent':
          await this.handleBridgeCompletedEvent(event);
          break;

        // Reward Events
        case 'RewardIssuedEvent':
          await this.handleRewardIssuedEvent(event);
          break;
        case 'RewardClaimedEvent':
          await this.handleRewardClaimedEvent(event);
          break;
        case 'RewardPoolFundedEvent':
          await this.handleRewardPoolFundedEvent(event);
          break;

        // Escrow Events
        case 'EscrowCreatedEvent':
          await this.handleEscrowCreatedEvent(event);
          break;
        case 'EscrowApprovedEvent':
          await this.handleEscrowApprovedEvent(event);
          break;
        case 'EscrowReleasedEvent':
          await this.handleEscrowReleasedEvent(event);
          break;
        case 'EscrowRefundedEvent':
          await this.handleEscrowRefundedEvent(event);
          break;
        case 'EscrowDisputedEvent':
          await this.handleEscrowDisputedEvent(event);
          break;
        case 'EscrowResolvedEvent':
          await this.handleEscrowResolvedEvent(event);
          break;

        // Tokenization Events
        case 'ContentMintedEvent':
          await this.handleContentMintedEvent(event);
          break;
        case 'OwnershipTransferredEvent':
          await this.handleOwnershipTransferredEvent(event);
          break;
        case 'ProvenanceRecordedEvent':
          await this.handleProvenanceRecordedEvent(event);
          break;
        case 'MetadataUpdatedEvent':
          await this.handleMetadataUpdatedEvent(event);
          break;

        // Scoring Events
        case 'CreditScoreUpdatedEvent':
          await this.handleCreditScoreUpdatedEvent(event);
          break;
        case 'CourseCompletedEvent':
          await this.handleCourseCompletedEvent(event);
          break;
        case 'ContributionRecordedEvent':
          await this.handleContributionRecordedEvent(event);
          break;

        default:
          this.logger.warn(`Unknown event type: ${eventType}`);
      }
    } catch (error) {
      this.logger.error(`Error processing event: ${error.message}`, error.stack);
      throw error;
    }
  }

  // Bridge Event Handlers
  private async handleDepositEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;

    const bridgeTx = this.bridgeTransactionRepo.create({
      nonce: data.nonce,
      from: data.from,
      amount: data.amount,
      destinationChain: data.destination_chain,
      destinationAddress: data.destination_address,
      status: BridgeStatus.INITIATED,
      ledger: event.ledger,
      txHash: event.txHash,
      timestamp: event.timestamp,
    });

    await this.bridgeTransactionRepo.save(bridgeTx);
    this.logger.log(`Indexed DepositEvent for nonce ${data.nonce}`);
  }

  private async handleReleaseEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;

    const existingTx = await this.bridgeTransactionRepo.findOne({
      where: { nonce: data.nonce },
    });

    if (existingTx) {
      existingTx.recipient = data.recipient;
      existingTx.sourceChain = data.source_chain;
      existingTx.status = BridgeStatus.COMPLETED;
      await this.bridgeTransactionRepo.save(existingTx);
    } else {
      const bridgeTx = this.bridgeTransactionRepo.create({
        nonce: data.nonce,
        recipient: data.recipient,
        amount: data.amount,
        sourceChain: data.source_chain,
        status: BridgeStatus.COMPLETED,
        ledger: event.ledger,
        txHash: event.txHash,
        timestamp: event.timestamp,
      } as any);
      await this.bridgeTransactionRepo.save(bridgeTx);
    }

    this.logger.log(`Indexed ReleaseEvent for nonce ${data.nonce}`);
  }

  private async handleBridgeInitiatedEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;
    const tx = data.transaction;

    const bridgeTx = this.bridgeTransactionRepo.create({
      nonce: data.nonce,
      from: tx.from,
      amount: tx.amount,
      destinationChain: tx.destination_chain,
      destinationAddress: tx.destination_address,
      status: BridgeStatus.INITIATED,
      ledger: event.ledger,
      txHash: event.txHash,
      timestamp: event.timestamp,
    });

    await this.bridgeTransactionRepo.save(bridgeTx);
    this.logger.log(`Indexed BridgeInitiatedEvent for nonce ${data.nonce}`);
  }

  private async handleBridgeCompletedEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;

    const bridgeTx = await this.bridgeTransactionRepo.findOne({
      where: { nonce: data.nonce },
    });

    if (bridgeTx) {
      bridgeTx.status = BridgeStatus.COMPLETED;
      await this.bridgeTransactionRepo.save(bridgeTx);
      this.logger.log(`Indexed BridgeCompletedEvent for nonce ${data.nonce}`);
    }
  }

  // Reward Event Handlers
  private async handleRewardIssuedEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;

    const reward = this.rewardRepo.create({
      recipient: data.recipient,
      amount: data.amount,
      rewardType: data.reward_type,
      status: RewardStatus.ISSUED,
      timestamp: data.timestamp,
      ledger: event.ledger,
      txHash: event.txHash,
    });

    await this.rewardRepo.save(reward);
    this.logger.log(`Indexed RewardIssuedEvent for ${data.recipient}`);
  }

  private async handleRewardClaimedEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;

    // Find all unclaimed rewards for this user
    const rewards = await this.rewardRepo.find({
      where: {
        recipient: data.user,
        status: RewardStatus.ISSUED,
      },
    });

    // Mark rewards as claimed
    for (const reward of rewards) {
      reward.status = RewardStatus.CLAIMED;
      reward.claimedAt = data.timestamp;
      await this.rewardRepo.save(reward);
    }

    this.logger.log(`Indexed RewardClaimedEvent for ${data.user}`);
  }

  private async handleRewardPoolFundedEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;

    let pool = await this.rewardPoolRepo.findOne({ where: {} });

    if (!pool) {
      pool = this.rewardPoolRepo.create({
        totalPoolBalance: data.amount,
        totalRewardsIssued: '0',
        totalRewardsClaimed: '0',
        lastFundedLedger: event.ledger,
        lastFundedTxHash: event.txHash,
        lastFundedTimestamp: data.timestamp,
        lastFunder: data.funder,
        lastFundedAmount: data.amount,
      });
    } else {
      const currentBalance = BigInt(pool.totalPoolBalance);
      const additionalAmount = BigInt(data.amount);
      pool.totalPoolBalance = (currentBalance + additionalAmount).toString();
      pool.lastFundedLedger = event.ledger;
      pool.lastFundedTxHash = event.txHash;
      pool.lastFundedTimestamp = data.timestamp;
      pool.lastFunder = data.funder;
      pool.lastFundedAmount = data.amount;
    }

    await this.rewardPoolRepo.save(pool);
    this.logger.log(`Indexed RewardPoolFundedEvent from ${data.funder}`);
  }

  // Escrow Event Handlers
  private async handleEscrowCreatedEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;
    const escrowData = data.escrow;

    const escrow = this.escrowRepo.create({
      escrowId: escrowData.id,
      depositor: escrowData.depositor,
      beneficiary: escrowData.beneficiary,
      amount: escrowData.amount,
      requiredSigners: escrowData.required_signers,
      requiredApprovals: escrowData.required_approvals,
      deadline: escrowData.deadline,
      status: EscrowStatus.ACTIVE,
      createdAtLedger: event.ledger,
      createdTxHash: event.txHash,
    });

    await this.escrowRepo.save(escrow);
    this.logger.log(`Indexed EscrowCreatedEvent for escrow ${escrowData.id}`);
  }

  private async handleEscrowApprovedEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;

    const escrow = await this.escrowRepo.findOne({
      where: { escrowId: data.escrow_id },
    });

    if (escrow) {
      if (!escrow.approvers.includes(data.signer)) {
        escrow.approvers = [...escrow.approvers, data.signer];
      }
      escrow.approvalCount = data.approval_count;

      if (escrow.approvalCount >= escrow.requiredApprovals) {
        escrow.status = EscrowStatus.APPROVED;
      }

      await this.escrowRepo.save(escrow);
      this.logger.log(`Indexed EscrowApprovedEvent for escrow ${data.escrow_id}`);
    }
  }

  private async handleEscrowReleasedEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;

    const escrow = await this.escrowRepo.findOne({
      where: { escrowId: data.escrow_id },
    });

    if (escrow) {
      escrow.status = EscrowStatus.RELEASED;
      escrow.completedAtLedger = event.ledger;
      escrow.completedTxHash = event.txHash;
      await this.escrowRepo.save(escrow);
      this.logger.log(`Indexed EscrowReleasedEvent for escrow ${data.escrow_id}`);
    }
  }

  private async handleEscrowRefundedEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;

    const escrow = await this.escrowRepo.findOne({
      where: { escrowId: data.escrow_id },
    });

    if (escrow) {
      escrow.status = EscrowStatus.REFUNDED;
      escrow.completedAtLedger = event.ledger;
      escrow.completedTxHash = event.txHash;
      await this.escrowRepo.save(escrow);
      this.logger.log(`Indexed EscrowRefundedEvent for escrow ${data.escrow_id}`);
    }
  }

  private async handleEscrowDisputedEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;

    const escrow = await this.escrowRepo.findOne({
      where: { escrowId: data.escrow_id },
    });

    if (escrow) {
      escrow.status = EscrowStatus.DISPUTED;
      escrow.disputeReason = data.reason;
      escrow.disputer = data.disputer;
      await this.escrowRepo.save(escrow);
      this.logger.log(`Indexed EscrowDisputedEvent for escrow ${data.escrow_id}`);
    }
  }

  private async handleEscrowResolvedEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;

    const escrow = await this.escrowRepo.findOne({
      where: { escrowId: data.escrow_id },
    });

    if (escrow) {
      escrow.status = EscrowStatus.RESOLVED;
      escrow.resolutionOutcome = data.outcome;
      escrow.completedAtLedger = event.ledger;
      escrow.completedTxHash = event.txHash;
      await this.escrowRepo.save(escrow);
      this.logger.log(`Indexed EscrowResolvedEvent for escrow ${data.escrow_id}`);
    }
  }

  // Tokenization Event Handlers
  private async handleContentMintedEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;
    const metadata = data.metadata;

    const token = this.contentTokenRepo.create({
      tokenId: data.token_id,
      creator: data.creator,
      currentOwner: data.creator,
      contentHash: metadata.content_hash,
      metadataUri: metadata.metadata_uri,
      transferable: metadata.transferable,
      royaltyPercentage: metadata.royalty_percentage,
      mintedAtLedger: event.ledger,
      mintedTxHash: event.txHash,
      mintedTimestamp: event.timestamp,
    });

    await this.contentTokenRepo.save(token);

    // Create provenance record
    await this.createProvenanceRecord({
      tokenId: data.token_id,
      eventType: ProvenanceEventType.MINT,
      fromAddress: null,
      toAddress: data.creator,
      timestamp: event.timestamp,
      ledger: event.ledger,
      txHash: event.txHash,
    });

    this.logger.log(`Indexed ContentMintedEvent for token ${data.token_id}`);
  }

  private async handleOwnershipTransferredEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;

    const token = await this.contentTokenRepo.findOne({
      where: { tokenId: data.token_id },
    });

    if (token) {
      token.currentOwner = data.to;
      token.transferCount += 1;
      token.lastTransferLedger = event.ledger;
      token.lastTransferTxHash = event.txHash;
      token.lastTransferTimestamp = data.timestamp;
      await this.contentTokenRepo.save(token);

      // Create provenance record
      await this.createProvenanceRecord({
        tokenId: data.token_id,
        eventType: ProvenanceEventType.TRANSFER,
        fromAddress: data.from,
        toAddress: data.to,
        timestamp: data.timestamp,
        ledger: event.ledger,
        txHash: event.txHash,
      });

      this.logger.log(`Indexed OwnershipTransferredEvent for token ${data.token_id}`);
    }
  }

  private async handleProvenanceRecordedEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;
    const record = data.record;

    await this.createProvenanceRecord({
      tokenId: data.token_id,
      eventType: this.mapProvenanceEventType(record.event_type),
      fromAddress: record.from,
      toAddress: record.to,
      timestamp: record.timestamp,
      ledger: event.ledger,
      txHash: event.txHash,
    });

    this.logger.log(`Indexed ProvenanceRecordedEvent for token ${data.token_id}`);
  }

  private async handleMetadataUpdatedEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;

    const token = await this.contentTokenRepo.findOne({
      where: { tokenId: data.token_id },
    });

    if (token) {
      // Create provenance record for metadata update
      await this.createProvenanceRecord({
        tokenId: data.token_id,
        eventType: ProvenanceEventType.METADATA_UPDATE,
        fromAddress: null,
        toAddress: data.owner,
        timestamp: data.timestamp,
        ledger: event.ledger,
        txHash: event.txHash,
      });

      this.logger.log(`Indexed MetadataUpdatedEvent for token ${data.token_id}`);
    }
  }

  // Scoring Event Handlers
  private async handleCreditScoreUpdatedEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;

    let creditScore = await this.creditScoreRepo.findOne({
      where: { userAddress: data.user },
    });

    if (!creditScore) {
      creditScore = this.creditScoreRepo.create({
        userAddress: data.user,
        score: data.new_score,
        lastUpdatedLedger: event.ledger,
        lastUpdatedTxHash: event.txHash,
        lastUpdatedTimestamp: event.timestamp,
      });
    } else {
      creditScore.score = data.new_score;
      creditScore.lastUpdatedLedger = event.ledger;
      creditScore.lastUpdatedTxHash = event.txHash;
      creditScore.lastUpdatedTimestamp = event.timestamp;
    }

    await this.creditScoreRepo.save(creditScore);
    this.logger.log(`Indexed CreditScoreUpdatedEvent for ${data.user}`);
  }

  private async handleCourseCompletedEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;

    const courseCompletion = this.courseCompletionRepo.create({
      userAddress: data.user,
      courseId: data.course_id,
      pointsEarned: data.points,
      completedAt: event.timestamp,
      ledger: event.ledger,
      txHash: event.txHash,
    });

    await this.courseCompletionRepo.save(courseCompletion);

    // Update credit score
    const creditScore = await this.creditScoreRepo.findOne({
      where: { userAddress: data.user },
    });

    if (creditScore) {
      creditScore.coursesCompleted += 1;
      await this.creditScoreRepo.save(creditScore);
    }

    this.logger.log(`Indexed CourseCompletedEvent for ${data.user}`);
  }

  private async handleContributionRecordedEvent(event: ProcessedEvent): Promise<void> {
    const data = event.data;

    const contribution = this.contributionRepo.create({
      userAddress: data.user,
      contributionType: data.c_type,
      pointsEarned: data.points,
      timestamp: event.timestamp,
      ledger: event.ledger,
      txHash: event.txHash,
    });

    await this.contributionRepo.save(contribution);

    // Update credit score
    const creditScore = await this.creditScoreRepo.findOne({
      where: { userAddress: data.user },
    });

    if (creditScore) {
      creditScore.contributionsCount += 1;
      await this.creditScoreRepo.save(creditScore);
    }

    this.logger.log(`Indexed ContributionRecordedEvent for ${data.user}`);
  }

  // Helper Methods
  private async createProvenanceRecord(data: {
    tokenId: string;
    eventType: ProvenanceEventType;
    fromAddress: string | null;
    toAddress: string;
    timestamp: string;
    ledger: string;
    txHash: string;
  }): Promise<void> {
    const record = this.provenanceRepo.create({
      ...data,
      fromAddress: data.fromAddress || undefined,
    });
    await this.provenanceRepo.save(record);
  }

  private mapProvenanceEventType(eventType: string): ProvenanceEventType {
    switch (eventType.toLowerCase()) {
      case 'mint':
        return ProvenanceEventType.MINT;
      case 'transfer':
        return ProvenanceEventType.TRANSFER;
      case 'metadata_update':
        return ProvenanceEventType.METADATA_UPDATE;
      default:
        return ProvenanceEventType.TRANSFER;
    }
  }
}
