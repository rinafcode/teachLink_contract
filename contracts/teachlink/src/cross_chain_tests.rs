use crate::bridge::Bridge;
use crate::errors::BridgeError;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_bridge_initialization_and_access_control() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let min_validators = 2;

    // Test Initialization
    let result = Bridge::initialize(&env, token.clone(), admin.clone(), min_validators, fee_recipient.clone());
    assert!(result.is_ok());

    // Test Double Initialization (Should Fail)
    let result_fail = Bridge::initialize(&env, token, admin, min_validators, fee_recipient);
    assert_eq!(result_fail.unwrap_err(), BridgeError::AlreadyInitialized);
}
