//! Delegated Voting (Liquid Democracy) Module
//!
//! Implements vote delegation where token holders can delegate their
//! voting power to trusted representatives. Supports:
//!
//! - Direct delegation (A -> B)
//! - Delegation chain resolution (A -> B -> C)
//! - Delegation revocation
//! - Time-bounded delegations with expiry
//! - Maximum delegation depth to prevent infinite chains
//!
//! # Liquid Democracy
//!
//! Token holders can delegate their votes while retaining the ability to:
//! - Override their delegate by voting directly
//! - Revoke delegation at any time
//! - Set delegation expiry dates

use soroban_sdk::{token, Address, Env};

use crate::events;
use crate::storage::{DELEGATIONS, DELEG_PWR};
use crate::types::{DelegatedPower, Delegation, DelegationKey, GovernanceConfig};

/// Maximum allowed delegation chain depth (hardcoded safety limit)
const MAX_CHAIN_DEPTH: u32 = 5;

pub struct DelegationManager;

impl DelegationManager {
    /// Delegate voting power to another address
    ///
    /// Creates a delegation from `delegator` to `delegate`. The delegate
    /// will receive the delegator's voting power when casting votes,
    /// unless the delegator votes directly (which overrides delegation).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `config` - Current governance configuration
    /// * `delegator` - Address delegating their voting power
    /// * `delegate` - Address receiving the voting power
    /// * `expires_at` - Optional expiry timestamp (0 = no expiry)
    ///
    /// # Panics
    /// * If delegator tries to delegate to themselves
    /// * If a circular delegation would be created
    /// * If max delegation depth would be exceeded
    pub fn delegate(
        env: &Env,
        config: &GovernanceConfig,
        delegator: Address,
        delegate: Address,
        expires_at: u64,
    ) {
        delegator.require_auth();

        // Cannot delegate to self
        assert!(
            delegator != delegate,
            "ERR_SELF_DELEGATION: Cannot delegate to yourself"
        );

        // Check for circular delegation
        assert!(
            !Self::would_create_cycle(env, &delegate, &delegator, config.max_delegation_depth),
            "ERR_CIRCULAR_DELEGATION: Delegation would create a circular chain"
        );

        let now = env.ledger().timestamp();
        let del_key = DelegationKey {
            delegator: delegator.clone(),
        };

        // Revoke existing delegation if any
        if env
            .storage()
            .persistent()
            .has(&(DELEGATIONS, del_key.clone()))
        {
            Self::_revoke_internal(env, config, &delegator);
        }

        // Create delegation record
        let delegation = Delegation {
            delegator: delegator.clone(),
            delegate: delegate.clone(),
            created_at: now,
            active: true,
            expires_at,
        };

        env.storage()
            .persistent()
            .set(&(DELEGATIONS, del_key), &delegation);

        // Update delegated power for the delegate
        Self::add_delegated_power(env, config, &delegator, &delegate);

        // Emit event
        events::delegation_created(env, &delegator, &delegate, expires_at);
    }

    /// Revoke an existing delegation
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `config` - Current governance configuration
    /// * `delegator` - Address revoking their delegation
    ///
    /// # Panics
    /// * If no delegation exists for the delegator
    pub fn revoke_delegation(env: &Env, config: &GovernanceConfig, delegator: Address) {
        delegator.require_auth();

        Self::_revoke_internal(env, config, &delegator);

        events::delegation_revoked(env, &delegator);
    }

    /// Internal revocation logic (no auth check)
    fn _revoke_internal(env: &Env, config: &GovernanceConfig, delegator: &Address) {
        let del_key = DelegationKey {
            delegator: delegator.clone(),
        };

        let delegation: Delegation = env
            .storage()
            .persistent()
            .get(&(DELEGATIONS, del_key.clone()))
            .expect("ERR_DELEGATION_NOT_FOUND: No active delegation found");

        // Remove delegated power from the delegate
        Self::remove_delegated_power(env, config, delegator, &delegation.delegate);

        // Remove delegation record
        env.storage().persistent().remove(&(DELEGATIONS, del_key));
    }

