#![allow(clippy::all)]
#![allow(unused)]

//! TeachLink Smart Contract
//!
//! A comprehensive Soroban smart contract for the TeachLink decentralized
//! knowledge-sharing platform on the Stellar network.
//!
//! # Overview
//!
//! TeachLink provides the following core features:
//!
//! - **Cross-Chain Bridge**: Bridge tokens between Stellar and other blockchains
//! - **Advanced BFT Consensus**: Byzantine Fault Tolerant validator consensus
//! - **Validator Slashing**: Economic penalties for malicious validators
//! - **Multi-Chain Support**: Support for multiple blockchain networks
//! - **Liquidity Optimization**: AMM and dynamic fee pricing
//! - **Message Passing**: Guaranteed cross-chain message delivery
//! - **Emergency Controls**: Circuit breaker and pause mechanisms
//! - **Atomic Swaps**: Cross-chain token exchanges
//! - **Audit & Compliance**: Comprehensive logging and reporting
//! - **Token Rewards**: Incentivize learning and contributions with token rewards
//! - **Multi-Sig Escrow**: Secure payments with multi-signature escrow and arbitration
//! - **Content Tokenization**: Mint NFTs representing educational content ownership
//! - **Provenance Tracking**: Full chain-of-custody for content tokens
//! - **User Reputation**: Track user participation, completion rates, and contribution quality
//! - **Credit Scoring**: Calculate user credit scores based on courses and contributions
//!
//! # Contract Modules
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`bridge`] | Cross-chain token bridging with validator consensus |
//! | [`bft_consensus`] | Byzantine Fault Tolerant consensus mechanism |
//! | [`slashing`] | Validator slashing and reward mechanisms |
//! | [`multichain`] | Multi-chain support and asset management |
//! | [`liquidity`] | Bridge liquidity pools and AMM |
//! | [`message_passing`] | Cross-chain message passing |
//! | [`emergency`] | Emergency pause and circuit breaker |
//! | [`audit`] | Audit trail and compliance reporting |
//! | [`atomic_swap`] | Cross-chain atomic swaps |
//! | [`analytics`] | Bridge monitoring and analytics |
//! | [`rewards`] | Reward pool management and distribution |
//! | [`escrow`] | Multi-signature escrow with dispute resolution |
//! | [`tokenization`] | Educational content NFT minting and management |
//! | [`provenance`] | Ownership history tracking for content tokens |
//! | [`reputation`] | User reputation scoring system |
//! | [`score`] | Credit score calculation from activities |
//!
//! # Quick Start
//!
//! ```ignore
//! // Initialize the contract
//! TeachLinkBridge::initialize(env, token, admin, min_validators, fee_recipient);
//!
//! // Register a validator with BFT consensus
//! TeachLinkBridge::register_validator(env, validator, stake);
//!
//! // Add a supported chain
//! TeachLinkBridge::add_supported_chain_config(env, chain_id, chain_name, bridge_address);
//!
//! // Bridge tokens with advanced features
//! let nonce = TeachLinkBridge::bridge_out(env, from, amount, destination_chain, destination_address);
//!
//! // Create atomic swap
//! let swap_id = TeachLinkBridge::initiate_atomic_swap(env, params);
//! ```
//!
//! # Authorization
//!
//! Most state-changing functions require authorization:
//! - Admin functions require the admin address
//! - User functions require the user's address
//! - Validator functions require validator authorization
//! - Escrow functions require appropriate party authorization

#![no_std]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::needless_borrow)]

use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Map, String, Vec, Symbol};

mod analytics;
mod arbitration;
mod assessment;
mod atomic_swap;
mod audit;
mod bft_consensus;
mod bridge;
mod emergency;
mod errors;
mod escrow;
mod escrow_analytics;
mod events;
// TODO: Implement governance module
// mod governance;
mod liquidity;
mod message_passing;
mod multichain;
mod notification;
mod notification_events_basic;
// mod notification_tests; // TODO: Re-enable when testutils dependencies are resolved
mod notification_types;
mod rewards;
mod slashing;
// mod social_events;
mod social_learning;
mod storage;
mod tokenization;
mod types;
pub mod validation;

pub use errors::{BridgeError, EscrowError, RewardsError};
pub use types::{
    ArbitratorProfile, AtomicSwap, AuditRecord, BridgeMetrics, BridgeProposal, BridgeTransaction,
    ChainConfig, ChainMetrics, ComplianceReport, ConsensusState, ContentMetadata, ContentToken,
    ContentTokenParameters, CrossChainMessage, CrossChainPacket, DisputeOutcome, EmergencyState,
    Escrow, EscrowMetrics, EscrowParameters, EscrowStatus, LiquidityPool, MultiChainAsset,
    NotificationChannel, NotificationContent, NotificationPreference, NotificationSchedule,
    NotificationTemplate, NotificationTracking, OperationType, PacketStatus, ProposalStatus,
    ProvenanceRecord, RewardRate, RewardType, SlashingReason, SlashingRecord, SwapStatus,
    TransferType, UserNotificationSettings, UserReputation, UserReward, ValidatorInfo,
    ValidatorReward, ValidatorSignature,
};
pub use assessment::{Assessment, AssessmentSettings, AssessmentSubmission, Question, QuestionType};

/// TeachLink main contract.
///
/// This contract provides entry points for all TeachLink functionality
/// including bridging, rewards, escrow, tokenization, and reputation.
#[contract]
pub struct TeachLinkBridge;

#[contractimpl]
impl TeachLinkBridge {
    /// Initialize the bridge contract
    pub fn initialize(
        env: Env,
        token: Address,
        admin: Address,
        min_validators: u32,
        fee_recipient: Address,
    ) -> Result<(), BridgeError> {
        bridge::Bridge::initialize(&env, token, admin, min_validators, fee_recipient)
    }

    /// Bridge tokens out to another chain (lock/burn tokens on Stellar)
    pub fn bridge_out(
        env: Env,
        from: Address,
        amount: i128,
        destination_chain: u32,
        destination_address: Bytes,
    ) -> Result<u64, BridgeError> {
        bridge::Bridge::bridge_out(&env, from, amount, destination_chain, destination_address)
    }

