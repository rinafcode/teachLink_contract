#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Vec};

mod bridge;
mod events;
mod storage;
mod types;

pub use types::{BridgeTransaction, CrossChainMessage};

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
}
