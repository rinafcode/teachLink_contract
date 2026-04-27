//! Automatic Scaling & Load Management Module
//!
//! This module implements dynamic scaling mechanisms to handle high-load scenarios
//! in the TeachLink bridge and consensus operations.
//!
//! # Features
//!
//! - **Dynamic Batch Sizing**: Adjusts proposal batch sizes based on current load
//! - **Load Shedding**: Gracefully degrades non-critical operations under extreme load
//! - **Priority Queuing**: Ensures critical operations are processed first
//! - **Resource Allocation**: Optimizes gas and storage usage based on demand
//!
//! # Load Levels
//!
//! ```text
//! LOW:    < 50% capacity - normal operations
//! MEDIUM: 50-75% capacity - optimized batching
//! HIGH:   75-90% capacity - load shedding begins
//! CRITICAL: > 90% capacity - emergency measures
//! ```

use crate::errors::BridgeError;
use crate::storage::{LOAD_LEVEL, LOAD_METRICS, SCALING_CONFIG};
use crate::types::{LoadLevel, ScalingMetrics, ScalingPolicy};
use soroban_sdk::{Address, Env, Map};

/// Default maximum batch size for proposal processing
pub const DEFAULT_MAX_BATCH_SIZE: u32 = 10;

/// Default minimum batch size under load
pub const DEFAULT_MIN_BATCH_SIZE: u32 = 1;

/// Load threshold for entering MEDIUM level (50%)
pub const LOAD_THRESHOLD_MEDIUM: u64 = 50;

/// Load threshold for entering HIGH level (75%)
pub const LOAD_THRESHOLD_HIGH: u64 = 75;

/// Load threshold for entering CRITICAL level (90%)
pub const LOAD_THRESHOLD_CRITICAL: u64 = 90;

/// Gas limit per transaction (Stellar limit)
pub const GAS_LIMIT_PER_TX: u64 = 10_000_000;

/// Auto-scaling manager for handling high-load scenarios
pub struct AutoScaler;

impl AutoScaler {
    /// Initialize scaling configuration with default policy
    pub fn initialize(env: &Env, admin: &Address) -> Result<(), BridgeError> {
        admin.require_auth();

        let policy = ScalingPolicy {
            max_batch_size: DEFAULT_MAX_BATCH_SIZE,
            min_batch_size: DEFAULT_MIN_BATCH_SIZE,
            gas_budget_per_batch: 5_000_000, // 50% of max
            enable_load_shedding: true,
            load_shedding_threshold: LOAD_THRESHOLD_HIGH,
            priority_queue_enabled: true,
        };

        env.storage().instance().set(&SCALING_CONFIG, &policy);

        let metrics = ScalingMetrics {
            current_load: 0,
            processed_operations: 0,
            shed_operations: 0,
            average_batch_size: DEFAULT_MAX_BATCH_SIZE,
            last_scaling_adjustment: env.ledger().timestamp(),
        };

        env.storage().instance().set(&LOAD_METRICS, &metrics);
        env.storage().instance().set(&LOAD_LEVEL, &LoadLevel::Low);

        Ok(())
    }

    /// Get current load level based on recent metrics
    pub fn get_current_load_level(env: &Env) -> LoadLevel {
        env.storage()
            .instance()
            .get(&LOAD_LEVEL)
            .unwrap_or(LoadLevel::Low)
    }

    /// Calculate optimal batch size based on current load
    pub fn calculate_optimal_batch_size(env: &Env) -> u32 {
        let metrics: ScalingMetrics = env
            .storage()
            .instance()
            .get(&LOAD_METRICS)
            .unwrap_or_else(|| Self::default_metrics(env));

        let policy: ScalingPolicy = env
            .storage()
            .instance()
            .get(&SCALING_CONFIG)
            .unwrap_or_else(|| Self::default_policy());

        let load = metrics.current_load;

        // Linear interpolation between min and max batch size based on load
        if load >= LOAD_THRESHOLD_CRITICAL {
            policy.min_batch_size
        } else if load >= LOAD_THRESHOLD_HIGH {
            // Scale down from 50% to min as load goes from HIGH to CRITICAL
            let range = LOAD_THRESHOLD_CRITICAL - LOAD_THRESHOLD_HIGH;
            let position = load - LOAD_THRESHOLD_HIGH;
            let ratio = ((range - position) as u32 * 100 / range as u32) as u32;
            let batch_range = policy.max_batch_size - policy.min_batch_size;
            policy.min_batch_size + (batch_range * ratio / 200) // 50% at HIGH threshold
        } else if load >= LOAD_THRESHOLD_MEDIUM {
            // Scale down from max to 50% as load goes from MEDIUM to HIGH
            let range = LOAD_THRESHOLD_HIGH - LOAD_THRESHOLD_MEDIUM;
            let position = load - LOAD_THRESHOLD_MEDIUM;
            let ratio = (100 - (position as u32 * 100 / range as u32 / 2)) as u32;
            let batch_range = policy.max_batch_size - policy.min_batch_size;
            policy.min_batch_size + (batch_range * ratio / 100)
        } else {
            // Low load - use maximum batch size
            policy.max_batch_size
        }
    }