    /// Complete a bridge transaction (mint/release tokens on Stellar)
    pub fn complete_bridge(
        env: Env,
        message: CrossChainMessage,
        validator_signatures: Vec<Address>,
    ) -> Result<(), BridgeError> {
        bridge::Bridge::complete_bridge(&env, message, validator_signatures)
    }

    /// Cancel a bridge transaction and refund locked tokens
    pub fn cancel_bridge(env: Env, nonce: u64) -> Result<(), BridgeError> {
        bridge::Bridge::cancel_bridge(&env, nonce)
    }

    // ========== Admin Functions ==========

    /// Add a validator (admin only)
    pub fn add_validator(env: Env, validator: Address) {
        let _ = bridge::Bridge::add_validator(&env, validator);
    }

    /// Remove a validator (admin only)
    pub fn remove_validator(env: Env, validator: Address) {
        let _ = bridge::Bridge::remove_validator(&env, validator);
    }

    /// Add a supported destination chain (admin only)
    pub fn add_supported_chain(env: Env, chain_id: u32) {
        let _ = bridge::Bridge::add_supported_chain(&env, chain_id);
    }

    /// Remove a supported destination chain (admin only)
    pub fn remove_supported_chain(env: Env, chain_id: u32) {
        let _ = bridge::Bridge::remove_supported_chain(&env, chain_id);
    }

    /// Set bridge fee (admin only)
    pub fn set_bridge_fee(env: Env, fee: i128) -> Result<(), BridgeError> {
        bridge::Bridge::set_bridge_fee(&env, fee)
    }

    /// Set fee recipient (admin only)
    pub fn set_fee_recipient(env: Env, fee_recipient: Address) {
        let _ = bridge::Bridge::set_fee_recipient(&env, fee_recipient);
    }

    /// Set minimum validators (admin only)
    pub fn set_min_validators(env: Env, min_validators: u32) -> Result<(), BridgeError> {
        bridge::Bridge::set_min_validators(&env, min_validators)
    }

    // ========== View Functions ==========

    /// Get the bridge transaction by nonce
    pub fn get_bridge_transaction(env: Env, nonce: u64) -> Option<BridgeTransaction> {
        bridge::Bridge::get_bridge_transaction(&env, nonce)
    }

    /// Check if a chain is supported
    pub fn is_chain_supported(env: Env, chain_id: u32) -> bool {
        bridge::Bridge::is_chain_supported(&env, chain_id)
    }

    /// Check if an address is a validator
    pub fn is_validator(env: Env, address: Address) -> bool {
        bridge::Bridge::is_validator(&env, address)
    }

    /// Get the current nonce
    pub fn get_nonce(env: Env) -> u64 {
        bridge::Bridge::get_nonce(&env)
    }

    /// Get the bridge fee
    pub fn get_bridge_fee(env: Env) -> i128 {
        bridge::Bridge::get_bridge_fee(&env)
    }

    /// Get the token address
    pub fn get_token(env: Env) -> Address {
        bridge::Bridge::get_token(&env)
    }

    /// Get the admin address
    pub fn get_admin(env: Env) -> Address {
        bridge::Bridge::get_admin(&env)
    }

    // ========== BFT Consensus Functions ==========

    /// Register a validator with stake for BFT consensus
    pub fn register_validator(
        env: Env,
        validator: Address,
        stake: i128,
    ) -> Result<(), BridgeError> {
        bft_consensus::BFTConsensus::register_validator(&env, validator, stake)
    }

    /// Unregister a validator and unstake
    pub fn unregister_validator(env: Env, validator: Address) -> Result<(), BridgeError> {
        bft_consensus::BFTConsensus::unregister_validator(&env, validator)
    }

    /// Create a bridge proposal for BFT consensus
    pub fn create_bridge_proposal(
        env: Env,
        message: CrossChainMessage,
    ) -> Result<u64, BridgeError> {
        bft_consensus::BFTConsensus::create_proposal(&env, message)
    }

    /// Vote on a bridge proposal
    pub fn vote_on_proposal(
        env: Env,
        validator: Address,
        proposal_id: u64,
        approve: bool,
    ) -> Result<(), BridgeError> {
        bft_consensus::BFTConsensus::vote_on_proposal(&env, validator, proposal_id, approve)
    }

    /// Get validator information
    pub fn get_validator_info(env: Env, validator: Address) -> Option<ValidatorInfo> {
        bft_consensus::BFTConsensus::get_validator_info(&env, validator)
    }

    /// Get consensus state
    pub fn get_consensus_state(env: Env) -> ConsensusState {
        bft_consensus::BFTConsensus::get_consensus_state(&env)
    }

    /// Get proposal by ID
    pub fn get_proposal(env: Env, proposal_id: u64) -> Option<BridgeProposal> {
        bft_consensus::BFTConsensus::get_proposal(&env, proposal_id)
    }

    /// Check if consensus is reached for a proposal
    pub fn is_consensus_reached(env: Env, proposal_id: u64) -> bool {
        bft_consensus::BFTConsensus::is_consensus_reached(&env, proposal_id)
    }

    // ========== Slashing and Rewards Functions ==========

    /// Deposit stake for a validator
    pub fn deposit_stake(env: Env, validator: Address, amount: i128) -> Result<(), BridgeError> {
        slashing::SlashingManager::deposit_stake(&env, validator, amount)
    }

    /// Withdraw stake
    pub fn withdraw_stake(env: Env, validator: Address, amount: i128) -> Result<(), BridgeError> {
        slashing::SlashingManager::withdraw_stake(&env, validator, amount)
    }

    /// Slash a validator for malicious behavior
    pub fn slash_validator(
        env: Env,
        validator: Address,
        reason: types::SlashingReason,
        evidence: Bytes,
        slasher: Address,
    ) -> Result<i128, BridgeError> {
        slashing::SlashingManager::slash_validator(&env, validator, reason, evidence, slasher)
    }

