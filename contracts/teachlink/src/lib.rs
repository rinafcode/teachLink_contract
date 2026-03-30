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

#[cfg(not(test))]
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
}
