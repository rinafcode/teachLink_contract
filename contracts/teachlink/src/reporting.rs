//! Advanced Analytics and Reporting Module
//!
//! Provides report templates, scheduled reports, snapshots, usage tracking,
//! collaboration comments, alert rules, and dashboard-ready aggregate analytics
//! for visualization.

use crate::analytics::AnalyticsManager;
use crate::audit::AuditManager;
use crate::errors::BridgeError;
use crate::escrow_analytics::EscrowAnalyticsManager;
use crate::events::{
    AlertTriggeredEvent, ReportCommentAddedEvent, ReportGeneratedEvent, ReportScheduledEvent,
};
use crate::storage::{
    ALERT_RULES, ALERT_RULE_COUNTER, REPORT_COMMENTS, REPORT_COMMENT_COUNTER, REPORT_SCHEDULES,
    REPORT_SCHEDULE_COUNTER, REPORT_SNAPSHOTS, REPORT_SNAPSHOT_COUNTER, REPORT_TEMPLATES,
    REPORT_TEMPLATE_COUNTER, REPORT_USAGE,
};
use crate::types::{
    AlertConditionType, AlertRule, DashboardAnalytics, ReportComment, ReportSchedule,
    ReportSnapshot, ReportTemplate, ReportType, ReportUsage,
};
use soroban_sdk::{Address, Bytes, Env, Map, Vec};

/// Reporting Manager for dashboard, templates, scheduling, and alerts
pub struct ReportingManager;

impl ReportingManager {
    /// Create a report template
    pub fn create_report_template(
        env: &Env,
        creator: Address,
        name: Bytes,
        report_type: ReportType,
        config: Bytes,
    ) -> Result<u64, BridgeError> {
        creator.require_auth();

        let mut counter: u64 = env
            .storage()
            .instance()
            .get(&REPORT_TEMPLATE_COUNTER)
            .unwrap_or(0u64);
        counter += 1;

        let template = ReportTemplate {
            template_id: counter,
            name,
            report_type: report_type.clone(),
            created_by: creator,
            created_at: env.ledger().timestamp(),
            config,
        };

        let mut templates: Map<u64, ReportTemplate> = env
            .storage()
            .instance()
            .get(&REPORT_TEMPLATES)
            .unwrap_or_else(|| Map::new(env));
        templates.set(counter, template);
        env.storage().instance().set(&REPORT_TEMPLATES, &templates);
        env.storage()
            .instance()
            .set(&REPORT_TEMPLATE_COUNTER, &counter);

        Ok(counter)
    }

    /// Get report template by id
    pub fn get_report_template(env: &Env, template_id: u64) -> Option<ReportTemplate> {
        let templates: Map<u64, ReportTemplate> = env
            .storage()
            .instance()
            .get(&REPORT_TEMPLATES)
            .unwrap_or_else(|| Map::new(env));
        templates.get(template_id)
    }

    /// Schedule a report (owner must auth)
    pub fn schedule_report(
        env: &Env,
        owner: Address,
        template_id: u64,
        next_run_at: u64,
        interval_seconds: u64,
    ) -> Result<u64, BridgeError> {
        owner.require_auth();

        if Self::get_report_template(env, template_id).is_none() {
            return Err(BridgeError::InvalidInput);
        }

        let mut counter: u64 = env
            .storage()
            .instance()
            .get(&REPORT_SCHEDULE_COUNTER)
            .unwrap_or(0u64);
        counter += 1;

        let schedule = ReportSchedule {
            schedule_id: counter,
            template_id,
            owner: owner.clone(),
            next_run_at,
            interval_seconds,
            enabled: true,
            created_at: env.ledger().timestamp(),
        };

        let mut schedules: Map<u64, ReportSchedule> = env
            .storage()
            .instance()
            .get(&REPORT_SCHEDULES)
            .unwrap_or_else(|| Map::new(env));
        schedules.set(counter, schedule);
        env.storage().instance().set(&REPORT_SCHEDULES, &schedules);
        env.storage()
            .instance()
            .set(&REPORT_SCHEDULE_COUNTER, &counter);

        ReportScheduledEvent {
            schedule_id: counter,
            template_id,
            owner,
            next_run_at,
        }
        .publish(env);

        Ok(counter)
    }

    /// Get scheduled reports for an owner
    pub fn get_scheduled_reports(env: &Env, owner: Address) -> Vec<ReportSchedule> {
        let schedules: Map<u64, ReportSchedule> = env
            .storage()
            .instance()
            .get(&REPORT_SCHEDULES)
            .unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        for (_id, s) in schedules.iter() {
            if s.owner == owner {
                result.push_back(s);
            }
        }
        result
    }

