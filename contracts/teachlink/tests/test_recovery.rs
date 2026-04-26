#![cfg(test)]

//! Recovery tests covering:
//! - Backup recovery
//! - State restoration
//! - Error recovery
//! - Data consistency

use soroban_sdk::{testutils::Address as _, Address, Bytes, Env};
use teachlink_contract::{RtoTier, TeachLinkBridge, TeachLinkBridgeClient};

fn setup() -> (Env, TeachLinkBridgeClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    (env, client, admin)
}

// ── Backup Recovery ──────────────────────────────────────────────────────────

#[test]
fn test_backup_recovery_success() {
    let (env, client, admin) = setup();

    let hash = Bytes::from_array(&env, &[1u8; 8]);
    let backup_id = client.create_backup(&admin, &hash, &RtoTier::Critical, &0u64);

    let recovery_id = client.record_recovery(&backup_id, &admin, &120u64, &true);
    assert!(recovery_id > 0);

    let records = client.get_recovery_records(&10u32);
    assert_eq!(records.len(), 1);
    let rec = records.get(0).unwrap();
    assert_eq!(rec.backup_id, backup_id);
    assert!(rec.success);
    assert_eq!(rec.recovery_duration_secs, 120);
}

#[test]
fn test_backup_recovery_failure_recorded() {
    let (env, client, admin) = setup();

    let hash = Bytes::from_array(&env, &[2u8; 8]);
    let backup_id = client.create_backup(&admin, &hash, &RtoTier::High, &0u64);

    let recovery_id = client.record_recovery(&backup_id, &admin, &600u64, &false);
    assert!(recovery_id > 0);

    let records = client.get_recovery_records(&10u32);
    let rec = records.get(0).unwrap();
    assert!(!rec.success);
    assert_eq!(rec.recovery_duration_secs, 600);
}

#[test]
fn test_multiple_recovery_attempts_tracked() {
    let (env, client, admin) = setup();

    let hash = Bytes::from_array(&env, &[3u8; 8]);
    let backup_id = client.create_backup(&admin, &hash, &RtoTier::Standard, &0u64);

    client.record_recovery(&backup_id, &admin, &300u64, &false);
    client.record_recovery(&backup_id, &admin, &120u64, &true);

    let records = client.get_recovery_records(&10u32);
    assert_eq!(records.len(), 2);
    // IDs are monotonically increasing
    assert!(records.get(1).unwrap().recovery_id > records.get(0).unwrap().recovery_id);
}

// ── State Restoration ────────────────────────────────────────────────────────

#[test]
fn test_state_restored_from_backup_manifest() {
    let (env, client, admin) = setup();

    let hash = Bytes::from_array(&env, &[10, 20, 30, 40, 50, 60, 70, 80]);
    let backup_id = client.create_backup(&admin, &hash, &RtoTier::Critical, &99u64);

    // Simulate state restoration: retrieve manifest and verify fields are intact
    let manifest = client.get_backup_manifest(&backup_id).unwrap();
    assert_eq!(manifest.backup_id, backup_id);
    assert_eq!(manifest.integrity_hash, hash);
    assert_eq!(manifest.rto_tier, RtoTier::Critical);
    assert_eq!(manifest.encryption_ref, 99u64);
    assert_eq!(manifest.created_by, admin);
}

#[test]
fn test_recent_backups_reflect_current_state() {
    let (env, client, admin) = setup();

    let hash_a = Bytes::from_array(&env, &[1u8; 8]);
    let hash_b = Bytes::from_array(&env, &[2u8; 8]);
    client.create_backup(&admin, &hash_a, &RtoTier::High, &0u64);
    let id_b = client.create_backup(&admin, &hash_b, &RtoTier::Critical, &0u64);

    let recent = client.get_recent_backups(&1u32);
    assert_eq!(recent.len(), 1);
    assert_eq!(recent.get(0).unwrap().backup_id, id_b);
}

