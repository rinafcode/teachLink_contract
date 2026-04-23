//! Cross-Chain Atomic Swap Module
//!
//! This module implements atomic swap functionality for cross-chain token exchanges
//! using hash time-locked contracts (HTLC).

use crate::errors::BridgeError;
use crate::events::{SwapCompletedEvent, SwapInitiatedEvent, SwapRefundedEvent};
use crate::reentrancy;
use crate::storage::{ATOMIC_SWAPS, SWAP_COUNTER, SWAP_GUARD};
use crate::types::{AtomicSwap, SwapStatus};
use soroban_sdk::{symbol_short, vec, Address, Bytes, Env, IntoVal, Vec};

/// Minimum timelock duration (1 hour)
pub const MIN_TIMELOCK: u64 = 3_600;

/// Maximum timelock duration (7 days)
pub const MAX_TIMELOCK: u64 = 604_800;

/// Hash length (32 bytes for SHA256)
pub const HASH_LENGTH: u32 = 32;

/// Atomic Swap Manager
pub struct AtomicSwapManager;

impl AtomicSwapManager {
    /// Initiate an atomic swap
    pub fn initiate_swap(
        env: &Env,
        initiator: Address,
        initiator_token: Address,
        initiator_amount: i128,
        counterparty: Address,
        counterparty_token: Address,
        counterparty_amount: i128,
        hashlock: Bytes,
        timelock: u64,
    ) -> Result<u64, BridgeError> {
        initiator.require_auth();

        reentrancy::with_guard(env, &SWAP_GUARD, BridgeError::ReentrancyDetected, || {
            // Validate address inputs
            crate::validation::AddressValidator::validate(env, &initiator)
                .map_err(|_| BridgeError::InvalidInput)?;
            crate::validation::AddressValidator::validate(env, &counterparty)
                .map_err(|_| BridgeError::InvalidInput)?;
            crate::validation::AddressValidator::validate(env, &initiator_token)
                .map_err(|_| BridgeError::InvalidInput)?;
            crate::validation::AddressValidator::validate(env, &counterparty_token)
                .map_err(|_| BridgeError::InvalidInput)?;

            crate::validation::InputSanitizer::sanitize_amount(initiator_amount)
                .map_err(|_| BridgeError::AmountMustBePositive)?;
            crate::validation::InputSanitizer::sanitize_amount(counterparty_amount)
                .map_err(|_| BridgeError::AmountMustBePositive)?;

            if hashlock.len() != HASH_LENGTH {
                return Err(BridgeError::InvalidHashlock);
            }
            if timelock < MIN_TIMELOCK || timelock > MAX_TIMELOCK {
                return Err(BridgeError::InvalidInput);
            }
            if initiator == counterparty {
                return Err(BridgeError::InvalidInput);
            }

            let mut swap_counter: u64 = env.storage().instance().get(&SWAP_COUNTER).unwrap_or(0u64);
            swap_counter += 1;

            let swap = AtomicSwap {
                swap_id: swap_counter,
                initiator: initiator.clone(),
                initiator_token: initiator_token.clone(),
                initiator_amount,
                counterparty: counterparty.clone(),
                counterparty_token: counterparty_token.clone(),
                counterparty_amount,
                hashlock: hashlock.clone(),
                timelock: env.ledger().timestamp() + timelock,
                status: SwapStatus::Initiated,
                created_at: env.ledger().timestamp(),
            };

            let mut swaps: soroban_sdk::Map<u64, AtomicSwap> = env
                .storage()
                .instance()
                .get(&ATOMIC_SWAPS)
                .unwrap_or_else(|| soroban_sdk::Map::new(env));
            swaps.set(swap_counter, swap);
            env.storage().instance().set(&ATOMIC_SWAPS, &swaps);
            env.storage().instance().set(&SWAP_COUNTER, &swap_counter);

            env.invoke_contract::<()>(
                &initiator_token,
                &symbol_short!("transfer"),
                vec![
                    env,
                    initiator.clone().into_val(env),
                    env.current_contract_address().into_val(env),
                    initiator_amount.into_val(env),
                ],
            );

            SwapInitiatedEvent {
                swap_id: swap_counter,
                initiator: initiator.clone(),
                initiator_amount,
                counterparty: counterparty.clone(),
                counterparty_amount,
                timelock: env.ledger().timestamp() + timelock,
            }
            .publish(env);

            Ok(swap_counter)
        })
    }

