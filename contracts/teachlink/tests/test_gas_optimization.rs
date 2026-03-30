#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]

//! Tests verifying the gas-optimized storage patterns introduced in #161.
//!
//! The key optimizations verified here:
//! - Escrows are stored per-key in persistent storage (not as a single Map blob).
//! - User rewards and reward rates are stored per-key in persistent storage.
//!
//! Correct behaviour after the refactor is verified by:
//! 1. Multiple escrows can be created and retrieved independently.
//! 2. Updating one escrow does not affect another.
//! 3. User rewards are isolated per address.
//! 4. Reward rates are isolated per reward type.

use soroban_sdk::{
    contract, contractimpl, symbol_short, testutils::Address as _, Address, Bytes, Env, Map, Vec,
};

use teachlink_contract::{
    EscrowParameters, EscrowRole, EscrowSigner, EscrowStatus, TeachLinkBridge,
    TeachLinkBridgeClient,
};

// ---------------------------------------------------------------------------
// Minimal test token contract
// ---------------------------------------------------------------------------

#[contract]
pub struct GasTestToken;

#[contractimpl]
impl GasTestToken {
    pub fn initialize(env: Env, admin: Address) {
        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &admin);
        let balances: Map<Address, i128> = Map::new(&env);
        env.storage()
            .instance()
            .set(&symbol_short!("bals"), &balances);
    }

    pub fn balance(env: Env, address: Address) -> i128 {
        let balances: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&symbol_short!("bals"))
            .unwrap_or_else(|| Map::new(&env));
        balances.get(address).unwrap_or(0)
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .unwrap();
        admin.require_auth();
        let mut balances: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&symbol_short!("bals"))
            .unwrap_or_else(|| Map::new(&env));
        let cur = balances.get(to.clone()).unwrap_or(0);
        balances.set(to, cur + amount);
        env.storage()
            .instance()
            .set(&symbol_short!("bals"), &balances);
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        let mut balances: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&symbol_short!("bals"))
            .unwrap_or_else(|| Map::new(&env));
        let from_bal = balances.get(from.clone()).unwrap_or(0);
        assert!(from_bal >= amount, "Insufficient balance");
        let to_bal = balances.get(to.clone()).unwrap_or(0);
        balances.set(from, from_bal - amount);
        balances.set(to, to_bal + amount);
        env.storage()
            .instance()
            .set(&symbol_short!("bals"), &balances);
    }
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn make_signer(env: &Env, addr: Address) -> EscrowSigner {
    EscrowSigner {
        address: addr,
        role: EscrowRole::Primary,
        weight: 1,
    }
}

fn setup_env() -> (
    Env,
    TeachLinkBridgeClient<'static>,
    GasTestTokenClient<'static>,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let token_id = env.register(GasTestToken, ());
    let token = GasTestTokenClient::new(&env, &token_id);

    let admin = Address::generate(&env);
    token.initialize(&admin);

    // No insurance for these tests
    client.initialize_insurance_pool(&token_id, &0);

    (env, client, token, token_id)
}

// ---------------------------------------------------------------------------
// Escrow per-key storage tests
// ---------------------------------------------------------------------------

#[test]
fn test_two_escrows_stored_independently() {
    let (env, client, token, token_id) = setup_env();

    let depositor1 = Address::generate(&env);
    let depositor2 = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let arb = Address::generate(&env);

    token.mint(&depositor1, &1000);
    token.mint(&depositor2, &1000);

    let mut signers1 = Vec::new(&env);
    signers1.push_back(make_signer(&env, signer1.clone()));

    let mut signers2 = Vec::new(&env);
    signers2.push_back(make_signer(&env, signer2.clone()));

    let id1 = client.create_escrow(&EscrowParameters {
        depositor: depositor1.clone(),
        beneficiary: beneficiary.clone(),
        token: token_id.clone(),
        amount: 300,
        signers: signers1,
        threshold: 1,
        release_time: None,
        refund_time: None,
        arbitrator: arb.clone(),
    });

    let id2 = client.create_escrow(&EscrowParameters {
        depositor: depositor2.clone(),
        beneficiary: beneficiary.clone(),
        token: token_id.clone(),
        amount: 500,
        signers: signers2,
        threshold: 1,
        release_time: None,
        refund_time: None,
        arbitrator: arb.clone(),
    });

    // IDs are distinct
    assert_ne!(id1, id2);

    // Each escrow has its own amount stored correctly
    let e1 = client.get_escrow(&id1).unwrap();
    let e2 = client.get_escrow(&id2).unwrap();
    assert_eq!(e1.amount, 300);
    assert_eq!(e2.amount, 500);
    assert_eq!(e1.depositor, depositor1);
    assert_eq!(e2.depositor, depositor2);
    assert_eq!(e1.status, EscrowStatus::Pending);
    assert_eq!(e2.status, EscrowStatus::Pending);
}