    /// Generate a report snapshot (stores result, emits event)
    pub fn generate_report_snapshot(
        env: &Env,
        generator: Address,
        template_id: u64,
        period_start: u64,
        period_end: u64,
    ) -> Result<u64, BridgeError> {
        generator.require_auth();

        let template =
            Self::get_report_template(env, template_id).ok_or(BridgeError::InvalidInput)?;

        let analytics = Self::get_dashboard_analytics(env);
        // Summary stored as placeholder; full data available via get_dashboard_analytics
        let _ = analytics;
        let summary = Bytes::new(env);

        let mut counter: u64 = env
            .storage()
            .instance()
            .get(&REPORT_SNAPSHOT_COUNTER)
            .unwrap_or(0u64);
        counter += 1;

        let snapshot = ReportSnapshot {
            report_id: counter,
            template_id,
            report_type: template.report_type.clone(),
            generated_at: env.ledger().timestamp(),
            period_start,
            period_end,
            generated_by: generator.clone(),
            summary,
        };

        let mut snapshots: Map<u64, ReportSnapshot> = env
            .storage()
            .instance()
            .get(&REPORT_SNAPSHOTS)
            .unwrap_or_else(|| Map::new(env));
        snapshots.set(counter, snapshot);
        env.storage().instance().set(&REPORT_SNAPSHOTS, &snapshots);
        env.storage()
            .instance()
            .set(&REPORT_SNAPSHOT_COUNTER, &counter);

        ReportGeneratedEvent {
            report_id: counter,
            report_type: template.report_type,
            generated_by: generator,
            period_start,
            period_end,
        }
        .publish(env);

        Ok(counter)
    }

    /// Get report snapshot by id
    pub fn get_report_snapshot(env: &Env, report_id: u64) -> Option<ReportSnapshot> {
        let snapshots: Map<u64, ReportSnapshot> = env
            .storage()
            .instance()
            .get(&REPORT_SNAPSHOTS)
            .unwrap_or_else(|| Map::new(env));
        snapshots.get(report_id)
    }

    /// Record report view for usage analytics
    pub fn record_report_view(
        env: &Env,
        report_id: u64,
        viewer: Address,
    ) -> Result<(), BridgeError> {
        viewer.require_auth();

        if Self::get_report_snapshot(env, report_id).is_none() {
            return Err(BridgeError::InvalidInput);
        }

        let usage = ReportUsage {
            report_id,
            viewer: viewer.clone(),
            viewed_at: env.ledger().timestamp(),
        };

        let key = (report_id, viewer);
        let mut usage_map: Map<(u64, Address), ReportUsage> = env
            .storage()
            .instance()
            .get(&REPORT_USAGE)
            .unwrap_or_else(|| Map::new(env));
        usage_map.set(key, usage);
        env.storage().instance().set(&REPORT_USAGE, &usage_map);

        Ok(())
    }

    /// Get usage count for a report
    pub fn get_report_usage_count(env: &Env, report_id: u64) -> u32 {
        let usage_map: Map<(u64, Address), ReportUsage> = env
            .storage()
            .instance()
            .get(&REPORT_USAGE)
            .unwrap_or_else(|| Map::new(env));

        let mut count: u32 = 0;
        for ((rid, _), _) in usage_map.iter() {
            if rid == report_id {
                count += 1;
            }
        }
        count
    }

    /// Add a comment to a report (collaboration)
    pub fn add_report_comment(
        env: &Env,
        report_id: u64,
        author: Address,
        body: Bytes,
    ) -> Result<u64, BridgeError> {
        author.require_auth();

        if Self::get_report_snapshot(env, report_id).is_none() {
            return Err(BridgeError::InvalidInput);
        }

        let mut counter: u64 = env
            .storage()
            .instance()
            .get(&REPORT_COMMENT_COUNTER)
            .unwrap_or(0u64);
        counter += 1;

        let comment = ReportComment {
            comment_id: counter,
            report_id,
            author: author.clone(),
            body,
            created_at: env.ledger().timestamp(),
        };

        let mut comments: Map<u64, ReportComment> = env
            .storage()
            .instance()
            .get(&REPORT_COMMENTS)
            .unwrap_or_else(|| Map::new(env));
        comments.set(counter, comment);
        env.storage().instance().set(&REPORT_COMMENTS, &comments);
        env.storage()
            .instance()
            .set(&REPORT_COMMENT_COUNTER, &counter);

        ReportCommentAddedEvent {
            report_id,
            comment_id: counter,
            author,
        }
        .publish(env);

        Ok(counter)
    }

    /// Get comments for a report
    pub fn get_report_comments(env: &Env, report_id: u64) -> Vec<ReportComment> {
        let comments: Map<u64, ReportComment> = env
            .storage()
            .instance()
            .get(&REPORT_COMMENTS)
            .unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        for (_id, c) in comments.iter() {
            if c.report_id == report_id {
                result.push_back(c);
            }
        }
        result
    }

