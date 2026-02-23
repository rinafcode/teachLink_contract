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
        let ts: u64 = env.storage().instance().get(&PERF_TS)?;
        let now = env.ledger().timestamp();
        if now.saturating_sub(ts) > CACHE_TTL_SECS {
            return None;
        }
        env.storage().instance().get(&PERF_CACHE)
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
}
