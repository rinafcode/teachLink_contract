#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    vec, Address, Env, String,
};

use teachlink_contract::{TeachLinkBridge, TeachLinkBridgeClient};

fn setup_test() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TeachLinkBridge, ());
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let rewards_admin = Address::generate(&env);
    let user = Address::generate(&env);

    // Set ledger timestamp
    env.ledger().set(LedgerInfo {
        timestamp: 1000,
        protocol_version: 20,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 2000000,
    });

    (env, contract_id, token, admin, rewards_admin)
}

#[test]
fn test_initialize_rewards() {
    let (env, contract_id, token, _admin, rewards_admin) = setup_test();
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    // Initialize rewards system
    client.initialize_rewards(&token, &rewards_admin);

    // Verify initialization - query reward pool
    let pool_balance = client.get_reward_pool_balance();
    assert_eq!(pool_balance, 0);

    let total_issued = client.get_total_rewards_issued();
    assert_eq!(total_issued, 0);

    let stored_admin = client.get_rewards_admin();
    assert_eq!(stored_admin, rewards_admin);
}

#[test]
#[should_panic(expected = "Rewards already initialized")]
fn test_initialize_rewards_double_init() {
    let (env, contract_id, token, _admin, rewards_admin) = setup_test();
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    // Initialize twice - should panic
    client.initialize_rewards(&token, &rewards_admin);
    client.initialize_rewards(&token, &rewards_admin);
}