    /// Reward a validator
    pub fn reward_validator(
        env: Env,
        validator: Address,
        amount: i128,
        reward_type: types::RewardType,
    ) -> Result<(), BridgeError> {
        slashing::SlashingManager::reward_validator(&env, validator, amount, reward_type)
    }

    /// Fund the reward pool
    pub fn fund_validator_reward_pool(
        env: Env,
        funder: Address,
        amount: i128,
    ) -> Result<(), BridgeError> {
        slashing::SlashingManager::fund_reward_pool(&env, funder, amount)
    }

    /// Get validator stake
    pub fn get_validator_stake(env: Env, validator: Address) -> i128 {
        slashing::SlashingManager::get_stake(&env, validator)
    }

    // ========== Multi-Chain Functions ==========

    /// Add a supported chain with configuration
    pub fn add_supported_chain_config(
        env: Env,
        chain_id: u32,
        chain_name: Bytes,
        bridge_contract_address: Bytes,
        confirmation_blocks: u32,
        gas_price: u64,
    ) -> Result<(), BridgeError> {
        multichain::MultiChainManager::add_chain(
            &env,
            chain_id,
            chain_name,
            bridge_contract_address,
            confirmation_blocks,
            gas_price,
        )
    }

    /// Update chain configuration
    pub fn update_chain_config(
        env: Env,
        chain_id: u32,
        is_active: bool,
        confirmation_blocks: Option<u32>,
        gas_price: Option<u64>,
    ) -> Result<(), BridgeError> {
        multichain::MultiChainManager::update_chain(
            &env,
            chain_id,
            is_active,
            confirmation_blocks,
            gas_price,
        )
    }

    /// Register a multi-chain asset
    pub fn register_multi_chain_asset(
        env: Env,
        asset_id: Bytes,
        stellar_token: Address,
        chain_configs: Map<u32, types::ChainAssetInfo>,
    ) -> Result<u64, BridgeError> {
        multichain::MultiChainManager::register_asset(&env, asset_id, stellar_token, chain_configs)
    }

    /// Get chain configuration
    pub fn get_chain_config(env: Env, chain_id: u32) -> Option<ChainConfig> {
        multichain::MultiChainManager::get_chain_config(&env, chain_id)
    }

    /// Check if chain is active
    pub fn is_chain_active(env: Env, chain_id: u32) -> bool {
        multichain::MultiChainManager::is_chain_active(&env, chain_id)
    }

    /// Get supported chains
    pub fn get_supported_chains(env: Env) -> Vec<u32> {
        multichain::MultiChainManager::get_supported_chains(&env)
    }

    // ========== Liquidity and AMM Functions ==========

    /// Initialize liquidity pool for a chain
    pub fn initialize_liquidity_pool(
        env: Env,
        chain_id: u32,
        token: Address,
    ) -> Result<(), BridgeError> {
        liquidity::LiquidityManager::initialize_pool(&env, chain_id, token)
    }

    /// Add liquidity to a pool
    pub fn add_liquidity(
        env: Env,
        provider: Address,
        chain_id: u32,
        amount: i128,
    ) -> Result<u32, BridgeError> {
        liquidity::LiquidityManager::add_liquidity(&env, provider, chain_id, amount)
    }

    /// Remove liquidity from a pool
    pub fn remove_liquidity(
        env: Env,
        provider: Address,
        chain_id: u32,
        amount: i128,
    ) -> Result<i128, BridgeError> {
        liquidity::LiquidityManager::remove_liquidity(&env, provider, chain_id, amount)
    }

    /// Calculate dynamic bridge fee
    pub fn calculate_bridge_fee(
        env: Env,
        chain_id: u32,
        amount: i128,
        user_volume_24h: i128,
    ) -> Result<i128, BridgeError> {
        liquidity::LiquidityManager::calculate_bridge_fee(&env, chain_id, amount, user_volume_24h)
    }

    /// Update fee structure
    pub fn update_fee_structure(
        env: Env,
        base_fee: i128,
        dynamic_multiplier: u32,
        volume_discount_tiers: Map<u32, u32>,
    ) -> Result<(), BridgeError> {
        liquidity::LiquidityManager::update_fee_structure(
            &env,
            base_fee,
            dynamic_multiplier,
            volume_discount_tiers,
        )
    }

    /// Get available liquidity for a chain
    pub fn get_available_liquidity(env: Env, chain_id: u32) -> i128 {
        liquidity::LiquidityManager::get_available_liquidity(&env, chain_id)
    }

    // ========== Message Passing Functions ==========

    /// Send a cross-chain packet
    pub fn send_cross_chain_packet(
        env: Env,
        source_chain: u32,
        destination_chain: u32,
        sender: Bytes,
        recipient: Bytes,
        payload: Bytes,
        timeout: Option<u64>,
    ) -> Result<u64, BridgeError> {
        message_passing::MessagePassing::send_packet(
            &env,
            source_chain,
            destination_chain,
            sender,
            recipient,
            payload,
            timeout,
        )
    }

    /// Get packet by ID
    pub fn get_packet(env: Env, packet_id: u64) -> Option<CrossChainPacket> {
        message_passing::MessagePassing::get_packet(&env, packet_id)
    }

    /// Get packet receipt
    pub fn get_packet_receipt(env: Env, packet_id: u64) -> Option<types::MessageReceipt> {
        message_passing::MessagePassing::get_receipt(&env, packet_id)
    }

    /// Verify packet delivery
    pub fn verify_packet_delivery(env: Env, packet_id: u64) -> bool {
        message_passing::MessagePassing::verify_delivery(&env, packet_id)
    }

    // ========== Emergency Functions ==========

    /// Pause the entire bridge
    pub fn pause_bridge(env: Env, pauser: Address, reason: Bytes) -> Result<(), BridgeError> {
        emergency::EmergencyManager::pause_bridge(&env, pauser, reason)
    }

    /// Resume the bridge
    pub fn resume_bridge(env: Env, resumer: Address) -> Result<(), BridgeError> {
        emergency::EmergencyManager::resume_bridge(&env, resumer)
    }

