#![cfg(test)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unreadable_literal)]

//! Tests for advanced analytics and reporting dashboard.
//!
//! When the contract impl is enabled (uncommented in lib.rs), extend these tests to:
//! - get_dashboard_analytics: assert bridge_health_score, escrow totals, etc.
//! - create_report_template: assert template_id returned and get_report_template matches
//! - schedule_report: assert schedule_id and get_scheduled_reports contains it
//! - generate_report_snapshot: assert report_id and get_report_snapshot
//! - record_report_view / get_report_usage_count
//! - add_report_comment / get_report_comments
//! - create_alert_rule / get_alert_rules / evaluate_alerts
//! - get_recent_report_snapshots

use soroban_sdk::Env;

use teachlink_contract::{ReportType, TeachLinkBridge};

#[test]
fn test_contract_with_reporting_module_registers() {
    let env = Env::default();
    env.mock_all_auths();

    let _contract_id = env.register(TeachLinkBridge, ());
    assert!(true);
}

#[test]
fn test_report_type_variants() {
    // Ensure ReportType is usable for template creation
    let _ = ReportType::BridgeHealth;
    let _ = ReportType::EscrowSummary;
    let _ = ReportType::ComplianceAudit;
    let _ = ReportType::RewardsSummary;
    let _ = ReportType::TokenizationSummary;
    let _ = ReportType::Custom;
    assert!(true);
}

#[test]
fn test_dashboard_analytics_type_available() {
    use teachlink_contract::DashboardAnalytics;
    let env = Env::default();
    let _analytics = DashboardAnalytics {
        bridge_health_score: 100,
        bridge_total_volume: 0,
        bridge_total_transactions: 0,
        bridge_success_rate: 10000,
        escrow_total_count: 0,
        escrow_total_volume: 0,
        escrow_dispute_count: 0,
        escrow_avg_resolution_time: 0,
        compliance_report_count: 0,
        audit_record_count: 0,
        generated_at: env.ledger().timestamp(),
    };
    assert_eq!(_analytics.bridge_health_score, 100);
}
