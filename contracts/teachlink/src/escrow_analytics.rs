use crate::storage::ESCROW_ANALYTICS;
use crate::types::EscrowMetrics;
use soroban_sdk::{Env, Map};

pub struct EscrowAnalyticsManager;

impl EscrowAnalyticsManager {
    pub fn update_creation(env: &Env, amount: i128) {
        let mut metrics = Self::get_metrics(env);
        metrics.total_escrows += 1;
        metrics.total_volume += amount;
        env.storage().instance().set(&ESCROW_ANALYTICS, &metrics);
    }

    pub fn update_dispute(env: &Env) {
        let mut metrics = Self::get_metrics(env);
        metrics.total_disputes += 1;
        env.storage().instance().set(&ESCROW_ANALYTICS, &metrics);
    }

    pub fn update_resolution(env: &Env, resolution_time: u64) {
        let mut metrics = Self::get_metrics(env);
        metrics.total_resolved += 1;

        // Update average resolution time
        if metrics.total_resolved == 1 {
            metrics.average_resolution_time = resolution_time;
        } else {
            metrics.average_resolution_time =
                (metrics.average_resolution_time * (metrics.total_resolved - 1) + resolution_time)
                    / metrics.total_resolved;
        }

        env.storage().instance().set(&ESCROW_ANALYTICS, &metrics);
    }

    pub fn get_metrics(env: &Env) -> EscrowMetrics {
        env.storage()
            .instance()
            .get(&ESCROW_ANALYTICS)
            .unwrap_or(EscrowMetrics {
                total_escrows: 0,
                total_volume: 0,
                total_disputes: 0,
                total_resolved: 0,
                average_resolution_time: 0,
            })
    }
}
