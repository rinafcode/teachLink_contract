//! Ledger time helpers
//!
//! Soroban `ledger().timestamp()` is not a monotonic wall-clock guarantee. For critical
//! deadline checks, we also store and compare against `ledger().sequence()` as a fallback.

use soroban_sdk::Env;

/// Conservative estimate of seconds per ledger close on Stellar — re-exported from config.
pub use crate::config::LEDGER_EST_SECS as EST_SECS_PER_LEDGER;

pub fn seconds_to_ledger_delta(seconds: u64) -> u32 {
    // Ceil division to avoid shortening timeouts.
    let d = (seconds + EST_SECS_PER_LEDGER - 1) / EST_SECS_PER_LEDGER;
    u32::try_from(d).unwrap_or(u32::MAX)
}

pub fn seq_now(env: &Env) -> u32 {
    env.ledger().sequence()
}