    /// Pause specific chains
    pub fn pause_chains(
        env: Env,
        pauser: Address,
        chain_ids: Vec<u32>,
        reason: Bytes,
    ) -> Result<(), BridgeError> {
        emergency::EmergencyManager::pause_chains(&env, pauser, chain_ids, reason)
    }

    /// Resume specific chains
    pub fn resume_chains(
        env: Env,
        resumer: Address,
        chain_ids: Vec<u32>,
    ) -> Result<(), BridgeError> {
        emergency::EmergencyManager::resume_chains(&env, resumer, chain_ids)
    }

    /// Initialize circuit breaker for a chain
    pub fn initialize_circuit_breaker(
        env: Env,
        chain_id: u32,
        max_daily_volume: i128,
        max_transaction_amount: i128,
    ) -> Result<(), BridgeError> {
        emergency::EmergencyManager::initialize_circuit_breaker(
            &env,
            chain_id,
            max_daily_volume,
            max_transaction_amount,
        )
    }

    /// Check if bridge is paused
    pub fn is_bridge_paused(env: Env) -> bool {
        emergency::EmergencyManager::is_bridge_paused(&env)
    }

    /// Check if a chain is paused
    pub fn is_chain_paused(env: Env, chain_id: u32) -> bool {
        emergency::EmergencyManager::is_chain_paused(&env, chain_id)
    }

    /// Get emergency state
    pub fn get_emergency_state(env: Env) -> EmergencyState {
        emergency::EmergencyManager::get_emergency_state(&env)
    }

    // ========== Audit and Compliance Functions ==========

    /// Create an audit record
    pub fn create_audit_record(
        env: Env,
        operation_type: types::OperationType,
        operator: Address,
        details: Bytes,
        tx_hash: Bytes,
    ) -> Result<u64, BridgeError> {
        audit::AuditManager::create_audit_record(&env, operation_type, operator, details, tx_hash)
    }

    /// Get audit record by ID
    pub fn get_audit_record(env: Env, record_id: u64) -> Option<AuditRecord> {
        audit::AuditManager::get_audit_record(&env, record_id)
    }

    /// Generate compliance report
    pub fn generate_compliance_report(
        env: Env,
        period_start: u64,
        period_end: u64,
    ) -> Result<u64, BridgeError> {
        audit::AuditManager::generate_compliance_report(&env, period_start, period_end)
    }

    /// Get compliance report
    pub fn get_compliance_report(env: Env, report_id: u64) -> Option<ComplianceReport> {
        audit::AuditManager::get_compliance_report(&env, report_id)
    }

    // ========== Atomic Swap Functions ==========

    /// Initiate an atomic swap
    pub fn initiate_atomic_swap(
        env: Env,
        initiator: Address,
        initiator_token: Address,
        initiator_amount: i128,
        counterparty: Address,
        counterparty_token: Address,
        counterparty_amount: i128,
        hashlock: Bytes,
        timelock: u64,
    ) -> Result<u64, BridgeError> {
        atomic_swap::AtomicSwapManager::initiate_swap(
            &env,
            initiator,
            initiator_token,
            initiator_amount,
            counterparty,
            counterparty_token,
            counterparty_amount,
            hashlock,
            timelock,
        )
    }

    /// Accept and complete an atomic swap
    pub fn accept_atomic_swap(
        env: Env,
        swap_id: u64,
        counterparty: Address,
        preimage: Bytes,
    ) -> Result<(), BridgeError> {
        atomic_swap::AtomicSwapManager::accept_swap(&env, swap_id, counterparty, preimage)
    }

    /// Refund an expired atomic swap
    pub fn refund_atomic_swap(
        env: Env,
        swap_id: u64,
        initiator: Address,
    ) -> Result<(), BridgeError> {
        atomic_swap::AtomicSwapManager::refund_swap(&env, swap_id, initiator)
    }

    /// Get atomic swap by ID
    pub fn get_atomic_swap(env: Env, swap_id: u64) -> Option<AtomicSwap> {
        atomic_swap::AtomicSwapManager::get_swap(&env, swap_id)
    }

    /// Get active atomic swaps
    pub fn get_active_atomic_swaps(env: Env) -> Vec<u64> {
        atomic_swap::AtomicSwapManager::get_active_swaps(&env)
    }

    // ========== Analytics Functions ==========

    /// Initialize bridge metrics
    pub fn initialize_bridge_metrics(env: Env) -> Result<(), BridgeError> {
        analytics::AnalyticsManager::initialize_metrics(&env)
    }

    /// Get bridge metrics
    pub fn get_bridge_metrics(env: Env) -> BridgeMetrics {
        analytics::AnalyticsManager::get_bridge_metrics(&env)
    }

    /// Get chain metrics
    pub fn get_chain_metrics(env: Env, chain_id: u32) -> Option<ChainMetrics> {
        analytics::AnalyticsManager::get_chain_metrics(&env, chain_id)
    }

    /// Calculate bridge health score
    pub fn calculate_bridge_health_score(env: Env) -> u32 {
        analytics::AnalyticsManager::calculate_health_score(&env)
    }

    /// Get bridge statistics
    pub fn get_bridge_statistics(env: Env) -> Map<Bytes, i128> {
        analytics::AnalyticsManager::get_bridge_statistics(&env)
    }

    // ========== Rewards Functions ==========

    /// Initialize the rewards system
    pub fn initialize_rewards(
        env: Env,
        token: Address,
        rewards_admin: Address,
    ) -> Result<(), RewardsError> {
        rewards::Rewards::initialize_rewards(&env, token, rewards_admin)
    }

    /// Fund the reward pool
    pub fn fund_reward_pool(env: Env, funder: Address, amount: i128) -> Result<(), RewardsError> {
        rewards::Rewards::fund_reward_pool(&env, funder, amount)
    }

    /// Issue rewards to a user
    pub fn issue_reward(
        env: Env,
        recipient: Address,
        amount: i128,
        reward_type: String,
    ) -> Result<(), RewardsError> {
        rewards::Rewards::issue_reward(&env, recipient, amount, reward_type)
    }

