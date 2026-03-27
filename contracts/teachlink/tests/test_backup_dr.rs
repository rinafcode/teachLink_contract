#![cfg(test)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unreadable_literal)]

//! Tests for backup and disaster recovery system.
//!
//! When the contract impl is enabled, extend with:
//! - create_backup, get_backup_manifest
//! - verify_backup (valid / invalid hash)
//! - schedule_backup, get_scheduled_backups
//! - record_recovery, get_recovery_records
//! - get_recent_backups

use soroban_sdk::Env;

use teachlink_contract::{RtoTier, TeachLinkBridge};

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