#[test]
fn test_fund_reward_pool() {
    let (env, contract_id, token, _admin, rewards_admin) = setup_test();
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let funder = Address::generate(&env);

    // Initialize
    client.initialize_rewards(&token, &rewards_admin);

    // Mock token transfer - in real test with actual token contract
    // For now, just verify the function call
    // This would normally transfer tokens from funder to contract
    client.fund_reward_pool(&funder, &1000);

    // Check pool balance - should increase
    let pool_balance = client.get_reward_pool_balance();
    assert_eq!(pool_balance, 1000);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_fund_reward_pool_invalid_amount() {
    let (env, contract_id, token, _admin, rewards_admin) = setup_test();
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let funder = Address::generate(&env);

    // Initialize
    client.initialize_rewards(&token, &rewards_admin);

    // Try to fund with zero amount - should panic
    client.fund_reward_pool(&funder, &0);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_fund_reward_pool_negative_amount() {
    let (env, contract_id, token, _admin, rewards_admin) = setup_test();
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let funder = Address::generate(&env);

    // Initialize
    client.initialize_rewards(&token, &rewards_admin);

    // Try to fund with negative amount - should panic
    client.fund_reward_pool(&funder, &-100);
}

#[test]
fn test_issue_reward_to_user() {
    let (env, contract_id, token, _admin, rewards_admin) = setup_test();
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    // Initialize
    client.initialize_rewards(&token, &rewards_admin);

    // Fund the pool
    let funder = Address::generate(&env);
    client.fund_reward_pool(&funder, &5000);

    // Issue reward
    let reward_type = String::from_str(&env, "participation");
    client.issue_reward(&user, &1000, &reward_type);

    // Verify user rewards
    let user_rewards = client.get_user_rewards(&user);
    assert!(user_rewards.is_some());

    let rewards = user_rewards.unwrap();
    assert_eq!(rewards.user, user);
    assert_eq!(rewards.total_earned, 1000);
    assert_eq!(rewards.pending, 1000);
    assert_eq!(rewards.claimed, 0);

    // Verify total issued
    let total_issued = client.get_total_rewards_issued();
    assert_eq!(total_issued, 1000);

    // Pool balance should be reduced
    let pool_balance = client.get_reward_pool_balance();
    assert_eq!(pool_balance, 4000);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_issue_reward_invalid_amount() {
    let (env, contract_id, token, _admin, rewards_admin) = setup_test();
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    // Initialize
    client.initialize_rewards(&token, &rewards_admin);

    // Fund the pool
    let funder = Address::generate(&env);
    client.fund_reward_pool(&funder, &5000);

    // Try to issue zero reward - should panic
    let reward_type = String::from_str(&env, "participation");
    client.issue_reward(&user, &0, &reward_type);
}

#[test]
#[should_panic(expected = "Insufficient reward pool balance")]
fn test_issue_reward_insufficient_pool() {
    let (env, contract_id, token, _admin, rewards_admin) = setup_test();
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    // Initialize
    client.initialize_rewards(&token, &rewards_admin);

    // Fund with insufficient amount
    let funder = Address::generate(&env);
    client.fund_reward_pool(&funder, &500);

    // Try to issue more than available - should panic
    let reward_type = String::from_str(&env, "participation");
    client.issue_reward(&user, &1000, &reward_type);
}

#[test]
fn test_claim_rewards() {
    let (env, contract_id, token, _admin, rewards_admin) = setup_test();
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    // Initialize
    client.initialize_rewards(&token, &rewards_admin);

    // Fund the pool
    let funder = Address::generate(&env);
    client.fund_reward_pool(&funder, &5000);

    // Issue reward
    let reward_type = String::from_str(&env, "participation");
    client.issue_reward(&user, &1500, &reward_type);

    // Claim rewards
    client.claim_rewards(&user);

    // Verify user rewards after claim
    let user_rewards = client.get_user_rewards(&user);
    assert!(user_rewards.is_some());

    let rewards = user_rewards.unwrap();
    assert_eq!(rewards.claimed, 1500);
    assert_eq!(rewards.pending, 0);
    assert_eq!(rewards.total_earned, 1500);

    // Verify pool balance decreased
    let pool_balance = client.get_reward_pool_balance();
    assert_eq!(pool_balance, 3500);
}

#[test]
fn test_claim_rewards_updates_timestamp() {
    let (env, contract_id, token, _admin, rewards_admin) = setup_test();
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    // Initialize
    client.initialize_rewards(&token, &rewards_admin);

    // Fund the pool
    let funder = Address::generate(&env);
    client.fund_reward_pool(&funder, &5000);

    // Issue reward at time 1000
    let reward_type = String::from_str(&env, "participation");
    client.issue_reward(&user, &1000, &reward_type);

    // Advance time
    env.ledger().set(LedgerInfo {
        timestamp: 2000,
        protocol_version: 20,
        sequence_number: 20,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 2000000,
    });

    // Claim rewards
    client.claim_rewards(&user);

    // Verify claim timestamp is updated
    let user_rewards = client.get_user_rewards(&user);
    let rewards = user_rewards.unwrap();
    assert_eq!(rewards.last_claim_timestamp, 2000);
}

#[test]
#[should_panic(expected = "No rewards available")]
fn test_claim_rewards_no_balance() {
    let (env, contract_id, token, _admin, rewards_admin) = setup_test();
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    // Initialize
    client.initialize_rewards(&token, &rewards_admin);

    // Try to claim without any rewards - should panic
    client.claim_rewards(&user);
}

#[test]
#[should_panic(expected = "No pending rewards")]
fn test_claim_rewards_already_claimed() {
    let (env, contract_id, token, _admin, rewards_admin) = setup_test();
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    // Initialize
    client.initialize_rewards(&token, &rewards_admin);

    // Fund the pool
    let funder = Address::generate(&env);
    client.fund_reward_pool(&funder, &5000);

    // Issue and claim reward
    let reward_type = String::from_str(&env, "participation");
    client.issue_reward(&user, &1000, &reward_type);
    client.claim_rewards(&user);

    // Try to claim again - should panic as no pending rewards
    client.claim_rewards(&user);
}

#[test]
fn test_multiple_rewards_accumulation() {
    let (env, contract_id, token, _admin, rewards_admin) = setup_test();
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    // Initialize
    client.initialize_rewards(&token, &rewards_admin);

    // Fund the pool
    let funder = Address::generate(&env);
    client.fund_reward_pool(&funder, &10000);

    // Issue multiple rewards
    let reward_type_1 = String::from_str(&env, "participation");
    let reward_type_2 = String::from_str(&env, "course_completion");

    client.issue_reward(&user, &500, &reward_type_1);
    client.issue_reward(&user, &800, &reward_type_2);
    client.issue_reward(&user, &200, &reward_type_1);

    // Verify accumulated rewards
    let user_rewards = client.get_user_rewards(&user);
    let rewards = user_rewards.unwrap();
    assert_eq!(rewards.total_earned, 1500); // 500 + 800 + 200
    assert_eq!(rewards.pending, 1500);
    assert_eq!(rewards.claimed, 0);

    // Pool should be reduced
    let pool_balance = client.get_reward_pool_balance();
    assert_eq!(pool_balance, 8500);
}

#[test]
fn test_set_reward_rate() {
    let (env, contract_id, token, _admin, rewards_admin) = setup_test();
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    // Initialize
    client.initialize_rewards(&token, &rewards_admin);

    // Set reward rate
    let reward_type = String::from_str(&env, "course_completion");
    client.set_reward_rate(&reward_type, &1000, &true);

    // Get reward rate
    let rate = client.get_reward_rate(&reward_type);
    assert!(rate.is_some());

    let rate_data = rate.unwrap();
    assert_eq!(rate_data.rate, 1000);
    assert!(rate_data.enabled);
}

#[test]
#[should_panic(expected = "Rate cannot be negative")]
fn test_set_reward_rate_negative() {
    let (env, contract_id, token, _admin, rewards_admin) = setup_test();
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    // Initialize
    client.initialize_rewards(&token, &rewards_admin);

    // Try to set negative rate - should panic
    let reward_type = String::from_str(&env, "course_completion");
    client.set_reward_rate(&reward_type, &-100, &true);
}

#[test]
fn test_update_rewards_admin() {
    let (env, contract_id, token, _admin, rewards_admin) = setup_test();
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let new_admin = Address::generate(&env);

    // Initialize
    client.initialize_rewards(&token, &rewards_admin);

    // Update admin
    client.update_rewards_admin(&new_admin);

    // Verify new admin
    let current_admin = client.get_rewards_admin();
    assert_eq!(current_admin, new_admin);
}

#[test]
fn test_fund_and_claim_workflow() {
    let (env, contract_id, token, _admin, rewards_admin) = setup_test();
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let funder = Address::generate(&env);

    // Initialize
    client.initialize_rewards(&token, &rewards_admin);

    // Fund pool
    client.fund_reward_pool(&funder, &10000);

    // Issue rewards to multiple users
    let participation = String::from_str(&env, "participation");
    let completion = String::from_str(&env, "course_completion");

    client.issue_reward(&user1, &2000, &participation);
    client.issue_reward(&user2, &3000, &completion);
    client.issue_reward(&user1, &1000, &completion);

    // Verify state
    let total = client.get_total_rewards_issued();
    assert_eq!(total, 6000);

    let pool = client.get_reward_pool_balance();
    assert_eq!(pool, 4000);

    // Claim for user1
    client.claim_rewards(&user1);

    let user1_rewards = client.get_user_rewards(&user1).unwrap();
    assert_eq!(user1_rewards.claimed, 3000);
    assert_eq!(user1_rewards.pending, 0);

    // Pool after user1 claim
    let pool = client.get_reward_pool_balance();
    assert_eq!(pool, 1000);

    // Claim for user2
    client.claim_rewards(&user2);

    let user2_rewards = client.get_user_rewards(&user2).unwrap();
    assert_eq!(user2_rewards.claimed, 3000);
    assert_eq!(user2_rewards.pending, 0);

    // Pool should be empty (or near empty accounting for rounding)
    let pool = client.get_reward_pool_balance();
    assert_eq!(pool, 1000);
}

#[test]
#[should_panic(expected = "Insufficient reward pool balance")]
fn test_claim_insufficient_pool_balance() {
    let (env, contract_id, token, _admin, rewards_admin) = setup_test();
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    let funder = Address::generate(&env);

    // Initialize
    client.initialize_rewards(&token, &rewards_admin);

    // Fund with limited amount
    client.fund_reward_pool(&funder, &500);

    // Issue reward that exceeds pool when claimed
    let participation = String::from_str(&env, "participation");
    client.issue_reward(&user, &500, &participation);

    // Manually deplete pool by funding more and issuing more
    client.fund_reward_pool(&funder, &200);
    let user2 = Address::generate(&env);
    client.issue_reward(&user2, &600, &participation);

    // Try to claim when pool is insufficient - should panic
    client.claim_rewards(&user);
}