#[test]
fn test_schedule_state_persists_after_creation() {
    let (env, client, admin) = setup();

    client.schedule_backup(&admin, &1_800_000_000u64, &3600u64, &RtoTier::Standard);

    let schedules = client.get_scheduled_backups(&admin);
    assert_eq!(schedules.len(), 1);
    let s = schedules.get(0).unwrap();
    assert_eq!(s.owner, admin);
    assert_eq!(s.interval_seconds, 3600u64);
    assert_eq!(s.next_run_at, 1_800_000_000u64);
    assert!(s.enabled);
}

// ── Error Recovery ───────────────────────────────────────────────────────────

#[test]
fn test_recovery_on_nonexistent_backup_returns_error() {
    let (env, client, admin) = setup();

    let result = client.try_record_recovery(&999u64, &admin, &60u64, &true);
    assert!(result.is_err());
}

#[test]
fn test_verify_backup_with_wrong_hash_returns_false() {
    let (env, client, admin) = setup();

    let correct = Bytes::from_array(&env, &[5u8; 8]);
    let wrong = Bytes::from_array(&env, &[9u8; 8]);
    let backup_id = client.create_backup(&admin, &correct, &RtoTier::High, &0u64);

    let valid = client.verify_backup(&backup_id, &admin, &wrong);
    assert!(!valid);
}

#[test]
fn test_system_continues_after_failed_recovery() {
    let (env, client, admin) = setup();

    let hash = Bytes::from_array(&env, &[7u8; 8]);
    let backup_id = client.create_backup(&admin, &hash, &RtoTier::Standard, &0u64);

    // Failed recovery
    client.record_recovery(&backup_id, &admin, &500u64, &false);
    // Successful retry
    let ok_id = client.record_recovery(&backup_id, &admin, &90u64, &true);
    assert!(ok_id > 0);

    let records = client.get_recovery_records(&10u32);
    let last = records.get(records.len() - 1).unwrap();
    assert!(last.success);
}

// ── Data Consistency ─────────────────────────────────────────────────────────

#[test]
fn test_backup_ids_are_sequential() {
    let (env, client, admin) = setup();

    let hash = Bytes::from_array(&env, &[0u8; 8]);
    let id1 = client.create_backup(&admin, &hash, &RtoTier::Standard, &0u64);
    let id2 = client.create_backup(&admin, &hash, &RtoTier::High, &0u64);
    let id3 = client.create_backup(&admin, &hash, &RtoTier::Critical, &0u64);

    assert_eq!(id2, id1 + 1);
    assert_eq!(id3, id2 + 1);
}

#[test]
fn test_recovery_ids_are_sequential() {
    let (env, client, admin) = setup();

    let hash = Bytes::from_array(&env, &[0u8; 8]);
    let backup_id = client.create_backup(&admin, &hash, &RtoTier::Standard, &0u64);

    let r1 = client.record_recovery(&backup_id, &admin, &60u64, &true);
    let r2 = client.record_recovery(&backup_id, &admin, &60u64, &true);
    assert_eq!(r2, r1 + 1);
}

#[test]
fn test_integrity_hash_preserved_exactly() {
    let (env, client, admin) = setup();

    let hash = Bytes::from_array(&env, &[0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE]);
    let backup_id = client.create_backup(&admin, &hash, &RtoTier::Critical, &0u64);

    let manifest = client.get_backup_manifest(&backup_id).unwrap();
    assert_eq!(manifest.integrity_hash, hash);

    // Verify confirms the same hash
    assert!(client.verify_backup(&backup_id, &admin, &hash));
}

#[test]
fn test_recovery_record_references_correct_backup() {
    let (env, client, admin) = setup();

    let hash_a = Bytes::from_array(&env, &[1u8; 8]);
    let hash_b = Bytes::from_array(&env, &[2u8; 8]);
    let id_a = client.create_backup(&admin, &hash_a, &RtoTier::High, &0u64);
    let id_b = client.create_backup(&admin, &hash_b, &RtoTier::Critical, &0u64);

    client.record_recovery(&id_a, &admin, &100u64, &true);
    client.record_recovery(&id_b, &admin, &200u64, &true);

    let records = client.get_recovery_records(&10u32);
    assert_eq!(records.get(0).unwrap().backup_id, id_a);
    assert_eq!(records.get(1).unwrap().backup_id, id_b);
}
