//! TeachLink Contract Types
//!
//! This module defines all data structures used throughout the TeachLink smart contract.
//! These types are organized into the following categories:
//!
//! - **Bridge Types**: Cross-chain token bridging data structures
//! - **Rewards Types**: User reward tracking and rate configuration
//! - **Escrow Types**: Multi-signature escrow and dispute resolution
//! - **Credit Score Types**: User contribution and scoring
//! - **Reputation Types**: User reputation and participation tracking
//! - **Tokenization Types**: Educational content NFTs and provenance

use soroban_sdk::{contracttype, Address, Bytes, String, Vec};

// ========== Bridge Types ==========

/// Represents a cross-chain bridge transaction for token transfers.
///
/// This struct tracks the state of tokens being bridged from Stellar
/// to another blockchain network. The transaction is identified by
/// a unique nonce and contains all information needed to complete
/// or reverse the bridge operation.
///
/// # Fields
/// * `nonce` - Unique transaction identifier, auto-incremented per bridge
/// * `token` - Address of the token being bridged
/// * `amount` - Amount of tokens being transferred
/// * `recipient` - Stellar address that initiated the bridge
/// * `destination_chain` - Target chain ID (e.g., 1 for Ethereum mainnet)
/// * `destination_address` - Recipient address on the destination chain
/// * `timestamp` - Unix timestamp when the bridge was initiated
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

/// Message payload for completing cross-chain token transfers.
///
/// This struct contains the verified information from the source chain
/// needed to mint/release tokens on Stellar. Validators must sign
/// this message to authorize the completion of a bridge transaction.
///
/// # Fields
/// * `source_chain` - Chain ID where the tokens originated
/// * `source_tx_hash` - Transaction hash on the source chain
/// * `nonce` - Unique identifier matching the original transaction
/// * `token` - Token address on Stellar
/// * `amount` - Amount to mint/release
/// * `recipient` - Stellar address to receive the tokens
/// * `destination_chain` - Should be Stellar's chain ID
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

// ========== Rewards Types ==========

/// Tracks a user's reward balance and claim history.
///
/// This struct maintains the complete reward state for a single user,
/// including lifetime earnings, amounts already claimed, and pending
/// rewards available for withdrawal.
///
/// # Fields
/// * `user` - Address of the reward recipient
/// * `total_earned` - Lifetime total rewards earned
/// * `claimed` - Total amount already claimed/withdrawn
/// * `pending` - Available balance (total_earned - claimed)
/// * `last_claim_timestamp` - Unix timestamp of the last claim
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserReward {
    pub user: Address,
    pub total_earned: i128,
    pub claimed: i128,
    pub pending: i128,
    pub last_claim_timestamp: u64,
}

/// Configuration for a specific reward type's rate.
///
/// Reward rates can be configured per activity type, allowing
/// different amounts for course completion, contributions, etc.
///
/// # Fields
/// * `reward_type` - Identifier for the reward category (e.g., "course_completion")
/// * `rate` - Amount of tokens rewarded per unit
/// * `enabled` - Whether this reward type is currently active
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardRate {
    pub reward_type: String,
    pub rate: i128,
    pub enabled: bool,
}

// ========== Escrow Types ==========

/// Current state of an escrow transaction.
///
/// Escrows progress through these states based on signer actions
/// and time-based conditions.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    /// Awaiting approval signatures or release conditions
    Pending,
    /// Funds successfully released to beneficiary
    Released,
    /// Funds returned to depositor
    Refunded,
    /// Dispute raised, awaiting arbitrator decision
    Disputed,
    /// Escrow cancelled before any approvals
    Cancelled,
}

/// Multi-signature escrow for secure conditional payments.
///
/// Escrows hold funds until release conditions are met, supporting
/// multiple signers with configurable thresholds, time-based
/// release/refund windows, and dispute arbitration.
///
/// # Fields
/// * `id` - Unique escrow identifier
/// * `depositor` - Address that funded the escrow
/// * `beneficiary` - Address that receives funds on release
/// * `token` - Token address being held
/// * `amount` - Amount held in escrow
/// * `signers` - List of addresses that can approve release
/// * `threshold` - Number of approvals required for release
/// * `approval_count` - Current number of approvals received
/// * `release_time` - Earliest time funds can be released (optional)
/// * `refund_time` - Time after which depositor can request refund (optional)
/// * `arbitrator` - Address authorized to resolve disputes
/// * `status` - Current escrow state
/// * `created_at` - Unix timestamp of escrow creation
/// * `dispute_reason` - Reason provided when dispute was raised (optional)
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

