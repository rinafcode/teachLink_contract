use crate::storage::ESCROW_ANALYTICS;
use crate::types::EscrowMetrics;
use crate::safe_stats::{safe_add_i128, safe_inc_u64};
use soroban_sdk::{Env, Map};

pub struct EscrowAnalyticsManager;

impl EscrowAnalyticsManager {
    pub fn update_creation(env: &Env, amount: i128) {
        let mut metrics = Self::get_metrics(env);
        let (new_total, overflowed) = safe_inc_u64(metrics.total_escrows);
        metrics.total_escrows = new_total;
        if overflowed {
            metrics.resets += 1;
        }

        let (new_vol, vol_overflow) = safe_add_i128(metrics.total_volume, amount);
        metrics.total_volume = new_vol;
        if vol_overflow {
            metrics.resets += 1;
        }
        env.storage().instance().set(&ESCROW_ANALYTICS, &metrics);
    }

    pub fn update_dispute(env: &Env) {
        let mut metrics = Self::get_metrics(env);
        let (new_d, overflowed) = safe_inc_u64(metrics.total_disputes);
        metrics.total_disputes = new_d;
        if overflowed {
            metrics.resets += 1;
        }
        env.storage().instance().set(&ESCROW_ANALYTICS, &metrics);
    }

    pub fn update_resolution(env: &Env, resolution_time: u64) {
        let mut metrics = Self::get_metrics(env);
        let (new_resolved, overflowed) = safe_inc_u64(metrics.total_resolved);
        metrics.total_resolved = new_resolved;
        if overflowed {
            metrics.resets += 1;
        }

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
                resets: 0,
            })
    }

    /// Reset escrow metrics (requires admin auth by passing admin address)
    pub fn reset_metrics(env: &Env, admin: soroban_sdk::Address) {
        admin.require_auth();
        let metrics = EscrowMetrics {
            total_escrows: 0,
            total_volume: 0,
            total_disputes: 0,
            total_resolved: 0,
            average_resolution_time: 0,
            resets: 0,
        };
        env.storage().instance().set(&ESCROW_ANALYTICS, &metrics);
    }
}
