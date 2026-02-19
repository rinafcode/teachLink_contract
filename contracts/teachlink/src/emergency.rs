//! Emergency Pause and Recovery Module
//!
//! This module implements circuit breaker functionality and emergency controls
//! to protect the bridge during critical situations.

use crate::errors::BridgeError;
use crate::events::{
    BridgePausedEvent, BridgeResumedEvent, CircuitBreakerTriggeredEvent,
};
use crate::storage::{CIRCUIT_BREAKERS, EMERGENCY_STATE, PAUSED_CHAINS};
use crate::types::{CircuitBreaker, EmergencyState};
use soroban_sdk::{Address, Bytes, Env, Map, Vec};

/// Authorized pausers (admin + security council)
pub const SECURITY_COUNCIL_SIZE: u32 = 5;

/// Daily volume reset period (24 hours)
pub const DAILY_VOLUME_RESET: u64 = 86_400;

/// Emergency Manager
pub struct EmergencyManager;

impl EmergencyManager {
    /// Pause the entire bridge
    pub fn pause_bridge(
        env: &Env,
        pauser: Address,
        reason: Bytes,
    ) -> Result<(), BridgeError> {
        pauser.require_auth();

        // Check if already paused
        let emergency_state: EmergencyState = env
            .storage()
            .instance()
            .get(&EMERGENCY_STATE)
            .unwrap_or(EmergencyState {
                is_paused: false,
                paused_at: 0,
                paused_by: pauser.clone(),
                reason: Bytes::new(env),
                affected_chains: Vec::new(env),
            });

        if emergency_state.is_paused {
            return Err(BridgeError::BridgePaused);
        }

        // Create emergency state
        let new_state = EmergencyState {
            is_paused: true,
            paused_at: env.ledger().timestamp(),
            paused_by: pauser.clone(),
            reason: reason.clone(),
            affected_chains: Vec::new(env),
        };

        env.storage().instance().set(&EMERGENCY_STATE, &new_state);

        // Emit event
        BridgePausedEvent {
            paused_by: pauser.clone(),
            reason,
            paused_at: env.ledger().timestamp(),
            affected_chains: Vec::new(env),
        }
        .publish(env);

        Ok(())
    }

    /// Resume the bridge
    pub fn resume_bridge(env: &Env, resumer: Address) -> Result<(), BridgeError> {
        resumer.require_auth();

        // Check if paused
        let mut emergency_state: EmergencyState = env
            .storage()
            .instance()
            .get(&EMERGENCY_STATE)
            .unwrap_or(EmergencyState {
                is_paused: false,
                paused_at: 0,
                paused_by: resumer.clone(),
                reason: Bytes::new(env),
                affected_chains: Vec::new(env),
            });

        if !emergency_state.is_paused {
            return Err(BridgeError::InvalidInput);
        }

        // Resume bridge
        emergency_state.is_paused = false;
        env.storage().instance().set(&EMERGENCY_STATE, &emergency_state);

        // Emit event
        BridgeResumedEvent {
            resumed_by: resumer.clone(),
            resumed_at: env.ledger().timestamp(),
            affected_chains: Vec::new(env),
        }
        .publish(env);

        Ok(())
    }

    /// Pause specific chains
    pub fn pause_chains(
        env: &Env,
        pauser: Address,
        chain_ids: Vec<u32>,
        reason: Bytes,
    ) -> Result<(), BridgeError> {
        pauser.require_auth();

        let mut paused_chains: Map<u32, bool> = env
            .storage()
            .instance()
            .get(&PAUSED_CHAINS)
            .unwrap_or_else(|| Map::new(env));

        for chain_id in chain_ids.iter() {
            paused_chains.set(chain_id, true);
        }

        env.storage().instance().set(&PAUSED_CHAINS, &paused_chains);

        // Emit event
        BridgePausedEvent {
            paused_by: pauser.clone(),
            reason,
            paused_at: env.ledger().timestamp(),
            affected_chains: chain_ids,
        }
        .publish(env);

        Ok(())
    }

