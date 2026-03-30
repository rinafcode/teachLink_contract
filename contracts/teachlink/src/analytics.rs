#![no_std]

//! Bridge monitoring and analytics for bridge operations, validator performance, and chain metrics.

use crate::errors::BridgeError;

use crate::types::{BridgeMetrics, ChainMetrics};
use soroban_sdk::{Address, Bytes, Env, Map, Vec};

/// Metrics update interval (1 hour)
pub const METRICS_UPDATE_INTERVAL: u64 = 3_600;

/// Analytics Manager
pub struct AnalyticsManager;

impl AnalyticsManager {
    /// Initialize bridge metrics
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // initialize_metrics(...);
    /// ```
    pub fn initialize_metrics(env: &Env) -> Result<(), BridgeError> {
        let metrics = BridgeMetrics {
            total_volume: 0,
            total_transactions: 0,
            active_validators: 0,
            average_confirmation_time: 0,
            success_rate: 10000, // 100% in basis points
            last_updated: env.ledger().timestamp(),
        };

        env.storage().instance().set(&BRIDGE_METRICS, &metrics);

        Ok(())
    }

    /// Update bridge metrics
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // update_bridge_metrics(...);
    /// ```
    pub fn update_bridge_metrics(
        env: &Env,
        volume: i128,
        transactions: u64,
        confirmation_time: u64,
        success: bool,
    ) -> Result<(), BridgeError> {
        let mut metrics: BridgeMetrics =
            env.storage()
                .instance()
                .get(&BRIDGE_METRICS)
                .unwrap_or(BridgeMetrics {
                    total_volume: 0,
                    total_transactions: 0,
                    active_validators: 0,
                    average_confirmation_time: 0,
                    success_rate: 10000,
                    last_updated: env.ledger().timestamp(),
                });

        // Update metrics
        metrics.total_volume += volume;
        metrics.total_transactions += transactions;

        // Update average confirmation time (exponential moving average)
        if metrics.total_transactions > 0 {
            let alpha = 10; // Smoothing factor (10% weight to new value)
            metrics.average_confirmation_time = ((metrics.average_confirmation_time
                * (100 - alpha) as u64)
                + (confirmation_time * alpha as u64))
                / 100;
        } else {
            metrics.average_confirmation_time = confirmation_time;
        }

        // Update success rate
        if success {
            metrics.success_rate = ((metrics.success_rate * 99) + 10000) / 100;
        } else {
            metrics.success_rate = (metrics.success_rate * 99) / 100;
        }

        metrics.last_updated = env.ledger().timestamp();

        env.storage().instance().set(&BRIDGE_METRICS, &metrics);

        Ok(())
    }

    /// Update validator count
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // update_validator_count(...);
    /// ```
    pub fn update_validator_count(env: &Env, active_validators: u32) -> Result<(), BridgeError> {
        let mut metrics: BridgeMetrics =
            env.storage()
                .instance()
                .get(&BRIDGE_METRICS)
                .unwrap_or(BridgeMetrics {
                    total_volume: 0,
                    total_transactions: 0,
                    active_validators: 0,
                    average_confirmation_time: 0,
                    success_rate: 10000,
                    last_updated: env.ledger().timestamp(),
                });

        metrics.active_validators = active_validators;
        metrics.last_updated = env.ledger().timestamp();

        env.storage().instance().set(&BRIDGE_METRICS, &metrics);

        Ok(())
    }

    /// Initialize chain metrics
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // initialize_chain_metrics(...);
    /// ```
    pub fn initialize_chain_metrics(env: &Env, chain_id: u32) -> Result<(), BridgeError> {
        let metrics = ChainMetrics {
            chain_id,
            volume_in: 0,
            volume_out: 0,
            transaction_count: 0,
            average_fee: 0,
            last_updated: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&crate::storage::DataKey::ChainMetrics(chain_id), &metrics);

        // Initialize volume index for this chain
        let mut volume_index: Map<u32, i128> = env
            .storage()
            .instance()
            .get(&CHAIN_VOLUME_INDEX)
            .unwrap_or_else(|| Map::new(env));
        volume_index.set(chain_id, 0i128);
        env.storage().instance().set(&CHAIN_VOLUME_INDEX, &volume_index);

        Ok(())
    }

