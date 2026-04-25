//! DoS protection: batch-size limits, resource quotas, and per-address rate limiting.
//!
//! All bulk-operation limits live here so they can be reviewed and tuned in
//! one place.

use soroban_sdk::{symbol_short, Address, Env, Map, Symbol};

use crate::errors::BridgeError;

// ── Resource quotas / batch-size limits ──────────────────────────────────────

/// Maximum validator signatures accepted in a single `complete_bridge` call.
pub const MAX_VALIDATORS_PER_COMPLETION: u32 = 50;

/// Maximum chain IDs that may be paused or resumed in a single call.
pub const MAX_CHAIN_BATCH_SIZE: u32 = 50;

/// Maximum notification preferences a user may submit in a single call.
pub const MAX_PREFERENCE_BATCH_SIZE: u32 = 20;

/// Maximum packets scanned per `check_timeouts` invocation.
pub const MAX_TIMEOUT_SCAN_BATCH: u32 = 100;

/// Maximum questions an assessment may contain.
pub const MAX_QUESTIONS_PER_ASSESSMENT: u32 = 200;

/// Maximum proctor-log entries per assessment submission.
pub const MAX_PROCTOR_LOGS_PER_SUBMISSION: u32 = 50;

// ── Instruction budget (gas) limits ──────────────────────────────────────────
//
// Soroban measures execution cost in CPU instructions and memory bytes.
// The constants below document the expected instruction budget for each
// bulk-operation category so callers can reason about worst-case costs.
// Exceeding the network's per-transaction limit causes an automatic abort.

/// Approximate CPU-instruction budget consumed per validator checked in
/// `complete_bridge` (signature verification is the dominant cost).
pub const INSTRUCTIONS_PER_VALIDATOR_CHECK: u64 = 5_000;

/// Approximate CPU-instruction budget consumed per chain ID processed in
/// `pause_chains` / `resume_chains`.
pub const INSTRUCTIONS_PER_CHAIN_OP: u64 = 2_000;

/// Approximate CPU-instruction budget consumed per packet evaluated in
/// `check_timeouts`.
pub const INSTRUCTIONS_PER_TIMEOUT_CHECK: u64 = 1_500;

/// Approximate CPU-instruction budget consumed per notification preference.
pub const INSTRUCTIONS_PER_PREFERENCE: u64 = 1_000;

/// Absolute upper bound on CPU instructions a single contract invocation may
/// consume before hitting the Soroban network limit (~100 million).
/// Used as a soft guard: if a batch's estimated cost exceeds this, reject early.
pub const MAX_INSTRUCTIONS_PER_INVOCATION: u64 = 50_000_000;

// ── Rate limiting ─────────────────────────────────────────────────────────────

/// Minimum gap (seconds) between two `bridge_out` calls from the same address.
pub const BRIDGE_OUT_RATE_LIMIT_SECONDS: u64 = 60;

/// Minimum gap (seconds) between two admin bulk operations (pause/resume/scan)
/// from the same address.  Prevents a rogue admin from looping bulk calls.
pub const ADMIN_OP_RATE_LIMIT_SECONDS: u64 = 10;

const BRIDGE_RATE_LIMIT_KEY: Symbol = symbol_short!("BR_RLMT");

/// Storage key for per-address admin operation rate-limit timestamps.
const ADMIN_RATE_LIMIT_KEY: Symbol = symbol_short!("ADM_RLMT");

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Return `BatchSizeLimitExceeded` if `actual > max`.
pub fn check_batch_size(actual: u32, max: u32) -> Result<(), BridgeError> {
    if actual > max {
        Err(BridgeError::BatchSizeLimitExceeded)
    } else {
        Ok(())
    }
}

/// Estimate instruction cost for `batch_size` items at `cost_per_item` and
/// return `BatchSizeLimitExceeded` if the estimate would exceed
/// `MAX_INSTRUCTIONS_PER_INVOCATION`.
///
/// Call this alongside `check_batch_size` when the per-item cost is non-trivial
/// (e.g. crypto operations, storage writes) to provide a second, cost-aware guard.
pub fn check_instruction_budget(batch_size: u32, cost_per_item: u64) -> Result<(), BridgeError> {
    let estimated = (batch_size as u64).saturating_mul(cost_per_item);
    if estimated > MAX_INSTRUCTIONS_PER_INVOCATION {
        Err(BridgeError::BatchSizeLimitExceeded)
    } else {
        Ok(())
    }
}

/// Enforce a per-sender cooldown for `bridge_out`.
///
/// Reads and updates a per-address timestamp under `BRIDGE_RATE_LIMIT_KEY`.
/// Returns `BridgeError::RetryBackoffActive` if the caller is within the
/// cooldown window.
pub fn check_bridge_out_rate_limit(env: &Env, sender: &Address) -> Result<(), BridgeError> {
    let mut limits: Map<Address, u64> = env
        .storage()
        .instance()
        .get(&BRIDGE_RATE_LIMIT_KEY)
        .unwrap_or_else(|| Map::new(env));

    let now = env.ledger().timestamp();
    if let Some(last) = limits.get(sender.clone()) {
        if now < last.saturating_add(BRIDGE_OUT_RATE_LIMIT_SECONDS) {
            return Err(BridgeError::RetryBackoffActive);
        }
    }

    limits.set(sender.clone(), now);
    env.storage()
        .instance()
        .set(&BRIDGE_RATE_LIMIT_KEY, &limits);
    Ok(())
}

/// Enforce a per-sender cooldown for admin bulk operations (pause/resume/timeout
/// scan).  Prevents a rogue or compromised admin key from flooding the ledger
/// with back-to-back bulk writes.
///
/// Returns `BridgeError::RetryBackoffActive` if the caller last performed an
/// admin op within `ADMIN_OP_RATE_LIMIT_SECONDS`.
pub fn check_admin_rate_limit(env: &Env, admin: &Address) -> Result<(), BridgeError> {
    let mut limits: Map<Address, u64> = env
        .storage()
        .instance()
        .get(&ADMIN_RATE_LIMIT_KEY)
        .unwrap_or_else(|| Map::new(env));

    let now = env.ledger().timestamp();
    if let Some(last) = limits.get(admin.clone()) {
        if now < last.saturating_add(ADMIN_OP_RATE_LIMIT_SECONDS) {
            return Err(BridgeError::RetryBackoffActive);
        }
    }

    limits.set(admin.clone(), now);
    env.storage()
        .instance()
        .set(&ADMIN_RATE_LIMIT_KEY, &limits);
    Ok(())
}
