#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, Vec};
use teachlink_contract::TeachLinkBridge;

mod helpers {
    use super::*;
    use soroban_sdk::testutils::Ledger;

    pub fn setup() -> (Env, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());
        (env, contract_id)
    }

    pub fn set_time(env: &Env, ts: u64) {
        env.ledger().with_mut(|l| l.timestamp = ts);
    }
}

use helpers::*;

// ── Pause / Resume Bridge ──────────────────────────────────────────

#[test]
fn test_pause_bridge() {
    let (env, cid) = setup();
    let pauser = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.pause_bridge(&pauser, &Bytes::from_slice(&env, b"security incident"));
    assert!(client.is_bridge_paused());
}

#[test]
fn test_pause_bridge_already_paused_fails() {
    let (env, cid) = setup();
    let pauser = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.pause_bridge(&pauser, &Bytes::from_slice(&env, b"reason"));
    let result = client.try_pause_bridge(&pauser, &Bytes::from_slice(&env, b"again"));
    assert!(result.is_err());
}

#[test]
fn test_resume_bridge() {
    let (env, cid) = setup();
    let actor = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.pause_bridge(&actor, &Bytes::from_slice(&env, b"reason"));
    assert!(client.is_bridge_paused());

    client.resume_bridge(&actor);
    assert!(!client.is_bridge_paused());
}

#[test]
fn test_resume_bridge_not_paused_fails() {
    let (env, cid) = setup();
    let actor = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    let result = client.try_resume_bridge(&actor);
    assert!(result.is_err());
}

// ── Pause / Resume Chains ──────────────────────────────────────────

#[test]
fn test_pause_and_resume_chains() {
    let (env, cid) = setup();
    let pauser = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    let chain_ids: Vec<u32> = Vec::from_array(&env, [1u32, 2u32]);
    client.pause_chains(&pauser, &chain_ids, &Bytes::from_slice(&env, b"exploit"));

    assert!(client.is_chain_paused(&1));
    assert!(client.is_chain_paused(&2));
    assert!(!client.is_chain_paused(&3));

    let paused = client.get_paused_chains();
    assert_eq!(paused.len(), 2);

    client.resume_chains(&pauser, &chain_ids);
    assert!(!client.is_chain_paused(&1));
    assert!(!client.is_chain_paused(&2));
}

// ── Circuit Breaker ────────────────────────────────────────────────

#[test]
fn test_initialize_circuit_breaker() {
    let (env, cid) = setup();
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.initialize_circuit_breaker(&1, &1_000_000, &100_000);

    let cb = client.get_circuit_breaker(&1).unwrap();
    assert_eq!(cb.chain_id, 1);
    assert_eq!(cb.max_daily_volume, 1_000_000);
    assert_eq!(cb.max_transaction_amount, 100_000);
    assert!(!cb.is_triggered);
}

#[test]
fn test_circuit_breaker_triggers_on_max_transaction() {
    let (env, cid) = setup();
    set_time(&env, 1000);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.initialize_circuit_breaker(&1, &1_000_000, &50_000);

    // Amount exceeds max_transaction_amount → returns error
    let result = client.try_check_circuit_breaker(&1, &60_000);
    assert!(result.is_err());
}

#[test]
fn test_circuit_breaker_triggers_on_daily_volume() {
    let (env, cid) = setup();
    set_time(&env, 1000);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.initialize_circuit_breaker(&1, &100_000, &80_000);

    // First transaction OK
    client.check_circuit_breaker(&1, &70_000);

    // Second pushes over daily volume → error
    let result = client.try_check_circuit_breaker(&1, &40_000);
    assert!(result.is_err());
}

#[test]
fn test_circuit_breaker_resets_after_24h() {
    let (env, cid) = setup();
    set_time(&env, 1000);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.initialize_circuit_breaker(&1, &100_000, &80_000);
    client.check_circuit_breaker(&1, &70_000);

    // Advance past 24h reset
    set_time(&env, 1000 + 86_401);
    client.check_circuit_breaker(&1, &70_000);
}

#[test]
fn test_reset_circuit_breaker() {
    let (env, cid) = setup();
    set_time(&env, 1000);
    let resetter = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.initialize_circuit_breaker(&1, &100_000, &80_000);

    // Add some volume
    client.check_circuit_breaker(&1, &50_000);
    let cb = client.get_circuit_breaker(&1).unwrap();
    assert_eq!(cb.current_daily_volume, 50_000);

    // Reset
    client.reset_circuit_breaker(&1, &resetter);
    let cb = client.get_circuit_breaker(&1).unwrap();
    assert!(!cb.is_triggered);
    assert_eq!(cb.current_daily_volume, 0);
}

#[test]
fn test_circuit_breaker_nonexistent_chain_fails() {
    let (env, cid) = setup();
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    let result = client.try_check_circuit_breaker(&99, &1000);
    assert!(result.is_err());
}

#[test]
fn test_update_circuit_breaker_limits() {
    let (env, cid) = setup();
    let updater = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.initialize_circuit_breaker(&1, &100_000, &50_000);
    client.update_circuit_breaker_limits(&1, &200_000, &150_000, &updater);

    let cb = client.get_circuit_breaker(&1).unwrap();
    assert_eq!(cb.max_daily_volume, 200_000);
    assert_eq!(cb.max_transaction_amount, 150_000);
}

// ── Emergency State View ───────────────────────────────────────────

#[test]
fn test_get_emergency_state_default() {
    let (env, cid) = setup();
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    let state = client.get_emergency_state();
    assert!(!state.is_paused);
}