    /// Accept and complete an atomic swap (counterparty)
    pub fn accept_swap(
        env: &Env,
        swap_id: u64,
        counterparty: Address,
        preimage: Bytes,
    ) -> Result<(), BridgeError> {
        counterparty.require_auth();

        reentrancy::with_guard(env, &SWAP_GUARD, BridgeError::ReentrancyDetected, || {
            let mut swaps: soroban_sdk::Map<u64, AtomicSwap> = env
                .storage()
                .instance()
                .get(&ATOMIC_SWAPS)
                .unwrap_or_else(|| soroban_sdk::Map::new(env));
            let mut swap = swaps.get(swap_id).ok_or(BridgeError::SwapNotFound)?;

            if swap.status != SwapStatus::Initiated {
                return Err(BridgeError::SwapAlreadyCompleted);
            }

            if env.ledger().timestamp() > swap.timelock {
                swap.status = SwapStatus::Expired;
                swaps.set(swap_id, swap);
                env.storage().instance().set(&ATOMIC_SWAPS, &swaps);
                return Err(BridgeError::TimelockExpired);
            }

            if swap.counterparty != counterparty {
                return Err(BridgeError::Unauthorized);
            }
            if !Self::verify_hashlock(env, &preimage, &swap.hashlock) {
                return Err(BridgeError::InvalidHashlock);
            }

            swap.status = SwapStatus::Completed;
            swaps.set(swap_id, swap.clone());
            env.storage().instance().set(&ATOMIC_SWAPS, &swaps);

            env.invoke_contract::<()>(
                &swap.counterparty_token,
                &symbol_short!("transfer"),
                vec![
                    env,
                    counterparty.clone().into_val(env),
                    env.current_contract_address().into_val(env),
                    swap.counterparty_amount.into_val(env),
                ],
            );

            env.invoke_contract::<()>(
                &swap.counterparty_token,
                &symbol_short!("transfer"),
                vec![
                    env,
                    env.current_contract_address().into_val(env),
                    swap.initiator.clone().into_val(env),
                    swap.counterparty_amount.into_val(env),
                ],
            );

            env.invoke_contract::<()>(
                &swap.initiator_token,
                &symbol_short!("transfer"),
                vec![
                    env,
                    env.current_contract_address().into_val(env),
                    counterparty.clone().into_val(env),
                    swap.initiator_amount.into_val(env),
                ],
            );

            SwapCompletedEvent {
                swap_id,
                completed_at: env.ledger().timestamp(),
            }
            .publish(env);

            Ok(())
        })
    }

    /// Refund a swap after timelock expires (initiator only)
    pub fn refund_swap(env: &Env, swap_id: u64, initiator: Address) -> Result<(), BridgeError> {
        initiator.require_auth();

        reentrancy::with_guard(env, &SWAP_GUARD, BridgeError::ReentrancyDetected, || {
            let mut swaps: soroban_sdk::Map<u64, AtomicSwap> = env
                .storage()
                .instance()
                .get(&ATOMIC_SWAPS)
                .unwrap_or_else(|| soroban_sdk::Map::new(env));
            let mut swap = swaps.get(swap_id).ok_or(BridgeError::SwapNotFound)?;

            if swap.initiator != initiator {
                return Err(BridgeError::Unauthorized);
            }
            if swap.status == SwapStatus::Completed || swap.status == SwapStatus::Refunded {
                return Err(BridgeError::SwapAlreadyCompleted);
            }
            if env.ledger().timestamp() <= swap.timelock {
                return Err(BridgeError::TimeoutNotReached);
            }

            swap.status = SwapStatus::Refunded;
            swaps.set(swap_id, swap.clone());
            env.storage().instance().set(&ATOMIC_SWAPS, &swaps);

            env.invoke_contract::<()>(
                &swap.initiator_token,
                &symbol_short!("transfer"),
                vec![
                    env,
                    env.current_contract_address().into_val(env),
                    initiator.clone().into_val(env),
                    swap.initiator_amount.into_val(env),
                ],
            );

            SwapRefundedEvent {
                swap_id,
                refunded_to: initiator.clone(),
                amount: swap.initiator_amount,
            }
            .publish(env);

            Ok(())
        })
    }

    /// Get swap by ID
    pub fn get_swap(env: &Env, swap_id: u64) -> Option<AtomicSwap> {
        let swaps: soroban_sdk::Map<u64, AtomicSwap> = env
            .storage()
            .instance()
            .get(&ATOMIC_SWAPS)
            .unwrap_or_else(|| soroban_sdk::Map::new(env));
        swaps.get(swap_id)
    }