    /// Update chain metrics
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // update_chain_metrics(...);
    /// ```
    pub fn update_chain_metrics(
        env: &Env,
        chain_id: u32,
        volume: i128,
        is_incoming: bool,
        fee: i128,
    ) -> Result<(), BridgeError> {
        let mut metrics = env
            .storage()
            .instance()
            .get::<_, ChainMetrics>(&crate::storage::DataKey::ChainMetrics(chain_id))
            .unwrap_or(ChainMetrics {
                chain_id,
                volume_in: 0,
                volume_out: 0,
                transaction_count: 0,
                average_fee: 0,
                last_updated: env.ledger().timestamp(),
            });

        // Update volume
        if is_incoming {
            metrics.volume_in += volume;
        } else {
            metrics.volume_out += volume;
        }

        // Update transaction count
        metrics.transaction_count += 1;

        // Update average fee
        if metrics.transaction_count > 0 {
            metrics.average_fee = ((metrics.average_fee * (metrics.transaction_count - 1) as i128)
                + fee)
                / metrics.transaction_count as i128;
        } else {
            metrics.average_fee = fee;
        }

        metrics.last_updated = env.ledger().timestamp();

        env.storage()
            .instance()
            .set(&crate::storage::DataKey::ChainMetrics(chain_id), &metrics);

        // Update volume index
        let mut volume_index: Map<u32, i128> = env
            .storage()
            .instance()
            .get(&CHAIN_VOLUME_INDEX)
            .unwrap_or_else(|| Map::new(env));
        let current_total = volume_index.get(chain_id).unwrap_or(0i128);
        volume_index.set(chain_id, current_total + volume);
        env.storage().instance().set(&CHAIN_VOLUME_INDEX, &volume_index);

        Ok(())
    }

    /// Record daily volume
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // record_daily_volume(...);
    /// ```
    pub fn record_daily_volume(
        env: &Env,
        day_timestamp: u64,
        volume: i128,
        chain_id: u32,
    ) -> Result<(), BridgeError> {
        let key = crate::storage::DataKey::DailyVolume(day_timestamp, chain_id);
        let current_volume = env.storage().instance().get(&key).unwrap_or(0i128);
        env.storage()
            .instance()
            .set(&key, &(current_volume + volume));

        Ok(())
    }

    /// Get daily volume
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // get_daily_volume(...);
    /// ```
    pub fn get_daily_volume(env: &Env, day_timestamp: u64, chain_id: u32) -> i128 {
        env.storage()
            .instance()
            .get(&crate::storage::DataKey::DailyVolume(
                day_timestamp,
                chain_id,
            ))
            .unwrap_or(0i128)
    }

    /// Get bridge metrics
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // get_bridge_metrics(...);
    /// ```
    pub fn get_bridge_metrics(env: &Env) -> BridgeMetrics {
        env.storage()
            .instance()
            .get(&BRIDGE_METRICS)
            .unwrap_or(BridgeMetrics {
                total_volume: 0,
                total_transactions: 0,
                active_validators: 0,
                average_confirmation_time: 0,
                success_rate: 10000,
                last_updated: env.ledger().timestamp(),
            })
    }

    /// Get chain metrics
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // get_chain_metrics(...);
    /// ```
    pub fn get_chain_metrics(env: &Env, chain_id: u32) -> Option<ChainMetrics> {
        env.storage()
            .instance()
            .get(&crate::storage::DataKey::ChainMetrics(chain_id))
    }

    /// Get all chain metrics
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // get_all_chain_metrics(...);
    /// ```
    pub fn get_all_chain_metrics(env: &Env) -> Vec<ChainMetrics> {
        let chains: Vec<u32> = env
            .storage()
            .instance()
            .get(&crate::storage::SUPPORTED_CHAINS_LIST)
            .unwrap_or_else(|| Vec::new(env));
        let mut result = Vec::new(env);
        for chain_id in chains.iter() {
            if let Some(metrics) = Self::get_chain_metrics(env, chain_id) {
                result.push_back(metrics);
            }
        }
        result
    }

