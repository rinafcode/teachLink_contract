//! Bulk Operation DoS Protection
//!
//! This module enforces limits on all bulk and batch operations to prevent
//! denial-of-service attacks caused by unbounded iteration, excessive compute
//! budgets, or rapid repeated calls from a single caller.
//!
//! # Limits applied
//!
//! | Resource          | Constant                      | Default |
//! |-------------------|-------------------------------|---------|
//! | Batch size        | `MAX_BATCH_SIZE`              | 50      |
//! | Gas budget        | `MAX_GAS_BUDGET`              | 500_000 |
//! | Calls per window  | `RATE_LIMIT_MAX_CALLS`        | 10      |
//! | Rate-limit window | `RATE_LIMIT_WINDOW_SECONDS`   | 3_600   |
//! | Validators (batch)| `MAX_VALIDATOR_BATCH`         | 20      |
//! | Chain IDs (batch) | `MAX_CHAIN_ID_BATCH`          | 10      |
//! | Packets (batch)   | `MAX_PACKET_BATCH`            | 30      |
//! | Tags in metadata  | `MAX_CONTENT_TAGS`            | 20      |
//! | Signers per escrow| `MAX_ESCROW_SIGNERS`          | 20      |

use crate::errors::BridgeError;
use soroban_sdk::{Address, Env, Map, Symbol};

// ── Batch-size limits ──────────────────────────────────────────────────────

/// Hard upper bound on items processed in a single bulk call.
pub const MAX_BATCH_SIZE: u32 = 50;

/// Maximum number of validator signatures accepted in `complete_bridge`.
pub const MAX_VALIDATOR_BATCH: u32 = 20;

/// Maximum number of chain IDs accepted in `pause_chains` / `resume_chains`.
pub const MAX_CHAIN_ID_BATCH: u32 = 10;

/// Maximum number of packets processed in `check_cross_chain_timeouts`.
pub const MAX_PACKET_BATCH: u32 = 30;

/// Maximum number of content tags in a `ContentTokenParameters` or metadata struct.
pub const MAX_CONTENT_TAGS: u32 = 20;

/// Maximum number of signers in a single escrow.
pub const MAX_ESCROW_SIGNERS: u32 = 20;

// ── Gas / compute budget ───────────────────────────────────────────────────

/// Soft cap on CPU instructions consumed by a single bulk call.
/// Soroban reports consumed budget via `env.budget().cpu_instructions_consumed()`.
pub const MAX_GAS_BUDGET: u64 = 500_000;

// ── Rate-limiting ──────────────────────────────────────────────────────────

/// Rolling window (seconds) for per-caller rate limiting.
pub const RATE_LIMIT_WINDOW_SECONDS: u64 = 3_600; // 1 hour

/// Maximum number of bulk calls a single caller may make within the window.
pub const RATE_LIMIT_MAX_CALLS: u32 = 10;

// Storage key for the rate-limit map (caller → (count, window_start))
const RATE_LIMIT_KEY: Symbol = soroban_sdk::symbol_short!("rl_bulk");

// ── Public guards ──────────────────────────────────────────────────────────

/// Verify that `count` does not exceed `MAX_BATCH_SIZE`.
///
/// Call this at the **start** of every function that iterates over a
/// caller-supplied list.
pub fn check_batch_size(count: u32) -> Result<(), BridgeError> {
    if count > MAX_BATCH_SIZE {
        return Err(BridgeError::InvalidInput);
    }
    Ok(())
}

/// Verify that `count` does not exceed `limit`.
///
/// Use the typed constants (`MAX_VALIDATOR_BATCH`, `MAX_CHAIN_ID_BATCH`, …)
/// as the `limit` argument for self-documenting call sites.
pub fn check_batch_size_limit(count: u32, limit: u32) -> Result<(), BridgeError> {
    if count > limit {
        return Err(BridgeError::InvalidInput);
    }
    Ok(())
}

/// Verify that the CPU instructions consumed so far are within budget.
///
/// Insert this check **inside** tight loops to abort early when compute
/// is unexpectedly high.
pub fn check_gas_budget(env: &Env) -> Result<(), BridgeError> {
    // Soroban's budget is metered by the runtime; if the contract exhausts it
    // the invocation traps automatically.  This explicit check lets us return a
    // clean error instead of an opaque trap, and caps work at our chosen bound
    // rather than the full network limit.
    let consumed = env.budget().cpu_instructions_consumed();
    if consumed > MAX_GAS_BUDGET {
        return Err(BridgeError::InvalidInput);
    }
    Ok(())
}

/// Rate-limit bulk calls per caller.
///
/// Each caller gets a rolling window of `RATE_LIMIT_WINDOW_SECONDS`.
/// Within that window they may issue at most `RATE_LIMIT_MAX_CALLS` bulk
/// operations.  The counter resets when the window expires.
pub fn check_rate_limit(env: &Env, caller: &Address) -> Result<(), BridgeError> {
    let now = env.ledger().timestamp();

    // Load the current rate-limit map: caller → (call_count, window_start)
    let mut rl: Map<Address, (u32, u64)> = env
        .storage()
        .instance()
        .get(&RATE_LIMIT_KEY)
        .unwrap_or_else(|| Map::new(env));

    let (mut count, mut window_start) = rl.get(caller.clone()).unwrap_or((0u32, now));

    // Reset window if it has expired
    if now.saturating_sub(window_start) >= RATE_LIMIT_WINDOW_SECONDS {
        count = 0;
        window_start = now;
    }

    if count >= RATE_LIMIT_MAX_CALLS {
        return Err(BridgeError::InvalidInput);
    }

    count += 1;
    rl.set(caller.clone(), (count, window_start));
    env.storage().instance().set(&RATE_LIMIT_KEY, &rl);

    Ok(())
}

