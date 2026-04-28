#![no_std]

//! Sustainability metrics for the TeachLink contract.
//!
//! Tracks resource usage, efficiency, and platform health KPIs.

use crate::events::SustainabilityMetricsUpdatedEvent;
use crate::storage::SUSTAINABILITY_METRICS;
use crate::types::SustainabilityMetrics;
use soroban_sdk::Env;

pub struct SustainabilityManager;

impl SustainabilityManager {
    fn load(env: &Env) -> SustainabilityMetrics {
        env.storage()
            .instance()
            .get(&SUSTAINABILITY_METRICS)
            .unwrap_or(SustainabilityMetrics {
                total_invocations: 0,
                total_storage_writes: 0,
                total_events_emitted: 0,
                total_rewards_distributed: 0,
                total_content_minted: 0,
                total_active_users: 0,
                efficiency_score: 10000, // 100% initially
                last_updated: env.ledger().timestamp(),
            })
    }

    fn save(env: &Env, metrics: &SustainabilityMetrics) {
        env.storage()
            .instance()
            .set(&SUSTAINABILITY_METRICS, metrics);
    }

    fn publish(env: &Env, metrics: &SustainabilityMetrics) {
        SustainabilityMetricsUpdatedEvent {
            total_invocations: metrics.total_invocations,
            total_storage_writes: metrics.total_storage_writes,
            total_events_emitted: metrics.total_events_emitted,
            efficiency_score: metrics.efficiency_score,
            updated_at: metrics.last_updated,
        }
        .publish(env);
    }

    /// Record a contract invocation and optional storage write.
    pub fn record_invocation(env: &Env, storage_write: bool) {
        let mut m = Self::load(env);
        m.total_invocations += 1;
        if storage_write {
            m.total_storage_writes += 1;
        }
        m.last_updated = env.ledger().timestamp();
        Self::save(env, &m);
        Self::publish(env, &m);
    }

    /// Record an emitted event.
    pub fn record_event(env: &Env) {
        let mut m = Self::load(env);
        m.total_events_emitted += 1;
        m.last_updated = env.ledger().timestamp();
        Self::save(env, &m);
    }

    /// Record rewards distributed.
    pub fn record_rewards(env: &Env, amount: i128) {
        let mut m = Self::load(env);
        m.total_rewards_distributed += amount;
        m.last_updated = env.ledger().timestamp();
        Self::save(env, &m);
    }

    /// Record a content token minted.
    pub fn record_content_minted(env: &Env) {
        let mut m = Self::load(env);
        m.total_content_minted += 1;
        m.last_updated = env.ledger().timestamp();
        Self::save(env, &m);
    }

    /// Record a new active user.
    pub fn record_active_user(env: &Env) {
        let mut m = Self::load(env);
        m.total_active_users += 1;
        m.last_updated = env.ledger().timestamp();
        Self::save(env, &m);
    }

    /// Update the efficiency score (basis points, 0-10000).
    /// `successful_ops` and `total_ops` are the caller-supplied window counts.
    pub fn update_efficiency(env: &Env, successful_ops: u64, total_ops: u64) {
        let mut m = Self::load(env);
        m.efficiency_score = if total_ops == 0 {
            10000
        } else {
            ((successful_ops * 10000) / total_ops) as u32
        };
        m.last_updated = env.ledger().timestamp();
        Self::save(env, &m);
        Self::publish(env, &m);
    }

    /// Return the current sustainability metrics snapshot.
    pub fn get_metrics(env: &Env) -> SustainabilityMetrics {
        Self::load(env)
    }

    /// Compute a composite sustainability health score (0-100).
    ///
    /// Weights:
    /// - Efficiency score: 50%
    /// - Content creation activity: 25% (capped at 1000 tokens = full score)
    /// - User adoption: 25% (capped at 1000 users = full score)
    pub fn health_score(env: &Env) -> u32 {
        let m = Self::load(env);

        let efficiency_component = m.efficiency_score / 100; // 0-100

        let content_component = if m.total_content_minted >= 1000 {
            100u32
        } else {
            (m.total_content_minted as u32 * 100) / 1000
        };

        let user_component = if m.total_active_users >= 1000 {
            100u32
        } else {
            (m.total_active_users as u32 * 100) / 1000
        };

        (efficiency_component * 50 + content_component * 25 + user_component * 25) / 100
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_record_invocation() {
        let env = Env::default();
        SustainabilityManager::record_invocation(&env, true);
        let m = SustainabilityManager::get_metrics(&env);
        assert_eq!(m.total_invocations, 1);
        assert_eq!(m.total_storage_writes, 1);
    }

    #[test]
    fn test_update_efficiency() {
        let env = Env::default();
        SustainabilityManager::update_efficiency(&env, 90, 100);
        let m = SustainabilityManager::get_metrics(&env);
        assert_eq!(m.efficiency_score, 9000);
    }

    #[test]
    fn test_health_score_full() {
        let env = Env::default();
        // Set efficiency to 100%, content and users to max
        SustainabilityManager::update_efficiency(&env, 1, 1);
        let mut m = SustainabilityManager::get_metrics(&env);
        m.total_content_minted = 1000;
        m.total_active_users = 1000;
        env.storage().instance().set(&SUSTAINABILITY_METRICS, &m);
        assert_eq!(SustainabilityManager::health_score(&env), 100);
    }

    #[test]
    fn test_health_score_zero_ops() {
        let env = Env::default();
        // No ops yet — efficiency defaults to 10000 (100%)
        let score = SustainabilityManager::health_score(&env);
        // efficiency=100, content=0, users=0 → (100*50 + 0 + 0)/100 = 50
        assert_eq!(score, 50);
    }
}
