//! Storage key constants and low-level storage helpers.

use soroban_sdk::{symbol_short, Env, Symbol};

use crate::types::BridgeConfig;

// symbol_short! max length is 9 chars
pub const ADMIN: Symbol = symbol_short!("admin");
pub const NONCE: Symbol = symbol_short!("nonce");
pub const BRIDGE_TXS: Symbol = symbol_short!("brdg_txs");
pub const FALLBACK_ENABLED: Symbol = symbol_short!("fallback");
pub const ERROR_COUNT: Symbol = symbol_short!("err_count");
pub const CONFIG: Symbol = symbol_short!("config");

/// Fetch the current config or return the default.
pub fn get_config(env: &Env) -> BridgeConfig {
    env.storage().instance().get(&CONFIG).unwrap_or_default()
}

/// Return the next nonce (increments the stored value).
pub fn get_next_nonce(env: &Env) -> u64 {
    let nonce: u64 = env.storage().instance().get(&NONCE).unwrap_or(0);
    let next = nonce + 1;
    env.storage().instance().set(&NONCE, &next);
    next
}
