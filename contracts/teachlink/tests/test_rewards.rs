#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]

use soroban_sdk::{
    contract, contractimpl, symbol_short, testutils::Address as _, Address, Env, Map, String,
};

use teachlink_contract::{RewardsError, TeachLinkBridge, TeachLinkBridgeClient, UserReward};

// ========== Mock Token ==========

#[contract]
pub struct TestToken;

#[contractimpl]
impl TestToken {
    pub fn initialize(env: Env, admin: Address) {
        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &admin);
        let balances: Map<Address, i128> = Map::new(&env);
        env.storage()
            .instance()
            .set(&symbol_short!("balances"), &balances);
    }

    pub fn balance(env: Env, address: Address) -> i128 {
        let balances: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&symbol_short!("balances"))
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
            .get(&symbol_short!("balances"))
            .unwrap_or_else(|| Map::new(&env));
        let current = balances.get(to.clone()).unwrap_or(0);
        balances.set(to, current + amount);
        env.storage()
            .instance()
            .set(&symbol_short!("balances"), &balances);
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        let mut balances: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&symbol_short!("balances"))
            .unwrap_or_else(|| Map::new(&env));
        let from_bal = balances.get(from.clone()).unwrap_or(0);
        let to_bal = balances.get(to.clone()).unwrap_or(0);
        assert!(from_bal >= amount, "Insufficient balance");
        balances.set(from, from_bal - amount);
        balances.set(to, to_bal + amount);
        env.storage()
            .instance()
            .set(&symbol_short!("balances"), &balances);
    }
}

// ========== Helpers ==========

struct Setup {
    env: Env,
    client: TeachLinkBridgeClient<'static>,
    token_client: TestTokenClient<'static>,
    token_id: Address,
    admin: Address,
    rewards_admin: Address,
    contract_id: Address,
}

fn setup() -> Setup {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let token_id = env.register(TestToken, ());
    let token_client = TestTokenClient::new(&env, &token_id);

    let token_admin = Address::generate(&env);
    let admin = Address::generate(&env);
    let rewards_admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);

    token_client.initialize(&token_admin);

    client.initialize(&token_id, &admin, &1, &fee_recipient);
    client.initialize_rewards(&token_id, &rewards_admin);

    Setup {
        env,
        client,
        token_client,
        token_id,
        admin,
        rewards_admin,
        contract_id,
    }
}

// ========== Initialization Tests ==========

#[test]
fn test_initialize_rewards_sets_admin() {
    let s = setup();
    assert_eq!(s.client.get_rewards_admin(), s.rewards_admin);
}

#[test]
fn test_initialize_rewards_pool_starts_at_zero() {
    let s = setup();
    assert_eq!(s.client.get_reward_pool_balance(), 0);
}

#[test]
fn test_initialize_rewards_total_issued_starts_at_zero() {
    let s = setup();
    assert_eq!(s.client.get_total_rewards_issued(), 0);
}

#[test]
fn test_initialize_rewards_twice_fails() {
    let s = setup();
    let result = s
        .client
        .try_initialize_rewards(&s.token_id, &s.rewards_admin);
    assert_eq!(result, Err(Ok(RewardsError::AlreadyInitialized)));
}

// ========== Fund Reward Pool Tests ==========

#[test]
fn test_fund_reward_pool() {
    let s = setup();
    let funder = Address::generate(&s.env);
    s.token_client.mint(&funder, &1000);

    s.client.fund_reward_pool(&funder, &500);

    assert_eq!(s.client.get_reward_pool_balance(), 500);
    assert_eq!(s.token_client.balance(&funder), 500);
    assert_eq!(s.token_client.balance(&s.contract_id), 500);
}

#[test]
fn test_fund_reward_pool_multiple_times() {
    let s = setup();
    let funder = Address::generate(&s.env);
    s.token_client.mint(&funder, &1000);

    s.client.fund_reward_pool(&funder, &300);
    s.client.fund_reward_pool(&funder, &200);

    assert_eq!(s.client.get_reward_pool_balance(), 500);
}

// ========== Issue Reward Tests ==========

#[test]
fn test_issue_reward() {
    let s = setup();
    let funder = Address::generate(&s.env);
    let recipient = Address::generate(&s.env);
    s.token_client.mint(&funder, &1000);
    s.client.fund_reward_pool(&funder, &1000);

    let reward_type = String::from_str(&s.env, "course_completion");
    s.client.issue_reward(&recipient, &100, &reward_type);

    let user_reward: UserReward = s.client.get_user_rewards(&recipient).unwrap();
    assert_eq!(user_reward.total_earned, 100);
    assert_eq!(user_reward.pending, 100);
    assert_eq!(user_reward.claimed, 0);
    assert_eq!(s.client.get_total_rewards_issued(), 100);
}

