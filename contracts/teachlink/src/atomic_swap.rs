//! Cross-Chain Atomic Swap Module
//!
//! This module implements atomic swap functionality for cross-chain token exchanges
//! using hash time-locked contracts (HTLC).

use crate::errors::BridgeError;
use crate::events::{SwapCompletedEvent, SwapInitiatedEvent, SwapRefundedEvent};
use crate::storage::{ATOMIC_SWAPS, SWAP_COUNTER, SWAPS_BY_INITIATOR, SWAPS_BY_COUNTERPARTY, SWAPS_BY_STATUS};
use crate::types::{AtomicSwap, SwapStatus};
use soroban_sdk::{symbol_short, vec, Address, Bytes, Env, IntoVal, Vec, Map};

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
        let mut swap_counter: u64 = env.storage().instance().get(&SWAP_COUNTER).unwrap_or(0u64);
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
        swaps.set(swap_counter, swap.clone());
        env.storage().instance().set(&ATOMIC_SWAPS, &swaps);
        env.storage().instance().set(&SWAP_COUNTER, &swap_counter);

        // Update indexes
        Self::update_indexes(env, swap_counter, &swap);

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
        let mut swap = swaps.get(swap_id).ok_or(BridgeError::SwapNotFound)?;

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
        swaps.set(swap_id, swap.clone());
        env.storage().instance().set(&ATOMIC_SWAPS, &swaps);

        // Update indexes
        Self::update_indexes(env, swap_id, &swap);

        // Emit event
        SwapCompletedEvent {
            swap_id,
            completed_at: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    /// Refund a swap after timelock expires (initiator only)
    pub fn refund_swap(env: &Env, swap_id: u64, initiator: Address) -> Result<(), BridgeError> {
        initiator.require_auth();

        // Get swap
        let mut swaps: soroban_sdk::Map<u64, AtomicSwap> = env
            .storage()
            .instance()
            .get(&ATOMIC_SWAPS)
            .unwrap_or_else(|| soroban_sdk::Map::new(env));
        let mut swap = swaps.get(swap_id).ok_or(BridgeError::SwapNotFound)?;

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

        // Update indexes
        Self::update_indexes(env, swap_id, &swap);

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

    /// Get swaps by initiator (using indexed lookup)
    pub fn get_swaps_by_initiator(env: &Env, initiator: Address) -> Vec<u64> {
        let swaps_by_initiator: Map<Address, Vec<u64>> = env
            .storage()
            .instance()
            .get(&SWAPS_BY_INITIATOR)
            .unwrap_or_else(|| Map::new(env));
        swaps_by_initiator.get(initiator).unwrap_or_else(|| Vec::new(env))
    }

    /// Get swaps by counterparty (using indexed lookup)
    pub fn get_swaps_by_counterparty(env: &Env, counterparty: Address) -> Vec<u64> {
        let swaps_by_counterparty: Map<Address, Vec<u64>> = env
            .storage()
            .instance()
            .get(&SWAPS_BY_COUNTERPARTY)
            .unwrap_or_else(|| Map::new(env));
        swaps_by_counterparty.get(counterparty).unwrap_or_else(|| Vec::new(env))
    }

    /// Get active swaps (using indexed lookup)
    pub fn get_active_swaps(env: &Env) -> Vec<u64> {
        let swaps_by_status: Map<SwapStatus, Vec<u64>> = env
            .storage()
            .instance()
            .get(&SWAPS_BY_STATUS)
            .unwrap_or_else(|| Map::new(env));
        swaps_by_status.get(SwapStatus::Initiated).unwrap_or_else(|| Vec::new(env))
    }

    /// Get expired swaps (using indexed lookup)
    pub fn get_expired_swaps(env: &Env) -> Vec<u64> {
        let swaps_by_status: Map<SwapStatus, Vec<u64>> = env
            .storage()
            .instance()
            .get(&SWAPS_BY_STATUS)
            .unwrap_or_else(|| Map::new(env));
        
        let active_swaps = swaps_by_status.get(SwapStatus::Initiated).unwrap_or_else(|| Vec::new(env));
        let current_time = env.ledger().timestamp();
        let mut result = Vec::new(env);

        // Check each active swap for expiration
        for swap_id in active_swaps.iter() {
            if let Some(swap) = Self::get_swap(env, *swap_id) {
                if current_time > swap.timelock {
                    result.push_back(*swap_id);
                }
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

    /// Update indexes for a swap
    fn update_indexes(env: &Env, swap_id: u64, swap: &AtomicSwap) {
        // Update initiator index
        let mut swaps_by_initiator: Map<Address, Vec<u64>> = env
            .storage()
            .instance()
            .get(&SWAPS_BY_INITIATOR)
            .unwrap_or_else(|| Map::new(env));
        
        let mut initiator_swaps = swaps_by_initiator
            .get(swap.initiator.clone())
            .unwrap_or_else(|| Vec::new(env));
        
        // Add swap_id if not already present
        if !initiator_swaps.iter().any(|&id| id == swap_id) {
            initiator_swaps.push_back(swap_id);
        }
        swaps_by_initiator.set(swap.initiator.clone(), initiator_swaps);
        env.storage().instance().set(&SWAPS_BY_INITIATOR, &swaps_by_initiator);

        // Update counterparty index
        let mut swaps_by_counterparty: Map<Address, Vec<u64>> = env
            .storage()
            .instance()
            .get(&SWAPS_BY_COUNTERPARTY)
            .unwrap_or_else(|| Map::new(env));
        
        let mut counterparty_swaps = swaps_by_counterparty
            .get(swap.counterparty.clone())
            .unwrap_or_else(|| Vec::new(env));
        
        // Add swap_id if not already present
        if !counterparty_swaps.iter().any(|&id| id == swap_id) {
            counterparty_swaps.push_back(swap_id);
        }
        swaps_by_counterparty.set(swap.counterparty.clone(), counterparty_swaps);
        env.storage().instance().set(&SWAPS_BY_COUNTERPARTY, &swaps_by_counterparty);

        // Update status index
        let mut swaps_by_status: Map<SwapStatus, Vec<u64>> = env
            .storage()
            .instance()
            .get(&SWAPS_BY_STATUS)
            .unwrap_or_else(|| Map::new(env));
        
        // Remove from all status indexes first (to handle status changes)
        for status in [SwapStatus::Initiated, SwapStatus::Completed, SwapStatus::Refunded, SwapStatus::Expired] {
            if let Some(mut status_swaps) = swaps_by_status.get(status) {
                status_swaps = status_swaps.iter().filter(|&&id| id != swap_id).copied().collect::<Vec<_>>(env);
                swaps_by_status.set(status, status_swaps);
            }
        }
        
        // Add to current status index
        let mut current_status_swaps = swaps_by_status
            .get(swap.status.clone())
            .unwrap_or_else(|| Vec::new(env));
        current_status_swaps.push_back(swap_id);
        swaps_by_status.set(swap.status.clone(), current_status_swaps);
        env.storage().instance().set(&SWAPS_BY_STATUS, &swaps_by_status);
    }
}
