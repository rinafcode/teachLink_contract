//! Backup and Disaster Recovery Module
//!
//! Provides backup scheduling, integrity verification, recovery recording,
//! and audit trails for compliance. Off-chain systems use events to replicate
//! data; this module records manifests, verification, and RTO recovery metrics.

use crate::audit::AuditManager;
use crate::errors::BridgeError;
use crate::events::{BackupCreatedEvent, BackupVerifiedEvent, RecoveryExecutedEvent};
use crate::storage::{
    BACKUP_COUNTER, BACKUP_MANIFESTS, BACKUP_SCHED_CNT, BACKUP_SCHEDULES, RECOVERY_CNT,
    RECOVERY_RECORDS,
};
use crate::types::{BackupManifest, BackupSchedule, OperationType, RecoveryRecord, RtoTier};
use soroban_sdk::{Address, Bytes, Env, Map, Vec};

/// Backup and disaster recovery manager
pub struct BackupManager;

impl BackupManager {
    /// Create a backup manifest (authorized caller). Integrity hash is supplied by off-chain.
    pub fn create_backup(
        env: &Env,
        creator: Address,
        integrity_hash: Bytes,
        rto_tier: RtoTier,
        encryption_ref: u64,
    ) -> Result<u64, BridgeError> {
        creator.require_auth();

        let mut counter: u64 = env.storage().instance().get(&BACKUP_COUNTER).unwrap_or(0u64);
        counter += 1;

        let manifest = BackupManifest {
            backup_id: counter,
            created_at: env.ledger().timestamp(),
            created_by: creator.clone(),
            integrity_hash: integrity_hash.clone(),
            rto_tier: rto_tier.clone(),
            encryption_ref,
        };

        let mut manifests: Map<u64, BackupManifest> = env
            .storage()
            .instance()
            .get(&BACKUP_MANIFESTS)
            .unwrap_or_else(|| Map::new(env));
        manifests.set(counter, manifest);
        env.storage().instance().set(&BACKUP_MANIFESTS, &manifests);
        env.storage().instance().set(&BACKUP_COUNTER, &counter);

        BackupCreatedEvent {
            backup_id: counter,
            created_by: creator.clone(),
            integrity_hash,
            rto_tier: rto_tier.clone(),
            created_at: env.ledger().timestamp(),
        }
        .publish(env);

        let details = Bytes::from_slice(env, &counter.to_be_bytes());
        AuditManager::create_audit_record(
            env,
            OperationType::BackupCreated,
            creator,
            details,
            Bytes::new(env),
        )?;

        Ok(counter)
    }

    /// Get backup manifest by id
    pub fn get_backup_manifest(env: &Env, backup_id: u64) -> Option<BackupManifest> {
        let manifests: Map<u64, BackupManifest> = env
            .storage()
            .instance()
            .get(&BACKUP_MANIFESTS)
            .unwrap_or_else(|| Map::new(env));
        manifests.get(backup_id)
    }

    /// Verify backup integrity (compare expected hash to stored). Emit event and audit.
    pub fn verify_backup(
        env: &Env,
        backup_id: u64,
        verifier: Address,
        expected_hash: Bytes,
    ) -> Result<bool, BridgeError> {
        verifier.require_auth();

        let manifest = Self::get_backup_manifest(env, backup_id).ok_or(BridgeError::InvalidInput)?;
        let valid = manifest.integrity_hash == expected_hash;

        BackupVerifiedEvent {
            backup_id,
            verified_by: verifier.clone(),
            verified_at: env.ledger().timestamp(),
            valid,
        }
        .publish(env);

        let details = Bytes::from_slice(env, &[if valid { 1u8 } else { 0u8 }]);
        AuditManager::create_audit_record(
            env,
            OperationType::BackupVerified,
            verifier,
            details,
            Bytes::new(env),
        )?;

        Ok(valid)
    }

