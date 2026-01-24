use soroban_sdk::{contracttype, Address, Bytes, Vec, String};

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserReward {
    pub user: Address,
    pub total_earned: i128,
    pub claimed: i128,
    pub pending: i128,
    pub last_claim_timestamp: u64,
pub enum EscrowStatus {
    Pending,
    Released,
    Refunded,
    Disputed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardRate {
    pub reward_type: String,
    pub rate: i128,
    pub enabled: bool,
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

// ========== Educational Content Tokenization Types ==========

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
    pub content_hash: Bytes, // IPFS hash or content identifier
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
    pub royalty_percentage: u32, // Basis points (e.g., 500 = 5%)
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceRecord {
    pub token_id: u64,
    pub from: Option<Address>, // None for initial mint
    pub to: Address,
    pub timestamp: u64,
    pub transaction_hash: Bytes,
    pub transfer_type: TransferType,
    pub notes: Option<Bytes>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransferType {
    Mint,        // Initial creation
    Transfer,    // Standard ownership transfer
    License,     // Licensing agreement
    Revoke,      // Ownership revoked
}
