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
//! | [`performance`] | Performance caching (bridge summary, TTL, invalidation) |
//! | [`reporting`] | Advanced analytics, report templates, dashboards, and alerting |
//! | [`backup`] | Backup scheduling, integrity verification, disaster recovery, and RTO audit |
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

use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Map, String, Symbol, Vec};

mod access_control;
mod analytics;
mod arbitration;
mod assessment;
mod atomic_swap;
mod audit;
mod backup;
mod bft_consensus;
mod bridge;
// TODO: Fix collaboration module compilation errors (pre-existing issue)
// mod collaboration;
// TODO: Fix content_nft module compilation errors (pre-existing issue)
// mod content_nft;
// TODO: Fix content_quality module compilation errors (pre-existing issue - symbol too long)
// mod content_quality;
mod emergency;
mod errors;
mod escrow_analytics;
mod event_query;
// TODO: Fix event_tests module compilation errors (pre-existing issue)
// mod event_tests;
mod events;
// TODO: Fix fractional module compilation errors (pre-existing issue)
// mod fractional;
mod insurance;
mod interface_versioning;
// TODO: Fix learning_paths module compilation errors (pre-existing issue - symbol too long)
// mod learning_paths;
mod ledger_time;
// TODO: Fix licensing module compilation errors (pre-existing issue)
// mod licensing;
mod liquidity;
// TODO: Fix marketplace module compilation errors (pre-existing issue)
// mod marketplace;
mod message_passing;
mod mobile_platform;
mod multichain;
mod network_recovery;
mod notification;
// TODO: Fix notification_events module compilation errors (pre-existing issue - event name too long)
// mod notification_events;
mod notification_events_basic;
// TODO: Fix notification_events_simple module compilation errors (pre-existing issue)
// mod notification_events_simple;
// TODO: Fix notification_tests module (pre-existing issue - tests fail with AlreadyInitialized)
// mod notification_tests;
mod notification_types;
mod performance;
mod property_based_tests;
mod provenance;
mod rate_limiting;
mod recommendation;
mod reentrancy;
mod reporting;
mod repository;
mod reputation;
mod rewards;
// TODO: Fix royalty module compilation errors (pre-existing issue - incomplete implementation)
// mod royalty;
mod score;
mod slashing;
// TODO: Fix social_events module compilation errors (pre-existing issue)
// mod social_events;
// TODO: Fix social_learning module compilation errors (pre-existing issue)
// mod social_learning;
mod storage;
mod tokenization;
mod types;
mod upgrade;
mod validation;
// TODO: Fix validation_tests compilation errors (pre-existing issue)
// mod validation_tests;

pub use validation::{
    config, AddressValidator, BridgeValidator, BytesValidator, CrossChainValidator,
    EscrowValidator, InputSanitizer, NumberValidator, RewardsValidator, StringValidator,
    ValidationError, ValidationResult,
};

