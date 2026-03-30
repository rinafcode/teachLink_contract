#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::too_many_lines)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    vec, Address, Bytes, Env, Vec, Symbol, Map,
};

use teachlink_contract::{
    BridgeError, BridgeTransaction, BridgeParameters, TeachLinkBridge, TeachLinkBridgeClient,
    ValidatorSignature, ChainConfiguration,
};

fn create_bridge_params(
    env: &Env,
    from_chain: u32,
    to_chain: u32,
    token: Address,
    amount: i128,
    recipient: Address,
    fee: i128,
) -> BridgeParameters {
    BridgeParameters {
        from_chain,
        to_chain,
        token,
        amount,
        recipient,
        fee,
        nonce: 12345,
        timeout: 1000,
    }
}

fn create_validator_signatures(env: &Env, count: u32) -> Vec<ValidatorSignature> {
    let mut signatures = Vec::new(env);
    for i in 0..count {
        let validator = Address::generate(env);
        let signature = Bytes::from_slice(env, &format!("signature_{}", i).as_bytes());
        signatures.push_back(ValidatorSignature {
            validator,
            signature,
        });
    }
    signatures
}

#[test]
fn test_bridge_initialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let min_validators = 3;

    // Test successful initialization
    client.initialize(&admin, &min_validators);

    // Test double initialization
    let result = client.try_initialize(&admin, &min_validators);
    assert_eq!(result.error(), Some(Ok(BridgeError::AlreadyInitialized)));
}

#[test]
fn test_bridge_transaction_validation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let token = Address::generate(&env);
    
    client.initialize(&admin, &3);

    // Test amount must be positive
    let params = create_bridge_params(&env, 1, 2, token.clone(), 0, user.clone(), 100);
    let result = client.try_initiate_bridge(&params);
    assert_eq!(result.error(), Some(Ok(BridgeError::AmountMustBePositive)));

    // Test fee cannot be negative
    let params = create_bridge_params(&env, 1, 2, token.clone(), 1000, user.clone(), -1);
    let result = client.try_initiate_bridge(&params);
    assert_eq!(result.error(), Some(Ok(BridgeError::FeeCannotBeNegative)));

    // Test successful bridge initiation
    let params = create_bridge_params(&env, 1, 2, token, 1000, user, 100);
    let tx_id = client.initiate_bridge(&params);
    assert!(tx_id > 0);
}

#[test]
fn test_validator_signature_validation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let token = Address::generate(&env);
    
    client.initialize(&admin, &3);

    let params = create_bridge_params(&env, 1, 2, token, 1000, user, 100);
    let tx_id = client.initiate_bridge(&params);

    // Test insufficient validator signatures
    let signatures = create_validator_signatures(&env, 2); // Less than required 3
    let result = client.try_complete_bridge(&tx_id, &signatures);
    assert_eq!(result.error(), Some(Ok(BridgeError::InsufficientValidatorSignatures)));

    // Test sufficient signatures
    let signatures = create_validator_signatures(&env, 3);
    client.complete_bridge(&tx_id, &signatures);
}

#[test]
fn test_chain_configuration() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &3);

    // Test adding chain configuration
    let chain_config = ChainConfiguration {
        chain_id: 1,
        chain_name: Bytes::from_slice(&env, b"Ethereum"),
        is_active: true,
        min_confirmations: 6,
        block_time: 12,
    };

    client.add_chain_configuration(&chain_config);

    // Test adding duplicate chain
    let result = client.try_add_chain_configuration(&chain_config);
    assert_eq!(result.error(), Some(Ok(BridgeError::InvalidChainConfiguration)));

    // Test pausing chain
    client.pause_chain(&1);
    
    // Test bridge to paused chain
    let user = Address::generate(&env);
    let token = Address::generate(&env);
    let params = create_bridge_params(&env, 1, 2, token, 1000, user, 100);
    let result = client.try_initiate_bridge(&params);
    assert_eq!(result.error(), Some(Ok(BridgeError::ChainPaused)));
}

#[test]
fn test_nonce_handling() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let token = Address::generate(&env);
    
    client.initialize(&admin, &3);

    // First transaction with nonce
    let params = create_bridge_params(&env, 1, 2, token.clone(), 1000, user.clone(), 100);
    let tx_id = client.initiate_bridge(&params);

    // Attempt to use same nonce again
    let params2 = create_bridge_params(&env, 1, 2, token, 2000, user, 100);
    let result = client.try_initiate_bridge(&params2);
    assert_eq!(result.error(), Some(Ok(BridgeError::NonceAlreadyProcessed)));
}

#[test]
fn test_emergency_controls() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let token = Address::generate(&env);
    
    client.initialize(&admin, &3);

    // Test emergency pause
    client.emergency_pause();

    // Test operations during pause
    let params = create_bridge_params(&env, 1, 2, token, 1000, user, 100);
    let result = client.try_initiate_bridge(&params);
    assert_eq!(result.error(), Some(Ok(BridgeError::BridgePaused)));

    // Test unauthorized pause
    let unauthorized_user = Address::generate(&env);
    let result = client.try_emergency_pause_auth(&unauthorized_user);
    assert_eq!(result.error(), Some(Ok(BridgeError::UnauthorizedPause)));

    // Test resume
    client.emergency_resume();
    let tx_id = client.initiate_bridge(&params);
    assert!(tx_id > 0);
}

#[test]
fn test_bridge_transaction_limits() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let token = Address::generate(&env);
    
    client.initialize(&admin, &3);

    // Test very large amount (should succeed if within bounds)
    let large_amount = i128::MAX / 2;
    let params = create_bridge_params(&env, 1, 2, token.clone(), large_amount, user.clone(), 100);
    let tx_id = client.initiate_bridge(&params);
    assert!(tx_id > 0);

    // Test amount overflow (should fail)
    let overflow_amount = i128::MAX;
    let params = create_bridge_params(&env, 1, 2, token, overflow_amount, user, 100);
    let result = client.try_initiate_bridge(&params);
    // This should trigger some form of overflow error
    assert!(result.error().is_some());
}

#[test]
fn test_bridge_transaction_query() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let token = Address::generate(&env);
    
    client.initialize(&admin, &3);

    // Create a transaction
    let params = create_bridge_params(&env, 1, 2, token.clone(), 1000, user.clone(), 100);
    let tx_id = client.initiate_bridge(&params);

    // Query transaction
    let tx = client.get_bridge_transaction(&tx_id);
    assert_eq!(tx.amount, 1000);
    assert_eq!(tx.from_chain, 1);
    assert_eq!(tx.to_chain, 2);

    // Query non-existent transaction
    let result = client.try_get_bridge_transaction(&999999);
    assert_eq!(result.error(), Some(Ok(BridgeError::BridgeTransactionNotFound)));
}
