#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::too_many_lines)]

use soroban_sdk::{testutils::Address as _, Address, Bytes, Env};

use teachlink_contract::{BridgeError, TeachLinkBridge, TeachLinkBridgeClient};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());

    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);

    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    client.initialize(&token, &admin, &2, &fee_recipient);

    assert_eq!(client.get_token(), token);
    assert_eq!(client.get_bridge_fee(), 0i128);
    assert_eq!(client.get_nonce(), 0u64);
}

#[test]
fn test_add_validator() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());

    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);

    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    client.initialize(&token, &admin, &2, &fee_recipient);

    let validator = Address::generate(&env);
    env.mock_all_auths(); // Mock authentication for admin
    client.add_validator(&validator);
    assert!(client.is_validator(&validator));
}

#[test]
fn test_add_supported_chain() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());

    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);

    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    client.initialize(&token, &admin, &2, &fee_recipient);

    env.mock_all_auths(); // Mock authentication for admin
    client.add_supported_chain(&1); // Ethereum
    client.add_supported_chain(&2); // Polygon
    assert!(client.is_chain_supported(&1));
    assert!(client.is_chain_supported(&2));
    assert!(!client.is_chain_supported(&3));
}

#[test]
fn test_bridge_out_unsupported_chain() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());

    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let user = Address::generate(&env);

    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    client.initialize(&token, &admin, &2, &fee_recipient);

    env.mock_all_auths();
    // Try to bridge to unsupported chain
    let dest_addr = Bytes::from_array(&env, &[0; 20]);
    let result = client.bridge_out(&user, &1000, &999, &dest_addr);
    // Check if the result is an error (should be for unsupported chain)
    match result {
        Ok(_) => panic!("Expected error but got success"),
        Err(_) => (), // Expected error
    }
}

#[test]
fn test_bridge_out_invalid_amount() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());

    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let user = Address::generate(&env);

    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    client.initialize(&token, &admin, &2, &fee_recipient);

    env.mock_all_auths();
    client.add_supported_chain(&1);
    let dest_addr = Bytes::from_array(&env, &[0; 20]);
    let result = client.bridge_out(&user, &0, &1, &dest_addr);
    // Check if the result is an error (should be for invalid amount)
    match result {
        Ok(_) => panic!("Expected error but got success"),
        Err(_) => (), // Expected error
    }
}

#[test]
fn test_set_bridge_fee() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());

    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);

    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    client.initialize(&token, &admin, &2, &fee_recipient);

    assert_eq!(client.get_bridge_fee(), 0i128);

    env.mock_all_auths();
    client.set_bridge_fee(&100);
    assert_eq!(client.get_bridge_fee(), 100i128);
}

#[test]
fn test_set_min_validators() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());

    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);

    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    client.initialize(&token, &admin, &2, &fee_recipient);

    env.mock_all_auths();
    client.set_min_validators(&3);
    // Verify by attempting complete_bridge with insufficient validators
}
