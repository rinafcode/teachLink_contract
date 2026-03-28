#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]

use soroban_sdk::{testutils::Address as _, Address, Bytes, Env};
use teachlink_contract::{BridgeError, TeachLinkBridge, TeachLinkBridgeClient};

fn setup() -> (Env, TeachLinkBridgeClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    (env, client)
}

fn init_bridge(env: &Env, client: &TeachLinkBridgeClient) -> (Address, Address, Address) {
    let token = Address::generate(env);
    let admin = Address::generate(env);
    let fee_recipient = Address::generate(env);
    client.initialize(&token, &admin, &2, &fee_recipient);
    (token, admin, fee_recipient)
}

// ── Initialize ─────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let (env, client) = setup();
    let (token, _admin, _fee_recipient) = init_bridge(&env, &client);

    assert_eq!(client.get_token(), token);
    assert_eq!(client.get_bridge_fee(), 0i128);
    assert_eq!(client.get_nonce(), 0u64);
}

#[test]
fn test_initialize_twice_fails() {
    let (env, client) = setup();
    init_bridge(&env, &client);

    let token2 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let fee2 = Address::generate(&env);
    let result = client.try_initialize(&token2, &admin2, &1, &fee2);
    assert_eq!(result, Err(Ok(BridgeError::AlreadyInitialized)));
}

#[test]
fn test_initialize_zero_validators_fails() {
    let (env, client) = setup();
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee = Address::generate(&env);

    let result = client.try_initialize(&token, &admin, &0, &fee);
    assert_eq!(result, Err(Ok(BridgeError::MinimumValidatorsMustBeAtLeastOne)));
}

// ── Validators ─────────────────────────────────────────────────────

#[test]
fn test_add_and_remove_validator() {
    let (env, client) = setup();
    init_bridge(&env, &client);

    let validator = Address::generate(&env);
    client.add_validator(&validator);
    assert!(client.is_validator(&validator));

    client.remove_validator(&validator);
    assert!(!client.is_validator(&validator));
}

// ── Supported Chains ───────────────────────────────────────────────

#[test]
fn test_add_and_remove_supported_chain() {
    let (env, client) = setup();
    init_bridge(&env, &client);

    client.add_supported_chain(&1);
    client.add_supported_chain(&2);
    assert!(client.is_chain_supported(&1));
    assert!(client.is_chain_supported(&2));
    assert!(!client.is_chain_supported(&3));

    client.remove_supported_chain(&1);
    assert!(!client.is_chain_supported(&1));
}

// ── Bridge Fee ─────────────────────────────────────────────────────

#[test]
fn test_set_bridge_fee() {
    let (env, client) = setup();
    init_bridge(&env, &client);

    assert_eq!(client.get_bridge_fee(), 0i128);
    client.set_bridge_fee(&100);
    assert_eq!(client.get_bridge_fee(), 100i128);
}

#[test]
fn test_set_bridge_fee_negative_fails() {
    let (env, client) = setup();
    init_bridge(&env, &client);

    let result = client.try_set_bridge_fee(&-1);
    assert_eq!(result, Err(Ok(BridgeError::FeeCannotBeNegative)));
}

// ── Min Validators ─────────────────────────────────────────────────

#[test]
fn test_set_min_validators() {
    let (env, client) = setup();
    init_bridge(&env, &client);

    client.set_min_validators(&3);
    // Verify it took effect by checking that complete_bridge would need 3 validators
}

#[test]
fn test_set_min_validators_zero_fails() {
    let (env, client) = setup();
    init_bridge(&env, &client);

    let result = client.try_set_min_validators(&0);
    assert_eq!(result, Err(Ok(BridgeError::MinimumValidatorsMustBeAtLeastOne)));
}

// ── Fee Recipient ──────────────────────────────────────────────────

#[test]
fn test_set_fee_recipient() {
    let (env, client) = setup();
    init_bridge(&env, &client);

    let new_recipient = Address::generate(&env);
    client.set_fee_recipient(&new_recipient);
    // No getter exposed, but no panic means success
}

// ── Bridge Out Validation ──────────────────────────────────────────

#[test]
fn test_bridge_out_unsupported_chain_fails() {
    let (env, client) = setup();
    init_bridge(&env, &client);

    let user = Address::generate(&env);
    let dest = Bytes::from_array(&env, &[0; 20]);
    let result = client.try_bridge_out(&user, &1000, &999, &dest);
    assert!(result.is_err());
}

#[test]
fn test_bridge_out_zero_amount_fails() {
    let (env, client) = setup();
    init_bridge(&env, &client);
    client.add_supported_chain(&1);

    let user = Address::generate(&env);
    let dest = Bytes::from_array(&env, &[0; 20]);
    let result = client.try_bridge_out(&user, &0, &1, &dest);
    assert!(result.is_err());
}

// ── Get Bridge Transaction ─────────────────────────────────────────

#[test]
fn test_get_nonexistent_bridge_transaction() {
    let (env, client) = setup();
    init_bridge(&env, &client);

    assert!(client.get_bridge_transaction(&999).is_none());
}

// ── Mark Failed / Cancel ───────────────────────────────────────────

#[test]
fn test_mark_bridge_failed_nonexistent_fails() {
    let (env, client) = setup();
    init_bridge(&env, &client);

    let result = client.try_mark_bridge_failed(&99, &Bytes::from_slice(&env, b"fail"));
    assert_eq!(result, Err(Ok(BridgeError::BridgeTransactionNotFound)));
}

#[test]
fn test_mark_bridge_failed_empty_reason_fails() {
    let (env, client) = setup();
    init_bridge(&env, &client);

    let result = client.try_mark_bridge_failed(&1, &Bytes::new(&env));
    assert_eq!(result, Err(Ok(BridgeError::InvalidInput)));
}