#[test]
fn test_releasing_one_escrow_does_not_affect_another() {
    let (env, client, token, token_id) = setup_env();

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let signer_a = Address::generate(&env);
    let signer_b = Address::generate(&env);
    let arb = Address::generate(&env);

    token.mint(&depositor, &2000);

    let mut signers_a = Vec::new(&env);
    signers_a.push_back(make_signer(&env, signer_a.clone()));

    let mut signers_b = Vec::new(&env);
    signers_b.push_back(make_signer(&env, signer_b.clone()));

    let id_a = client.create_escrow(&EscrowParameters {
        depositor: depositor.clone(),
        beneficiary: beneficiary.clone(),
        token: token_id.clone(),
        amount: 400,
        signers: signers_a,
        threshold: 1,
        release_time: None,
        refund_time: None,
        arbitrator: arb.clone(),
    });

    let id_b = client.create_escrow(&EscrowParameters {
        depositor: depositor.clone(),
        beneficiary: beneficiary.clone(),
        token: token_id.clone(),
        amount: 600,
        signers: signers_b,
        threshold: 1,
        release_time: None,
        refund_time: None,
        arbitrator: arb.clone(),
    });

    // Approve and release escrow A only
    client.approve_escrow_release(&id_a, &signer_a);
    client.release_escrow(&id_a, &signer_a);

    let ea = client.get_escrow(&id_a).unwrap();
    let eb = client.get_escrow(&id_b).unwrap();

    assert_eq!(ea.status, EscrowStatus::Released);
    // Escrow B must remain Pending — its storage slot is independent
    assert_eq!(eb.status, EscrowStatus::Pending);
}

// ---------------------------------------------------------------------------
// Rewards per-key storage tests
// ---------------------------------------------------------------------------

#[test]
fn test_reward_rates_stored_per_type() {
    let (env, client, token, token_id) = setup_env();

    let rewards_admin = Address::generate(&env);
    client.initialize_rewards(&token_id, &rewards_admin);

    let type_a = soroban_sdk::String::from_str(&env, "course_complete");
    let type_b = soroban_sdk::String::from_str(&env, "contribution");

    client.set_reward_rate(&type_a, &100, &true);
    client.set_reward_rate(&type_b, &250, &true);

    let rate_a = client.get_reward_rate(&type_a).unwrap();
    let rate_b = client.get_reward_rate(&type_b).unwrap();

    assert_eq!(rate_a.rate, 100);
    assert_eq!(rate_b.rate, 250);
}

#[test]
fn test_user_rewards_isolated_per_address() {
    let (env, client, token, token_id) = setup_env();

    let rewards_admin = Address::generate(&env);
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);
    let funder = Address::generate(&env);

    client.initialize_rewards(&token_id, &rewards_admin);

    // Fund the pool
    token.mint(&funder, &10_000);
    client.fund_reward_pool(&funder, &10_000);

    let rtype = soroban_sdk::String::from_str(&env, "test");

    client.issue_reward(&user_a, &300, &rtype);
    client.issue_reward(&user_b, &700, &rtype);

    let ra = client.get_user_rewards(&user_a).unwrap();
    let rb = client.get_user_rewards(&user_b).unwrap();

    // Each user's reward record is stored independently
    assert_eq!(ra.pending, 300);
    assert_eq!(rb.pending, 700);
    assert_eq!(ra.total_earned, 300);
    assert_eq!(rb.total_earned, 700);
}

#[test]
fn test_issuing_reward_does_not_affect_other_user() {
    let (env, client, token, token_id) = setup_env();

    let rewards_admin = Address::generate(&env);
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);
    let funder = Address::generate(&env);

    client.initialize_rewards(&token_id, &rewards_admin);
    token.mint(&funder, &10_000);
    client.fund_reward_pool(&funder, &10_000);

    let rtype = soroban_sdk::String::from_str(&env, "test");
    client.issue_reward(&user_a, &500, &rtype);

    // user_b has no rewards yet
    let rb = client.get_user_rewards(&user_b);
    assert!(rb.is_none());

    // Issuing more to user_a does not corrupt user_b
    client.issue_reward(&user_a, &200, &rtype);
    let ra = client.get_user_rewards(&user_a).unwrap();
    assert_eq!(ra.total_earned, 700);
    assert_eq!(ra.pending, 700);
}