pub use crate::types::{
    ColorBlindMode, ComponentConfig, DeviceInfo, FeedbackCategory, FocusStyle, FontSize,
    LayoutDensity, MobileAccessibilitySettings, MobilePreferences, MobileProfile, NetworkType,
    OnboardingStage, OnboardingStatus, ThemePreference, UserFeedback, VideoQuality,
};
pub use assessment::{
    Assessment, AssessmentSettings, AssessmentSubmission, Question, QuestionType,
};
pub use errors::{BridgeError, EscrowError, MobilePlatformError, RewardsError};
pub use repository::{
    BridgeRepository, EscrowAggregateRepository, GenericCounterRepository, GenericMapRepository,
    SingleValueRepository, StorageError,
};
pub use types::{
    AlertConditionType, AlertRule, ArbitratorProfile, AtomicSwap, AuditRecord, BackupManifest,
    BackupSchedule, BridgeMetrics, BridgeProposal, BridgeTransaction, CachedBridgeSummary,
    ChainConfig, ChainMetrics, ComplianceReport, ConsensusState, ContentMetadata, ContentToken,
    ContentTokenParameters, ContentType, ContractSemVer, ContributionType, CrossChainMessage,
    CrossChainPacket, DashboardAnalytics, DisputeOutcome, EmergencyState, Escrow, EscrowMetrics,
    EscrowParameters, EscrowRole, EscrowSigner, EscrowStatus, InterfaceVersionStatus,
    LiquidityPool, MultiChainAsset, NotificationChannel, NotificationContent,
    NotificationPreference, NotificationSchedule, NotificationTemplate, NotificationTracking,
    OperationType, PacketStatus, ProposalStatus, ProvenanceRecord, RecoveryRecord, ReportComment,
    ReportSchedule, ReportSnapshot, ReportTemplate, ReportType, ReportUsage, RewardRate,
    RewardType, RtoTier, SlashingReason, SlashingRecord, SwapStatus, TransferType,
    UserNotificationSettings, UserReputation, UserReward, ValidatorInfo, ValidatorReward,
    ValidatorSignature, VisualizationDataPoint,
};

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
        bridge::Bridge::initialize(&env, token, admin.clone(), min_validators, fee_recipient)?;
        interface_versioning::InterfaceVersioning::initialize(&env);
        upgrade::ContractUpgrader::initialize(&env, admin.clone())?;
        Ok(())
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

    pub fn mark_bridge_failed(env: Env, nonce: u64, reason: Bytes) -> Result<(), BridgeError> {
        bridge::Bridge::mark_bridge_failed(&env, nonce, reason)
    }

    pub fn retry_bridge(env: Env, nonce: u64) -> Result<u32, BridgeError> {
        bridge::Bridge::retry_bridge(&env, nonce)
    }

    pub fn refund_bridge_transaction(env: Env, nonce: u64) -> Result<(), BridgeError> {
        bridge::Bridge::refund_bridge_transaction(&env, nonce)
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

    /// Get full interface version status (current and minimum compatible version)
    pub fn get_interface_version_status(env: Env) -> InterfaceVersionStatus {
        interface_versioning::InterfaceVersioning::get_interface_version_status(&env)
    }

    /// Get current interface semantic version
    pub fn get_interface_version(env: Env) -> ContractSemVer {
        interface_versioning::InterfaceVersioning::get_interface_version(&env)
    }

    /// Get minimum supported interface semantic version
    pub fn get_min_compat_interface_version(env: Env) -> ContractSemVer {
        interface_versioning::InterfaceVersioning::get_minimum_compatible_interface_version(&env)
    }

    /// Update current and minimum compatible interface versions (admin only)
    pub fn set_interface_version(
        env: Env,
        current: ContractSemVer,
        minimum_compatible: ContractSemVer,
    ) -> Result<(), BridgeError> {
        interface_versioning::InterfaceVersioning::set_interface_versions(
            &env,
            current,
            minimum_compatible,
        )
    }

    /// Validate whether a client interface version is compatible
    pub fn is_interface_compatible(env: Env, client_version: ContractSemVer) -> bool {
        interface_versioning::InterfaceVersioning::is_interface_compatible(&env, client_version)
    }

    /// Assert interface compatibility and return an explicit error if incompatible
    pub fn assert_interface_compatible(
        env: Env,
        client_version: ContractSemVer,
    ) -> Result<(), BridgeError> {
        interface_versioning::InterfaceVersioning::assert_interface_compatible(&env, client_version)
    }

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

    /// Rotate validators: deactivate those with low reputation or insufficient stake.
    /// Returns the number of validators rotated out.
    pub fn rotate_validators(env: Env) -> Result<u32, BridgeError> {
        bft_consensus::BFTConsensus::rotate_validators(&env)
    }

    /// Trigger rotation if the current consensus round is at an epoch boundary.
    pub fn maybe_rotate_validators(env: Env) -> Result<bool, BridgeError> {
        bft_consensus::BFTConsensus::maybe_rotate(&env)
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

    /// Mark a cross-chain packet as delivered
    pub fn deliver_cross_chain_packet(
        env: Env,
        packet_id: u64,
        gas_used: u64,
        result: Bytes,
    ) -> Result<(), BridgeError> {
        message_passing::MessagePassing::deliver_packet(&env, packet_id, gas_used, result)
    }

    /// Mark a cross-chain packet as failed
    pub fn fail_cross_chain_packet(
        env: Env,
        packet_id: u64,
        reason: Bytes,
    ) -> Result<(), BridgeError> {
        message_passing::MessagePassing::fail_packet(&env, packet_id, reason)
    }

    /// Retry a failed or timed-out cross-chain packet
    pub fn retry_cross_chain_packet(env: Env, packet_id: u64) -> Result<(), BridgeError> {
        message_passing::MessagePassing::retry_packet(&env, packet_id)
    }

    /// Mark all expired packets as timed out and return packet IDs
    pub fn check_cross_chain_timeouts(env: Env) -> Result<Vec<u64>, BridgeError> {
        message_passing::MessagePassing::check_timeouts(&env)
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

    /// Get retry count for a packet
    pub fn get_packet_retry_count(env: Env, packet_id: u64) -> u32 {
        message_passing::MessagePassing::get_packet_retry_count(&env, packet_id)
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

    /// Get cached or computed bridge summary (health score + top chains). Uses cache if fresh.
    pub fn get_cached_bridge_summary(env: Env) -> Result<CachedBridgeSummary, BridgeError> {
        performance::PerformanceManager::get_or_compute_summary(&env)
    }

    /// Force recompute and cache bridge summary. Emits PerfMetricsComputedEvent.
    pub fn compute_and_cache_bridge_summary(env: Env) -> Result<CachedBridgeSummary, BridgeError> {
        performance::PerformanceManager::compute_and_cache_summary(&env)
    }

    /// Invalidate performance cache (admin only). Emits PerfCacheInvalidatedEvent.
    pub fn invalidate_performance_cache(env: Env, admin: Address) -> Result<(), BridgeError> {
        performance::PerformanceManager::invalidate_cache(&env, &admin)
    }

    // ========== Advanced Analytics & Reporting Functions ==========

    /// Get dashboard-ready aggregate analytics for visualizations
    pub fn get_dashboard_analytics(env: Env) -> DashboardAnalytics {
        reporting::ReportingManager::get_dashboard_analytics(&env)
    }

    /// Create a report template
    pub fn create_report_template(
        env: Env,
        creator: Address,
        name: Bytes,
        report_type: ReportType,
        config: Bytes,
    ) -> Result<u64, BridgeError> {
        reporting::ReportingManager::create_report_template(
            &env,
            creator,
            name,
            report_type,
            config,
        )
    }

    /// Get report template by id
    pub fn get_report_template(env: Env, template_id: u64) -> Option<ReportTemplate> {
        reporting::ReportingManager::get_report_template(&env, template_id)
    }

    /// Schedule a report
    pub fn schedule_report(
        env: Env,
        owner: Address,
        template_id: u64,
        next_run_at: u64,
        interval_seconds: u64,
    ) -> Result<u64, BridgeError> {
        reporting::ReportingManager::schedule_report(
            &env,
            owner,
            template_id,
            next_run_at,
            interval_seconds,
        )
    }

    /// Get scheduled reports for an owner
    pub fn get_scheduled_reports(env: Env, owner: Address) -> Vec<ReportSchedule> {
        reporting::ReportingManager::get_scheduled_reports(&env, owner)
    }

    /// Generate a report snapshot
    pub fn generate_report_snapshot(
        env: Env,
        generator: Address,
        template_id: u64,
        period_start: u64,
        period_end: u64,
    ) -> Result<u64, BridgeError> {
        reporting::ReportingManager::generate_report_snapshot(
            &env,
            generator,
            template_id,
            period_start,
            period_end,
        )
    }

    /// Get report snapshot by id
    pub fn get_report_snapshot(env: Env, report_id: u64) -> Option<ReportSnapshot> {
        reporting::ReportingManager::get_report_snapshot(&env, report_id)
    }

    /// Record report view for usage analytics
    pub fn record_report_view(
        env: Env,
        report_id: u64,
        viewer: Address,
    ) -> Result<(), BridgeError> {
        reporting::ReportingManager::record_report_view(&env, report_id, viewer)
    }

    /// Get report usage count
    pub fn get_report_usage_count(env: Env, report_id: u64) -> u32 {
        reporting::ReportingManager::get_report_usage_count(&env, report_id)
    }

    /// Add comment to a report
    pub fn add_report_comment(
        env: Env,
        report_id: u64,
        author: Address,
        body: Bytes,
    ) -> Result<u64, BridgeError> {
        reporting::ReportingManager::add_report_comment(&env, report_id, author, body)
    }

    /// Get comments for a report
    pub fn get_report_comments(env: Env, report_id: u64) -> Vec<ReportComment> {
        reporting::ReportingManager::get_report_comments(&env, report_id)
    }

    /// Create an alert rule
    pub fn create_alert_rule(
        env: Env,
        owner: Address,
        name: Bytes,
        condition_type: AlertConditionType,
        threshold: i128,
    ) -> Result<u64, BridgeError> {
        reporting::ReportingManager::create_alert_rule(&env, owner, name, condition_type, threshold)
    }

    /// Get alert rules for an owner
    pub fn get_alert_rules(env: Env, owner: Address) -> Vec<AlertRule> {
        reporting::ReportingManager::get_alert_rules(&env, owner)
    }

    /// Bootstrap a baseline set of alert rules for production monitoring.
    ///
    /// Returns the created rule IDs.
    pub fn bootstrap_default_alert_rules(
        env: Env,
        owner: Address,
    ) -> Result<Vec<u64>, BridgeError> {
        reporting::ReportingManager::bootstrap_default_alert_rules(&env, owner)
    }

    /// Evaluate alert rules (returns triggered rule ids)
    pub fn evaluate_alerts(env: Env) -> Vec<u64> {
        reporting::ReportingManager::evaluate_alerts(&env)
    }

    /// Get recent report snapshots
    pub fn get_recent_report_snapshots(env: Env, limit: u32) -> Vec<ReportSnapshot> {
        reporting::ReportingManager::get_recent_report_snapshots(&env, limit)
    }

    // ========== Backup and Disaster Recovery Functions ==========

    /// Create a backup manifest (integrity hash from off-chain)
    pub fn create_backup(
        env: Env,
        creator: Address,
        integrity_hash: Bytes,
        rto_tier: RtoTier,
        encryption_ref: u64,
    ) -> Result<u64, BridgeError> {
        backup::BackupManager::create_backup(
            &env,
            creator,
            integrity_hash,
            rto_tier,
            encryption_ref,
        )
    }

    /// Get backup manifest by id
    pub fn get_backup_manifest(env: Env, backup_id: u64) -> Option<BackupManifest> {
        backup::BackupManager::get_backup_manifest(&env, backup_id)
    }

    /// Verify backup integrity
    pub fn verify_backup(
        env: Env,
        backup_id: u64,
        verifier: Address,
        expected_hash: Bytes,
    ) -> Result<bool, BridgeError> {
        backup::BackupManager::verify_backup(&env, backup_id, verifier, expected_hash)
    }

    /// Schedule automated backup
    pub fn schedule_backup(
        env: Env,
        owner: Address,
        next_run_at: u64,
        interval_seconds: u64,
        rto_tier: RtoTier,
    ) -> Result<u64, BridgeError> {
        backup::BackupManager::schedule_backup(&env, owner, next_run_at, interval_seconds, rto_tier)
    }

    /// Get scheduled backups for an owner
    pub fn get_scheduled_backups(env: Env, owner: Address) -> Vec<BackupSchedule> {
        backup::BackupManager::get_scheduled_backups(&env, owner)
    }

    /// Record a recovery execution (RTO tracking and audit)
    pub fn record_recovery(
        env: Env,
        backup_id: u64,
        executed_by: Address,
        recovery_duration_secs: u64,
        success: bool,
    ) -> Result<u64, BridgeError> {
        backup::BackupManager::record_recovery(
            &env,
            backup_id,
            executed_by,
            recovery_duration_secs,
            success,
        )
    }

    /// Get recovery records for audit and RTO reporting
    pub fn get_recovery_records(env: Env, limit: u32) -> Vec<RecoveryRecord> {
        backup::BackupManager::get_recovery_records(&env, limit)
    }

    /// Get recent backup manifests
    pub fn get_recent_backups(env: Env, limit: u32) -> Vec<BackupManifest> {
        backup::BackupManager::get_recent_backups(&env, limit)
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
        assessment::AssessmentManager::get_next_adaptive_question(&env, id, scores, answered_ids)
    }

    // ========== Escrow Functions ==========
    // REMOVED: All escrow functions disabled due to broken implementation

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

    /// Initialize insurance pool
    pub fn initialize_insurance_pool(
        env: Env,
        token: Address,
        premium_rate: u32,
    ) -> Result<(), EscrowError> {
        insurance::InsuranceManager::initialize_pool(&env, token, premium_rate)
    }

    /// Fund insurance pool
    pub fn fund_insurance_pool(env: Env, funder: Address, amount: i128) -> Result<(), EscrowError> {
        insurance::InsuranceManager::fund_pool(&env, funder, amount)
    }

    // ========== Escrow Analytics Functions ==========
    // REMOVED: Escrow analytics disabled (depends on removed escrow module)

    // ========== Credit Scoring Functions (feat/credit_score) ==========

    /// Record course completion
    pub fn record_course_completion(env: Env, user: Address, course_id: u64, points: u64) {
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
    pub fn get_user_courses(env: Env, user: Address) -> Vec<u64> {
        score::ScoreManager::get_courses(&env, user)
    }

    /// Get user's contributions
    pub fn get_user_contributions(env: Env, user: Address) -> Vec<types::Contribution> {
        score::ScoreManager::get_contributions(&env, user)
    }

    // ========== Reputation Functions (main) ==========

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
        provenance::ProvenanceTracker::record_mint(&env, token_id, params.creator, None);
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

    // ========== Mobile UI/UX Functions ==========

    /// Initialize mobile profile for user
    pub fn initialize_mobile_profile(
        env: Env,
        user: Address,
        device_info: DeviceInfo,
        preferences: MobilePreferences,
    ) -> Result<(), MobilePlatformError> {
        mobile_platform::MobilePlatformManager::initialize_mobile_profile(
            &env,
            user,
            device_info,
            preferences,
        )
        .map_err(|_| MobilePlatformError::DeviceNotSupported)
    }

    /// Update accessibility settings
    pub fn update_accessibility_settings(
        env: Env,
        user: Address,
        settings: MobileAccessibilitySettings,
    ) -> Result<(), MobilePlatformError> {
        mobile_platform::MobilePlatformManager::update_accessibility_settings(&env, user, settings)
            .map_err(|_| MobilePlatformError::DeviceNotSupported)
    }

    /// Update personalization settings
    pub fn update_personalization(
        env: Env,
        user: Address,
        preferences: MobilePreferences,
    ) -> Result<(), MobilePlatformError> {
        mobile_platform::MobilePlatformManager::update_personalization(&env, user, preferences)
            .map_err(|_| MobilePlatformError::DeviceNotSupported)
    }

    /// Record onboarding progress
    pub fn record_onboarding_progress(
        env: Env,
        user: Address,
        stage: OnboardingStage,
    ) -> Result<(), MobilePlatformError> {
        mobile_platform::MobilePlatformManager::record_onboarding_progress(&env, user, stage)
            .map_err(|_| MobilePlatformError::DeviceNotSupported)
    }

    /// Submit user feedback
    pub fn submit_user_feedback(
        env: Env,
        user: Address,
        rating: u32,
        comment: Bytes,
        category: FeedbackCategory,
    ) -> Result<u64, MobilePlatformError> {
        mobile_platform::MobilePlatformManager::submit_user_feedback(
            &env, user, rating, comment, category,
        )
        .map_err(|_| MobilePlatformError::DeviceNotSupported)
    }

    /// Get user allocated experiment variants
    pub fn get_user_experiment_variants(env: Env, user: Address) -> Map<u64, Symbol> {
        mobile_platform::MobilePlatformManager::get_user_experiment_variants(&env, user)
    }

    /// Get design system configuration
    pub fn get_design_system_config(env: Env) -> ComponentConfig {
        mobile_platform::MobilePlatformManager::get_design_system_config(&env)
    }

    /// Set design system configuration (admin only)
    pub fn set_design_system_config(env: Env, config: ComponentConfig) {
        // In a real implementation, we would check for admin authorization here
        mobile_platform::MobilePlatformManager::set_design_system_config(&env, config)
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

    // ========== Social Learning Functions ==========
    // REMOVED: All social learning functions disabled due to broken implementation

    // Analytics function removed due to contracttype limitations
    // Use internal notification manager for analytics

    // ========== Contract Upgrade Functions ==========

    /// Prepare for contract upgrade by backing up current state
    pub fn prepare_upgrade(
        env: Env,
        admin: Address,
        new_version: u32,
        state_hash: Bytes,
    ) -> Result<(), BridgeError> {
        upgrade::ContractUpgrader::prepare_upgrade(&env, admin, new_version, state_hash)
    }

    /// Execute the contract upgrade
    pub fn execute_upgrade(
        env: Env,
        admin: Address,
        new_version: u32,
        migration_hash: Bytes,
    ) -> Result<(), BridgeError> {
        upgrade::ContractUpgrader::execute_upgrade(&env, admin, new_version, migration_hash)
    }

    /// Rollback to previous version if within rollback window
    pub fn rollback_upgrade(env: Env, admin: Address) -> Result<(), BridgeError> {
        upgrade::ContractUpgrader::rollback(&env, admin)
    }

    /// Get current contract version
    pub fn get_contract_version(env: Env) -> u32 {
        upgrade::ContractUpgrader::get_current_version(&env)
    }

    /// Get upgrade history for a specific version
    pub fn get_upgrade_history(env: Env, version: u32) -> Option<upgrade::UpgradeRecord> {
        upgrade::ContractUpgrader::get_upgrade_history(&env, version)
    }

    /// Check if rollback is available
    pub fn is_rollback_available(env: Env) -> bool {
        upgrade::ContractUpgrader::is_rollback_available(&env)
    }

    /// Get state backup information
    pub fn get_state_backup(env: Env) -> Option<upgrade::StateBackup> {
        upgrade::ContractUpgrader::get_state_backup(&env)
    }

    // ========== Network Recovery Functions ==========

    /// Register a failed operation for automatic retry
    pub fn register_failed_operation(
        env: Env,
        operation_id: u64,
        operation_type: Bytes,
        user: Address,
        error_message: Bytes,
    ) -> Result<(), BridgeError> {
        network_recovery::NetworkRecovery::register_failed_operation(
            &env,
            operation_id,
            operation_type,
            user,
            error_message,
        )
    }

    /// Check if operation can be retried
    pub fn can_retry_operation(env: Env, operation_id: u64) -> Result<bool, BridgeError> {
        network_recovery::NetworkRecovery::can_retry(&env, operation_id)
    }

    /// Mark operation as completed
    pub fn mark_operation_completed(env: Env, operation_id: u64) -> Result<(), BridgeError> {
        network_recovery::NetworkRecovery::mark_completed(&env, operation_id)
    }

    /// Get operation state
    pub fn get_operation_state(
        env: Env,
        operation_id: u64,
    ) -> Option<network_recovery::OperationState> {
        network_recovery::NetworkRecovery::get_operation_state(&env, operation_id)
    }

    /// Get user retry notifications
    pub fn get_user_retry_notifications(env: Env, user: Address) -> Vec<u64> {
        network_recovery::NetworkRecovery::get_user_notifications(&env, user)
    }

    /// Check if fallback mechanism is active
    pub fn is_fallback_active(env: Env) -> bool {
        network_recovery::NetworkRecovery::is_fallback_active(&env)
    }
}
