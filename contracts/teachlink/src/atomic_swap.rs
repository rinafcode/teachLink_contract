//! Cross-Chain Atomic Swap Module
//!
//! This module implements atomic swap functionality for cross-chain token exchanges
//! using hash time-locked contracts (HTLC).

use crate::errors::BridgeError;
use crate::events::{SwapCompletedEvent, SwapInitiatedEvent, SwapRefundedEvent};
use crate::storage::{ATOMIC_SWAPS, SWAP_COUNTER};
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

        // Validate inputs
        if initiator_amount <= 0 || counterparty_amount <= 0 {
            return Err(BridgeError::AmountMustBePositive);
        }

        if hashlock.len() != HASH_LENGTH {
            return Err(BridgeError::InvalidHashlock);
        }

        if timelock < MIN_TIMELOCK || timelock > MAX_TIMELOCK {
            return Err(BridgeError::InvalidInput);
        }

        if initiator == counterparty {
            return Err(BridgeError::InvalidInput);
        }

        // Get swap counter
        let mut swap_counter: u64 = env
            .storage()
            .instance()
            .get(&SWAP_COUNTER)
            .unwrap_or(0u64);
        swap_counter += 1;

        // Transfer initiator tokens to contract
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

        // Create swap
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

        // Store swap
        let mut swaps: soroban_sdk::Map<u64, AtomicSwap> = env
            .storage()
            .instance()
            .get(&ATOMIC_SWAPS)
            .unwrap_or_else(|| soroban_sdk::Map::new(env));
        swaps.set(swap_counter, swap);
        env.storage().instance().set(&ATOMIC_SWAPS, &swaps);
        env.storage().instance().set(&SWAP_COUNTER, &swap_counter);

        // Emit event
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
    }

    /// Accept and complete an atomic swap (counterparty)
    pub fn accept_swap(
        env: &Env,
        swap_id: u64,
        counterparty: Address,
        preimage: Bytes,
    ) -> Result<(), BridgeError> {
        counterparty.require_auth();

        // Get swap
        let mut swaps: soroban_sdk::Map<u64, AtomicSwap> = env
            .storage()
            .instance()
            .get(&ATOMIC_SWAPS)
            .unwrap_or_else(|| soroban_sdk::Map::new(env));
        let mut swap = swaps
            .get(swap_id)
            .ok_or(BridgeError::SwapNotFound)?;

        // Validate swap state
        if swap.status != SwapStatus::Initiated {
            return Err(BridgeError::SwapAlreadyCompleted);
        }

        // Check timelock
        if env.ledger().timestamp() > swap.timelock {
            swap.status = SwapStatus::Expired;
            swaps.set(swap_id, swap);
            env.storage().instance().set(&ATOMIC_SWAPS, &swaps);
            return Err(BridgeError::TimelockExpired);
        }

        // Verify counterparty
        if swap.counterparty != counterparty {
            return Err(BridgeError::Unauthorized);
        }

        // Verify hashlock
        if !Self::verify_hashlock(env, &preimage, &swap.hashlock) {
            return Err(BridgeError::InvalidHashlock);
        }

        // Transfer counterparty tokens to contract
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

        // Execute the swap
        // 1. Send counterparty tokens to initiator
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

        // 2. Send initiator tokens to counterparty
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

        // Update swap status
        swap.status = SwapStatus::Completed;
        swaps.set(swap_id, swap);
        env.storage().instance().set(&ATOMIC_SWAPS, &swaps);

        // Emit event
        SwapCompletedEvent {
            swap_id,
            completed_at: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    /// Refund a swap after timelock expires (initiator only)
    pub fn refund_swap(
        env: &Env,
        swap_id: u64,
        initiator: Address,
    ) -> Result<(), BridgeError> {
        initiator.require_auth();

        // Get swap
        let mut swaps: soroban_sdk::Map<u64, AtomicSwap> = env
            .storage()
            .instance()
            .get(&ATOMIC_SWAPS)
            .unwrap_or_else(|| soroban_sdk::Map::new(env));
        let mut swap = swaps
            .get(swap_id)
            .ok_or(BridgeError::SwapNotFound)?;

        // Verify initiator
        if swap.initiator != initiator {
            return Err(BridgeError::Unauthorized);
        }

        // Check if already completed or refunded
        if swap.status == SwapStatus::Completed {
            return Err(BridgeError::SwapAlreadyCompleted);
        }
        if swap.status == SwapStatus::Refunded {
            return Err(BridgeError::SwapAlreadyCompleted);
        }

        // Check timelock
        if env.ledger().timestamp() <= swap.timelock {
            return Err(BridgeError::TimeoutNotReached);
        }

        // Refund initiator tokens
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

        // Update swap status
        swap.status = SwapStatus::Refunded;
        swaps.set(swap_id, swap.clone());
        env.storage().instance().set(&ATOMIC_SWAPS, &swaps);

        // Emit event
        SwapRefundedEvent {
            swap_id,
            refunded_to: initiator.clone(),
            amount: swap.initiator_amount,
        }
        .publish(env);

        Ok(())
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
        // In a real implementation, this would hash the preimage and compare
        // For Soroban, we'd use the SDK's crypto functions
        // This is a simplified placeholder
        
        if preimage.len() == 0 {
            return false;
        }

        // TODO: Implement actual SHA256 hashing
        // For now, we'll do a simple length check as placeholder
        hashlock.len() == HASH_LENGTH
    }

    /// Get swap count
    pub fn get_swap_count(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&SWAP_COUNTER)
            .unwrap_or(0u64)
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