    /// Get swaps by initiator
    pub fn get_swaps_by_initiator(env: &Env, initiator: Address) -> Vec<u64> {
        let swaps: soroban_sdk::Map<u64, AtomicSwap> = env
            .storage()
            .instance()
            .get(&ATOMIC_SWAPS)
            .unwrap_or_else(|| soroban_sdk::Map::new(env));

        let mut result = Vec::new(env);
        for (swap_id, swap) in swaps.iter() {
            if swap.initiator == initiator {
                result.push_back(swap_id);
            }
        }
        result
    }

    /// Get swaps by counterparty
    pub fn get_swaps_by_counterparty(env: &Env, counterparty: Address) -> Vec<u64> {
        let swaps: soroban_sdk::Map<u64, AtomicSwap> = env
            .storage()
            .instance()
            .get(&ATOMIC_SWAPS)
            .unwrap_or_else(|| soroban_sdk::Map::new(env));

        let mut result = Vec::new(env);
        for (swap_id, swap) in swaps.iter() {
            if swap.counterparty == counterparty {
                result.push_back(swap_id);
            }
        }
        result
    }

    /// Get active swaps (initiated but not completed)
    pub fn get_active_swaps(env: &Env) -> Vec<u64> {
        let swaps: soroban_sdk::Map<u64, AtomicSwap> = env
            .storage()
            .instance()
            .get(&ATOMIC_SWAPS)
            .unwrap_or_else(|| soroban_sdk::Map::new(env));

        let mut result = Vec::new(env);
        for (swap_id, swap) in swaps.iter() {
            if swap.status == SwapStatus::Initiated {
                result.push_back(swap_id);
            }
        }
        result
    }

    /// Get expired swaps
    pub fn get_expired_swaps(env: &Env) -> Vec<u64> {
        let swaps: soroban_sdk::Map<u64, AtomicSwap> = env
            .storage()
            .instance()
            .get(&ATOMIC_SWAPS)
            .unwrap_or_else(|| soroban_sdk::Map::new(env));

        let current_time = env.ledger().timestamp();
        let mut result = Vec::new(env);

        for (swap_id, swap) in swaps.iter() {
            if swap.status == SwapStatus::Initiated && current_time > swap.timelock {
                result.push_back(swap_id);
            }
        }
        result
    }

    /// Verify hashlock against preimage
    fn verify_hashlock(env: &Env, preimage: &Bytes, hashlock: &Bytes) -> bool {
        if preimage.is_empty() {
            return false;
        }

        // Hash the preimage
        let hash_bytesn = env.crypto().sha256(preimage);

        // Convert BytesN<32> to Bytes for comparison
        let expected_bytes: Bytes = hash_bytesn.into();

        hashlock == &expected_bytes
    }

    /// Get swap count
    pub fn get_swap_count(env: &Env) -> u64 {
        env.storage().instance().get(&SWAP_COUNTER).unwrap_or(0u64)
    }

    /// Check if swap is expired
    pub fn is_swap_expired(env: &Env, swap_id: u64) -> bool {
        if let Some(swap) = Self::get_swap(env, swap_id) {
            env.ledger().timestamp() > swap.timelock && swap.status == SwapStatus::Initiated
        } else {
            false
        }
    }

    /// Calculate swap rate (price ratio)
    pub fn calculate_swap_rate(initiator_amount: i128, counterparty_amount: i128) -> f64 {
        if initiator_amount == 0 {
            return 0.0;
        }
        (counterparty_amount as f64) / (initiator_amount as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::AtomicSwapManager;
    use crate::errors::BridgeError;
    use crate::storage::SWAP_GUARD;
    use crate::TeachLinkBridge;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env};

    #[test]
    fn initiate_swap_rejects_when_reentrancy_guard_active() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());

        env.as_contract(&contract_id, || {
            env.storage().instance().set(&SWAP_GUARD, &true);

            let initiator = Address::generate(&env);
            let counterparty = Address::generate(&env);
            let token_a = Address::generate(&env);
            let token_b = Address::generate(&env);
            let hashlock = Bytes::from_slice(&env, &[0xaa; 32]);

            let result = AtomicSwapManager::initiate_swap(
                &env,
                initiator,
                token_a,
                100,
                counterparty,
                token_b,
                100,
                hashlock,
                super::MIN_TIMELOCK,
            );

            assert_eq!(result, Err(BridgeError::ReentrancyDetected));
        });
    }
}
