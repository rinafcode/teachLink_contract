//! Error types and panic helper for the TeachLink contract.

use soroban_sdk::{symbol_short, Env};

/// All recoverable error conditions in the contract.
#[derive(Clone, Debug)]
pub enum TeachLinkError {
    Unauthorized,
    InvalidAmount,
    InvalidAddress,
    ChainNotSupported,
    RateLimitExceeded,
    InsufficientBalance,
    BridgeFailed,
    NotInitialized,
    InvalidChainId,
    FeeTooHigh,
    ChainExists,
    InvalidPrice,
    InvalidConfidence,
    UnauthorizedOracle,
}

/// Increment the error counter and panic with a descriptive symbol.
pub fn handle_error(env: &Env, error: TeachLinkError) -> ! {
    use crate::storage::ERROR_COUNT;

    let mut count: u64 = env.storage().instance().get(&ERROR_COUNT).unwrap_or(0);
    count += 1;
    env.storage().instance().set(&ERROR_COUNT, &count);

    match error {
        TeachLinkError::Unauthorized => {
            env.panic_with_error_data(&symbol_short!("unauth"), "Unauthorized access")
        }
        TeachLinkError::InvalidAmount => {
            env.panic_with_error_data(&symbol_short!("inv_amt"), "Invalid amount")
        }
        TeachLinkError::InvalidAddress => {
            env.panic_with_error_data(&symbol_short!("inv_addr"), "Invalid address")
        }
        TeachLinkError::ChainNotSupported => {
            env.panic_with_error_data(&symbol_short!("no_chain"), "Chain not supported")
        }
        TeachLinkError::RateLimitExceeded => {
            env.panic_with_error_data(&symbol_short!("rate_lim"), "Rate limit exceeded")
        }
        TeachLinkError::InsufficientBalance => {
            env.panic_with_error_data(&symbol_short!("no_bal"), "Insufficient balance")
        }
        TeachLinkError::BridgeFailed => {
            env.panic_with_error_data(&symbol_short!("br_fail"), "Bridge operation failed")
        }
        TeachLinkError::NotInitialized => {
            env.panic_with_error_data(&symbol_short!("no_init"), "Contract not initialized")
        }
        TeachLinkError::InvalidChainId => {
            env.panic_with_error_data(&symbol_short!("inv_chn"), "Invalid chain ID")
        }
        TeachLinkError::FeeTooHigh => {
            env.panic_with_error_data(&symbol_short!("fee_hi"), "Fee rate too high")
        }
        TeachLinkError::ChainExists => {
            env.panic_with_error_data(&symbol_short!("chn_ex"), "Chain already exists")
        }
        TeachLinkError::InvalidPrice => {
            env.panic_with_error_data(&symbol_short!("inv_prc"), "Invalid price")
        }
        TeachLinkError::InvalidConfidence => {
            env.panic_with_error_data(&symbol_short!("inv_conf"), "Invalid confidence")
        }
        TeachLinkError::UnauthorizedOracle => {
            env.panic_with_error_data(&symbol_short!("unauth_or"), "Unauthorized oracle")
        }
    }
}
