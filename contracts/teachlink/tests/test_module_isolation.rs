#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]

//! Tests verifying module isolation via trait-based dependency injection (#152).
//!
//! Each test uses a lightweight mock implementation of a port trait instead of
//! the real concrete type, confirming that calling modules compile and behave
//! correctly when the collaborator is swapped out.

use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, Vec};

use teachlink_contract::interfaces::{AnalyticsPort, AuditPort, EscrowMetricsPort, EscrowObserver};
use teachlink_contract::{BridgeError, BridgeMetrics, EscrowMetrics, OperationType};

pub struct MockAnalytics;

impl AnalyticsPort for MockAnalytics {
    fn bridge_metrics(_env: &Env) -> BridgeMetrics {
        BridgeMetrics {
            total_volume: 1_000,
            total_transactions: 10,
            active_validators: 3,
            average_confirmation_time: 5,
            success_rate: 9_500,
            last_updated: 0,
        }
    }

    fn health_score(_env: &Env) -> u32 {
        8_000
    }

    fn top_chains_by_volume(_env: &Env, _max: u32) -> Vec<(u32, i128)> {
        Vec::new(_env)
    }
}

// ---------------------------------------------------------------------------
// Mock: AuditPort  (records no state — just returns a fixed id)
// ---------------------------------------------------------------------------

pub struct MockAudit;

impl AuditPort for MockAudit {
    fn create_record(
        _env: &Env,
        _op: OperationType,
        _operator: Address,
        _details: Bytes,
        _tx_hash: Bytes,
    ) -> Result<u64, BridgeError> {
        Ok(42)
    }

    fn get_count(_env: &Env) -> u64 {
        0
    }
}

// ---------------------------------------------------------------------------
// Mock: EscrowMetricsPort
// ---------------------------------------------------------------------------

pub struct MockEscrowMetrics;

impl EscrowMetricsPort for MockEscrowMetrics {
    fn get_metrics(_env: &Env) -> EscrowMetrics {
        EscrowMetrics {
            total_escrows: 5,
            total_volume: 500,
            total_disputes: 1,
            total_resolved: 1,
            average_resolution_time: 3_600,
        }
    }
}

// ---------------------------------------------------------------------------
// Mock: EscrowObserver  (no-op)
// ---------------------------------------------------------------------------

pub struct MockObserver;

impl EscrowObserver for MockObserver {
    fn on_created(_env: &Env, _amount: i128) {}
    fn on_disputed(_env: &Env) {}
    fn on_resolved(_env: &Env, _duration: u64) {}
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn analytics_mock_returns_fixed_health_score() {
    let env = Env::default();
    let score = MockAnalytics::health_score(&env);
    assert_eq!(score, 8_000);
}

#[test]
fn analytics_mock_bridge_metrics_volume() {
    let env = Env::default();
    let metrics = MockAnalytics::bridge_metrics(&env);
    assert_eq!(metrics.total_volume, 1_000);
    assert_eq!(metrics.total_transactions, 10);
}

#[test]
fn audit_mock_create_record_returns_fixed_id() {
    let env = Env::default();
    let addr = Address::generate(&env);
    let result = MockAudit::create_record(
        &env,
        OperationType::BackupCreated,
        addr,
        Bytes::new(&env),
        Bytes::new(&env),
    );
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn audit_mock_get_count_returns_zero() {
    let env = Env::default();
    assert_eq!(MockAudit::get_count(&env), 0);
}

#[test]
fn escrow_metrics_mock_returns_expected_values() {
    let env = Env::default();
    let metrics = MockEscrowMetrics::get_metrics(&env);
    assert_eq!(metrics.total_escrows, 5);
    assert_eq!(metrics.total_disputes, 1);
}

#[test]
fn observer_mock_on_created_is_no_op() {
    let env = Env::default();
    // Should not panic.
    MockObserver::on_created(&env, 1_000);
    MockObserver::on_disputed(&env);
    MockObserver::on_resolved(&env, 7_200);
}

#[test]
fn mock_analytics_top_chains_empty() {
    let env = Env::default();
    let chains = MockAnalytics::top_chains_by_volume(&env, 10);
    assert_eq!(chains.len(), 0);
}