    /// Claim pending rewards
    pub fn claim_rewards(env: Env, user: Address) -> Result<(), RewardsError> {
        rewards::Rewards::claim_rewards(&env, user)
    }

    /// Set reward rate for a specific reward type (admin only)
    pub fn set_reward_rate(
        env: Env,
        reward_type: String,
        rate: i128,
        enabled: bool,
    ) -> Result<(), RewardsError> {
        rewards::Rewards::set_reward_rate(&env, reward_type, rate, enabled)
    }

    /// Update rewards admin (admin only)
    pub fn update_rewards_admin(env: Env, new_admin: Address) {
        rewards::Rewards::update_rewards_admin(&env, new_admin);
    }

    /// Get user reward information
    pub fn get_user_rewards(env: Env, user: Address) -> Option<UserReward> {
        rewards::Rewards::get_user_rewards(&env, user)
    }

    /// Get reward pool balance
    pub fn get_reward_pool_balance(env: Env) -> i128 {
        rewards::Rewards::get_reward_pool_balance(&env)
    }

    /// Get total rewards issued
    pub fn get_total_rewards_issued(env: Env) -> i128 {
        rewards::Rewards::get_total_rewards_issued(&env)
    }

    /// Get reward rate for a specific type
    pub fn get_reward_rate(env: Env, reward_type: String) -> Option<RewardRate> {
        rewards::Rewards::get_reward_rate(&env, reward_type)
    }

    /// Get rewards admin address
    pub fn get_rewards_admin(env: Env) -> Address {
        rewards::Rewards::get_rewards_admin(&env)
    }

    // ========== Assessment and Testing Platform Functions ==========

    /// Create a new assessment
    pub fn create_assessment(
        env: Env,
        creator: Address,
        title: Bytes,
        description: Bytes,
        questions: Vec<u64>,
        settings: AssessmentSettings,
    ) -> Result<u64, assessment::AssessmentError> {
        assessment::AssessmentManager::create_assessment(
            &env,
            creator,
            title,
            description,
            questions,
            settings,
        )
    }

    /// Add a question to the pool
    pub fn add_assessment_question(
        env: Env,
        creator: Address,
        q_type: QuestionType,
        content_hash: Bytes,
        points: u32,
        difficulty: u32,
        correct_answer_hash: Bytes,
        metadata: Map<Symbol, Bytes>,
    ) -> Result<u64, assessment::AssessmentError> {
        assessment::AssessmentManager::add_question(
            &env,
            creator,
            q_type,
            content_hash,
            points,
            difficulty,
            correct_answer_hash,
            metadata,
        )
    }

    /// Submit an assessment
    pub fn submit_assessment(
        env: Env,
        student: Address,
        assessment_id: u64,
        answers: Map<u64, Bytes>,
        proctor_logs: Vec<Bytes>,
    ) -> Result<u32, assessment::AssessmentError> {
        assessment::AssessmentManager::submit_assessment(
            &env,
            student,
            assessment_id,
            answers,
            proctor_logs,
        )
    }

    /// Get assessment details
    pub fn get_assessment(env: Env, id: u64) -> Option<Assessment> {
        assessment::AssessmentManager::get_assessment(&env, id)
    }

    /// Get user submission
    pub fn get_assessment_submission(
        env: Env,
        student: Address,
        assessment_id: u64,
    ) -> Option<AssessmentSubmission> {
        assessment::AssessmentManager::get_submission(&env, student, assessment_id)
    }

    /// Report a proctoring violation
    pub fn report_proctor_violation(
        env: Env,
        student: Address,
        assessment_id: u64,
        violation_type: Bytes,
    ) -> Result<(), assessment::AssessmentError> {
        assessment::AssessmentManager::report_proctoring_violation(
            &env,
            student,
            assessment_id,
            violation_type,
        )
    }

    /// Get next adaptive question
    pub fn get_next_adaptive_question(
        env: Env,
        id: u64,
        scores: Vec<u32>,
        answered_ids: Vec<u64>,
    ) -> Result<u64, assessment::AssessmentError> {
        assessment::AssessmentManager::get_next_adaptive_question(
            &env,
            id,
            scores,
            answered_ids,
        )
    }

    // ========== Escrow Functions ==========

    /// Create a multi-signature escrow
    pub fn create_escrow(env: Env, params: EscrowParameters) -> Result<u64, EscrowError> {
        escrow::EscrowManager::create_escrow(
            &env,
            params.depositor,
            params.beneficiary,
            params.token,
            params.amount,
            params.signers,
            params.threshold,
            params.release_time,
            params.refund_time,
            params.arbitrator,
        )
    }

    /// Approve escrow release (multi-signature)
    pub fn approve_escrow_release(
        env: Env,
        escrow_id: u64,
        signer: Address,
    ) -> Result<u32, EscrowError> {
        escrow::EscrowManager::approve_release(&env, escrow_id, signer)
    }

    /// Release funds to the beneficiary once conditions are met
    pub fn release_escrow(env: Env, escrow_id: u64, caller: Address) -> Result<(), EscrowError> {
        escrow::EscrowManager::release(&env, escrow_id, caller)
    }

    /// Refund escrow to the depositor after refund time
    pub fn refund_escrow(env: Env, escrow_id: u64, depositor: Address) -> Result<(), EscrowError> {
        escrow::EscrowManager::refund(&env, escrow_id, depositor)
    }

    /// Cancel escrow before any approvals
    pub fn cancel_escrow(env: Env, escrow_id: u64, depositor: Address) -> Result<(), EscrowError> {
        escrow::EscrowManager::cancel(&env, escrow_id, depositor)
    }

    /// Raise a dispute on the escrow
    pub fn dispute_escrow(
        env: Env,
        escrow_id: u64,
        disputer: Address,
        reason: Bytes,
    ) -> Result<(), EscrowError> {
        escrow::EscrowManager::dispute(&env, escrow_id, disputer, reason)
    }

    /// Automatically check if an escrow has stalled and trigger a dispute
    pub fn auto_check_escrow_dispute(env: Env, escrow_id: u64) -> Result<(), EscrowError> {
        escrow::EscrowManager::auto_check_dispute(&env, escrow_id)
    }

