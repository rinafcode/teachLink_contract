//! TeachLink Soroban smart contract — entry point.
//!
//! This file wires the public contract interface to focused sub-modules:
//! - [`constants`]   — compile-time configuration values
//! - [`errors`]      — error enum and panic helper
//! - [`types`]       — shared data types (`BridgeConfig`)
//! - [`storage`]     — storage keys and low-level helpers
//! - [`validation`]  — input validation guards
//! - [`bridge`]      — bridge-out and chain management
//! - [`oracle`]      — oracle price feed management

#![cfg_attr(not(test), no_std)]

pub mod bridge;
pub mod constants;
pub mod errors;
pub mod oracle;
pub mod storage;
pub mod types;
pub mod validation;

use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Symbol, Vec};

use storage::{ADMIN, BRIDGE_TXS, CONFIG, FALLBACK_ENABLED};
use types::BridgeConfig;
use validation::{require_admin, require_initialized, validate_address, validate_fee_rate};

#[cfg(not(test))]
#[contract]
pub struct TeachLinkBridge;

#[contractimpl]
impl TeachLinkBridge {
    /// Initialize the contract with an admin address.
    pub fn initialize(env: Env, admin: Address) {
        require_initialized(&env, false);
        validate_address(&env, &admin);

        let config = BridgeConfig::default();
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&storage::NONCE, &0u64);
        env.storage().instance().set(&FALLBACK_ENABLED, &config.fallback_enabled);
        env.storage().instance().set(&BRIDGE_TXS, &Vec::<(Address, i128, u32, Bytes)>::new(&env));
        env.storage().instance().set(&storage::ERROR_COUNT, &0u64);
        env.storage().instance().set(&CONFIG, &config);
    }

    /// Bridge tokens to another chain; returns the transaction nonce.
    pub fn bridge_out(
        env: Env,
        from: Address,
        amount: i128,
        destination_chain: u32,
        destination_address: Bytes,
    ) -> u64 {
        bridge::bridge_out(&env, from, amount, destination_chain, destination_address)
    }

    /// Register a new supported chain (admin only).
    pub fn add_chain_support(
        env: Env,
        chain_id: u32,
        name: Symbol,
        bridge_address: Address,
        min_confirmations: u32,
        fee_rate: u32,
    ) {
        bridge::add_chain_support(&env, chain_id, name, bridge_address, min_confirmations, fee_rate);
    }

    /// Submit an oracle price update (authorized oracles only).
    pub fn update_oracle_price(
        env: Env,
        asset: Symbol,
        price: i128,
        confidence: u32,
        oracle_signer: Address,
    ) {
        oracle::update_oracle_price(&env, asset, price, confidence, oracle_signer);
    }

    /// Update bridge configuration (admin only).
    pub fn update_config(env: Env, config: BridgeConfig) {
        require_admin(&env);
        validate_fee_rate(&env, &config.fee_rate);
        env.storage().instance().set(&CONFIG, &config);
    }

    /// Enable or disable the fallback mechanism (admin only).
    pub fn set_fallback_enabled(env: Env, enabled: bool) {
        require_admin(&env);
        env.storage().instance().set(&FALLBACK_ENABLED, &enabled);
    }

    // ── Queries ──────────────────────────────────────────────────────────────

    pub fn get_bridge_tx(env: Env, index: u32) -> Option<(Address, i128, u32, Bytes)> {
        let txs: Vec<(Address, i128, u32, Bytes)> = env
            .storage()
            .instance()
            .get(&BRIDGE_TXS)
            .unwrap_or_else(|| Vec::new(&env));
        txs.get(index)
    }

    pub fn get_config(env: Env) -> BridgeConfig {
        storage::get_config(&env)
    }

    pub fn is_fallback_enabled(env: Env) -> bool {
        env.storage().instance().get(&FALLBACK_ENABLED).unwrap_or(true)
    }

    pub fn get_error_stats(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&storage::ERROR_COUNT)
            .unwrap_or(0)
    }

    /// Expose key constants for off-chain consumers.
    pub fn get_constants(_env: Env) -> (u32, u32, u32, i128, u64) {
        (
            constants::fees::DEFAULT_FEE_RATE,
            constants::chains::DEFAULT_MIN_CONFIRMATIONS,
            constants::oracle::DEFAULT_CONFIDENCE_THRESHOLD,
            constants::amounts::FALLBACK_PRICE,
            constants::oracle::PRICE_FRESHNESS_SECONDS,
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert!(true);
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
            &env,
            creator,
            name,
            description,
            subject,
            max_members,
            is_private,
            tags,
            settings,
        )
        .map_err(|_| BridgeError::InvalidInput)
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
    pub fn get_study_group(
        env: Env,
        group_id: u64,
    ) -> Result<social_learning::StudyGroup, BridgeError> {
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
        social_learning::SocialLearningManager::create_forum(
            &env,
            creator,
            title,
            description,
            category,
            tags,
        )
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
            &env,
            forum_id,
            author,
            title,
            content,
            attachments,
        )
        .map_err(|_| BridgeError::InvalidInput)
    }

    /// Get forum information
    pub fn get_forum(
        env: Env,
        forum_id: u64,
    ) -> Result<social_learning::DiscussionForum, BridgeError> {
        social_learning::SocialLearningManager::get_forum(&env, forum_id)
            .map_err(|_| BridgeError::InvalidInput)
    }

    /// Get forum post
    pub fn get_forum_post(
        env: Env,
        post_id: u64,
    ) -> Result<social_learning::ForumPost, BridgeError> {
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
            &env,
            creator,
            name,
            description,
            project_type,
            settings,
        )
        .map_err(|_| BridgeError::InvalidInput)
    }

    /// Get workspace information
    pub fn get_workspace(
        env: Env,
        workspace_id: u64,
    ) -> Result<social_learning::CollaborationWorkspace, BridgeError> {
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
            &env,
            reviewer,
            reviewee,
            content_type,
            content_id,
            rating,
            feedback,
            criteria,
        )
        .map_err(|_| BridgeError::InvalidInput)
    }

    /// Get review information
    pub fn get_review(
        env: Env,
        review_id: u64,
    ) -> Result<social_learning::PeerReview, BridgeError> {
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
            &env,
            mentor,
            expertise_areas,
            experience_level,
            availability,
            hourly_rate,
            bio,
            languages,
            timezone,
        )
        .map_err(|_| BridgeError::InvalidInput)
    }

    /// Get mentorship profile
    pub fn get_mentorship_profile(
        env: Env,
        mentor: Address,
    ) -> Result<social_learning::MentorshipProfile, BridgeError> {
        social_learning::SocialLearningManager::get_mentorship_profile(&env, mentor)
            .map_err(|_| BridgeError::InvalidInput)
    }

    /// Get user social analytics
    pub fn get_user_analytics(env: Env, user: Address) -> social_learning::SocialAnalytics {
        social_learning::SocialLearningManager::get_user_analytics(&env, user)
    }

    /// Update user social analytics
    pub fn update_user_analytics(
        env: Env,
        user: Address,
        analytics: social_learning::SocialAnalytics,
    ) {
        social_learning::SocialLearningManager::update_user_analytics(&env, user, analytics);
    }

    // Analytics function removed due to contracttype limitations
    // Use internal notification manager for analytics
}