    /// Schedule automated backup (owner auth)
    pub fn schedule_backup(
        env: &Env,
        owner: Address,
        next_run_at: u64,
        interval_seconds: u64,
        rto_tier: RtoTier,
    ) -> Result<u64, BridgeError> {
        owner.require_auth();

        let mut counter: u64 = env.storage().instance().get(&BACKUP_SCHED_CNT).unwrap_or(0u64);
        counter += 1;

        let schedule = BackupSchedule {
            schedule_id: counter,
            owner: owner.clone(),
            next_run_at,
            interval_seconds,
            rto_tier: rto_tier.clone(),
            enabled: true,
            created_at: env.ledger().timestamp(),
        };

        let mut schedules: Map<u64, BackupSchedule> = env
            .storage()
            .instance()
            .get(&BACKUP_SCHEDULES)
            .unwrap_or_else(|| Map::new(env));
        schedules.set(counter, schedule);
        env.storage().instance().set(&BACKUP_SCHEDULES, &schedules);
        env.storage().instance().set(&BACKUP_SCHED_CNT, &counter);

        Ok(counter)
    }

    /// Get scheduled backups for an owner
    pub fn get_scheduled_backups(env: &Env, owner: Address) -> Vec<BackupSchedule> {
        let schedules: Map<u64, BackupSchedule> = env
            .storage()
            .instance()
            .get(&BACKUP_SCHEDULES)
            .unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        for (_id, s) in schedules.iter() {
            if s.owner == owner {
                result.push_back(s);
            }
        }
        result
    }

    /// Record a recovery execution (RTO tracking and audit trail)
    pub fn record_recovery(
        env: &Env,
        backup_id: u64,
        executed_by: Address,
        recovery_duration_secs: u64,
        success: bool,
    ) -> Result<u64, BridgeError> {
        executed_by.require_auth();

        if Self::get_backup_manifest(env, backup_id).is_none() {
            return Err(BridgeError::InvalidInput);
        }

        let mut counter: u64 = env.storage().instance().get(&RECOVERY_CNT).unwrap_or(0u64);
        counter += 1;

        let record = RecoveryRecord {
            recovery_id: counter,
            backup_id,
            executed_at: env.ledger().timestamp(),
            executed_by: executed_by.clone(),
            recovery_duration_secs,
            success,
        };

        let mut records: Map<u64, RecoveryRecord> = env
            .storage()
            .instance()
            .get(&RECOVERY_RECORDS)
            .unwrap_or_else(|| Map::new(env));
        records.set(counter, record);
        env.storage().instance().set(&RECOVERY_RECORDS, &records);
        env.storage().instance().set(&RECOVERY_CNT, &counter);

        RecoveryExecutedEvent {
            recovery_id: counter,
            backup_id,
            executed_by: executed_by.clone(),
            recovery_duration_secs,
            success,
        }
        .publish(env);

        let details = Bytes::from_slice(env, &recovery_duration_secs.to_be_bytes());
        AuditManager::create_audit_record(
            env,
            OperationType::RecoveryExecuted,
            executed_by,
            details,
            Bytes::new(env),
        )?;

        Ok(counter)
    }

    /// Get recovery records (for audit trail and RTO reporting)
    pub fn get_recovery_records(env: &Env, limit: u32) -> Vec<RecoveryRecord> {
        let counter: u64 = env.storage().instance().get(&RECOVERY_CNT).unwrap_or(0u64);
        let records: Map<u64, RecoveryRecord> = env
            .storage()
            .instance()
            .get(&RECOVERY_RECORDS)
            .unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        let start = if counter > limit as u64 {
            counter - limit as u64
        } else {
            1
        };
        for id in start..=counter {
            if let Some(r) = records.get(id) {
                result.push_back(r);
            }
        }
        result
    }

    /// Get recent backup manifests (for monitoring and compliance)
    pub fn get_recent_backups(env: &Env, limit: u32) -> Vec<BackupManifest> {
        let counter: u64 = env.storage().instance().get(&BACKUP_COUNTER).unwrap_or(0u64);
        let manifests: Map<u64, BackupManifest> = env
            .storage()
            .instance()
            .get(&BACKUP_MANIFESTS)
            .unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        let start = if counter > limit as u64 {
            counter - limit as u64
        } else {
            1
        };
        for id in start..=counter {
            if let Some(m) = manifests.get(id) {
                result.push_back(m);
            }
        }
        result
    }
}