    /// Resolve a dispute as the arbitrator
    pub fn resolve_escrow(
        env: Env,
        escrow_id: u64,
        arbitrator: Address,
        outcome: DisputeOutcome,
    ) -> Result<(), EscrowError> {
        escrow::EscrowManager::resolve(&env, escrow_id, arbitrator, outcome)
    }

    // ========== Arbitration Management Functions ==========

    /// Register a new professional arbitrator
    pub fn register_arbitrator(env: Env, profile: ArbitratorProfile) -> Result<(), EscrowError> {
        arbitration::ArbitrationManager::register_arbitrator(&env, profile)
    }

    /// Update arbitrator profile
    pub fn update_arbitrator_profile(
        env: Env,
        address: Address,
        profile: ArbitratorProfile,
    ) -> Result<(), EscrowError> {
        arbitration::ArbitrationManager::update_profile(&env, address, profile)
    }

    /// Get arbitrator profile
    pub fn get_arbitrator_profile(env: Env, address: Address) -> Option<ArbitratorProfile> {
        arbitration::ArbitrationManager::get_arbitrator(&env, address)
    }

    // ========== Insurance Pool Functions ==========

    // TODO: Implement insurance module
    /*
    /// Initialize insurance pool
    pub fn initialize_insurance_pool(
        env: Env,
        token: Address,
        premium_rate: u32,
    ) -> Result<(), BridgeError> {
        insurance::InsuranceManager::initialize_pool(&env, token, premium_rate)
    }

    /// Fund insurance pool
    pub fn fund_insurance_pool(env: Env, funder: Address, amount: i128) -> Result<(), BridgeError> {
        insurance::InsuranceManager::fund_pool(&env, funder, amount)
    }
    */

    // ========== Escrow Analytics Functions ==========

    /// Get aggregate escrow metrics
    pub fn get_escrow_metrics(env: Env) -> EscrowMetrics {
        escrow_analytics::EscrowAnalyticsManager::get_metrics(&env)
    }

    /// Get escrow by id
    pub fn get_escrow(env: Env, escrow_id: u64) -> Option<Escrow> {
        escrow::EscrowManager::get_escrow(&env, escrow_id)
    }

    /// Check if a signer approved
    pub fn has_escrow_approval(env: Env, escrow_id: u64, signer: Address) -> bool {
        escrow::EscrowManager::has_approved(&env, escrow_id, signer)
    }

    /// Get the current escrow count
    pub fn get_escrow_count(env: Env) -> u64 {
        escrow::EscrowManager::get_escrow_count(&env)
    }

    // ========== Credit Scoring Functions (feat/credit_score) ==========

    // TODO: Implement score module
    /*
    /// Record course completion
    pub fn record_course_completion(
        env: Env,
        user: Address,
        course_id: u64,
        points: u64,
    ) {
        let admin = bridge::Bridge::get_admin(&env);
        admin.require_auth();
        score::ScoreManager::record_course_completion(&env, user, course_id, points);
    }

    /// Record contribution
    pub fn record_contribution(
        env: Env,
        user: Address,
        c_type: types::ContributionType,
        description: Bytes,
        points: u64,
    ) {
        score::ScoreManager::record_contribution(&env, user, c_type, description, points);
    }

    /// Get user's credit score
    pub fn get_credit_score(env: Env, user: Address) -> u64 {
        score::ScoreManager::get_score(&env, user)
    }

    /// Get user's courses
    pub fn get_user_courses(env: Env, user: Address) -> Vec<types::Course> {
        score::ScoreManager::get_courses(&env, user)
    }

    /// Get user's contributions
    pub fn get_user_contributions(env: Env, user: Address) -> Vec<types::Contribution> {
        score::ScoreManager::get_contributions(&env, user)
    }
    */

    // ========== Reputation Functions (main) ==========

    // TODO: Implement missing modules
    /*
    pub fn update_participation(env: Env, user: Address, points: u32) {
        reputation::update_participation(&env, user, points);
    }

    pub fn update_course_progress(env: Env, user: Address, is_completion: bool) {
        reputation::update_course_progress(&env, user, is_completion);
    }

    pub fn rate_contribution(env: Env, user: Address, rating: u32) {
        reputation::rate_contribution(&env, user, rating);
    }

    pub fn get_user_reputation(env: Env, user: Address) -> types::UserReputation {
        reputation::get_reputation(&env, &user)
    }
    */

    // ========== Content Tokenization Functions ==========

    /// Mint a new educational content token
    pub fn mint_content_token(env: Env, params: ContentTokenParameters) -> u64 {
        let token_id = tokenization::ContentTokenization::mint(
            &env,
            params.creator.clone(),
            params.title,
            params.description,
            params.content_type,
            params.content_hash,
            params.license_type,
            params.tags,
            params.is_transferable,
            params.royalty_percentage,
        );
        // TODO: Implement provenance module
        // provenance::ProvenanceTracker::record_mint(&env, token_id, params.creator, None);
        token_id
    }

    /// Transfer ownership of a content token
    pub fn transfer_content_token(
        env: Env,
        from: Address,
        to: Address,
        token_id: u64,
        notes: Option<Bytes>,
    ) {
        tokenization::ContentTokenization::transfer(&env, from, to, token_id, notes);
    }

    /// Get a content token by ID
    pub fn get_content_token(env: Env, token_id: u64) -> Option<ContentToken> {
        tokenization::ContentTokenization::get_token(&env, token_id)
    }

    /// Get the owner of a content token
    pub fn get_content_token_owner(env: Env, token_id: u64) -> Option<Address> {
        tokenization::ContentTokenization::get_owner(&env, token_id)
    }

    /// Check if an address owns a content token
    pub fn is_content_token_owner(env: Env, token_id: u64, address: Address) -> bool {
        tokenization::ContentTokenization::is_owner(&env, token_id, address)
    }

    /// Get all tokens owned by an address
    pub fn get_owner_content_tokens(env: Env, owner: Address) -> Vec<u64> {
        tokenization::ContentTokenization::get_owner_tokens(&env, owner)
    }

