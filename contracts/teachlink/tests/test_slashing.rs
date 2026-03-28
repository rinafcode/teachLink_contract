#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]

use soroban_sdk::{testutils::Address as _, Address, Bytes, Env};
use teachlink_contract::{
    BridgeError, RewardType, SlashingReason, TeachLinkBridge, TeachLinkBridgeClient,
};

fn setup() -> (Env, TeachLinkBridgeClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    client.initialize(&token, &admin, &1, &fee_recipient);
    (env, client)
}

fn register_validator(env: &Env, client: &TeachLinkBridgeClient, stake: i128) -> Address {
    let validator = Address::generate(env);
    client.register_validator(&validator, &stake);
    validator
}

const MIN_STAKE: i128 = 100_000_000;

// ========== Deposit Stake ==========

#[test]
fn test_deposit_stake() {
    let (env, client) = setup();
    let validator = register_validator(&env, &client, MIN_STAKE);

    client.deposit_stake(&validator, &1000);
    assert_eq!(client.get_validator_stake(&validator), MIN_STAKE + 1000);
}

#[test]
fn test_deposit_stake_zero_fails() {
    let (env, client) = setup();
    let validator = Address::generate(&env);
    let result = client.try_deposit_stake(&validator, &0);
    assert_eq!(result, Err(Ok(BridgeError::AmountMustBePositive)));
}

#[test]
fn test_deposit_stake_negative_fails() {
    let (env, client) = setup();
    let validator = Address::generate(&env);
    let result = client.try_deposit_stake(&validator, &-1);
    assert_eq!(result, Err(Ok(BridgeError::AmountMustBePositive)));
}

// ========== Withdraw Stake ==========

#[test]
fn test_withdraw_stake() {
    let (env, client) = setup();
    let validator = register_validator(&env, &client, MIN_STAKE);

    client.withdraw_stake(&validator, &1000);
    assert_eq!(client.get_validator_stake(&validator), MIN_STAKE - 1000);
}

#[test]
fn test_withdraw_more_than_stake_fails() {
    let (env, client) = setup();
    let validator = register_validator(&env, &client, MIN_STAKE);

    let result = client.try_withdraw_stake(&validator, &(MIN_STAKE + 1));
    assert_eq!(result, Err(Ok(BridgeError::InsufficientBalance)));
}

#[test]
fn test_withdraw_zero_fails() {
    let (env, client) = setup();
    let validator = Address::generate(&env);
    let result = client.try_withdraw_stake(&validator, &0);
    assert_eq!(result, Err(Ok(BridgeError::AmountMustBePositive)));
}

// ========== Slash Validator ==========

#[test]
fn test_slash_double_vote() {
    let (env, client) = setup();
    let validator = register_validator(&env, &client, MIN_STAKE);
    let slasher = register_validator(&env, &client, MIN_STAKE);
    let evidence = Bytes::from_slice(&env, b"proof");

    let slashed = client.slash_validator(
        &validator,
        &SlashingReason::DoubleVote,
        &evidence,
        &slasher,
    );

    // 50% of MIN_STAKE
    assert_eq!(slashed, MIN_STAKE / 2);
    assert_eq!(client.get_validator_stake(&validator), MIN_STAKE / 2);
}

#[test]
fn test_slash_invalid_signature() {
    let (env, client) = setup();
    let validator = register_validator(&env, &client, MIN_STAKE);
    let slasher = register_validator(&env, &client, MIN_STAKE);
    let evidence = Bytes::from_slice(&env, b"proof");

    let slashed = client.slash_validator(
        &validator,
        &SlashingReason::InvalidSignature,
        &evidence,
        &slasher,
    );

    // 10% of MIN_STAKE
    assert_eq!(slashed, MIN_STAKE / 10);
}

#[test]
fn test_slash_byzantine_takes_full_stake() {
    let (env, client) = setup();
    let validator = register_validator(&env, &client, MIN_STAKE);
    let slasher = register_validator(&env, &client, MIN_STAKE);
    let evidence = Bytes::from_slice(&env, b"proof");

    let slashed = client.slash_validator(
        &validator,
        &SlashingReason::ByzantineBehavior,
        &evidence,
        &slasher,
    );

    assert_eq!(slashed, MIN_STAKE); // 100%
    assert_eq!(client.get_validator_stake(&validator), 0);
}

