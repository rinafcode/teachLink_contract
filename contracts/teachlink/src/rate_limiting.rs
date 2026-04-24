//! Rate Limiting Module
//!
//! Provides per-user and per-endpoint call-rate enforcement for contract
//! entry points, addressing issue #266.
//!
//! # Design
//! - Each (user, endpoint) pair gets an independent window tracked by ledger
//!   sequence number so it works even when block timestamps are unreliable.
//! - Limits are configurable per endpoint via `EndpointConfig`.
//! - When a limit is exceeded the call returns `BridgeError::RetryLimitExceeded`
//!   so callers can implement graceful degradation.

use crate::errors::BridgeError;
use crate::storage::StorageKey;
use soroban_sdk::{contracttype, Address, Env, Map, Symbol};

/// Per-endpoint rate-limit configuration.
#[contracttype]
#[derive(Clone, Debug)]
pub struct EndpointConfig {
    /// Maximum calls allowed within `window_ledgers`.
    pub max_calls: u32,
    /// Window size in ledger sequences.
    pub window_ledgers: u32,
}

/// Per-(user, endpoint) rate-limit state stored on-chain.
#[contracttype]
#[derive(Clone, Debug)]
pub struct RateLimitState {
    /// Ledger sequence at which the current window started.
    pub window_start: u32,
    /// Number of calls made in the current window.
    pub call_count: u32,
}

/// Default per-user limit: 100 calls per ~600 ledgers (~1 hour at 6 s/ledger).
pub const DEFAULT_MAX_CALLS: u32 = 100;
pub const DEFAULT_WINDOW_LEDGERS: u32 = 600;

pub struct RateLimiter;

impl RateLimiter {
    /// Check and record a call from `caller` to `endpoint`.
    ///
    /// Returns `Ok(())` if within limits, or `Err(BridgeError::RetryLimitExceeded)`
    /// when the per-user limit is exceeded.
    ///
    /// Uses the default limit unless an endpoint-specific config is stored.
    pub fn check_rate_limit(
        env: &Env,
        caller: &Address,
        endpoint: &Symbol,
    ) -> Result<(), BridgeError> {
        let config = Self::get_config(env, endpoint);
        let key = Self::state_key(caller, endpoint);
        let current_seq = env.ledger().sequence();

        let mut state: RateLimitState = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or(RateLimitState {
                window_start: current_seq,
                call_count: 0,
            });

        // Reset window if it has expired
        if current_seq.saturating_sub(state.window_start) >= config.window_ledgers {
            state.window_start = current_seq;
            state.call_count = 0;
        }

        if state.call_count >= config.max_calls {
            return Err(BridgeError::RetryLimitExceeded);
        }

        state.call_count += 1;
        env.storage().instance().set(&key, &state);
        Ok(())
    }

    /// Set a custom rate-limit config for a specific endpoint (admin action).
    pub fn set_endpoint_config(env: &Env, endpoint: &Symbol, config: EndpointConfig) {
        let configs_key = StorageKey::RateLimitState;
        let mut configs: Map<Symbol, EndpointConfig> = env
            .storage()
            .instance()
            .get(&configs_key)
            .unwrap_or_else(|| Map::new(env));
        configs.set(endpoint.clone(), config);
        env.storage().instance().set(&configs_key, &configs);
    }

    /// Get the config for an endpoint, falling back to defaults.
    fn get_config(env: &Env, endpoint: &Symbol) -> EndpointConfig {
        let configs_key = StorageKey::RateLimitState;
        let configs: Map<Symbol, EndpointConfig> = env
            .storage()
            .instance()
            .get(&configs_key)
            .unwrap_or_else(|| Map::new(env));
        configs.get(endpoint.clone()).unwrap_or(EndpointConfig {
            max_calls: DEFAULT_MAX_CALLS,
            window_ledgers: DEFAULT_WINDOW_LEDGERS,
        })
    }

    /// Composite storage key for a (caller, endpoint) pair.
    /// Uses a tuple so each pair gets a unique slot — no collisions.
    fn state_key(caller: &Address, endpoint: &Symbol) -> (Address, Symbol) {
        (caller.clone(), endpoint.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TeachLinkBridge;
    use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};

    fn set_seq(env: &Env, seq: u32) {
        env.ledger().with_mut(|li| li.sequence_number = seq);
    }

    #[test]
    fn allows_calls_within_limit() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());
        let caller = Address::generate(&env);
        let ep = symbol_short!("bridge");

        env.as_contract(&contract_id, || {
            // Set a tight limit for testing
            RateLimiter::set_endpoint_config(
                &env,
                &ep,
                EndpointConfig {
                    max_calls: 3,
                    window_ledgers: 100,
                },
            );
            set_seq(&env, 1);
            assert!(RateLimiter::check_rate_limit(&env, &caller, &ep).is_ok());
            assert!(RateLimiter::check_rate_limit(&env, &caller, &ep).is_ok());
            assert!(RateLimiter::check_rate_limit(&env, &caller, &ep).is_ok());
            // 4th call should be rejected
            assert_eq!(
                RateLimiter::check_rate_limit(&env, &caller, &ep),
                Err(BridgeError::RetryLimitExceeded)
            );
        });
    }

    #[test]
    fn resets_after_window_expires() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());
        let caller = Address::generate(&env);
        let ep = symbol_short!("bridge");

        env.as_contract(&contract_id, || {
            RateLimiter::set_endpoint_config(
                &env,
                &ep,
                EndpointConfig {
                    max_calls: 1,
                    window_ledgers: 10,
                },
            );
            set_seq(&env, 1);
            assert!(RateLimiter::check_rate_limit(&env, &caller, &ep).is_ok());
            // Exhausted
            assert!(RateLimiter::check_rate_limit(&env, &caller, &ep).is_err());
            // Advance past window
            set_seq(&env, 12);
            // Should reset and allow again
            assert!(RateLimiter::check_rate_limit(&env, &caller, &ep).is_ok());
        });
    }

    #[test]
    fn different_users_have_independent_limits() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let ep = symbol_short!("bridge");

        env.as_contract(&contract_id, || {
            RateLimiter::set_endpoint_config(
                &env,
                &ep,
                EndpointConfig {
                    max_calls: 1,
                    window_ledgers: 100,
                },
            );
            set_seq(&env, 1);
            assert!(RateLimiter::check_rate_limit(&env, &alice, &ep).is_ok());
            // Alice exhausted, Bob still fine
            assert!(RateLimiter::check_rate_limit(&env, &alice, &ep).is_err());
            assert!(RateLimiter::check_rate_limit(&env, &bob, &ep).is_ok());
        });
    }
}
