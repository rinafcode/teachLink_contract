#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]

//! Atomic swap tests using internal module calls (swaps require token transfers
//! which need a real token contract; we test validation and state logic directly).

use soroban_sdk::{testutils::Address as _, testutils::Ledger, Address, Bytes, Env};
use teachlink_contract::{BridgeError, SwapStatus, TeachLinkBridge};

fn setup() -> (Env, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TeachLinkBridge, ());
    (env, contract_id)
}

fn set_time(env: &Env, ts: u64) {
    env.ledger().with_mut(|l| l.timestamp = ts);
}

// ── View Functions on Empty State ──────────────────────────────────

#[test]
fn test_get_nonexistent_swap_returns_none() {
    let (env, cid) = setup();
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    assert!(client.get_atomic_swap(&1).is_none());
}

#[test]
fn test_get_active_swaps_empty() {
    let (env, cid) = setup();
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    let active = client.get_active_atomic_swaps();
    assert_eq!(active.len(), 0);
}

// ── Initiate Swap Validation ───────────────────────────────────────

#[test]
fn test_initiate_swap_zero_amount_fails() {
    let (env, cid) = setup();
    set_time(&env, 1000);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    let initiator = Address::generate(&env);
    let counterparty = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    let hashlock = Bytes::from_slice(&env, &[0xaa; 32]);

    let result = client.try_initiate_atomic_swap(
        &initiator,
        &token_a,
        &0i128,
        &counterparty,
        &token_b,
        &100i128,
        &hashlock,
        &3600u64,
    );
    assert!(result.is_err());
}

#[test]
fn test_initiate_swap_invalid_hashlock_length_fails() {
    let (env, cid) = setup();
    set_time(&env, 1000);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    let initiator = Address::generate(&env);
    let counterparty = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    let bad_hashlock = Bytes::from_slice(&env, &[0xaa; 16]); // wrong length

    let result = client.try_initiate_atomic_swap(
        &initiator,
        &token_a,
        &100i128,
        &counterparty,
        &token_b,
        &100i128,
        &bad_hashlock,
        &3600u64,
    );
    assert!(result.is_err());
}

#[test]
fn test_initiate_swap_self_swap_fails() {
    let (env, cid) = setup();
    set_time(&env, 1000);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    let user = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    let hashlock = Bytes::from_slice(&env, &[0xaa; 32]);

    let result = client.try_initiate_atomic_swap(
        &user, &token_a, &100i128, &user, // same as initiator
        &token_b, &100i128, &hashlock, &3600u64,
    );
    assert!(result.is_err());
}

#[test]
fn test_initiate_swap_invalid_timelock_fails() {
    let (env, cid) = setup();
    set_time(&env, 1000);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    let initiator = Address::generate(&env);
    let counterparty = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    let hashlock = Bytes::from_slice(&env, &[0xaa; 32]);

    // Too short (< 1 hour)
    let result = client.try_initiate_atomic_swap(
        &initiator,
        &token_a,
        &100i128,
        &counterparty,
        &token_b,
        &100i128,
        &hashlock,
        &60u64,
    );
    assert!(result.is_err());

    // Too long (> 7 days)
    let result = client.try_initiate_atomic_swap(
        &initiator,
        &token_a,
        &100i128,
        &counterparty,
        &token_b,
        &100i128,
        &hashlock,
        &700_000u64,
    );
    assert!(result.is_err());
}

// ── Accept / Refund on Nonexistent Swap ────────────────────────────

#[test]
fn test_accept_nonexistent_swap_fails() {
    let (env, cid) = setup();
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    let counterparty = Address::generate(&env);
    let preimage = Bytes::from_slice(&env, b"secret");

    let result = client.try_accept_atomic_swap(&999, &counterparty, &preimage);
    assert!(result.is_err());
}

#[test]
fn test_refund_nonexistent_swap_fails() {
    let (env, cid) = setup();
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    let initiator = Address::generate(&env);
    let result = client.try_refund_atomic_swap(&999, &initiator);
    assert!(result.is_err());
}
