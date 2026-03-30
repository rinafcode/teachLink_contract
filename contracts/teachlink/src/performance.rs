//! Performance optimization and caching.
//!
//! Provides cached bridge summary (health score, top chains by volume) with
//! TTL-based freshness and admin-triggered invalidation to reduce gas for
//! repeated read-heavy calls.

use crate::analytics;
use crate::errors::BridgeError;
use crate::events::PerfCacheInvalidatedEvent;
use crate::events::PerfMetricsComputedEvent;
use crate::storage::{PERF_CACHE, PERF_TS};
use crate::types::CachedBridgeSummary;
use soroban_sdk::{Address, Env};

/// Cache TTL in ledger seconds (1 hour).
pub const CACHE_TTL_SECS: u64 = 3_600;

/// Max chains to include in cached top-by-volume (bounds gas).
pub const MAX_TOP_CHAINS: u32 = 20;

/// Performance cache manager.
pub struct PerformanceManager;

impl PerformanceManager {
    /// Returns cached bridge summary if present and fresh (within CACHE_TTL_SECS).
    pub fn get_cached_summary(env: &Env) -> Option<CachedBridgeSummary> {
        // Read timestamp; if missing, count as miss
        let now = env.ledger().timestamp();
        match env.storage().instance().get::<_, u64>(&PERF_TS) {
            None => {
                Self::inc_cache_miss(env);
                return None;
            }
            Some(ts) => {
                if now.saturating_sub(ts) > CACHE_TTL_SECS {
                    // stale
                    Self::inc_cache_miss(env);
                    return None;
                }
            }
        }

        // Try to read cached value
        if let Some(summary) = env.storage().instance().get(&PERF_CACHE) {
            Self::inc_cache_hit(env);
            Some(summary)
        } else {
            Self::inc_cache_miss(env);
            None
        }
    }

    /// Computes bridge summary (health score + top chains), writes cache, emits event.
    pub fn compute_and_cache_summary(env: &Env) -> Result<CachedBridgeSummary, BridgeError> {
        let health_score = analytics::AnalyticsManager::calculate_health_score(env);
        let top_chains =
            analytics::AnalyticsManager::get_top_chains_by_volume_bounded(env, MAX_TOP_CHAINS);
        let computed_at = env.ledger().timestamp();
        let summary = CachedBridgeSummary {
            health_score,
            top_chains,
            computed_at,
        };
        env.storage().instance().set(&PERF_CACHE, &summary);
        env.storage().instance().set(&PERF_TS, &computed_at);
        PerfMetricsComputedEvent {
            health_score,
            computed_at,
        }
        .publish(env);
        Ok(summary)
    }

    /// Returns cached summary if fresh; otherwise computes, caches, and returns.
    pub fn get_or_compute_summary(env: &Env) -> Result<CachedBridgeSummary, BridgeError> {
        if let Some(cached) = Self::get_cached_summary(env) {
            return Ok(cached);
        }
        Self::compute_and_cache_summary(env)
    }

    /// Invalidates performance cache (admin only). Emits PerfCacheInvalidatedEvent.
    pub fn invalidate_cache(env: &Env, admin: &Address) -> Result<(), BridgeError> {
        admin.require_auth();
        env.storage().instance().remove(&PERF_CACHE);
        env.storage().instance().remove(&PERF_TS);
        PerfCacheInvalidatedEvent {
            invalidated_at: env.ledger().timestamp(),
        }
        .publish(env);
        Ok(())
    }

    /// Increment cache hit counter
    fn inc_cache_hit(env: &Env) {
        use crate::storage::PERF_HITS;
        let current: u64 = env.storage().instance().get(&PERF_HITS).unwrap_or(0u64);
        env.storage().instance().set(&PERF_HITS, &(current.saturating_add(1u64)));
    }

    /// Increment cache miss counter
    fn inc_cache_miss(env: &Env) {
        use crate::storage::PERF_MISS;
        let current: u64 = env.storage().instance().get(&PERF_MISS).unwrap_or(0u64);
        env.storage().instance().set(&PERF_MISS, &(current.saturating_add(1u64)));
    }

    /// Get cache statistics (hits, misses, hit_rate_percent)
    pub fn get_cache_stats(env: &Env) -> soroban_sdk::Map<soroban_sdk::Bytes, i128> {
        use crate::storage::{PERF_HITS, PERF_MISS};
        use soroban_sdk::{Bytes, Map};

        let hits: u64 = env.storage().instance().get(&PERF_HITS).unwrap_or(0u64);
        let misses: u64 = env.storage().instance().get(&PERF_MISS).unwrap_or(0u64);
        let total = hits.saturating_add(misses);
        let hit_rate = if total == 0 {
            0i128
        } else {
            // percent as integer 0..100
            ((hits as u128 * 100u128) / (total as u128)) as i128
        };

        let mut m: Map<Bytes, i128> = Map::new(env);
        m.set(Bytes::from_slice(env, b"hits"), hits as i128);
        m.set(Bytes::from_slice(env, b"misses"), misses as i128);
        m.set(Bytes::from_slice(env, b"hit_rate"), hit_rate);
        m
    }

    /// Reset cache stats (admin only)
    pub fn reset_cache_stats(env: &Env, admin: &Address) -> Result<(), BridgeError> {
        admin.require_auth();
        use crate::storage::{PERF_HITS, PERF_MISS};
        env.storage().instance().set(&PERF_HITS, &0u64);
        env.storage().instance().set(&PERF_MISS, &0u64);
        Ok(())
    }
}
