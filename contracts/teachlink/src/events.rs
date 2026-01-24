use soroban_sdk::contractevent;

use crate::types::{BridgeTransaction, CrossChainMessage, DisputeOutcome, Escrow, EscrowStatus};
use soroban_sdk::{Address, Bytes, String};

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

// Rewards Events
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
pub struct EscrowCreatedEvent {
    pub escrow: Escrow,
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
pub struct RewardPoolFundedEvent {
    pub funder: Address,
    pub amount: i128,
    pub timestamp: u64,
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