#[test]
fn test_issue_reward_accumulates() {
    let s = setup();
    let funder = Address::generate(&s.env);
    let recipient = Address::generate(&s.env);
    s.token_client.mint(&funder, &1000);
    s.client.fund_reward_pool(&funder, &1000);

    let rt = String::from_str(&s.env, "completion");
    s.client.issue_reward(&recipient, &100, &rt);
    s.client.issue_reward(&recipient, &200, &rt);

    let user_reward = s.client.get_user_rewards(&recipient).unwrap();
    assert_eq!(user_reward.total_earned, 300);
    assert_eq!(user_reward.pending, 300);
    assert_eq!(s.client.get_total_rewards_issued(), 300);
}

#[test]
fn test_issue_reward_insufficient_pool() {
    let s = setup();
    let recipient = Address::generate(&s.env);
    let rt = String::from_str(&s.env, "completion");

    // Pool is empty
    let result = s.client.try_issue_reward(&recipient, &100, &rt);
    assert_eq!(result, Err(Ok(RewardsError::InsufficientRewardPoolBalance)));
}

// ========== Claim Rewards Tests ==========

#[test]
fn test_claim_rewards() {
    let s = setup();
    let funder = Address::generate(&s.env);
    let recipient = Address::generate(&s.env);
    s.token_client.mint(&funder, &1000);
    s.client.fund_reward_pool(&funder, &1000);

    let rt = String::from_str(&s.env, "completion");
    s.client.issue_reward(&recipient, &250, &rt);
    s.client.claim_rewards(&recipient);

    let user_reward = s.client.get_user_rewards(&recipient).unwrap();
    assert_eq!(user_reward.claimed, 250);
    assert_eq!(user_reward.pending, 0);
    assert_eq!(s.token_client.balance(&recipient), 250);
    assert_eq!(s.client.get_reward_pool_balance(), 750);
}

#[test]
fn test_claim_rewards_no_rewards() {
    let s = setup();
    let user = Address::generate(&s.env);
    let result = s.client.try_claim_rewards(&user);
    assert_eq!(result, Err(Ok(RewardsError::NoRewardsAvailable)));
}

#[test]
fn test_claim_rewards_already_claimed() {
    let s = setup();
    let funder = Address::generate(&s.env);
    let recipient = Address::generate(&s.env);
    s.token_client.mint(&funder, &1000);
    s.client.fund_reward_pool(&funder, &1000);

    let rt = String::from_str(&s.env, "completion");
    s.client.issue_reward(&recipient, &100, &rt);
    s.client.claim_rewards(&recipient);

    // Second claim should fail — no pending rewards
    let result = s.client.try_claim_rewards(&recipient);
    assert_eq!(result, Err(Ok(RewardsError::NoPendingRewards)));
}

// ========== Set Reward Rate Tests ==========

#[test]
fn test_set_and_get_reward_rate() {
    let s = setup();
    let rt = String::from_str(&s.env, "teaching");
    s.client.set_reward_rate(&rt, &50, &true);

    let rate = s.client.get_reward_rate(&rt).unwrap();
    assert_eq!(rate.rate, 50);
    assert_eq!(rate.enabled, true);
}

#[test]
fn test_set_reward_rate_negative_fails() {
    let s = setup();
    let rt = String::from_str(&s.env, "teaching");
    let result = s.client.try_set_reward_rate(&rt, &-1, &true);
    assert_eq!(result, Err(Ok(RewardsError::RateCannotBeNegative)));
}

#[test]
fn test_get_reward_rate_nonexistent() {
    let s = setup();
    let rt = String::from_str(&s.env, "nonexistent");
    assert!(s.client.get_reward_rate(&rt).is_none());
}

// ========== Admin Tests ==========

#[test]
fn test_update_rewards_admin() {
    let s = setup();
    let new_admin = Address::generate(&s.env);
    s.client.update_rewards_admin(&new_admin);
    assert_eq!(s.client.get_rewards_admin(), new_admin);
}

// ========== View Function Tests ==========

#[test]
fn test_get_user_rewards_nonexistent() {
    let s = setup();
    let user = Address::generate(&s.env);
    assert!(s.client.get_user_rewards(&user).is_none());
}