    /// Get the total number of content tokens minted
    pub fn get_content_token_count(env: Env) -> u64 {
        tokenization::ContentTokenization::get_token_count(&env)
    }

    /// Update content token metadata (only by owner)
    pub fn update_content_metadata(
        env: Env,
        owner: Address,
        token_id: u64,
        title: Option<Bytes>,
        description: Option<Bytes>,
        tags: Option<Vec<Bytes>>,
    ) {
        tokenization::ContentTokenization::update_metadata(
            &env,
            owner,
            token_id,
            title,
            description,
            tags,
        );
    }

    /// Set transferability of a content token (only by owner)
    pub fn set_content_token_transferable(
        env: Env,
        owner: Address,
        token_id: u64,
        transferable: bool,
    ) {
        tokenization::ContentTokenization::set_transferable(&env, owner, token_id, transferable);
    }

    // ========== Provenance Functions ==========

    // TODO: Implement provenance module
    /*
    /// Get full provenance history for a content token
    pub fn get_content_provenance(env: Env, token_id: u64) -> Vec<ProvenanceRecord> {
        provenance::ProvenanceTracker::get_provenance(&env, token_id)
    }

    /// Get the number of transfers for a content token
    #[must_use]
    pub fn get_content_transfer_count(env: &Env, token_id: u64) -> u32 {
        provenance::ProvenanceTracker::get_transfer_count(env, token_id)
    }

    /// Verify ownership chain integrity for a content token
    #[must_use]
    pub fn verify_content_chain(env: &Env, token_id: u64) -> bool {
        provenance::ProvenanceTracker::verify_chain(env, token_id)
    }
    */

    /// Get the creator of a content token
    #[must_use]
    pub fn get_content_creator(env: &Env, token_id: u64) -> Option<Address> {
        tokenization::ContentTokenization::get_creator(env, token_id)
    }

    /// Get all owners of a content token
    #[must_use]
    pub fn get_content_all_owners(env: &Env, token_id: u64) -> Vec<Address> {
        tokenization::ContentTokenization::get_all_owners(env, token_id)
    }

    // ========== Notification System Functions ==========

    /// Initialize notification system
    pub fn initialize_notifications(env: Env) -> Result<(), BridgeError> {
        notification::NotificationManager::initialize(&env)
    }

    /// Send immediate notification
    pub fn send_notification(
        env: Env,
        recipient: Address,
        channel: NotificationChannel,
        subject: Bytes,
        body: Bytes,
    ) -> Result<u64, BridgeError> {
        let content = NotificationContent {
            subject,
            body,
            data: Bytes::new(&env),
            localization: Map::new(&env),
        };
        notification::NotificationManager::send_notification(&env, recipient, channel, content)
    }

    /// Schedule notification for future delivery
    pub fn schedule_notification(
        env: Env,
        recipient: Address,
        channel: NotificationChannel,
        subject: Bytes,
        body: Bytes,
        scheduled_time: u64,
        timezone: Bytes,
    ) -> Result<u64, BridgeError> {
        let content = NotificationContent {
            subject,
            body,
            data: Bytes::new(&env),
            localization: Map::new(&env),
        };
        let schedule = NotificationSchedule {
            notification_id: 0, // Will be set by the function
            recipient: recipient.clone(),
            channel,
            scheduled_time,
            timezone,
            is_recurring: false,
            recurrence_pattern: 0,
            max_deliveries: None,
            delivery_count: 0,
        };
        notification::NotificationManager::schedule_notification(
            &env, recipient, channel, content, schedule,
        )
    }

    /// Process scheduled notifications
    pub fn process_scheduled_notifications(env: Env) -> Result<u32, BridgeError> {
        notification::NotificationManager::process_scheduled_notifications(&env)
    }

    /// Update user notification preferences
    pub fn update_notification_preferences(
        env: Env,
        user: Address,
        preferences: Vec<NotificationPreference>,
    ) -> Result<(), BridgeError> {
        notification::NotificationManager::update_preferences(&env, user, preferences)
    }

    /// Update user notification settings
    pub fn update_notification_settings(
        env: Env,
        user: Address,
        timezone: Bytes,
        quiet_hours_start: u32,
        quiet_hours_end: u32,
        max_daily_notifications: u32,
        do_not_disturb: bool,
    ) -> Result<(), BridgeError> {
        let settings = UserNotificationSettings {
            user: user.clone(),
            timezone,
            quiet_hours_start,
            quiet_hours_end,
            max_daily_notifications,
            do_not_disturb,
        };
        notification::NotificationManager::update_user_settings(&env, user, settings)
    }

    /// Create notification template
    pub fn create_notification_template(
        env: Env,
        admin: Address,
        name: Bytes,
        channels: Vec<NotificationChannel>,
        subject: Bytes,
        body: Bytes,
    ) -> Result<u64, BridgeError> {
        let content = NotificationContent {
            subject,
            body,
            data: Bytes::new(&env),
            localization: Map::new(&env),
        };
        notification::NotificationManager::create_template(&env, admin, name, channels, content)
    }

    /// Send notification using template
    pub fn send_template_notification(
        env: Env,
        recipient: Address,
        template_id: u64,
        variables: Map<Bytes, Bytes>,
    ) -> Result<u64, BridgeError> {
        notification::NotificationManager::send_template_notification(
            &env,
            recipient,
            template_id,
            variables,
        )
    }

    /// Get notification tracking information
    pub fn get_notification_tracking(
        env: Env,
        notification_id: u64,
    ) -> Option<NotificationTracking> {
        notification::NotificationManager::get_notification_tracking(&env, notification_id)
    }

    /// Get user notification history
    pub fn get_user_notifications(
        env: Env,
        user: Address,
        limit: u32,
    ) -> Vec<NotificationTracking> {
        notification::NotificationManager::get_user_notifications(&env, user, limit)
    }

    // ========== Social Learning Functions ==========