/// Convenience guard that enforces **all three** limits at once:
/// batch-size cap, gas budget, and rate limiting.
///
/// Use this for the highest-risk bulk entry points.
pub fn enforce_all(
    env: &Env,
    caller: &Address,
    batch_count: u32,
) -> Result<(), BridgeError> {
    check_batch_size(batch_count)?;
    check_gas_budget(env)?;
    check_rate_limit(env, caller)?;
    Ok(())
}

// ── Unit tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TeachLinkBridge;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::Env;

    fn make_env() -> (Env, soroban_sdk::Address) {
        let env = Env::default();
        let _contract_id = env.register(TeachLinkBridge, ());
        let caller = Address::generate(&env);
        (env, caller)
    }

    // ── batch-size ────────────────────────────────────────────────────────

    #[test]
    fn batch_size_at_limit_passes() {
        assert!(check_batch_size(MAX_BATCH_SIZE).is_ok());
    }

    #[test]
    fn batch_size_exceeds_limit_fails() {
        assert_eq!(
            check_batch_size(MAX_BATCH_SIZE + 1),
            Err(BridgeError::InvalidInput)
        );
    }

    #[test]
    fn batch_size_zero_passes() {
        // Empty batches are fine; the caller just does nothing.
        assert!(check_batch_size(0).is_ok());
    }

    #[test]
    fn typed_batch_limit_validator_batch() {
        assert!(check_batch_size_limit(MAX_VALIDATOR_BATCH, MAX_VALIDATOR_BATCH).is_ok());
        assert_eq!(
            check_batch_size_limit(MAX_VALIDATOR_BATCH + 1, MAX_VALIDATOR_BATCH),
            Err(BridgeError::InvalidInput)
        );
    }

    #[test]
    fn typed_batch_limit_chain_id_batch() {
        assert!(check_batch_size_limit(MAX_CHAIN_ID_BATCH, MAX_CHAIN_ID_BATCH).is_ok());
        assert_eq!(
            check_batch_size_limit(MAX_CHAIN_ID_BATCH + 1, MAX_CHAIN_ID_BATCH),
            Err(BridgeError::InvalidInput)
        );
    }

    #[test]
    fn typed_batch_limit_packet_batch() {
        assert!(check_batch_size_limit(MAX_PACKET_BATCH, MAX_PACKET_BATCH).is_ok());
        assert_eq!(
            check_batch_size_limit(MAX_PACKET_BATCH + 1, MAX_PACKET_BATCH),
            Err(BridgeError::InvalidInput)
        );
    }

    // ── gas budget ────────────────────────────────────────────────────────

    #[test]
    fn gas_budget_check_passes_on_fresh_env() {
        let (env, caller) = make_env();
        env.as_contract(&env.register(TeachLinkBridge, ()), || {
            // Fresh environment has consumed almost no instructions.
            assert!(check_gas_budget(&env).is_ok());
            let _ = caller; // suppress unused warning
        });
    }

    // ── rate limiting ─────────────────────────────────────────────────────

    #[test]
    fn rate_limit_allows_up_to_max_calls() {
        let (env, caller) = make_env();
        let contract_id = env.register(TeachLinkBridge, ());
        env.as_contract(&contract_id, || {
            for _ in 0..RATE_LIMIT_MAX_CALLS {
                assert!(check_rate_limit(&env, &caller).is_ok());
            }
            // One more should be rejected
            assert_eq!(
                check_rate_limit(&env, &caller),
                Err(BridgeError::InvalidInput)
            );
        });
    }

    #[test]
    fn rate_limit_resets_after_window_expires() {
        let (env, caller) = make_env();
        let contract_id = env.register(TeachLinkBridge, ());
        env.as_contract(&contract_id, || {
            // Exhaust the quota
            for _ in 0..RATE_LIMIT_MAX_CALLS {
                let _ = check_rate_limit(&env, &caller);
            }
            assert_eq!(
                check_rate_limit(&env, &caller),
                Err(BridgeError::InvalidInput)
            );

            // Advance ledger time past the window
            env.ledger().with_mut(|l| {
                l.timestamp += RATE_LIMIT_WINDOW_SECONDS;
            });

            // Quota should have reset
            assert!(check_rate_limit(&env, &caller).is_ok());
        });
    }

    #[test]
    fn different_callers_have_independent_quotas() {
        let (env, caller_a) = make_env();
        let caller_b = Address::generate(&env);
        let contract_id = env.register(TeachLinkBridge, ());
        env.as_contract(&contract_id, || {
            // Exhaust caller_a's quota
            for _ in 0..RATE_LIMIT_MAX_CALLS {
                let _ = check_rate_limit(&env, &caller_a);
            }
            assert_eq!(
                check_rate_limit(&env, &caller_a),
                Err(BridgeError::InvalidInput)
            );
            // caller_b is unaffected
            assert!(check_rate_limit(&env, &caller_b).is_ok());
        });
    }

    // ── enforce_all ───────────────────────────────────────────────────────

    #[test]
    fn enforce_all_rejects_oversized_batch() {
        let (env, caller) = make_env();
        let contract_id = env.register(TeachLinkBridge, ());
        env.as_contract(&contract_id, || {
            assert_eq!(
                enforce_all(&env, &caller, MAX_BATCH_SIZE + 1),
                Err(BridgeError::InvalidInput)
            );
        });
    }

    #[test]
    fn enforce_all_passes_within_all_limits() {
        let (env, caller) = make_env();
        let contract_id = env.register(TeachLinkBridge, ());
        env.as_contract(&contract_id, || {
            assert!(enforce_all(&env, &caller, MAX_BATCH_SIZE).is_ok());
        });
    }
}
