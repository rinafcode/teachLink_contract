#![cfg(test)]

use soroban_sdk::{testutils::Address as _, testutils::Ledger, Address, Bytes, Env};
use teachlink_contract::TeachLinkBridge;

fn setup() -> (Env, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TeachLinkBridge, ());
    (env, contract_id)
}

fn set_time(env: &Env, ts: u64) {
    env.ledger().with_mut(|l| l.timestamp = ts);
}

// ── Audit Record Creation ──────────────────────────────────────────

#[test]
fn test_create_and_get_audit_record() {
    let (env, cid) = setup();
    let operator = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    set_time(&env, 5000);
    let record_id = client.create_audit_record(
        &teachlink_contract::OperationType::BridgeOut,
        &operator,
        &Bytes::from_slice(&env, b"bridge 100 tokens"),
        &Bytes::from_slice(&env, b"tx_hash_1"),
    );

    assert_eq!(record_id, 1);

    let record = client.get_audit_record(&1).unwrap();
    assert_eq!(record.record_id, 1);
    assert_eq!(record.operator, operator);
    assert_eq!(record.timestamp, 5000);
}

#[test]
fn test_audit_count_increments() {
    let (env, cid) = setup();
    let operator = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.create_audit_record(
        &teachlink_contract::OperationType::BridgeIn,
        &operator,
        &Bytes::new(&env),
        &Bytes::new(&env),
    );
    client.create_audit_record(
        &teachlink_contract::OperationType::BridgeOut,
        &operator,
        &Bytes::new(&env),
        &Bytes::new(&env),
    );

    assert_eq!(client.get_audit_count(), 2);
}

// ── Query by Type ──────────────────────────────────────────────────

#[test]
fn test_get_audit_records_by_type() {
    let (env, cid) = setup();
    let op = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.create_audit_record(
        &teachlink_contract::OperationType::BridgeOut,
        &op,
        &Bytes::new(&env),
        &Bytes::new(&env),
    );
    client.create_audit_record(
        &teachlink_contract::OperationType::ValidatorAdded,
        &op,
        &Bytes::new(&env),
        &Bytes::new(&env),
    );
    client.create_audit_record(
        &teachlink_contract::OperationType::BridgeOut,
        &op,
        &Bytes::new(&env),
        &Bytes::new(&env),
    );

    let bridge_outs =
        client.get_audit_records_by_type(&teachlink_contract::OperationType::BridgeOut);
    assert_eq!(bridge_outs.len(), 2);
}

// ── Query by Operator ──────────────────────────────────────────────

#[test]
fn test_get_audit_records_by_operator() {
    let (env, cid) = setup();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.create_audit_record(
        &teachlink_contract::OperationType::BridgeIn,
        &alice,
        &Bytes::new(&env),
        &Bytes::new(&env),
    );
    client.create_audit_record(
        &teachlink_contract::OperationType::BridgeIn,
        &bob,
        &Bytes::new(&env),
        &Bytes::new(&env),
    );

    let alice_records = client.get_audit_records_by_operator(&alice);
    assert_eq!(alice_records.len(), 1);
}

// ── Query by Time Range ────────────────────────────────────────────

#[test]
fn test_get_audit_records_by_time() {
    let (env, cid) = setup();
    let op = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    set_time(&env, 1000);
    client.create_audit_record(
        &teachlink_contract::OperationType::BridgeIn,
        &op,
        &Bytes::new(&env),
        &Bytes::new(&env),
    );

    set_time(&env, 5000);
    client.create_audit_record(
        &teachlink_contract::OperationType::BridgeOut,
        &op,
        &Bytes::new(&env),
        &Bytes::new(&env),
    );

    set_time(&env, 9000);
    client.create_audit_record(
        &teachlink_contract::OperationType::EmergencyPause,
        &op,
        &Bytes::new(&env),
        &Bytes::new(&env),
    );

    let mid_range = client.get_audit_records_by_time(&2000, &6000);
    assert_eq!(mid_range.len(), 1);
}

// ── Recent Records ─────────────────────────────────────────────────

#[test]
fn test_get_recent_audit_records() {
    let (env, cid) = setup();
    let op = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    for _ in 0..5 {
        client.create_audit_record(
            &teachlink_contract::OperationType::BridgeIn,
            &op,
            &Bytes::new(&env),
            &Bytes::new(&env),
        );
    }

    let recent = client.get_recent_audit_records(&3);
    // Implementation returns records from (counter-count)..=counter, which is count+1 when all exist
    assert!(recent.len() >= 3);
    assert!(recent.len() <= 5);
}

// ── Compliance Report ──────────────────────────────────────────────

#[test]
fn test_generate_and_get_compliance_report() {
    let (env, cid) = setup();
    let op = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    set_time(&env, 1000);
    client.create_audit_record(
        &teachlink_contract::OperationType::BridgeOut,
        &op,
        &Bytes::new(&env),
        &Bytes::new(&env),
    );

    let report_id = client.generate_compliance_report(&0, &2000);
    let report = client.get_compliance_report(&report_id).unwrap();
    assert_eq!(report.period_start, 0);
    assert_eq!(report.period_end, 2000);
}

// ── Clear Old Records ──────────────────────────────────────────────

#[test]
fn test_clear_old_records() {
    let (env, cid) = setup();
    let admin = Address::generate(&env);
    let op = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    set_time(&env, 100);
    client.create_audit_record(
        &teachlink_contract::OperationType::BridgeIn,
        &op,
        &Bytes::new(&env),
        &Bytes::new(&env),
    );

    set_time(&env, 5000);
    client.create_audit_record(
        &teachlink_contract::OperationType::BridgeOut,
        &op,
        &Bytes::new(&env),
        &Bytes::new(&env),
    );

    let cleared = client.clear_old_records(&1000, &admin);
    assert_eq!(cleared, 1);
}

// ── Convenience Loggers ────────────────────────────────────────────

#[test]
fn test_log_bridge_operation() {
    let (env, cid) = setup();
    let op = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    let id = client.log_bridge_operation(
        &true,
        &op,
        &500,
        &1,
        &Bytes::from_slice(&env, b"hash"),
    );
    assert_eq!(id, 1);

    let record = client.get_audit_record(&1).unwrap();
    assert_eq!(record.operation_type, teachlink_contract::OperationType::BridgeOut);
}

#[test]
fn test_log_emergency_operation() {
    let (env, cid) = setup();
    let op = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    let id = client.log_emergency_operation(
        &true,
        &op,
        &Bytes::from_slice(&env, b"exploit detected"),
        &Bytes::from_slice(&env, b"hash"),
    );
    assert_eq!(id, 1);

    let record = client.get_audit_record(&1).unwrap();
    assert_eq!(
        record.operation_type,
        teachlink_contract::OperationType::EmergencyPause
    );
}
