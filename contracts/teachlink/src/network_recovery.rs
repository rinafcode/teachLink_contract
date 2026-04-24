//! Network Failure Recovery and Retry Mechanism
//!
//! This module provides robust retry logic with exponential backoff,
//! state preservation, user notifications, and fallback mechanisms.

use crate::errors::BridgeError;
use soroban_sdk::{contracttype, Address, Bytes, Env, Map, Symbol, Vec};

/// Storage keys for recovery mechanism
pub const RECOVERY_CONFIG: Symbol = soroban_sdk::symbol_short!("rec_cfg");
pub const RECOVERY_STATE: Symbol = soroban_sdk::symbol_short!("rec_state");
pub const RECOVERY_NOTIFICATIONS: Symbol = soroban_sdk::symbol_short!("rec_notif");
pub const FALLBACK_ACTIVE: Symbol = soroban_sdk::symbol_short!("fb_active");

/// Retry configuration constants
pub const MAX_RETRY_ATTEMPTS: u32 = 5;
pub const INITIAL_BACKOFF_SECONDS: u64 = 60; // 1 minute
pub const MAX_BACKOFF_SECONDS: u64 = 3600; // 1 hour
pub const BACKOFF_MULTIPLIER: u64 = 2;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryConfig {
    pub max_retries: u32,
    pub initial_backoff: u64,
    pub max_backoff: u64,
    pub backoff_multiplier: u64,
    pub circuit_breaker_threshold: u32,
    pub fallback_enabled: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationState {
    pub operation_id: u64,
    pub operation_type: Bytes,
    pub user: Address,
    pub retry_count: u32,
    pub last_attempt: u64,
    pub next_retry: u64,
    pub status: OperationStatus,
    pub error_message: Bytes,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationStatus {
    Pending,
    InProgress,
    Failed,
    Retrying,
    Completed,
    FallbackActive,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FallbackMechanism {
    pub operation_id: u64,
    pub fallback_type: Bytes,
    pub activated_at: u64,
    pub original_params: Bytes,
    pub fallback_params: Bytes,
}

pub struct NetworkRecovery;

impl NetworkRecovery {
    /// Initialize recovery system with default configuration
    pub fn initialize(env: &Env) -> Result<(), BridgeError> {
        if env.storage().instance().has(&RECOVERY_CONFIG) {
            return Err(BridgeError::AlreadyInitialized);
        }

        let config = RecoveryConfig {
            max_retries: MAX_RETRY_ATTEMPTS,
            initial_backoff: INITIAL_BACKOFF_SECONDS,
            max_backoff: MAX_BACKOFF_SECONDS,
            backoff_multiplier: BACKOFF_MULTIPLIER,
            circuit_breaker_threshold: 10,
            fallback_enabled: true,
        };

        env.storage().instance().set(&RECOVERY_CONFIG, &config);

        let operations: Map<u64, OperationState> = Map::new(env);
        env.storage().instance().set(&RECOVERY_STATE, &operations);

        let notifications: Map<Address, Vec<u64>> = Map::new(env);
        env.storage()
            .instance()
            .set(&RECOVERY_NOTIFICATIONS, &notifications);

        env.storage().instance().set(&FALLBACK_ACTIVE, &false);

        Ok(())
    }

    /// Calculate exponential backoff delay
    pub fn calculate_backoff(env: &Env, retry_count: u32) -> u64 {
        let config: RecoveryConfig = env.storage().instance().get(&RECOVERY_CONFIG).unwrap();

        let mut delay = config.initial_backoff;
        for _ in 0..retry_count {
            delay *= config.backoff_multiplier;
            if delay > config.max_backoff {
                delay = config.max_backoff;
                break;
            }
        }

        delay
    }

    /// Register a failed operation for retry
    pub fn register_failed_operation(
        env: &Env,
        operation_id: u64,
        operation_type: Bytes,
        user: Address,
        error_message: Bytes,
    ) -> Result<(), BridgeError> {
        let mut operations: Map<u64, OperationState> = env
            .storage()
            .instance()
            .get(&RECOVERY_STATE)
            .unwrap_or_else(|| Map::new(env));

        let existing = operations.get(operation_id);
        let retry_count = if let Some(op) = existing {
            op.retry_count + 1
        } else {
            0
        };

        let config: RecoveryConfig = env.storage().instance().get(&RECOVERY_CONFIG).unwrap();

        // Check if max retries exceeded
        if retry_count >= config.max_retries {
            // Activate fallback if enabled
            if config.fallback_enabled {
                Self::activate_fallback(env, operation_id, operation_type.clone())?;
            }

            let state = OperationState {
                operation_id,
                operation_type,
                user: user.clone(),
                retry_count,
                last_attempt: env.ledger().timestamp(),
                next_retry: 0, // No more retries
                status: OperationStatus::Failed,
                error_message,
            };

            operations.set(operation_id, state);
            env.storage().instance().set(&RECOVERY_STATE, &operations);

            return Err(BridgeError::RetryLimitExceeded);
        }

        let backoff = Self::calculate_backoff(env, retry_count);
        let next_retry = env.ledger().timestamp() + backoff;

        let state = OperationState {
            operation_id,
            operation_type: operation_type.clone(),
            user: user.clone(),
            retry_count,
            last_attempt: env.ledger().timestamp(),
            next_retry,
            status: OperationStatus::Retrying,
            error_message,
        };

        operations.set(operation_id, state);
        env.storage().instance().set(&RECOVERY_STATE, &operations);

        // Notify user about retry
        Self::notify_user_retry(env, operation_id, user, retry_count, next_retry)?;

        Ok(())
    }

    /// Check if operation can be retried
    pub fn can_retry(env: &Env, operation_id: u64) -> Result<bool, BridgeError> {
        let operations: Map<u64, OperationState> = env
            .storage()
            .instance()
            .get(&RECOVERY_STATE)
            .unwrap_or_else(|| Map::new(env));

        if let Some(state) = operations.get(operation_id) {
            let config: RecoveryConfig = env.storage().instance().get(&RECOVERY_CONFIG).unwrap();

            if state.retry_count >= config.max_retries {
                return Ok(false);
            }

            let current_time = env.ledger().timestamp();
            if current_time < state.next_retry {
                return Err(BridgeError::RetryBackoffActive);
            }

            return Ok(true);
        }

        Ok(true) // New operation can always retry
    }

    /// Mark operation as completed
    pub fn mark_completed(env: &Env, operation_id: u64) -> Result<(), BridgeError> {
        let mut operations: Map<u64, OperationState> = env
            .storage()
            .instance()
            .get(&RECOVERY_STATE)
            .unwrap_or_else(|| Map::new(env));

        if let Some(mut state) = operations.get(operation_id) {
            state.status = OperationStatus::Completed;
            state.retry_count = 0;
            operations.set(operation_id, state);
            env.storage().instance().set(&RECOVERY_STATE, &operations);
        }

        Ok(())
    }

    /// Activate fallback mechanism
    pub fn activate_fallback(
        env: &Env,
        operation_id: u64,
        operation_type: Bytes,
    ) -> Result<(), BridgeError> {
        let fallback = FallbackMechanism {
            operation_id,
            fallback_type: operation_type.clone(),
            activated_at: env.ledger().timestamp(),
            original_params: Bytes::new(env),
            fallback_params: Bytes::new(env),
        };

        // Store fallback state
        let mut fallbacks: Map<u64, FallbackMechanism> = env
            .storage()
            .instance()
            .get(&RECOVERY_STATE)
            .unwrap_or_else(|| Map::new(env));

        fallbacks.set(operation_id, fallback);
        env.storage().instance().set(&RECOVERY_STATE, &fallbacks);

        // Mark fallback as active
        env.storage().instance().set(&FALLBACK_ACTIVE, &true);

        Ok(())
    }

    /// Notify user about retry attempt
    pub fn notify_user_retry(
        env: &Env,
        operation_id: u64,
        user: Address,
        retry_count: u32,
        next_retry: u64,
    ) -> Result<(), BridgeError> {
        let mut notifications: Map<Address, Vec<u64>> = env
            .storage()
            .instance()
            .get(&RECOVERY_NOTIFICATIONS)
            .unwrap_or_else(|| Map::new(env));

        let mut user_notifications = notifications.get(user.clone()).unwrap_or(Vec::new(env));
        user_notifications.push_back(operation_id);
        notifications.set(user.clone(), user_notifications);
        env.storage()
            .instance()
            .set(&RECOVERY_NOTIFICATIONS, &notifications);

        Ok(())
    }

    /// Get operation state
    pub fn get_operation_state(env: &Env, operation_id: u64) -> Option<OperationState> {
        let operations: Map<u64, OperationState> = env
            .storage()
            .instance()
            .get(&RECOVERY_STATE)
            .unwrap_or_else(|| Map::new(env));

        operations.get(operation_id)
    }

    /// Get user notifications
    pub fn get_user_notifications(env: &Env, user: Address) -> Vec<u64> {
        let notifications: Map<Address, Vec<u64>> = env
            .storage()
            .instance()
            .get(&RECOVERY_NOTIFICATIONS)
            .unwrap_or_else(|| Map::new(env));

        notifications.get(user).unwrap_or(Vec::new(env))
    }

    /// Update recovery configuration
    pub fn update_config(
        env: &Env,
        admin: Address,
        config: RecoveryConfig,
    ) -> Result<(), BridgeError> {
        admin.require_auth();

        if config.max_retries == 0 || config.initial_backoff == 0 {
            return Err(BridgeError::InvalidInput);
        }

        env.storage().instance().set(&RECOVERY_CONFIG, &config);
        Ok(())
    }

    /// Check if fallback is active
    pub fn is_fallback_active(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&FALLBACK_ACTIVE)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::NetworkRecovery;
    use crate::TeachLinkBridge;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env};

    #[test]
    fn test_retry_with_backoff() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());

        env.as_contract(&contract_id, || {
            NetworkRecovery::initialize(&env).unwrap();

            let user = Address::generate(&env);
            let operation_id = 1u64;
            let operation_type = Bytes::from_slice(&env, b"bridge_transfer");
            let error_msg = Bytes::from_slice(&env, b"network_timeout");

            // First failure
            NetworkRecovery::register_failed_operation(
                &env,
                operation_id,
                operation_type.clone(),
                user.clone(),
                error_msg.clone(),
            )
            .unwrap();

            let state = NetworkRecovery::get_operation_state(&env, operation_id);
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(state.retry_count, 0);
            assert_eq!(state.status, super::OperationStatus::Retrying);

            // Should have a next_retry time in the future
            assert!(state.next_retry > env.ledger().timestamp());
        });
    }

    #[test]
    fn test_max_retries_exceeded() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());

        env.as_contract(&contract_id, || {
            NetworkRecovery::initialize(&env).unwrap();

            let user = Address::generate(&env);
            let operation_id = 1u64;
            let operation_type = Bytes::from_slice(&env, b"bridge_transfer");
            let error_msg = Bytes::from_slice(&env, b"network_timeout");

            // Simulate multiple failures
            for _ in 0..6 {
                let _ = NetworkRecovery::register_failed_operation(
                    &env,
                    operation_id,
                    operation_type.clone(),
                    user.clone(),
                    error_msg.clone(),
                );
            }

            let state = NetworkRecovery::get_operation_state(&env, operation_id);
            assert!(state.is_some());
            let state = state.unwrap();
            // After max retries, should be in Failed status
            assert_eq!(state.status, super::OperationStatus::Failed);
        });
    }

    #[test]
    fn test_backoff_calculation() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());

        env.as_contract(&contract_id, || {
            NetworkRecovery::initialize(&env).unwrap();

            // Test exponential backoff
            let backoff_0 = NetworkRecovery::calculate_backoff(&env, 0);
            let backoff_1 = NetworkRecovery::calculate_backoff(&env, 1);
            let backoff_2 = NetworkRecovery::calculate_backoff(&env, 2);

            assert_eq!(backoff_0, super::INITIAL_BACKOFF_SECONDS);
            assert_eq!(backoff_1, super::INITIAL_BACKOFF_SECONDS * 2);
            assert_eq!(backoff_2, super::INITIAL_BACKOFF_SECONDS * 4);
        });
    }
}