    /// Resume specific chains
    pub fn resume_chains(
        env: &Env,
        resumer: Address,
        chain_ids: Vec<u32>,
    ) -> Result<(), BridgeError> {
        resumer.require_auth();

        let mut paused_chains: Map<u32, bool> = env
            .storage()
            .instance()
            .get(&PAUSED_CHAINS)
            .unwrap_or_else(|| Map::new(env));

        for chain_id in chain_ids.iter() {
            paused_chains.set(chain_id, false);
        }

        env.storage().instance().set(&PAUSED_CHAINS, &paused_chains);

        // Emit event
        BridgeResumedEvent {
            resumed_by: resumer.clone(),
            resumed_at: env.ledger().timestamp(),
            affected_chains: chain_ids,
        }
        .publish(env);

        Ok(())
    }

    /// Initialize circuit breaker for a chain
    pub fn initialize_circuit_breaker(
        env: &Env,
        chain_id: u32,
        max_daily_volume: i128,
        max_transaction_amount: i128,
    ) -> Result<(), BridgeError> {
        let circuit_breaker = CircuitBreaker {
            chain_id,
            max_daily_volume,
            current_daily_volume: 0,
            max_transaction_amount,
            last_reset: env.ledger().timestamp(),
            is_triggered: false,
        };

        let mut circuit_breakers: Map<u32, CircuitBreaker> = env
            .storage()
            .instance()
            .get(&CIRCUIT_BREAKERS)
            .unwrap_or_else(|| Map::new(env));
        circuit_breakers.set(chain_id, circuit_breaker);
        env.storage().instance().set(&CIRCUIT_BREAKERS, &circuit_breakers);

        Ok(())
    }

    /// Check and update circuit breaker for a transaction
    pub fn check_circuit_breaker(
        env: &Env,
        chain_id: u32,
        amount: i128,
    ) -> Result<(), BridgeError> {
        let mut circuit_breakers: Map<u32, CircuitBreaker> = env
            .storage()
            .instance()
            .get(&CIRCUIT_BREAKERS)
            .unwrap_or_else(|| Map::new(env));
        
        let mut breaker = circuit_breakers
            .get(chain_id)
            .ok_or(BridgeError::DestinationChainNotSupported)?;

        // Check if already triggered
        if breaker.is_triggered {
            return Err(BridgeError::CircuitBreakerTriggered);
        }

        // Reset daily volume if needed
        let current_time = env.ledger().timestamp();
        if current_time - breaker.last_reset >= DAILY_VOLUME_RESET {
            breaker.current_daily_volume = 0;
            breaker.last_reset = current_time;
        }

        // Check transaction amount limit
        if amount > breaker.max_transaction_amount {
            // Trigger circuit breaker
            breaker.is_triggered = true;
            circuit_breakers.set(chain_id, breaker.clone());
            env.storage().instance().set(&CIRCUIT_BREAKERS, &circuit_breakers);

            // Emit event
            CircuitBreakerTriggeredEvent {
                chain_id,
                trigger_reason: Bytes::from_slice(env, b"max_transaction_exceeded"),
                triggered_at: current_time,
            }
            .publish(env);

            return Err(BridgeError::CircuitBreakerTriggered);
        }

        // Check daily volume limit
        if breaker.current_daily_volume + amount > breaker.max_daily_volume {
            // Trigger circuit breaker
            breaker.is_triggered = true;
            circuit_breakers.set(chain_id, breaker.clone());
            env.storage().instance().set(&CIRCUIT_BREAKERS, &circuit_breakers);

            // Emit event
            CircuitBreakerTriggeredEvent {
                chain_id,
                trigger_reason: Bytes::from_slice(env, b"daily_volume_exceeded"),
                triggered_at: current_time,
            }
            .publish(env);

            return Err(BridgeError::CircuitBreakerTriggered);
        }

        // Update volume
        breaker.current_daily_volume += amount;
        circuit_breakers.set(chain_id, breaker);
        env.storage().instance().set(&CIRCUIT_BREAKERS, &circuit_breakers);

        Ok(())
    }

