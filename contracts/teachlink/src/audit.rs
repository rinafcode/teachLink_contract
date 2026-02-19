//! Bridge Audit Trail and Compliance Module
//!
//! This module implements comprehensive audit logging and compliance reporting
//! for regulatory requirements and operational transparency.

use crate::errors::BridgeError;
use crate::events::AuditRecordCreatedEvent;
use crate::storage::{AUDIT_COUNTER, AUDIT_RECORDS, COMPLIANCE_REPORTS};
use crate::types::{AuditRecord, ComplianceReport, OperationType};
use soroban_sdk::{Address, Bytes, Env, Map, Vec};

/// Maximum audit records to store
pub const MAX_AUDIT_RECORDS: u64 = 100_000;

/// Compliance report period (7 days)
pub const COMPLIANCE_PERIOD: u64 = 604_800;

/// Audit Manager
pub struct AuditManager;

impl AuditManager {
    /// Create an audit record
    pub fn create_audit_record(
        env: &Env,
        operation_type: OperationType,
        operator: Address,
        details: Bytes,
        tx_hash: Bytes,
    ) -> Result<u64, BridgeError> {
        // Get audit counter
        let mut audit_counter: u64 = env.storage().instance().get(&AUDIT_COUNTER).unwrap_or(0u64);

        // Check if we've reached the maximum
        if audit_counter >= MAX_AUDIT_RECORDS {
            // Reset counter to implement circular buffer
            audit_counter = 0;
        }

        audit_counter += 1;

        // Create audit record
        let record = AuditRecord {
            record_id: audit_counter,
            operation_type: operation_type.clone(),
            operator: operator.clone(),
            timestamp: env.ledger().timestamp(),
            details,
            tx_hash,
        };

        // Store record
        let mut audit_records: Map<u64, AuditRecord> = env
            .storage()
            .instance()
            .get(&AUDIT_RECORDS)
            .unwrap_or_else(|| Map::new(env));
        audit_records.set(audit_counter, record);
        env.storage().instance().set(&AUDIT_RECORDS, &audit_records);
        env.storage().instance().set(&AUDIT_COUNTER, &audit_counter);

        // Emit event
        AuditRecordCreatedEvent {
            record_id: audit_counter,
            operation_type,
            operator: operator.clone(),
            timestamp: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(audit_counter)
    }

    /// Get audit record by ID
    pub fn get_audit_record(env: &Env, record_id: u64) -> Option<AuditRecord> {
        let audit_records: Map<u64, AuditRecord> = env
            .storage()
            .instance()
            .get(&AUDIT_RECORDS)
            .unwrap_or_else(|| Map::new(env));
        audit_records.get(record_id)
    }

    /// Get audit records by time range
    pub fn get_audit_records_by_time(
        env: &Env,
        start_time: u64,
        end_time: u64,
    ) -> Vec<AuditRecord> {
        let audit_records: Map<u64, AuditRecord> = env
            .storage()
            .instance()
            .get(&AUDIT_RECORDS)
            .unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        for (_record_id, record) in audit_records.iter() {
            if record.timestamp >= start_time && record.timestamp <= end_time {
                result.push_back(record);
            }
        }
        result
    }

    /// Get audit records by operation type
    pub fn get_audit_records_by_type(env: &Env, operation_type: OperationType) -> Vec<AuditRecord> {
        let audit_records: Map<u64, AuditRecord> = env
            .storage()
            .instance()
            .get(&AUDIT_RECORDS)
            .unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        for (_record_id, record) in audit_records.iter() {
            if record.operation_type == operation_type {
                result.push_back(record);
            }
        }
        result
    }

    /// Get audit records by operator
    pub fn get_audit_records_by_operator(env: &Env, operator: Address) -> Vec<AuditRecord> {
        let audit_records: Map<u64, AuditRecord> = env
            .storage()
            .instance()
            .get(&AUDIT_RECORDS)
            .unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        for (_record_id, record) in audit_records.iter() {
            if record.operator == operator {
                result.push_back(record);
            }
        }
        result
    }

    /// Generate compliance report
    pub fn generate_compliance_report(
        env: &Env,
        period_start: u64,
        period_end: u64,
    ) -> Result<u64, BridgeError> {
        let audit_records = Self::get_audit_records_by_time(env, period_start, period_end);

        // Calculate metrics
        let mut total_volume: i128 = 0;
        let mut total_transactions: u64 = 0;
        let mut unique_users: Map<Address, bool> = Map::new(env);
        let mut validator_performance: Map<Address, u32> = Map::new(env);

        for record in audit_records.iter() {
            match record.operation_type {
                OperationType::BridgeIn | OperationType::BridgeOut => {
                    total_transactions += 1;
                    unique_users.set(record.operator.clone(), true);

                    // Extract volume from details (simplified)
                    // In a real implementation, you'd parse the details bytes
                    total_volume += 0; // Placeholder
                }
                OperationType::ValidatorAdded | OperationType::ValidatorRemoved => {
                    let current_count = validator_performance
                        .get(record.operator.clone())
                        .unwrap_or(0);
                    validator_performance.set(record.operator.clone(), current_count + 1);
                }
                _ => {}
            }
        }

        // Create report
        let report = ComplianceReport {
            report_id: env.ledger().timestamp(),
            period_start,
            period_end,
            total_volume,
            total_transactions,
            unique_users: unique_users.len(),
            validator_performance,
        };

        // Store report
        let mut reports: Map<u64, ComplianceReport> = env
            .storage()
            .instance()
            .get(&COMPLIANCE_REPORTS)
            .unwrap_or_else(|| Map::new(env));
        reports.set(report.report_id, report.clone());
        env.storage().instance().set(&COMPLIANCE_REPORTS, &reports);

        Ok(report.report_id)
    }

    /// Get compliance report
    pub fn get_compliance_report(env: &Env, report_id: u64) -> Option<ComplianceReport> {
        let reports: Map<u64, ComplianceReport> = env
            .storage()
            .instance()
            .get(&COMPLIANCE_REPORTS)
            .unwrap_or_else(|| Map::new(env));
        reports.get(report_id)
    }

    /// Get total audit record count
    pub fn get_audit_count(env: &Env) -> u64 {
        env.storage().instance().get(&AUDIT_COUNTER).unwrap_or(0u64)
    }

    /// Get recent audit records (last N records)
    pub fn get_recent_audit_records(env: &Env, count: u32) -> Vec<AuditRecord> {
        let audit_counter: u64 = env.storage().instance().get(&AUDIT_COUNTER).unwrap_or(0u64);

        let audit_records: Map<u64, AuditRecord> = env
            .storage()
            .instance()
            .get(&AUDIT_RECORDS)
            .unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        let start = if audit_counter > count as u64 {
            audit_counter - count as u64
        } else {
            1
        };

        for i in start..=audit_counter {
            if let Some(record) = audit_records.get(i) {
                result.push_back(record);
            }
        }
        result
    }

    /// Log bridge operation
    pub fn log_bridge_operation(
        env: &Env,
        is_outgoing: bool,
        operator: Address,
        amount: i128,
        chain_id: u32,
        tx_hash: Bytes,
    ) -> Result<u64, BridgeError> {
        let operation_type = if is_outgoing {
            OperationType::BridgeOut
        } else {
            OperationType::BridgeIn
        };

        let details = Bytes::from_slice(env, &amount.to_be_bytes());

        Self::create_audit_record(env, operation_type, operator, details, tx_hash)
    }

    /// Log validator operation
    pub fn log_validator_operation(
        env: &Env,
        is_added: bool,
        validator: Address,
        admin: Address,
        tx_hash: Bytes,
    ) -> Result<u64, BridgeError> {
        let operation_type = if is_added {
            OperationType::ValidatorAdded
        } else {
            OperationType::ValidatorRemoved
        };

        Self::create_audit_record(env, operation_type, admin, Bytes::new(env), tx_hash)
    }

    /// Log emergency operation
    pub fn log_emergency_operation(
        env: &Env,
        is_pause: bool,
        operator: Address,
        reason: Bytes,
        tx_hash: Bytes,
    ) -> Result<u64, BridgeError> {
        let operation_type = if is_pause {
            OperationType::EmergencyPause
        } else {
            OperationType::EmergencyResume
        };

        Self::create_audit_record(env, operation_type, operator, reason, tx_hash)
    }

    /// Clear old audit records (maintenance)
    pub fn clear_old_records(
        env: &Env,
        before_timestamp: u64,
        admin: Address,
    ) -> Result<u32, BridgeError> {
        admin.require_auth();

        let audit_records: Map<u64, AuditRecord> = env
            .storage()
            .instance()
            .get(&AUDIT_RECORDS)
            .unwrap_or_else(|| Map::new(env));

        let mut cleared_count: u32 = 0;
        let mut new_records: Map<u64, AuditRecord> = Map::new(env);

        for (record_id, record) in audit_records.iter() {
            if record.timestamp >= before_timestamp {
                new_records.set(record_id, record);
            } else {
                cleared_count += 1;
            }
        }

        env.storage().instance().set(&AUDIT_RECORDS, &new_records);

        Ok(cleared_count)
    }
}
