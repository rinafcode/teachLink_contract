//! Security test suite (integration-style): access control, integer bounds, front-running ordering.
//!
//! Replay protection for `complete_bridge` (nonce deduplication) is covered in
//! `bridge::tests::complete_bridge_rejects_replay_when_nonce_already_processed` so the check runs
//! before token mint (full end-to-end completion would require extra SAC setup).

#![allow(clippy::needless_pass_by_value)]

mod common;

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, vec, Address, Bytes, Env, IntoVal, String};
use teachlink_contract::validation::{config, NumberValidator, ValidationError};
use teachlink_contract::{TeachLinkBridge, TeachLinkBridgeClient};
use common::{random_address, register_bridge_client, register_sac_token, test_env};

fn mint_sac_token(env: &Env, token: &Address, to: &Address, amount: i128) {
    env.invoke_contract::<()>(
        token,
        &symbol_short!("mint"),
        vec![env, to.clone().into_val(env), amount.into_val(env)],
    );
}

fn setup_rewards_with_sac(env: &Env) -> (TeachLinkBridgeClient<'_>, Address, Address, Address) {
    env.mock_all_auths();
    let client = register_bridge_client(env);
    let token = register_sac_token(env);

    let admin = random_address(env);
    let fee_recipient = random_address(env);
    let rewards_admin = random_address(env);
    let funder = random_address(env);
    let recipient = random_address(env);

    client.initialize(&token, &admin, &1, &fee_recipient);
    client.initialize_rewards(&token, &rewards_admin);

    mint_sac_token(env, &token, &funder, 50_000_000_000);
    client.fund_reward_pool(&funder, &10_000_000_000);

    (client, rewards_admin, funder, recipient)
}

#[test]
fn security_integer_overflow_amount_bounds_reject_extreme_values() {
    assert_eq!(
        NumberValidator::validate_amount(0),
        Err(ValidationError::InvalidAmountRange)
    );
    assert_eq!(
        NumberValidator::validate_amount(-1),
        Err(ValidationError::InvalidAmountRange)
    );
    assert_eq!(
        NumberValidator::validate_amount(i128::MAX),
        Err(ValidationError::InvalidAmountRange)
    );
    assert_eq!(
        NumberValidator::validate_amount(config::MAX_AMOUNT + 1),
        Err(ValidationError::InvalidAmountRange)
    );
    assert_eq!(NumberValidator::validate_amount(config::MAX_AMOUNT), Ok(()));
    assert_eq!(NumberValidator::validate_amount(config::MIN_AMOUNT), Ok(()));
}

#[test]
fn security_integer_underflow_saturating_math_avoids_wrap() {
    assert_eq!(10u64.saturating_sub(100), 0);
    assert_eq!(0i128.saturating_sub(1), -1);
}

#[test]
fn security_access_control_admin_bridge_fee_requires_auth() {
    let env = test_env();
    let client = register_bridge_client(&env);

    let token = register_sac_token(&env);
    let admin = random_address(&env);
    let fee_recipient = random_address(&env);

    client.initialize(&token, &admin, &1, &fee_recipient);

    env.mock_auths(&[]);
    let r = client.try_set_bridge_fee(&10i128);
    assert!(r.is_err());
}

#[test]
fn security_access_control_issue_reward_requires_rewards_admin_auth() {
    let env = test_env();
    let (client, _rewards_admin, _funder, recipient) = setup_rewards_with_sac(&env);

    let reward_type = String::from_str(&env, "learning");
    env.mock_auths(&[]);
    let r = client.try_issue_reward(&recipient, &100, &reward_type);
    assert!(r.is_err());
}

#[test]
fn security_front_running_ordering_bridge_nonce_increments_monotonically() {
    let env = test_env();
    let client = register_bridge_client(&env);
    let token = register_sac_token(&env);

    let admin = random_address(&env);
    let fee_recipient = random_address(&env);
    let user = random_address(&env);
    let dest = Bytes::from_slice(&env, &[0xcd; 20]);

    client.initialize(&token, &admin, &1, &fee_recipient);
    client.add_supported_chain(&1);

    mint_sac_token(&env, &token, &user, 10_000_000_000);

    let n0 = client.get_nonce();
    let n1 = client
        .try_bridge_out(&user, &100, &1, &dest)
        .expect("host")
        .expect("bridge_out");
    assert_eq!(n1, n0 + 1);

    let n2 = client
        .try_bridge_out(&user, &100, &1, &dest)
        .expect("host")
        .expect("bridge_out");
    assert_eq!(n2, n1 + 1);
    assert_eq!(client.get_nonce(), n2);
}