    /// Reset circuit breaker for a chain
    pub fn reset_circuit_breaker(
        env: &Env,
        chain_id: u32,
        resetter: Address,
    ) -> Result<(), BridgeError> {
        resetter.require_auth();

        let mut circuit_breakers: Map<u32, CircuitBreaker> = env
            .storage()
            .instance()
            .get(&CIRCUIT_BREAKERS)
            .unwrap_or_else(|| Map::new(env));
        
        let mut breaker = circuit_breakers
            .get(chain_id)
            .ok_or(BridgeError::DestinationChainNotSupported)?;

        breaker.is_triggered = false;
        breaker.current_daily_volume = 0;
        breaker.last_reset = env.ledger().timestamp();

        circuit_breakers.set(chain_id, breaker);
        env.storage().instance().set(&CIRCUIT_BREAKERS, &circuit_breakers);

        Ok(())
    }

    /// Check if bridge is paused
    pub fn is_bridge_paused(env: &Env) -> bool {
        let emergency_state: EmergencyState = env
            .storage()
            .instance()
            .get(&EMERGENCY_STATE)
            .unwrap_or(EmergencyState {
                is_paused: false,
                paused_at: 0,
                paused_by: env.current_contract_address(),
                reason: Bytes::new(env),
                affected_chains: Vec::new(env),
            });
        emergency_state.is_paused
    }

    /// Check if a chain is paused
    pub fn is_chain_paused(env: &Env, chain_id: u32) -> bool {
        let paused_chains: Map<u32, bool> = env
            .storage()
            .instance()
            .get(&PAUSED_CHAINS)
            .unwrap_or_else(|| Map::new(env));
        paused_chains.get(chain_id).unwrap_or(false)
    }

    /// Get emergency state
    pub fn get_emergency_state(env: &Env) -> EmergencyState {
        env.storage()
            .instance()
            .get(&EMERGENCY_STATE)
            .unwrap_or(EmergencyState {
                is_paused: false,
                paused_at: 0,
                paused_by: env.current_contract_address(),
                reason: Bytes::new(env),
                affected_chains: Vec::new(env),
            })
    }

    /// Get circuit breaker state
    pub fn get_circuit_breaker(env: &Env, chain_id: u32) -> Option<CircuitBreaker> {
        let circuit_breakers: Map<u32, CircuitBreaker> = env
            .storage()
            .instance()
            .get(&CIRCUIT_BREAKERS)
            .unwrap_or_else(|| Map::new(env));
        circuit_breakers.get(chain_id)
    }

    /// Get all paused chains
    pub fn get_paused_chains(env: &Env) -> Vec<u32> {
        let paused_chains: Map<u32, bool> = env
            .storage()
            .instance()
            .get(&PAUSED_CHAINS)
            .unwrap_or_else(|| Map::new(env));
        
        let mut result = Vec::new(env);
        for (chain_id, is_paused) in paused_chains.iter() {
            if is_paused {
                result.push_back(chain_id);
            }
        }
        result
    }

    /// Update circuit breaker limits
    pub fn update_circuit_breaker_limits(
        env: &Env,
        chain_id: u32,
        max_daily_volume: i128,
        max_transaction_amount: i128,
        updater: Address,
    ) -> Result<(), BridgeError> {
        updater.require_auth();

        let mut circuit_breakers: Map<u32, CircuitBreaker> = env
            .storage()
            .instance()
            .get(&CIRCUIT_BREAKERS)
            .unwrap_or_else(|| Map::new(env));
        
        let mut breaker = circuit_breakers
            .get(chain_id)
            .ok_or(BridgeError::DestinationChainNotSupported)?;

        breaker.max_daily_volume = max_daily_volume;
        breaker.max_transaction_amount = max_transaction_amount;

        circuit_breakers.set(chain_id, breaker);
        env.storage().instance().set(&CIRCUIT_BREAKERS, &circuit_breakers);

        Ok(())
    }
}
