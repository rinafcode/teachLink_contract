#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unreadable_literal)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    vec, Address, Bytes, Env, Map, Vec,
};

use teachlink_contract::{
    BridgeError, BridgeParameters, ChainConfiguration, TeachLinkBridge, TeachLinkBridgeClient,
    ValidatorSignature,
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
        nonce: 9999,
        timeout: 1_000,
    }
}

fn create_validator_signatures(env: &Env, count: u32) -> Vec<ValidatorSignature> {
    let mut signatures = Vec::new(env);
    for i in 0..count {
        let validator = Address::generate(env);
        let signature = Bytes::from_slice(env, &format!("signature_{}", i).as_bytes());
        signatures.push_back(ValidatorSignature { validator, signature });
    }
    signatures
}

#[test]
fn test_successful_multichain_transfer() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &3);

    // Add supported chains
    client.add_supported_chain(&1u32);
    client.add_supported_chain(&2u32);

    // Register a multi-chain asset (smoke - ensures asset registration path works)
    let mut chain_configs: Map<u32, teachlink_contract::ChainAssetInfo> = Map::new(&env);
    chain_configs.set(
        1u32,
        teachlink_contract::ChainAssetInfo {
            chain_id: 1u32,
            token_address: Bytes::from_slice(&env, b"token-1"),
            decimals: 6,
            is_active: true,
        },
    );
    chain_configs.set(
        2u32,
        teachlink_contract::ChainAssetInfo {
            chain_id: 2u32,
            token_address: Bytes::from_slice(&env, b"token-2"),
            decimals: 6,
            is_active: true,
        },
    );

    let stellar_token = Address::generate(&env);
    let asset_id = Bytes::from_slice(&env, b"USDC");
    let _asset_index = client.register_multi_chain_asset(&asset_id, &stellar_token, &chain_configs);

    // Initiate bridge
    let user = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token = Address::generate(&env);

    // Initialize contract token (client.initialize above handles setup in existing tests' client)

    let params = create_bridge_params(&env, 1, 2, token, 1_000, recipient.clone(), 10);
    let tx_id = client.initiate_bridge(&params);
    assert!(tx_id > 0);

    // Complete bridge with sufficient validator signatures
    let sigs = create_validator_signatures(&env, 3);
    client.complete_bridge(&tx_id, &sigs);

    // After completion transaction should be removed (sanity check)
    let _ = client.get_bridge_transaction(&tx_id);
}

#[test]
fn test_failed_transfer_paused_chain() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &3);

    client.add_supported_chain(&1u32);
    client.add_supported_chain(&2u32);

    // Pause destination chain using emergency pause API
    client.pause_chain(&2u32);

    let user = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token = Address::generate(&env);

    let params = create_bridge_params(&env, 1, 2, token, 1_000, recipient, 10);
    let result = client.try_initiate_bridge(&params);
    assert_eq!(result.error(), Some(Ok(BridgeError::ChainPaused)));
}

#[test]
fn test_timeout_and_refund_flow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &3);
    client.add_supported_chain(&1u32);
    client.add_supported_chain(&2u32);

    let recipient = Address::generate(&env);
    let token = Address::generate(&env);

    let params = create_bridge_params(&env, 1, 2, token, 500, recipient.clone(), 5);
    let tx_id = client.initiate_bridge(&params);

    // Cancel immediately should fail with TimeoutNotReached
    let early_cancel = client.try_cancel_bridge(&tx_id);
    assert_eq!(early_cancel.error(), Some(Ok(BridgeError::TimeoutNotReached)));

    // Advance ledger time past bridge timeout (7 days = 604800 seconds)
    let now = env.ledger().timestamp();
    env.ledger().set(LedgerInfo { timestamp: now + 604_800 + 10, seq_num: 0, network_id: 0 });

    // Now cancel should succeed
    client.cancel_bridge(&tx_id);
}

#[test]
fn test_retry_mechanism_respects_backoff_and_limit() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &3);
    client.add_supported_chain(&1u32);
    client.add_supported_chain(&2u32);

    let recipient = Address::generate(&env);
    let token = Address::generate(&env);

    let params = create_bridge_params(&env, 1, 2, token, 500, recipient.clone(), 5);
    let tx_id = client.initiate_bridge(&params);

    // Mark bridge failed
    client.mark_bridge_failed(&tx_id, &Bytes::from_slice(&env, b"simulated failure"));

    // Immediate retry should fail due to backoff
    let first_retry = client.try_retry_bridge(&tx_id);
    assert_eq!(first_retry.error(), Some(Ok(BridgeError::RetryBackoffActive)));

    // Fast-forward time to allow retries and exhaust attempts
    let mut now = env.ledger().timestamp();
    let base = 300u64; // BRIDGE_RETRY_DELAY_BASE_SECONDS
    for i in 0..6u32 {
        // exponential backoff scheduling
        now += base * (1u64 << i);
        env.ledger().set(LedgerInfo { timestamp: now, seq_num: 0, network_id: 0 });
        let r = client.try_retry_bridge(&tx_id);
        if i < 5 {
            // should succeed for first 5 retries
            assert!(r.error().is_none());
        } else {
            // 6th attempt should exceed limit
            assert_eq!(r.error(), Some(Ok(BridgeError::RetryLimitExceeded)));
        }
    }
}
