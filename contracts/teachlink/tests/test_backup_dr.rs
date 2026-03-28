#![cfg(test)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unreadable_literal)]

//! Tests for backup and disaster recovery system.

use soroban_sdk::{testutils::Address as _, Address, Bytes, Env};

use teachlink_contract::{RtoTier, TeachLinkBridge, TeachLinkBridgeClient};

#[test]
fn test_contract_registers_with_backup_module() {
    let env = Env::default();
    env.mock_all_auths();

    let _ = env.register(TeachLinkBridge, ());
    assert!(true);
}

#[test]
fn test_rto_tier_variants() {
    let _ = RtoTier::Critical;
    let _ = RtoTier::High;
    let _ = RtoTier::Standard;
    assert!(true);
}

#[test]
fn test_create_and_verify_backup_integrity() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    let integrity_hash = Bytes::from_array(&env, &[1, 2, 3, 4, 5, 6, 7, 8]);
    let backup_id = client.create_backup(&admin, &integrity_hash, &RtoTier::Critical, &42u64);

    let manifest = client.get_backup_manifest(&backup_id).unwrap();
    assert_eq!(manifest.backup_id, backup_id);
    assert_eq!(manifest.integrity_hash, integrity_hash.clone());
    assert_eq!(manifest.rto_tier, RtoTier::Critical);

    let valid = client.verify_backup(&backup_id, &admin, &integrity_hash);
    assert!(valid);

    let wrong_hash = Bytes::from_array(&env, &[9, 9, 9, 9, 9, 9, 9, 9]);
    let invalid = client.verify_backup(&backup_id, &admin, &wrong_hash);
    assert!(!invalid);
}

#[test]
fn test_backup_scheduling_and_restore_records() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    let integrity_hash = Bytes::from_array(&env, &[11, 12, 13, 14, 15, 16, 17, 18]);
    let backup_id = client.create_backup(&admin, &integrity_hash, &RtoTier::High, &7u64);

    let schedule_id =
        client.schedule_backup(&admin, &1_800_000_000u64, &3600u64, &RtoTier::Standard);
    assert!(schedule_id > 0);

    let schedules = client.get_scheduled_backups(&admin);
    assert_eq!(schedules.len(), 1);
    let schedule = schedules.get(0).unwrap();
    assert_eq!(schedule.owner, admin);
    assert_eq!(schedule.interval_seconds, 3600);

    // Simulate a successful restoration drill and a failed one for monitoring.
    let recovery_ok = client.record_recovery(&backup_id, &admin, &120u64, &true);
    let recovery_fail = client.record_recovery(&backup_id, &admin, &300u64, &false);
    assert!(recovery_ok > 0);
    assert!(recovery_fail > recovery_ok);

    let recoveries = client.get_recovery_records(&10u32);
    assert_eq!(recoveries.len(), 2);

    let recent_backups = client.get_recent_backups(&10u32);
    assert_eq!(recent_backups.len(), 1);
    assert_eq!(recent_backups.get(0).unwrap().backup_id, backup_id);
}