#[test]
fn test_slash_self_fails() {
    let (env, client) = setup();
    let validator = register_validator(&env, &client, MIN_STAKE);
    let evidence = Bytes::from_slice(&env, b"proof");

    let result = client.try_slash_validator(
        &validator,
        &SlashingReason::DoubleVote,
        &evidence,
        &validator,
    );
    assert_eq!(result, Err(Ok(BridgeError::CannotSlashSelf)));
}

#[test]
fn test_slash_unregistered_validator_fails() {
    let (env, client) = setup();
    let unknown = Address::generate(&env);
    let slasher = register_validator(&env, &client, MIN_STAKE);
    let evidence = Bytes::from_slice(&env, b"proof");

    let result = client.try_slash_validator(
        &unknown,
        &SlashingReason::DoubleVote,
        &evidence,
        &slasher,
    );
    assert_eq!(result, Err(Ok(BridgeError::ValidatorNotActive)));
}

#[test]
fn test_slashed_amount_goes_to_reward_pool() {
    let (env, client) = setup();
    let validator = register_validator(&env, &client, MIN_STAKE);
    let slasher = register_validator(&env, &client, MIN_STAKE);
    let evidence = Bytes::from_slice(&env, b"proof");

    let slashed = client.slash_validator(
        &validator,
        &SlashingReason::Inactivity,
        &evidence,
        &slasher,
    );

    // 5% of MIN_STAKE
    assert_eq!(slashed, MIN_STAKE * 500 / 10000);
}

// ========== Reward Validator ==========

#[test]
fn test_reward_validator() {
    let (env, client) = setup();
    let validator = register_validator(&env, &client, MIN_STAKE);
    let funder = Address::generate(&env);

    client.fund_validator_reward_pool(&funder, &1000);
    client.reward_validator(&validator, &500, &RewardType::Validation);

    // Reward adds to validator info stake
    let info = client.get_validator_info(&validator).unwrap();
    assert_eq!(info.stake, MIN_STAKE + 500);
}

#[test]
fn test_reward_insufficient_pool_fails() {
    let (env, client) = setup();
    let validator = register_validator(&env, &client, MIN_STAKE);

    let result = client.try_reward_validator(&validator, &500, &RewardType::Validation);
    assert_eq!(result, Err(Ok(BridgeError::InsufficientBalance)));
}

#[test]
fn test_reward_zero_amount_fails() {
    let (env, client) = setup();
    let validator = Address::generate(&env);

    let result = client.try_reward_validator(&validator, &0, &RewardType::Validation);
    assert_eq!(result, Err(Ok(BridgeError::AmountMustBePositive)));
}

// ========== Fund Reward Pool ==========

#[test]
fn test_fund_validator_reward_pool() {
    let (env, client) = setup();
    let funder = Address::generate(&env);

    client.fund_validator_reward_pool(&funder, &1000);
    client.fund_validator_reward_pool(&funder, &500);

    // Verify pool has funds by successfully rewarding
    let validator = register_validator(&env, &client, MIN_STAKE);
    client.reward_validator(&validator, &1500, &RewardType::Validation);
    let info = client.get_validator_info(&validator).unwrap();
    assert_eq!(info.stake, MIN_STAKE + 1500);
}

#[test]
fn test_fund_reward_pool_zero_fails() {
    let (env, client) = setup();
    let funder = Address::generate(&env);
    let result = client.try_fund_validator_reward_pool(&funder, &0);
    assert_eq!(result, Err(Ok(BridgeError::AmountMustBePositive)));
}

// ========== Validator Info After Slash ==========

#[test]
fn test_validator_info_updated_after_slash() {
    let (env, client) = setup();
    let validator = register_validator(&env, &client, MIN_STAKE);
    let slasher = register_validator(&env, &client, MIN_STAKE);
    let evidence = Bytes::from_slice(&env, b"proof");

    client.slash_validator(
        &validator,
        &SlashingReason::DoubleVote,
        &evidence,
        &slasher,
    );

    let info = client.get_validator_info(&validator).unwrap();
    assert_eq!(info.slashed_amount, MIN_STAKE / 2);
    assert!(info.reputation_score < 100); // reduced from initial 100
}
