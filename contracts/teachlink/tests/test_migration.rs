//! Migration tests for TeachLink contract.
//!
//! Covers: data migration, schema updates, rollback, and data integrity.

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Bytes, Env};
use teachlink_contract::{TeachLinkBridge, TeachLinkBridgeClient};

fn setup(env: &Env) -> (TeachLinkBridgeClient<'_>, Address) {
    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(env, &contract_id);

    let token_admin = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(token_admin);
    let token = sac.address();

    let admin = Address::generate(env);
    let fee_recipient = Address::generate(env);

    env.mock_all_auths();
    client.initialize(&token, &admin, &1, &fee_recipient);

    (client, admin)
}

/// AC: Test data migration — state written before an upgrade is readable after.
#[test]
fn test_data_migration_preserves_state() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    // Record pre-upgrade version
    let version_before = client.get_contract_version();

    // Perform upgrade cycle
    let state_hash = Bytes::from_slice(&env, b"pre_upgrade_state");
    let migration_hash = Bytes::from_slice(&env, b"migration_v1_v2");
    client.prepare_upgrade(&admin, &(version_before + 1), &state_hash);
    client.execute_upgrade(&admin, &(version_before + 1), &migration_hash);

    // State (version counter) is accessible and correct after migration
    let version_after = client.get_contract_version();
    assert_eq!(version_after, version_before + 1);

    // Upgrade history record is present and links back to the previous version
    let record = client.get_upgrade_history(&version_after).unwrap();
    assert_eq!(record.previous_version, version_before);
    assert_eq!(record.version, version_after);
}

/// AC: Verify schema updates — upgrade history entry carries the migration hash.
#[test]
fn test_schema_update_recorded_in_history() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let current = client.get_contract_version();
    let state_hash = Bytes::from_slice(&env, b"schema_state_hash");
    let migration_hash = Bytes::from_slice(&env, b"schema_migration_hash");

    client.prepare_upgrade(&admin, &(current + 1), &state_hash);
    client.execute_upgrade(&admin, &(current + 1), &migration_hash);

    let record = client.get_upgrade_history(&(current + 1)).unwrap();
    assert_eq!(record.migration_hash, migration_hash);
    assert_eq!(record.upgraded_by, admin);
}

/// AC: Test rollback — after rollback the version reverts and no further rollback is possible.
#[test]
fn test_rollback_restores_previous_version() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let original = client.get_contract_version();

    // Prepare + execute upgrade so a backup exists
    let state_hash = Bytes::from_slice(&env, b"rollback_state");
    let migration_hash = Bytes::from_slice(&env, b"rollback_migration");
    client.prepare_upgrade(&admin, &(original + 1), &state_hash);
    client.execute_upgrade(&admin, &(original + 1), &migration_hash);

    assert_eq!(client.get_contract_version(), original + 1);
    assert!(client.is_rollback_available());

    // Rollback
    client.rollback_upgrade(&admin);

    assert_eq!(client.get_contract_version(), original);
    assert!(!client.is_rollback_available());
}

/// AC: Check data integrity — rollback without a prior prepare must fail.
#[test]
fn test_data_integrity_rollback_without_backup_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    // No prepare_upgrade called — rollback must be rejected
    assert!(!client.is_rollback_available());
    let result = client.try_rollback_upgrade(&admin);
    assert!(result.is_err());
}