    /// Determine if an operation should be shed based on priority and load
    pub fn should_shed_operation(env: &Env, priority: u32) -> bool {
        let metrics: ScalingMetrics = env
            .storage()
            .instance()
            .get(&LOAD_METRICS)
            .unwrap_or_else(|| Self::default_metrics(env));

        let policy: ScalingPolicy = env
            .storage()
            .instance()
            .get(&SCALING_CONFIG)
            .unwrap_or_else(|| Self::default_policy());

        if !policy.enable_load_shedding {
            return false;
        }

        let load = metrics.current_load;

        // Only shed if above threshold
        if load < policy.load_shedding_threshold {
            return false;
        }

        // Priority levels: 0-50 (critical), 51-100 (normal), 101-255 (low)
        let shed_threshold = if load >= LOAD_THRESHOLD_CRITICAL {
            150 // Shed everything except most critical
        } else {
            200 // Shed only low priority
        };

        priority > shed_threshold as u32
    }

    /// Update load metrics with new operation data
    pub fn update_load_metrics(
        env: &Env,
        operations_processed: u64,
        operations_shed: u64,
        current_gas_usage: u64,
    ) -> Result<(), BridgeError> {
        let mut metrics: ScalingMetrics = env
            .storage()
            .instance()
            .get(&LOAD_METRICS)
            .unwrap_or_else(|| Self::default_metrics(env));

        // Update counters
        metrics.processed_operations += operations_processed;
        metrics.shed_operations += operations_shed;

        // Calculate current load as percentage of gas budget used
        let policy: ScalingPolicy = env
            .storage()
            .instance()
            .get(&SCALING_CONFIG)
            .unwrap_or_else(|| Self::default_policy());

        metrics.current_load = if policy.gas_budget_per_batch > 0 {
            (current_gas_usage * 100 / policy.gas_budget_per_batch) as u64
        } else {
            0
        };

        // Update average batch size (exponential moving average)
        let current_batch = operations_processed;
        metrics.average_batch_size = (metrics.average_batch_size * 7 + current_batch as u32) / 8;

        metrics.last_scaling_adjustment = env.ledger().timestamp();

        env.storage().instance().set(&LOAD_METRICS, &metrics);

        // Update load level
        Self::update_load_level(env, metrics.current_load)?;

        Ok(())
    }

    /// Update the current load level based on load percentage
    fn update_load_level(env: &Env, load: u64) -> Result<(), BridgeError> {
        let new_level = if load >= LOAD_THRESHOLD_CRITICAL {
            LoadLevel::Critical
        } else if load >= LOAD_THRESHOLD_HIGH {
            LoadLevel::High
        } else if load >= LOAD_THRESHOLD_MEDIUM {
            LoadLevel::Medium
        } else {
            LoadLevel::Low
        };

        env.storage().instance().set(&LOAD_LEVEL, &new_level);

        Ok(())
    }

    /// Get priority-based queuing decision
    pub fn should_queue_operation(env: &Env, priority: u32) -> bool {
        let policy: ScalingPolicy = env
            .storage()
            .instance()
            .get(&SCALING_CONFIG)
            .unwrap_or_else(|| Self::default_policy());

        if !policy.priority_queue_enabled {
            return false;
        }

        let load_level = Self::get_current_load_level(env);

        match load_level {
            LoadLevel::Critical => priority < 100, // Queue non-critical
            LoadLevel::High => priority < 150,     // Queue normal and low
            LoadLevel::Medium => priority < 200,   // Queue low priority only
            LoadLevel::Low => false,               // No queuing needed
        }
    }

