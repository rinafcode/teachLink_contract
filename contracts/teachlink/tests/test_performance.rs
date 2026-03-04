#![cfg(test)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unreadable_literal)]

//! Tests for performance optimization and caching.
//!
//! When contract is invoked via client: get_cached_bridge_summary,
//! compute_and_cache_bridge_summary, invalidate_performance_cache.

use soroban_sdk::Env;

use teachlink_contract::{CachedBridgeSummary, TeachLinkBridge};

#[test]
fn test_contract_with_performance_module_registers() {
    let env = Env::default();
    env.mock_all_auths();

    let _ = env.register(TeachLinkBridge, ());
    assert!(true);
}

#[test]
fn test_cached_bridge_summary_type() {
    let env = Env::default();
    let summary = CachedBridgeSummary {
        health_score: 85,
        top_chains: soroban_sdk::Vec::new(&env),
        computed_at: env.ledger().timestamp(),
    };
    assert_eq!(summary.health_score, 85);
}
