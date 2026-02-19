use soroban_sdk::contractevent;

use crate::types::{
    BridgeTransaction, ContentMetadata, ContributionType, CrossChainMessage, DisputeOutcome,
    Escrow, EscrowStatus, ProvenanceRecord, ProposalStatus, PacketStatus, SwapStatus,
    SlashingReason, RewardType, OperationType,
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

// ================= Audit and Compliance Events =================

#[contractevent]
#[derive(Clone, Debug)]
pub struct AuditRecordCreatedEvent {
    pub record_id: u64,
    pub operation_type: OperationType,
    pub operator: Address,
    pub timestamp: u64,
}

// #[contractevent]
// #[derive(Clone, Debug)]
// pub struct ComplianceReportGeneratedEvent {
//     pub report_id: u64,
//     pub period_start: u64,
//     pub period_end: u64,
//     pub total_volume: i128,
// }

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