    /// Get the effective delegate for an address, resolving chains
    ///
    /// Follows the delegation chain to find the final delegate who
    /// will actually cast the vote. Respects expiry times.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `delegator` - Address to resolve delegation for
    /// * `max_depth` - Maximum chain depth to follow
    ///
    /// # Returns
    /// The final delegate address, or the original address if no delegation
    pub fn get_effective_delegate(env: &Env, delegator: &Address, max_depth: u32) -> Address {
        let mut current = delegator.clone();
        let now = env.ledger().timestamp();
        let effective_max = if max_depth > MAX_CHAIN_DEPTH {
            MAX_CHAIN_DEPTH
        } else {
            max_depth
        };

        for _depth in 0..effective_max {
            let del_key = DelegationKey {
                delegator: current.clone(),
            };

            match env
                .storage()
                .persistent()
                .get::<_, Delegation>(&(DELEGATIONS, del_key))
            {
                Some(delegation) => {
                    // Check if delegation is still active and not expired
                    if delegation.active
                        && (delegation.expires_at == 0 || delegation.expires_at > now)
                    {
                        current = delegation.delegate;
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }

        current
    }

    /// Get the total voting power for an address, including delegated power
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `config` - Governance configuration
    /// * `voter` - Address to get total voting power for
    ///
    /// # Returns
    /// Tuple of (own_power, delegated_power, total_power)
    pub fn get_total_voting_power(
        env: &Env,
        config: &GovernanceConfig,
        voter: &Address,
    ) -> (i128, i128, i128) {
        let token_client = token::Client::new(env, &config.token);
        let own_power = token_client.balance(voter);

        let delegated_power = Self::get_delegated_power(env, voter);

        (own_power, delegated_power, own_power + delegated_power)
    }

    /// Get the delegation record for an address
    pub fn get_delegation(env: &Env, delegator: &Address) -> Option<Delegation> {
        let del_key = DelegationKey {
            delegator: delegator.clone(),
        };
        env.storage().persistent().get(&(DELEGATIONS, del_key))
    }

    /// Check if an address has delegated their votes
    pub fn has_delegated(env: &Env, delegator: &Address) -> bool {
        let del_key = DelegationKey {
            delegator: delegator.clone(),
        };
        env.storage().persistent().has(&(DELEGATIONS, del_key))
    }

    /// Get the delegated power received by a delegate
    pub fn get_delegated_power(env: &Env, delegate: &Address) -> i128 {
        env.storage()
            .persistent()
            .get::<_, DelegatedPower>(&(DELEG_PWR, delegate.clone()))
            .map(|dp| dp.total_power)
            .unwrap_or(0)
    }

    /// Get full delegated power info for a delegate
    pub fn get_delegated_power_info(env: &Env, delegate: &Address) -> Option<DelegatedPower> {
        env.storage()
            .persistent()
            .get(&(DELEG_PWR, delegate.clone()))
    }

    // ========== Internal Helpers ==========

    /// Check if creating a delegation would result in a cycle
    fn would_create_cycle(env: &Env, from: &Address, target: &Address, max_depth: u32) -> bool {
        let mut current = from.clone();
        let effective_max = if max_depth > MAX_CHAIN_DEPTH {
            MAX_CHAIN_DEPTH
        } else {
            max_depth
        };

        for _depth in 0..effective_max {
            if current == *target {
                return true;
            }

            let del_key = DelegationKey {
                delegator: current.clone(),
            };

            match env
                .storage()
                .persistent()
                .get::<_, Delegation>(&(DELEGATIONS, del_key))
            {
                Some(delegation) if delegation.active => {
                    current = delegation.delegate;
                }
                _ => break,
            }
        }

        false
    }

    /// Add delegated power to a delegate's accumulator
    fn add_delegated_power(
        env: &Env,
        config: &GovernanceConfig,
        delegator: &Address,
        delegate: &Address,
    ) {
        let token_client = token::Client::new(env, &config.token);
        let power = token_client.balance(delegator);

        let mut dp = env
            .storage()
            .persistent()
            .get::<_, DelegatedPower>(&(DELEG_PWR, delegate.clone()))
            .unwrap_or(DelegatedPower {
                delegate: delegate.clone(),
                total_power: 0,
                delegator_count: 0,
            });

        dp.total_power += power;
        dp.delegator_count += 1;

        env.storage()
            .persistent()
            .set(&(DELEG_PWR, delegate.clone()), &dp);
    }

    /// Remove delegated power from a delegate's accumulator
    fn remove_delegated_power(
        env: &Env,
        config: &GovernanceConfig,
        delegator: &Address,
        delegate: &Address,
    ) {
        let token_client = token::Client::new(env, &config.token);
        let power = token_client.balance(delegator);

        if let Some(mut dp) = env
            .storage()
            .persistent()
            .get::<_, DelegatedPower>(&(DELEG_PWR, delegate.clone()))
        {
            dp.total_power = if dp.total_power > power {
                dp.total_power - power
            } else {
                0
            };

            if dp.delegator_count > 0 {
                dp.delegator_count -= 1;
            }

            env.storage()
                .persistent()
                .set(&(DELEG_PWR, delegate.clone()), &dp);
        }
    }
}