    /// Create an alert rule
    pub fn create_alert_rule(
        env: &Env,
        owner: Address,
        name: Bytes,
        condition_type: AlertConditionType,
        threshold: i128,
    ) -> Result<u64, BridgeError> {
        owner.require_auth();

        let mut counter: u64 = env
            .storage()
            .instance()
            .get(&ALERT_RULE_COUNTER)
            .unwrap_or(0u64);
        counter += 1;

        let rule = AlertRule {
            rule_id: counter,
            name,
            condition_type: condition_type.clone(),
            threshold,
            owner: owner.clone(),
            enabled: true,
            created_at: env.ledger().timestamp(),
        };

        let mut rules: Map<u64, AlertRule> = env
            .storage()
            .instance()
            .get(&ALERT_RULES)
            .unwrap_or_else(|| Map::new(env));
        rules.set(counter, rule);
        env.storage().instance().set(&ALERT_RULES, &rules);
        env.storage().instance().set(&ALERT_RULE_COUNTER, &counter);

        Ok(counter)
    }

    /// Get alert rules for an owner
    pub fn get_alert_rules(env: &Env, owner: Address) -> Vec<AlertRule> {
        let rules: Map<u64, AlertRule> = env
            .storage()
            .instance()
            .get(&ALERT_RULES)
            .unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        for (_id, r) in rules.iter() {
            if r.owner == owner {
                result.push_back(r);
            }
        }
        result
    }

    /// Evaluate alert rules and emit AlertTriggeredEvent if any threshold is breached
    pub fn evaluate_alerts(env: &Env) -> Vec<u64> {
        let rules: Map<u64, AlertRule> = env
            .storage()
            .instance()
            .get(&ALERT_RULES)
            .unwrap_or_else(|| Map::new(env));

        let bridge_metrics = AnalyticsManager::get_bridge_metrics(env);
        let health = AnalyticsManager::calculate_health_score(env);
        let escrow_metrics = EscrowAnalyticsManager::get_metrics(env);

        let mut triggered = Vec::new(env);
        for (rule_id, rule) in rules.iter() {
            if !rule.enabled {
                continue;
            }

            let (current_value, should_trigger) = match rule.condition_type {
                AlertConditionType::BridgeHealthBelow => {
                    let v = health as i128;
                    (v, v < rule.threshold)
                }
                AlertConditionType::EscrowDisputeRateAbove => {
                    let rate = if escrow_metrics.total_escrows > 0 {
                        (escrow_metrics.total_disputes as i128 * 10_000)
                            / escrow_metrics.total_escrows as i128
                    } else {
                        0
                    };
                    (rate, rate > rule.threshold)
                }
                AlertConditionType::VolumeAbove => {
                    let v = bridge_metrics.total_volume;
                    (v, v > rule.threshold)
                }
                AlertConditionType::VolumeBelow => {
                    let v = bridge_metrics.total_volume;
                    (v, v < rule.threshold)
                }
                AlertConditionType::TransactionCountAbove => {
                    let v = bridge_metrics.total_transactions as i128;
                    (v, v > rule.threshold)
                }
            };

            if should_trigger {
                AlertTriggeredEvent {
                    rule_id,
                    condition_type: rule.condition_type,
                    current_value,
                    threshold: rule.threshold,
                    triggered_at: env.ledger().timestamp(),
                }
                .publish(env);
                triggered.push_back(rule_id);
            }
        }
        triggered
    }

    /// Get dashboard-ready aggregate analytics for visualizations
    pub fn get_dashboard_analytics(env: &Env) -> DashboardAnalytics {
        let bridge_metrics = AnalyticsManager::get_bridge_metrics(env);
        let health = AnalyticsManager::calculate_health_score(env);
        let escrow_metrics = EscrowAnalyticsManager::get_metrics(env);
        let audit_count = AuditManager::get_audit_count(env);

        let compliance_count: u32 = 0; // Could be extended to count ComplianceReports if stored by id range

        DashboardAnalytics {
            bridge_health_score: health,
            bridge_total_volume: bridge_metrics.total_volume,
            bridge_total_transactions: bridge_metrics.total_transactions,
            bridge_success_rate: bridge_metrics.success_rate,
            escrow_total_count: escrow_metrics.total_escrows,
            escrow_total_volume: escrow_metrics.total_volume,
            escrow_dispute_count: escrow_metrics.total_disputes,
            escrow_avg_resolution_time: escrow_metrics.average_resolution_time,
            compliance_report_count: compliance_count,
            audit_record_count: audit_count,
            generated_at: env.ledger().timestamp(),
        }
    }

    /// Get recent report snapshots (for listing)
    pub fn get_recent_report_snapshots(env: &Env, limit: u32) -> Vec<ReportSnapshot> {
        let counter: u64 = env
            .storage()
            .instance()
            .get(&REPORT_SNAPSHOT_COUNTER)
            .unwrap_or(0u64);
        let snapshots: Map<u64, ReportSnapshot> = env
            .storage()
            .instance()
            .get(&REPORT_SNAPSHOTS)
            .unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        let start = if counter > limit as u64 {
            counter - limit as u64
        } else {
            1
        };
        for id in start..=counter {
            if let Some(s) = snapshots.get(id) {
                result.push_back(s);
            }
        }
        result
    }
}