    /// Get gas allocation for operation based on current load and priority
    pub fn allocate_gas_budget(env: &Env, priority: u32, base_gas: u64) -> u64 {
        let metrics: ScalingMetrics = env
            .storage()
            .instance()
            .get(&LOAD_METRICS)
            .unwrap_or_else(|| Self::default_metrics(env));

        let load = metrics.current_load;

        // Priority boost: critical operations get more gas allocation
        let priority_multiplier = if priority < 50 {
            120 // 120% for critical
        } else if priority < 100 {
            100 // 100% for high
        } else if priority < 200 {
            80 // 80% for normal
        } else {
            60 // 60% for low
        };

        // Load reduction: reduce allocation under high load
        let load_multiplier = if load >= LOAD_THRESHOLD_CRITICAL {
            50 // 50% under critical load
        } else if load >= LOAD_THRESHOLD_HIGH {
            70 // 70% under high load
        } else if load >= LOAD_THRESHOLD_MEDIUM {
            85 // 85% under medium load
        } else {
            100 // 100% under low load
        };

        let adjusted_gas = base_gas * priority_multiplier * load_multiplier / 10000;

        // Ensure within limits
        adjusted_gas.min(GAS_LIMIT_PER_TX).max(100_000) // Min 100k gas
    }

    /// Emergency scaling - triggered when system is under extreme load
    pub fn emergency_scaling(env: &Env) -> Result<(), BridgeError> {
        let mut policy: ScalingPolicy = env
            .storage()
            .instance()
            .get(&SCALING_CONFIG)
            .unwrap_or_else(|| Self::default_policy());

        // Reduce batch size to minimum
        policy.max_batch_size = policy.min_batch_size.max(1);

        // Enable aggressive load shedding
        policy.enable_load_shedding = true;
        policy.load_shedding_threshold = LOAD_THRESHOLD_MEDIUM;

        env.storage().instance().set(&SCALING_CONFIG, &policy);

        Ok(())
    }

    /// Reset scaling configuration to defaults
    pub fn reset_scaling(env: &Env, admin: &Address) -> Result<(), BridgeError> {
        admin.require_auth();

        let policy = Self::default_policy();
        env.storage().instance().set(&SCALING_CONFIG, &policy);

        let metrics = Self::default_metrics(env);
        env.storage().instance().set(&LOAD_METRICS, &metrics);

        env.storage().instance().set(&LOAD_LEVEL, &LoadLevel::Low);

        Ok(())
    }

    /// Get default scaling policy
    fn default_policy() -> ScalingPolicy {
        ScalingPolicy {
            max_batch_size: DEFAULT_MAX_BATCH_SIZE,
            min_batch_size: DEFAULT_MIN_BATCH_SIZE,
            gas_budget_per_batch: 5_000_000,
            enable_load_shedding: true,
            load_shedding_threshold: LOAD_THRESHOLD_HIGH,
            priority_queue_enabled: true,
        }
    }

    /// Get default metrics
    fn default_metrics(env: &Env) -> ScalingMetrics {
        ScalingMetrics {
            current_load: 0,
            processed_operations: 0,
            shed_operations: 0,
            average_batch_size: DEFAULT_MAX_BATCH_SIZE,
            last_scaling_adjustment: env.ledger().timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Ledger;

    #[test]
    #[ignore] // TODO: Fix test - requires proper contract context setup
    fn test_batch_size_scaling() {
        let env = Env::default();
        let admin = <soroban_sdk::Address as soroban_sdk::testutils::Address>::generate(&env);

        env.as_contract(&admin, || {
            AutoScaler::initialize(&env, &admin).unwrap();

            // Low load - should use max batch size
            let batch_size = AutoScaler::calculate_optimal_batch_size(&env);
            assert_eq!(batch_size, DEFAULT_MAX_BATCH_SIZE);
        });
    }

    #[test]
    #[ignore] // TODO: Fix test - requires proper contract context setup
    fn test_load_shedding_priority() {
        let env = Env::default();
        let admin = <soroban_sdk::Address as soroban_sdk::testutils::Address>::generate(&env);

        env.as_contract(&admin, || {
            AutoScaler::initialize(&env, &admin).unwrap();

            // Critical priority should not be shed
            assert!(!AutoScaler::should_shed_operation(&env, 10));

            // Low priority should be shed under high load
            // (requires setting up high load metrics first)
        });
    }

    #[test]
    #[ignore] // TODO: Fix test - requires proper contract context setup
    fn test_gas_allocation_priority() {
        let env = Env::default();
        let admin = <soroban_sdk::Address as soroban_sdk::testutils::Address>::generate(&env);

        env.as_contract(&admin, || {
            AutoScaler::initialize(&env, &admin).unwrap();

            // Critical priority gets more gas
            let critical_gas = AutoScaler::allocate_gas_budget(&env, 10, 1_000_000);
            let low_gas = AutoScaler::allocate_gas_budget(&env, 250, 1_000_000);

            assert!(critical_gas > low_gas);
        });
    }
}