    /// Create a study group
    pub fn create_study_group(
        env: Env,
        creator: Address,
        name: Bytes,
        description: Bytes,
        subject: Bytes,
        max_members: u32,
        is_private: bool,
        tags: Vec<Bytes>,
        settings: social_learning::StudyGroupSettings,
    ) -> Result<u64, BridgeError> {
        social_learning::SocialLearningManager::create_study_group(
            &env, creator, name, description, subject, max_members, is_private, tags, settings,
        ).map_err(|_| BridgeError::InvalidInput)
    }

    /// Join a study group
    pub fn join_study_group(env: Env, user: Address, group_id: u64) -> Result<(), BridgeError> {
        social_learning::SocialLearningManager::join_study_group(&env, user, group_id)
            .map_err(|_| BridgeError::InvalidInput)
    }

    /// Leave a study group
    pub fn leave_study_group(env: Env, user: Address, group_id: u64) -> Result<(), BridgeError> {
        social_learning::SocialLearningManager::leave_study_group(&env, user, group_id)
            .map_err(|_| BridgeError::InvalidInput)
    }

    /// Get study group information
    pub fn get_study_group(env: Env, group_id: u64) -> Result<social_learning::StudyGroup, BridgeError> {
        social_learning::SocialLearningManager::get_study_group(&env, group_id)
            .map_err(|_| BridgeError::InvalidInput)
    }

    /// Get user's study groups
    pub fn get_user_study_groups(env: Env, user: Address) -> Vec<u64> {
        social_learning::SocialLearningManager::get_user_study_groups(&env, user)
    }

    /// Create a discussion forum
    pub fn create_forum(
        env: Env,
        creator: Address,
        title: Bytes,
        description: Bytes,
        category: Bytes,
        tags: Vec<Bytes>,
    ) -> Result<u64, BridgeError> {
        social_learning::SocialLearningManager::create_forum(&env, creator, title, description, category, tags)
            .map_err(|_| BridgeError::InvalidInput)
    }

    /// Create a forum post
    pub fn create_forum_post(
        env: Env,
        forum_id: u64,
        author: Address,
        title: Bytes,
        content: Bytes,
        attachments: Vec<Bytes>,
    ) -> Result<u64, BridgeError> {
        social_learning::SocialLearningManager::create_forum_post(
            &env, forum_id, author, title, content, attachments,
        ).map_err(|_| BridgeError::InvalidInput)
    }

    /// Get forum information
    pub fn get_forum(env: Env, forum_id: u64) -> Result<social_learning::DiscussionForum, BridgeError> {
        social_learning::SocialLearningManager::get_forum(&env, forum_id)
            .map_err(|_| BridgeError::InvalidInput)
    }

    /// Get forum post
    pub fn get_forum_post(env: Env, post_id: u64) -> Result<social_learning::ForumPost, BridgeError> {
        social_learning::SocialLearningManager::get_forum_post(&env, post_id)
            .map_err(|_| BridgeError::InvalidInput)
    }

    /// Create a collaboration workspace
    pub fn create_workspace(
        env: Env,
        creator: Address,
        name: Bytes,
        description: Bytes,
        project_type: social_learning::ProjectType,
        settings: social_learning::WorkspaceSettings,
    ) -> Result<u64, BridgeError> {
        social_learning::SocialLearningManager::create_workspace(
            &env, creator, name, description, project_type, settings,
        ).map_err(|_| BridgeError::InvalidInput)
    }

    /// Get workspace information
    pub fn get_workspace(env: Env, workspace_id: u64) -> Result<social_learning::CollaborationWorkspace, BridgeError> {
        social_learning::SocialLearningManager::get_workspace(&env, workspace_id)
            .map_err(|_| BridgeError::InvalidInput)
    }

    /// Get user's workspaces
    pub fn get_user_workspaces(env: Env, user: Address) -> Vec<u64> {
        social_learning::SocialLearningManager::get_user_workspaces(&env, user)
    }

    /// Create a peer review
    pub fn create_review(
        env: Env,
        reviewer: Address,
        reviewee: Address,
        content_type: social_learning::ReviewContentType,
        content_id: u64,
        rating: u32,
        feedback: Bytes,
        criteria: Map<Bytes, u32>,
    ) -> Result<u64, BridgeError> {
        social_learning::SocialLearningManager::create_review(
            &env, reviewer, reviewee, content_type, content_id, rating, feedback, criteria,
        ).map_err(|_| BridgeError::InvalidInput)
    }

    /// Get review information
    pub fn get_review(env: Env, review_id: u64) -> Result<social_learning::PeerReview, BridgeError> {
        social_learning::SocialLearningManager::get_review(&env, review_id)
            .map_err(|_| BridgeError::InvalidInput)
    }

    /// Create mentorship profile
    pub fn create_mentorship_profile(
        env: Env,
        mentor: Address,
        expertise_areas: Vec<Bytes>,
        experience_level: social_learning::ExperienceLevel,
        availability: social_learning::AvailabilityStatus,
        hourly_rate: Option<u64>,
        bio: Bytes,
        languages: Vec<Bytes>,
        timezone: Bytes,
    ) -> Result<(), BridgeError> {
        social_learning::SocialLearningManager::create_mentorship_profile(
            &env, mentor, expertise_areas, experience_level, availability, hourly_rate, bio, languages, timezone,
        ).map_err(|_| BridgeError::InvalidInput)
    }

    /// Get mentorship profile
    pub fn get_mentorship_profile(env: Env, mentor: Address) -> Result<social_learning::MentorshipProfile, BridgeError> {
        social_learning::SocialLearningManager::get_mentorship_profile(&env, mentor)
            .map_err(|_| BridgeError::InvalidInput)
    }

    /// Get user social analytics
    pub fn get_user_analytics(env: Env, user: Address) -> social_learning::SocialAnalytics {
        social_learning::SocialLearningManager::get_user_analytics(&env, user)
    }

    /// Update user social analytics
    pub fn update_user_analytics(env: Env, user: Address, analytics: social_learning::SocialAnalytics) {
        social_learning::SocialLearningManager::update_user_analytics(&env, user, analytics);
    }

    // Analytics function removed due to contracttype limitations
    // Use internal notification manager for analytics
}
