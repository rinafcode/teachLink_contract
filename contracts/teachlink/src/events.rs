use soroban_sdk::contractevent;

use crate::types::{
    BridgeTransaction,
    ContentMetadata,
    ContributionType,
    CrossChainMessage,
    DisputeOutcome,
    Escrow,
    EscrowStatus,
    ProvenanceRecord,
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
