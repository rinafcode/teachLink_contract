use soroban_sdk::contractevent;

use crate::types::{
    BridgeTransaction, ContentMetadata, ContributionType, CrossChainMessage, DisputeOutcome,
    Escrow, EscrowStatus, OperationType, ProposalStatus, ProvenanceRecord, RewardType,
    SlashingReason,
};

use soroban_sdk::{Address, Bytes, String};

// ================= Bridge Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct DepositEvent {
    pub nonce: u64,
    pub from: Address,
    pub amount: i128,
    pub destination_chain: u32,
    pub destination_address: Bytes,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ReleaseEvent {
    pub nonce: u64,
    pub recipient: Address,
    pub amount: i128,
    pub source_chain: u32,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct BridgeInitiatedEvent {
    pub nonce: u64,
    pub transaction: BridgeTransaction,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct BridgeCompletedEvent {
    pub nonce: u64,
    pub message: CrossChainMessage,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct BridgeCancelledEvent {
    pub nonce: u64,
    pub refunded_to: Address,
    pub amount: i128,
    pub cancelled_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct BridgeFailedEvent {
    pub nonce: u64,
    pub reason: Bytes,
    pub failed_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct BridgeRetryEvent {
    pub nonce: u64,
    pub retry_count: u32,
    pub retried_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ValidatorAddedEvent {
    pub validator: Address,
    pub added_by: Address,
    pub added_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ValidatorRemovedEvent {
    pub validator: Address,
    pub removed_by: Address,
    pub removed_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ChainSupportedEvent {
    pub chain_id: u32,
    pub added_by: Address,
    pub added_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ChainUnsupportedEvent {
    pub chain_id: u32,
    pub removed_by: Address,
    pub removed_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct BridgeFeeUpdatedEvent {
    pub old_fee: i128,
    pub new_fee: i128,
    pub updated_by: Address,
    pub updated_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct FeeRecipientUpdatedEvent {
    pub old_recipient: Address,
    pub new_recipient: Address,
    pub updated_by: Address,
    pub updated_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct MinValidatorsUpdatedEvent {
    pub old_min: u32,
    pub new_min: u32,
    pub updated_by: Address,
    pub updated_at: u64,
}

// ================= BFT Consensus Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct ProposalCreatedEvent {
    pub proposal_id: u64,
    pub message: CrossChainMessage,
    pub required_votes: u32,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ProposalVotedEvent {
    pub proposal_id: u64,
    pub validator: Address,
    pub vote: bool,
    pub vote_count: u32,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ProposalExecutedEvent {
    pub proposal_id: u64,
    pub status: ProposalStatus,
    pub executed_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ValidatorRegisteredEvent {
    pub validator: Address,
    pub stake: i128,
    pub joined_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ValidatorUnregisteredEvent {
    pub validator: Address,
    pub unstaked_amount: i128,
    pub left_at: u64,
}

// ================= Slashing and Rewards Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct ValidatorSlashedEvent {
    pub validator: Address,
    pub amount: i128,
    pub reason: SlashingReason,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ValidatorRewardedEvent {
    pub validator: Address,
    pub amount: i128,
    pub reward_type: RewardType,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct StakeDepositedEvent {
    pub validator: Address,
    pub amount: i128,
    pub total_stake: i128,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct StakeWithdrawnEvent {
    pub validator: Address,
    pub amount: i128,
    pub remaining_stake: i128,
}

// Note: RewardPoolFundedExternalEvent removed - use RewardPoolFundedEvent instead

// ================= Multi-Chain Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct ChainAddedEvent {
    pub chain_id: u32,
    pub chain_name: Bytes,
    pub added_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ChainUpdatedEvent {
    pub chain_id: u32,
    pub is_active: bool,
    pub updated_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct AssetRegisteredEvent {
    pub asset_id: Bytes,
    pub stellar_token: Address,
    pub supported_chains: u32,
}

// ================= Liquidity and AMM Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct LiquidityAddedEvent {
    pub provider: Address,
    pub chain_id: u32,
    pub amount: i128,
    pub share_percentage: u32,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct LiquidityRemovedEvent {
    pub provider: Address,
    pub chain_id: u32,
    pub amount: i128,
    pub rewards: i128,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct FeeUpdatedEvent {
    pub old_fee: i128,
    pub new_fee: i128,
    pub multiplier: u32,
}

// ================= Message Passing Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct PacketSentEvent {
    pub packet_id: u64,
    pub source_chain: u32,
    pub destination_chain: u32,
    pub sender: Bytes,
    pub nonce: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct PacketDeliveredEvent {
    pub packet_id: u64,
    pub delivered_at: u64,
    pub gas_used: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct PacketFailedEvent {
    pub packet_id: u64,
    pub reason: Bytes,
    pub failed_at: u64,
}

// ================= Emergency and Security Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct BridgePausedEvent {
    pub paused_by: Address,
    pub reason: Bytes,
    pub paused_at: u64,
    pub affected_chains: soroban_sdk::Vec<u32>,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct BridgeResumedEvent {
    pub resumed_by: Address,
    pub resumed_at: u64,
    pub affected_chains: soroban_sdk::Vec<u32>,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct CircuitBreakerTriggeredEvent {
    pub chain_id: u32,
    pub trigger_reason: Bytes,
    pub triggered_at: u64,
}

// Note: CircuitBreakerInitializedEvent removed due to Soroban event field limitations
// Circuit breaker initialization is tracked through regular storage updates

#[contractevent]
#[derive(Clone, Debug)]
pub struct CircuitBreakerResetEvent {
    pub chain_id: u32,
    pub reset_by: Address,
    pub reset_at: u64,
}

// Note: CircuitBreakerLimitsUpdatedEvent removed due to Soroban event field limitations
// Use CircuitBreakerTriggeredEvent for circuit breaker state changes

// ================= Audit and Compliance Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct AuditRecordCreatedEvent {
    pub record_id: u64,
    pub operation_type: OperationType,
    pub operator: Address,
    pub timestamp: u64,
}

// ================= Atomic Swap Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct SwapInitiatedEvent {
    pub swap_id: u64,
    pub initiator: Address,
    pub initiator_amount: i128,
    pub counterparty: Address,
    pub counterparty_amount: i128,
    pub timelock: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct SwapCompletedEvent {
    pub swap_id: u64,
    pub completed_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct SwapRefundedEvent {
    pub swap_id: u64,
    pub refunded_to: Address,
    pub amount: i128,
}

// ================= Rewards Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct RewardIssuedEvent {
    pub recipient: Address,
    pub amount: i128,
    pub reward_type: String,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct RewardClaimedEvent {
    pub user: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct RewardPoolFundedEvent {
    pub funder: Address,
    pub amount: i128,
    pub timestamp: u64,
}

// ================= Escrow Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct EscrowCreatedEvent {
    pub escrow: Escrow,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct EscrowApprovedEvent {
    pub escrow_id: u64,
    pub signer: Address,
    pub approval_count: u32,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct EscrowReleasedEvent {
    pub escrow_id: u64,
    pub beneficiary: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct EscrowRefundedEvent {
    pub escrow_id: u64,
    pub depositor: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct EscrowCancelledEvent {
    pub escrow_id: u64,
    pub depositor: Address,
    pub amount: i128,
    pub cancelled_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct EscrowDisputedEvent {
    pub escrow_id: u64,
    pub disputer: Address,
    pub reason: Bytes,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct EscrowResolvedEvent {
    pub escrow_id: u64,
    pub outcome: DisputeOutcome,
    pub status: EscrowStatus,
}

// ================= Insurance Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct InsurancePoolInitializedEvent {
    pub token: Address,
    pub premium_rate: u32,
    pub initialized_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct InsurancePoolFundedEvent {
    pub funder: Address,
    pub amount: i128,
    pub new_balance: i128,
    pub funded_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct InsurancePremiumPaidEvent {
    pub user: Address,
    pub amount: i128,
    pub paid_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct InsuranceClaimProcessedEvent {
    pub recipient: Address,
    pub payout_amount: i128,
    pub new_balance: i128,
    pub processed_at: u64,
}

// ================= Credit Score Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct CreditScoreUpdatedEvent {
    pub user: Address,
    pub new_score: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct CourseCompletedEvent {
    pub user: Address,
    pub course_id: u64,
    pub points: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ContributionRecordedEvent {
    pub user: Address,
    pub c_type: ContributionType,
    pub points: u64,
}

// ================= Reputation Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct ParticipationUpdatedEvent {
    pub user: Address,
    pub points_added: u32,
    pub new_participation_score: u32,
    pub updated_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct CourseProgressUpdatedEvent {
    pub user: Address,
    pub total_courses_started: u32,
    pub total_courses_completed: u32,
    pub completion_rate: u32,
    pub updated_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ContributionRatedEvent {
    pub user: Address,
    pub rating: u32,
    pub new_contribution_quality: u32,
    pub total_contributions: u32,
    pub rated_at: u64,
}

// ================= Content Tokenization Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct ContentMintedEvent {
    pub token_id: u64,
    pub creator: Address,
    pub metadata: ContentMetadata,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct OwnershipTransferredEvent {
    pub token_id: u64,
    pub from: Address,
    pub to: Address,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ProvenanceRecordedEvent {
    pub token_id: u64,
    pub record: ProvenanceRecord,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct MetadataUpdatedEvent {
    pub token_id: u64,
    pub owner: Address,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct TransferabilityUpdatedEvent {
    pub token_id: u64,
    pub owner: Address,
    pub transferable: bool,
    pub updated_at: u64,
}

// ================= Advanced Analytics & Reporting Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct ReportGeneratedEvent {
    pub report_id: u64,
    pub report_type: crate::types::ReportType,
    pub generated_by: Address,
    pub period_start: u64,
    pub period_end: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ReportScheduledEvent {
    pub schedule_id: u64,
    pub template_id: u64,
    pub owner: Address,
    pub next_run_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ReportCommentAddedEvent {
    pub report_id: u64,
    pub comment_id: u64,
    pub author: Address,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct AlertTriggeredEvent {
    pub rule_id: u64,
    pub condition_type: crate::types::AlertConditionType,
    pub current_value: i128,
    pub threshold: i128,
    pub triggered_at: u64,
}

// ================= Backup and Disaster Recovery Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct BackupCreatedEvent {
    pub backup_id: u64,
    pub created_by: Address,
    pub integrity_hash: Bytes,
    pub rto_tier: crate::types::RtoTier,
    pub created_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct BackupVerifiedEvent {
    pub backup_id: u64,
    pub verified_by: Address,
    pub verified_at: u64,
    pub valid: bool,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct RecoveryExecutedEvent {
    pub recovery_id: u64,
    pub backup_id: u64,
    pub executed_by: Address,
    pub recovery_duration_secs: u64,
    pub success: bool,
}

// ================= Performance Optimization Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct PerfMetricsComputedEvent {
    pub health_score: u32,
    pub computed_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct PerfCacheInvalidatedEvent {
    pub invalidated_at: u64,
}

// ================= Observability Events =================

/// Emitted when bridge-level metrics are updated.
#[contractevent]
#[derive(Clone, Debug)]
pub struct BridgeMetricsUpdatedEvent {
    pub total_volume: i128,
    pub total_transactions: u64,
    pub active_validators: u32,
    pub average_confirmation_time: u64,
    /// Basis points (10000 = 100%).
    pub success_rate: u32,
    pub updated_at: u64,
}

/// Emitted when per-chain metrics are updated.
#[contractevent]
#[derive(Clone, Debug)]
pub struct ChainMetricsUpdatedEvent {
    pub chain_id: u32,
    pub volume_in: i128,
    pub volume_out: i128,
    pub transaction_count: u64,
    pub average_fee: i128,
    pub updated_at: u64,
}