    /// Calculate bridge health score (0-100)
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // calculate_health_score(...);
    /// ```
    pub fn calculate_health_score(env: &Env) -> u32 {
        let metrics = Self::get_bridge_metrics(env);

        // Success rate weight: 40%
        let success_score = metrics.success_rate / 100;

        // Validator participation weight: 30%
        let validator_score = if metrics.active_validators > 0 {
            100u32
        } else {
            0u32
        };

        // Confirmation time weight: 30%
        // Ideal: < 5 minutes (300 seconds)
        let confirmation_score = if metrics.average_confirmation_time < 300 {
            100u32
        } else if metrics.average_confirmation_time < 600 {
            80u32
        } else if metrics.average_confirmation_time < 1800 {
            60u32
        } else if metrics.average_confirmation_time < 3600 {
            40u32
        } else {
            20u32
        };

        // Weighted average
        ((success_score * 40) + (validator_score * 30) + (confirmation_score * 30)) / 100
    }

    /// Max chains to iterate when building top-by-volume (gas bound).
    const MAX_CHAINS_ITER: u32 = 50;

    /// Get top chains by volume with bounded iteration (for performance cache).
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // get_top_chains_by_volume_bounded(...);
    /// ```
    pub fn get_top_chains_by_volume_bounded(env: &Env, limit: u32) -> Vec<(u32, i128)> {

            if count >= Self::MAX_CHAINS_ITER {
                break;
            }
            count += 1;

        }

        // Efficient sort using built-in sorting
        chains.sort_by(|a, b| b.1.cmp(&a.1));

        let mut result = Vec::new(env);
        for i in 0..limit.min(chains.len()) {
            if let Some(chain) = chains.get(i) {
                result.push_back(*chain);
            }
        }
        result
    }

    /// Get top chains by volume (unbounded; use get_top_chains_by_volume_bounded for caching).
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // get_top_chains_by_volume(...);
    /// ```
    pub fn get_top_chains_by_volume(env: &Env, limit: u32) -> Vec<(u32, i128)> {

        // Efficient sort using built-in sorting (O(n log n))
        chains.sort_by(|a, b| b.1.cmp(&a.1));

        // Return top N
        let mut result = Vec::new(env);
        for i in 0..limit.min(chains.len()) {
            if let Some(chain) = chains.get(i) {
                result.push_back(*chain);
            }
        }
        result
    }

    /// Get bridge statistics
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // get_bridge_statistics(...);
    /// ```
    pub fn get_bridge_statistics(env: &Env) -> Map<Bytes, i128> {
        let metrics = Self::get_bridge_metrics(env);
        let mut stats: Map<Bytes, i128> = Map::new(env);

        stats.set(
            Bytes::from_slice(env, b"total_volume"),
            metrics.total_volume,
        );
        stats.set(
            Bytes::from_slice(env, b"total_transactions"),
            metrics.total_transactions as i128,
        );
        stats.set(
            Bytes::from_slice(env, b"active_validators"),
            metrics.active_validators as i128,
        );
        stats.set(
            Bytes::from_slice(env, b"avg_confirmation_time"),
            metrics.average_confirmation_time as i128,
        );
        stats.set(
            Bytes::from_slice(env, b"success_rate"),
            metrics.success_rate as i128,
        );
        stats.set(
            Bytes::from_slice(env, b"health_score"),
            Self::calculate_health_score(env) as i128,
        );

        stats
    }

    /// Reset metrics (admin only)
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // reset_metrics(...);
    /// ```
    pub fn reset_metrics(env: &Env, admin: Address) -> Result<(), BridgeError> {
        admin.require_auth();

        let metrics = BridgeMetrics {
            total_volume: 0,
            total_transactions: 0,
            active_validators: 0,
            average_confirmation_time: 0,
            success_rate: 10000,
            last_updated: env.ledger().timestamp(),
        };

        env.storage().instance().set(&BRIDGE_METRICS, &metrics);

        Ok(())
    }

    /// Check if metrics need update
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // needs_update(...);
    /// ```
    pub fn needs_update(env: &Env) -> bool {
        let metrics = Self::get_bridge_metrics(env);
        let current_time = env.ledger().timestamp();

        current_time - metrics.last_updated > METRICS_UPDATE_INTERVAL
    }
}

impl AnalyticsPort for AnalyticsManager {
    fn bridge_metrics(env: &Env) -> BridgeMetrics {
        Self::get_bridge_metrics(env)
    }

    fn health_score(env: &Env) -> u32 {
        Self::calculate_health_score(env)
    }

    fn top_chains_by_volume(env: &Env, max: u32) -> Vec<(u32, i128)> {
        Self::get_top_chains_by_volume_bounded(env, max)
    }
}
