//! Bridge Monitoring and Analytics Module
//!
//! This module implements comprehensive analytics and monitoring
//! for bridge operations, validator performance, and chain metrics.

use crate::errors::BridgeError;
use crate::storage::{BRIDGE_METRICS, CHAIN_METRICS, DAILY_VOLUMES};
use crate::types::{BridgeMetrics, ChainMetrics};

use soroban_sdk::{
    contracttype, Address, Bytes, Env, Map, Vec,
};

/// ==============================
/// Event Definitions
/// ==============================

#[contracttype]
pub struct ContentViewed {
    pub token_id: Bytes,
}

#[contracttype]
pub struct ContentPurchased {
    pub token_id: Bytes,
    pub buyer: Address,
}

/// ==============================
/// Analytics Data Structures
/// ==============================

pub struct ContentStats {
    pub views: u64,
    pub purchases: u64,
    pub revenue: u128,
}

/// Metrics update interval (1 hour)
pub const METRICS_UPDATE_INTERVAL: u64 = 3_600;

/// ==============================
/// Analytics Manager
/// ==============================

pub struct AnalyticsManager;

impl AnalyticsManager {
    // ============================================================
    // Event Emitters
    // ============================================================

    pub fn emit_content_viewed(env: &Env, token_id: Bytes) {
        let event = ContentViewed { token_id };
        env.events().publish(("content", "viewed"), event);
    }

    pub fn emit_content_purchased(env: &Env, token_id: Bytes, buyer: Address) {
        let event = ContentPurchased { token_id, buyer };
        env.events().publish(("content", "purchased"), event);
    }

    // ============================================================
    // Bridge Metrics
    // ============================================================

    pub fn initialize_metrics(env: &Env) -> Result<(), BridgeError> {
        let metrics = BridgeMetrics {
            total_volume: 0,
            total_transactions: 0,
            active_validators: 0,
            average_confirmation_time: 0,
            success_rate: 10000, // 100%
            last_updated: env.ledger().timestamp(),
        };

        env.storage().instance().set(&BRIDGE_METRICS, &metrics);
        Ok(())
    }

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

        metrics.total_volume += volume;
        metrics.total_transactions += transactions;

        // Exponential moving average
        if metrics.total_transactions > 0 {
            let alpha: u64 = 10;
            metrics.average_confirmation_time =
                ((metrics.average_confirmation_time * (100 - alpha))
                    + (confirmation_time * alpha))
                    / 100;
        } else {
            metrics.average_confirmation_time = confirmation_time;
        }

        // Success rate smoothing
        if success {
            metrics.success_rate = ((metrics.success_rate * 99) + 10000) / 100;
        } else {
            metrics.success_rate = (metrics.success_rate * 99) / 100;
        }

        metrics.last_updated = env.ledger().timestamp();
        env.storage().instance().set(&BRIDGE_METRICS, &metrics);

        Ok(())
    }

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

    // ============================================================
    // Chain Metrics
    // ============================================================

    pub fn initialize_chain_metrics(env: &Env, chain_id: u32) -> Result<(), BridgeError> {
        let metrics = ChainMetrics {
            chain_id,
            volume_in: 0,
            volume_out: 0,
            transaction_count: 0,
            average_fee: 0,
            last_updated: env.ledger().timestamp(),
        };

        let mut chain_metrics: Map<u32, ChainMetrics> =
            env.storage().instance().get(&CHAIN_METRICS).unwrap_or_else(|| Map::new(env));

        chain_metrics.set(chain_id, metrics);
        env.storage().instance().set(&CHAIN_METRICS, &chain_metrics);

        Ok(())
    }

    pub fn update_chain_metrics(
        env: &Env,
        chain_id: u32,
        volume: i128,
        is_incoming: bool,
        fee: i128,
    ) -> Result<(), BridgeError> {
        let mut chain_metrics: Map<u32, ChainMetrics> =
            env.storage().instance().get(&CHAIN_METRICS).unwrap_or_else(|| Map::new(env));

        let mut metrics = chain_metrics.get(chain_id).unwrap_or(ChainMetrics {
            chain_id,
            volume_in: 0,
            volume_out: 0,
            transaction_count: 0,
            average_fee: 0,
            last_updated: env.ledger().timestamp(),
        });

        if is_incoming {
            metrics.volume_in += volume;
        } else {
            metrics.volume_out += volume;
        }

        metrics.transaction_count += 1;

        metrics.average_fee =
            ((metrics.average_fee * (metrics.transaction_count - 1) as i128) + fee)
                / metrics.transaction_count as i128;

        metrics.last_updated = env.ledger().timestamp();

        chain_metrics.set(chain_id, metrics);
        env.storage().instance().set(&CHAIN_METRICS, &chain_metrics);

        Ok(())
    }

    // ============================================================
    // Daily Volume
    // ============================================================

    pub fn record_daily_volume(
        env: &Env,
        day_timestamp: u64,
        volume: i128,
        chain_id: u32,
    ) -> Result<(), BridgeError> {
        let mut daily_volumes: Map<(u64, u32), i128> =
            env.storage().instance().get(&DAILY_VOLUMES).unwrap_or_else(|| Map::new(env));

        let key = (day_timestamp, chain_id);
        let current_volume = daily_volumes.get(key.clone()).unwrap_or(0);
        daily_volumes.set(key, current_volume + volume);

        env.storage().instance().set(&DAILY_VOLUMES, &daily_volumes);
        Ok(())
    }

    pub fn get_daily_volume(env: &Env, day_timestamp: u64, chain_id: u32) -> i128 {
        let daily_volumes: Map<(u64, u32), i128> =
            env.storage().instance().get(&DAILY_VOLUMES).unwrap_or_else(|| Map::new(env));

        daily_volumes.get((day_timestamp, chain_id)).unwrap_or(0)
    }

    // ============================================================
    // Read APIs
    // ============================================================

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

    pub fn get_chain_metrics(env: &Env, chain_id: u32) -> Option<ChainMetrics> {
        let chain_metrics: Map<u32, ChainMetrics> =
            env.storage().instance().get(&CHAIN_METRICS).unwrap_or_else(|| Map::new(env));
        chain_metrics.get(chain_id)
    }

    pub fn get_all_chain_metrics(env: &Env) -> Vec<ChainMetrics> {
        let chain_metrics: Map<u32, ChainMetrics> =
            env.storage().instance().get(&CHAIN_METRICS).unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        for (_, metrics) in chain_metrics.iter() {
            result.push_back(metrics);
        }
        result
    }

    // ============================================================
    // Health Score
    // ============================================================

    pub fn calculate_health_score(env: &Env) -> u32 {
        let metrics = Self::get_bridge_metrics(env);

        let success_score = metrics.success_rate / 100;

        let validator_score = if metrics.active_validators > 0 { 100 } else { 0 };

        let confirmation_score = if metrics.average_confirmation_time < 300 {
            100
        } else if metrics.average_confirmation_time < 600 {
            80
        } else if metrics.average_confirmation_time < 1800 {
            60
        } else if metrics.average_confirmation_time < 3600 {
            40
        } else {
            20
        };

        ((success_score * 40) + (validator_score * 30) + (confirmation_score * 30)) / 100
    }

    // ============================================================
    // Admin
    // ============================================================

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

    pub fn needs_update(env: &Env) -> bool {
        let metrics = Self::get_bridge_metrics(env);
        env.ledger().timestamp() - metrics.last_updated > METRICS_UPDATE_INTERVAL
    }
}