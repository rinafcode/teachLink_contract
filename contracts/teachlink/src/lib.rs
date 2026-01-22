#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Vec};

mod bridge;
mod escrow;
mod events;
mod reputation;
mod storage;
mod types;

pub use types::{BridgeTransaction, CrossChainMessage, DisputeOutcome, Escrow, EscrowStatus};

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

    // ========== Reputation Functions ==========

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
}