/// Composite key for tracking individual escrow approvals.
///
/// Used in storage to record which signers have approved a specific escrow.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowApprovalKey {
    pub escrow_id: u64,
    pub signer: Address,
}

/// Possible outcomes when an arbitrator resolves a dispute.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeOutcome {
    /// Release funds to the beneficiary (service was provided)
    ReleaseToBeneficiary,
    /// Refund funds to the depositor (service was not provided)
    RefundToDepositor,
}

// ========== Credit Score / Contribution Types ==========

/// Categories of user contributions to the platform.
///
/// Different contribution types may have different reward rates
/// and impact on credit scores.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContributionType {
    /// Educational content creation (courses, materials)
    Content,
    /// Technical contributions (code, tools)
    Code,
    /// Community engagement (mentoring, moderation)
    Community,
    /// Governance participation (proposals, voting)
    Governance,
}

/// Record of a user's contribution to the platform.
///
/// Contributions are tracked to calculate credit scores and
/// determine reward eligibility.
///
/// # Fields
/// * `contributor` - Address of the contributing user
/// * `c_type` - Category of contribution
/// * `description` - Brief description of the contribution
/// * `timestamp` - When the contribution was recorded
/// * `points` - Credit points awarded for this contribution
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contribution {
    pub contributor: Address,
    pub c_type: ContributionType,
    pub description: Bytes,
    pub timestamp: u64,
    pub points: u64,
}

// ========== Reputation Types ==========

/// Comprehensive reputation profile for a user.
///
/// Reputation is calculated from multiple factors including
/// participation, course completion rates, and contribution quality.
/// This data influences access to features and reward multipliers.
///
/// # Fields
/// * `participation_score` - Points accumulated from general platform activity
/// * `completion_rate` - Course completion percentage in basis points (0-10000 = 0-100%)
/// * `contribution_quality` - Average rating of contributions (0-500 = 0-5 stars)
/// * `total_courses_started` - Number of courses the user has enrolled in
/// * `total_courses_completed` - Number of courses successfully completed
/// * `total_contributions` - Count of all contributions made
/// * `last_update` - Unix timestamp of the last reputation update
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

// ========== Educational Content Tokenization Types ==========

/// Categories of educational content that can be tokenized.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentType {
    /// Complete educational course
    Course,
    /// Supplementary learning material
    Material,
    /// Individual lesson or module
    Lesson,
    /// Quiz, test, or certification exam
    Assessment,
    /// Proof of completion or achievement
    Certificate,
}

/// Metadata associated with tokenized educational content.
///
/// Contains all descriptive information about a content token,
/// including creator attribution and content identification.
///
/// # Fields
/// * `title` - Human-readable content title
/// * `description` - Detailed description of the content
/// * `content_type` - Category of educational content
/// * `creator` - Address of the original content creator
/// * `content_hash` - IPFS CID or other content identifier for verification
/// * `license_type` - License terms (e.g., "CC-BY-4.0", "All Rights Reserved")
/// * `tags` - Searchable keywords/categories
/// * `created_at` - Unix timestamp of content creation
/// * `updated_at` - Unix timestamp of last metadata update
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

/// Non-fungible token representing educational content ownership.
///
/// Content tokens enable ownership, licensing, and trading of
/// educational materials on the Stellar network.
///
/// # Fields
/// * `token_id` - Unique identifier for this token
/// * `metadata` - Associated content metadata
/// * `owner` - Current owner's address
/// * `minted_at` - Unix timestamp when the token was created
/// * `is_transferable` - Whether ownership can be transferred
/// * `royalty_percentage` - Creator royalty in basis points (500 = 5%)
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

/// Record of a content token ownership change for provenance tracking.
///
/// Every transfer is recorded to maintain a complete chain of custody,
/// enabling verification of authentic ownership history.
///
/// # Fields
/// * `token_id` - Token being transferred
/// * `from` - Previous owner (None for initial mint)
/// * `to` - New owner
/// * `timestamp` - When the transfer occurred
/// * `transaction_hash` - On-chain transaction reference
/// * `transfer_type` - Category of transfer
/// * `notes` - Optional transfer details or comments
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

/// Types of content token transfers for provenance records.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransferType {
    /// Initial token creation
    Mint,
    /// Standard ownership transfer
    Transfer,
    /// Licensing agreement (owner retains ownership)
    License,
    /// Ownership revoked (e.g., terms violation)
    Revoke,
}
