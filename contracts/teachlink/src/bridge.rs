//! Bridge-out and chain-management logic.

use soroban_sdk::{symbol_short, Address, Bytes, Env, Symbol, Vec};

use crate::{
    constants,
    errors::{handle_error, TeachLinkError},
    storage::{self, BRIDGE_TXS},
    types::BridgeConfig,
    validation,
};

/// Calculate the fee for a given amount and rate (basis points).
pub fn calculate_fee(amount: i128, fee_rate: u32) -> i128 {
    amount * fee_rate as i128 / constants::fees::FEE_CALCULATION_DIVISOR as i128
}

/// Initiate a cross-chain bridge transfer and return the nonce.
pub fn bridge_out(
    env: &Env,
    from: Address,
    amount: i128,
    destination_chain: u32,
    destination_address: Bytes,
) -> u64 {
    validation::require_initialized(env, true);
    validation::validate_amount(env, &amount);
    validation::validate_chain_id(env, &destination_chain);
    validation::validate_bytes_address(env, &destination_address);

    let config: BridgeConfig = storage::get_config(env);
    let nonce = storage::get_next_nonce(env);

    let fee = calculate_fee(amount, config.fee_rate);
    let bridge_amount = amount - fee;
    validation::validate_amount(env, &bridge_amount);

    let mut txs: Vec<(Address, i128, u32, Bytes)> = env
        .storage()
        .instance()
        .get(&BRIDGE_TXS)
        .unwrap_or_else(|| Vec::new(env));

    if txs.len() >= constants::storage::MAX_BRIDGE_TXS {
        handle_error(env, TeachLinkError::BridgeFailed);
    }

    txs.push_back((from, bridge_amount, destination_chain, destination_address));
    env.storage().instance().set(&BRIDGE_TXS, &txs);

    nonce
}

/// Register a new supported chain (admin only).
pub fn add_chain_support(
    env: &Env,
    chain_id: u32,
    name: Symbol,
    bridge_address: Address,
    min_confirmations: u32,
    fee_rate: u32,
) {
    validation::require_admin(env);
    validation::validate_chain_id(env, &chain_id);
    validation::validate_fee_rate(env, &fee_rate);
    validation::validate_address(env, &bridge_address);

    if name.to_string().len() > constants::chains::MAX_CHAIN_NAME_LENGTH as usize {
        handle_error(env, TeachLinkError::InvalidAddress);
    }

    let chains_key = symbol_short!("chains");
    let chains: Vec<(u32, Symbol, Address, u32, u32)> = env
        .storage()
        .instance()
        .get(&chains_key)
        .unwrap_or_else(|| Vec::new(env));

    if chains.len() >= constants::storage::MAX_CHAIN_CONFIGS {
        handle_error(env, TeachLinkError::ChainExists);
    }

    for chain in chains.iter() {
        if chain.0 == chain_id {
            handle_error(env, TeachLinkError::ChainExists);
        }
    }

    let mut updated = chains;
    updated.push_back((chain_id, name, bridge_address, min_confirmations, fee_rate));
    env.storage().instance().set(&chains_key, &updated);
}
