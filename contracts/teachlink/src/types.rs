//! TeachLink Contract Types
//!
//! This module defines all data structures used throughout the TeachLink smart contract.

use soroban_sdk::{contracttype, Address, Bytes, Map, String, Vec};

// ========== Chain Configuration Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChainConfig {
    pub chain_id: u32,
    pub chain_name: Bytes,
    pub is_active: bool,
    pub bridge_contract_address: Bytes,
    pub confirmation_blocks: u32,
    pub gas_price: u64,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiChainAsset {
    pub asset_id: Bytes,
    pub stellar_token: Address,
    pub chain_configs: Map<u32, ChainAssetInfo>,
    pub total_bridged: i128,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChainAssetInfo {
    pub chain_id: u32,
    pub token_address: Bytes,
    pub decimals: u32,
    pub is_active: bool,
}

// ========== BFT Consensus Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorInfo {
    pub address: Address,
    pub stake: i128,
    pub reputation_score: u32,
    pub is_active: bool,
    pub joined_at: u64,
    pub last_activity: u64,
    pub total_validations: u64,
    pub missed_validations: u64,
    pub slashed_amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeProposal {
    pub proposal_id: u64,
    pub message: CrossChainMessage,
    pub votes: Map<Address, bool>,
    pub vote_count: u32,
    pub required_votes: u32,
    pub status: ProposalStatus,
    pub created_at: u64,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
    Executed,
    Expired,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsensusState {
    pub total_stake: i128,
    pub active_validators: u32,
    pub byzantine_threshold: u32,
    pub last_consensus_round: u64,
}

// ========== Slashing and Rewards Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlashingRecord {
    pub validator: Address,
    pub amount: i128,
    pub reason: SlashingReason,
    pub timestamp: u64,
    pub evidence: Bytes,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlashingReason {
    DoubleVote,
    InvalidSignature,
    Inactivity,
    ByzantineBehavior,
    MaliciousProposal,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorReward {
    pub validator: Address,
    pub amount: i128,
    pub reward_type: RewardType,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RewardType {
    Validation,
    Consensus,
    Uptime,
    Security,
}

// ========== Liquidity and AMM Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiquidityPool {
    pub chain_id: u32,
    pub token: Address,
    pub total_liquidity: i128,
    pub available_liquidity: i128,
    pub locked_liquidity: i128,
    pub lp_providers: Map<Address, LPPosition>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LPPosition {
    pub provider: Address,
    pub amount: i128,
    pub share_percentage: u32,
    pub deposited_at: u64,
    pub rewards_earned: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeFeeStructure {
    pub base_fee: i128,
    pub dynamic_multiplier: u32,
    pub congestion_multiplier: u32,
    pub volume_discount_tiers: Map<u32, u32>,
    pub last_updated: u64,
}

// ========== Message Passing Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChainPacket {
    pub packet_id: u64,
    pub source_chain: u32,
    pub destination_chain: u32,
    pub sender: Bytes,
    pub recipient: Bytes,
    pub payload: Bytes,
    pub nonce: u64,
    pub timeout: u64,
    pub status: PacketStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PacketStatus {
    Pending,
    Delivered,
    Failed,
    TimedOut,
    Retrying,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageReceipt {
    pub packet_id: u64,
    pub delivered_at: u64,
    pub gas_used: u64,
    pub result: Bytes,
}

// ========== Emergency and Security Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyState {
    pub is_paused: bool,
    pub paused_at: u64,
    pub paused_by: Address,
    pub reason: Bytes,
    pub affected_chains: Vec<u32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CircuitBreaker {
    pub chain_id: u32,
    pub max_daily_volume: i128,
    pub current_daily_volume: i128,
    pub max_transaction_amount: i128,
    pub last_reset: u64,
    pub is_triggered: bool,
}

// ========== Audit and Compliance Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditRecord {
    pub record_id: u64,
    pub operation_type: OperationType,
    pub operator: Address,
    pub timestamp: u64,
    pub details: Bytes,
    pub tx_hash: Bytes,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationType {
    BridgeIn,
    BridgeOut,
    ValidatorAdded,
    ValidatorRemoved,
    ValidatorSlashed,
    EmergencyPause,
    EmergencyResume,
    FeeUpdate,
    ConfigUpdate,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComplianceReport {
    pub report_id: u64,
    pub period_start: u64,
    pub period_end: u64,
    pub total_volume: i128,
    pub total_transactions: u64,
    pub unique_users: u32,
    pub validator_performance: Map<Address, u32>,
}

// ========== Atomic Swap Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtomicSwap {
    pub swap_id: u64,
    pub initiator: Address,
    pub initiator_token: Address,
    pub initiator_amount: i128,
    pub counterparty: Address,
    pub counterparty_token: Address,
    pub counterparty_amount: i128,
    pub hashlock: Bytes,
    pub timelock: u64,
    pub status: SwapStatus,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SwapStatus {
    Initiated,
    CounterpartyAccepted,
    Completed,
    Refunded,
    Expired,
}

// ========== Analytics Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeMetrics {
    pub total_volume: i128,
    pub total_transactions: u64,
    pub active_validators: u32,
    pub average_confirmation_time: u64,
    pub success_rate: u32,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChainMetrics {
    pub chain_id: u32,
    pub volume_in: i128,
    pub volume_out: i128,
    pub transaction_count: u64,
    pub average_fee: i128,
    pub last_updated: u64,
}

// ========== Validator Signature Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorSignature {
    pub validator: Address,
    pub signature: Bytes,
    pub timestamp: u64,
}

// ========== Content Tokenization Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentTokenParameters {
    pub creator: Address,
    pub title: Bytes,
    pub description: Bytes,
    pub content_type: ContentType,
    pub content_hash: Bytes,
    pub license_type: Bytes,
    pub tags: Vec<Bytes>,
    pub is_transferable: bool,
    pub royalty_percentage: u32,
}

// ========== Escrow Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowParameters {
    pub depositor: Address,
    pub beneficiary: Address,
    pub token: Address,
    pub amount: i128,
    pub signers: Vec<Address>,
    pub threshold: u32,
    pub release_time: Option<u64>,
    pub refund_time: Option<u64>,
    pub arbitrator: Address,
}

// ========== Bridge Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeTransaction {
    pub nonce: u64,
    pub token: Address,
    pub amount: i128,
    pub recipient: Address,
    pub destination_chain: u32,
    pub destination_address: Bytes,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChainMessage {
    pub source_chain: u32,
    pub source_tx_hash: Bytes,
    pub nonce: u64,
    pub token: Address,
    pub amount: i128,
    pub recipient: Address,
    pub destination_chain: u32,
}

//
// ==========================
// Rewards Types
// ==========================
//

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserReward {
    pub user: Address,
    pub total_earned: i128,
    pub claimed: i128,
    pub pending: i128,
    pub last_claim_timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardRate {
    pub reward_type: String,
    pub rate: i128,
    pub enabled: bool,
}

// ========== Escrow Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    Pending,
    Released,
    Refunded,
    Disputed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Escrow {
    pub id: u64,
    pub depositor: Address,
    pub beneficiary: Address,
    pub token: Address,
    pub amount: i128,
    pub signers: Vec<Address>,
    pub threshold: u32,
    pub approval_count: u32,
    pub release_time: Option<u64>,
    pub refund_time: Option<u64>,
    pub arbitrator: Address,
    pub status: EscrowStatus,
    pub created_at: u64,
    pub dispute_reason: Option<Bytes>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowApprovalKey {
    pub escrow_id: u64,
    pub signer: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeOutcome {
    ReleaseToBeneficiary,
    RefundToDepositor,
}

//
// ==========================
// Credit Score / Contribution Types
// ==========================
//

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContributionType {
    Content,
    Code,
    Community,
    Governance,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contribution {
    pub contributor: Address,
    pub c_type: ContributionType,
    pub description: Bytes,
    pub timestamp: u64,
    pub points: u64,
}

//
// ==========================
// Reputation Types
// ==========================
//

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserReputation {
    pub participation_score: u32,
    pub completion_rate: u32,
    pub contribution_quality: u32,
    pub total_courses_started: u32,
    pub total_courses_completed: u32,
    pub total_contributions: u32,
    pub last_update: u64,
}

//
// ==========================
// Content Tokenization Types
// ==========================
//

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentType {
    Course,
    Material,
    Lesson,
    Assessment,
    Certificate,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentMetadata {
    pub title: Bytes,
    pub description: Bytes,
    pub content_type: ContentType,
    pub creator: Address,
    pub content_hash: Bytes,
    pub license_type: Bytes,
    pub tags: Vec<Bytes>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentToken {
    pub token_id: u64,
    pub metadata: ContentMetadata,
    pub owner: Address,
    pub minted_at: u64,
    pub is_transferable: bool,
    pub royalty_percentage: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceRecord {
    pub token_id: u64,
    pub from: Option<Address>,
    pub to: Address,
    pub timestamp: u64,
    pub transaction_hash: Bytes,
    pub transfer_type: TransferType,
    pub notes: Option<Bytes>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransferType {
    Mint,
    Transfer,
    License,
    Revoke,
}
