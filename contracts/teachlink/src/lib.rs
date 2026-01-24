#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, String, Vec};

mod bridge;
mod escrow;
mod events;
mod rewards;
mod storage;
mod tokenization;
mod types;

pub use types::{
    BridgeTransaction, CrossChainMessage, DisputeOutcome, Escrow, EscrowStatus, RewardRate, UserReward
};

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
    ) {
        bridge::Bridge::initialize(&env, token, admin, min_validators, fee_recipient);
    }

    /// Bridge tokens out to another chain (lock/burn tokens on Stellar)
    pub fn bridge_out(
        env: Env,
        from: Address,
        amount: i128,
        destination_chain: u32,
        destination_address: Bytes,
    ) -> u64 {
        bridge::Bridge::bridge_out(&env, from, amount, destination_chain, destination_address)
    }

    /// Complete a bridge transaction (mint/release tokens on Stellar)
    pub fn complete_bridge(
        env: Env,
        message: CrossChainMessage,
        validator_signatures: Vec<Address>,
    ) {
        bridge::Bridge::complete_bridge(&env, message, validator_signatures);
    }

    /// Cancel a bridge transaction and refund locked tokens
    pub fn cancel_bridge(env: Env, nonce: u64) {
        bridge::Bridge::cancel_bridge(&env, nonce);
    }

    // ========== Admin Functions ==========

    /// Add a validator (admin only)
    pub fn add_validator(env: Env, validator: Address) {
        bridge::Bridge::add_validator(&env, validator);
    }

    /// Remove a validator (admin only)
    pub fn remove_validator(env: Env, validator: Address) {
        bridge::Bridge::remove_validator(&env, validator);
    }

    /// Add a supported destination chain (admin only)
    pub fn add_supported_chain(env: Env, chain_id: u32) {
        bridge::Bridge::add_supported_chain(&env, chain_id);
    }

    /// Remove a supported destination chain (admin only)
    pub fn remove_supported_chain(env: Env, chain_id: u32) {
        bridge::Bridge::remove_supported_chain(&env, chain_id);
    }

    /// Set bridge fee (admin only)
    pub fn set_bridge_fee(env: Env, fee: i128) {
        bridge::Bridge::set_bridge_fee(&env, fee);
    }

    /// Set fee recipient (admin only)
    pub fn set_fee_recipient(env: Env, fee_recipient: Address) {
        bridge::Bridge::set_fee_recipient(&env, fee_recipient);
    }

    /// Set minimum validators (admin only)
    pub fn set_min_validators(env: Env, min_validators: u32) {
        bridge::Bridge::set_min_validators(&env, min_validators);
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

    // ========== Rewards Functions ==========

    /// Initialize the rewards system
    pub fn initialize_rewards(env: Env, token: Address, rewards_admin: Address) {
        rewards::Rewards::initialize_rewards(&env, token, rewards_admin);
    }

    /// Fund the reward pool
    pub fn fund_reward_pool(env: Env, funder: Address, amount: i128) {
        rewards::Rewards::fund_reward_pool(&env, funder, amount);
    }

    /// Issue rewards to a user
    pub fn issue_reward(env: Env, recipient: Address, amount: i128, reward_type: String) {
        rewards::Rewards::issue_reward(&env, recipient, amount, reward_type);
    }

    /// Claim pending rewards
    pub fn claim_rewards(env: Env, user: Address) {
        rewards::Rewards::claim_rewards(&env, user);
    }

    /// Set reward rate for a specific reward type (admin only)
    pub fn set_reward_rate(env: Env, reward_type: String, rate: i128, enabled: bool) {
        rewards::Rewards::set_reward_rate(&env, reward_type, rate, enabled);
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
    // ========== Escrow Functions ==========

    /// Create a multi-signature escrow
    pub fn create_escrow(
        env: Env,
        depositor: Address,
        beneficiary: Address,
        token: Address,
        amount: i128,
        signers: Vec<Address>,
        threshold: u32,
        release_time: Option<u64>,
        refund_time: Option<u64>,
        arbitrator: Address,
    ) -> u64 {
        escrow::EscrowManager::create_escrow(
            &env,
            depositor,
            beneficiary,
            token,
            amount,
            signers,
            threshold,
            release_time,
            refund_time,
            arbitrator,
        )
    }

    /// Approve escrow release (multi-signature)
    pub fn approve_escrow_release(env: Env, escrow_id: u64, signer: Address) -> u32 {
        escrow::EscrowManager::approve_release(&env, escrow_id, signer)
    }

    /// Release funds to the beneficiary once conditions are met
    pub fn release_escrow(env: Env, escrow_id: u64, caller: Address) {
        escrow::EscrowManager::release(&env, escrow_id, caller)
    }

    /// Refund escrow to the depositor after refund time
    pub fn refund_escrow(env: Env, escrow_id: u64, depositor: Address) {
        escrow::EscrowManager::refund(&env, escrow_id, depositor)
    }

    /// Cancel escrow before any approvals
    pub fn cancel_escrow(env: Env, escrow_id: u64, depositor: Address) {
        escrow::EscrowManager::cancel(&env, escrow_id, depositor)
    }

    /// Raise a dispute on the escrow
    pub fn dispute_escrow(env: Env, escrow_id: u64, disputer: Address, reason: Bytes) {
        escrow::EscrowManager::dispute(&env, escrow_id, disputer, reason)
    }

    /// Resolve a dispute as the arbitrator
    pub fn resolve_escrow(env: Env, escrow_id: u64, arbitrator: Address, outcome: DisputeOutcome) {
        escrow::EscrowManager::resolve(&env, escrow_id, arbitrator, outcome)
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

    // ========== Content Tokenization Functions ==========

    /// Mint a new educational content token
    pub fn mint_content_token(
        env: Env,
        creator: Address,
        title: Bytes,
        description: Bytes,
        content_type: ContentType,
        content_hash: Bytes,
        license_type: Bytes,
        tags: Vec<Bytes>,
        is_transferable: bool,
        royalty_percentage: u32,
    ) -> u64 {
        let token_id = tokenization::ContentTokenization::mint(
            &env,
            creator.clone(),
            title,
            description,
            content_type,
            content_hash,
            license_type,
            tags,
            is_transferable,
            royalty_percentage,
        );

        // Record mint in provenance
        provenance::ProvenanceTracker::record_mint(&env, token_id, creator, None);

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
    pub fn get_content_transfer_count(env: Env, token_id: u64) -> u32 {
        provenance::ProvenanceTracker::get_transfer_count(&env, token_id)
    }

    /// Verify ownership chain integrity for a content token
    pub fn verify_content_chain(env: Env, token_id: u64) -> bool {
        provenance::ProvenanceTracker::verify_chain(&env, token_id)
    }

    /// Get the original creator of a content token
    pub fn get_content_creator(env: Env, token_id: u64) -> Option<Address> {
        provenance::ProvenanceTracker::get_creator(&env, token_id)
    }

    /// Get all addresses that have owned a content token
    pub fn get_content_all_owners(env: Env, token_id: u64) -> Vec<Address> {
        provenance::ProvenanceTracker::get_all_owners(&env, token_id)
    }
}
