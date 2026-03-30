#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]

use soroban_sdk::{testutils::Address as _, Address, Env};
use teachlink_contract::{TeachLinkBridge, TeachLinkBridgeClient};

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

// ========== Initial State ==========

#[test]
fn test_initial_reputation_zeroed() {
    let (env, client) = setup();
    let user = Address::generate(&env);
    let rep = client.get_user_reputation(&user);
    assert_eq!(rep.participation_score, 0);
    assert_eq!(rep.completion_rate, 0);
    assert_eq!(rep.contribution_quality, 0);
    assert_eq!(rep.total_courses_started, 0);
    assert_eq!(rep.total_courses_completed, 0);
    assert_eq!(rep.total_contributions, 0);
}

// ========== Participation ==========

#[test]
fn test_update_participation() {
    let (env, client) = setup();
    let user = Address::generate(&env);

    client.update_participation(&user, &10);
    assert_eq!(client.get_user_reputation(&user).participation_score, 10);

    client.update_participation(&user, &5);
    assert_eq!(client.get_user_reputation(&user).participation_score, 15);
}

// ========== Course Progress ==========

#[test]
fn test_course_start() {
    let (env, client) = setup();
    let user = Address::generate(&env);

    client.update_course_progress(&user, &false);
    let rep = client.get_user_reputation(&user);
    assert_eq!(rep.total_courses_started, 1);
    assert_eq!(rep.total_courses_completed, 0);
    assert_eq!(rep.completion_rate, 0);
}

#[test]
fn test_course_completion_updates_rate() {
    let (env, client) = setup();
    let user = Address::generate(&env);

    client.update_course_progress(&user, &false); // start
    client.update_course_progress(&user, &true); // complete

    let rep = client.get_user_reputation(&user);
    assert_eq!(rep.total_courses_started, 1);
    assert_eq!(rep.total_courses_completed, 1);
    assert_eq!(rep.completion_rate, 10000); // 100% in basis points
}

#[test]
fn test_partial_completion_rate() {
    let (env, client) = setup();
    let user = Address::generate(&env);

    // Start 2 courses, complete 1
    client.update_course_progress(&user, &false);
    client.update_course_progress(&user, &false);
    client.update_course_progress(&user, &true);

    let rep = client.get_user_reputation(&user);
    assert_eq!(rep.total_courses_started, 2);
    assert_eq!(rep.total_courses_completed, 1);
    assert_eq!(rep.completion_rate, 5000); // 50%
}

#[test]
fn test_completion_without_start_auto_adjusts() {
    let (env, client) = setup();
    let user = Address::generate(&env);

    // Complete without explicit start — should auto-adjust started count
    client.update_course_progress(&user, &true);

    let rep = client.get_user_reputation(&user);
    assert_eq!(rep.total_courses_started, 1);
    assert_eq!(rep.total_courses_completed, 1);
    assert_eq!(rep.completion_rate, 10000);
}

// ========== Contribution Rating ==========

#[test]
fn test_single_rating() {
    let (env, client) = setup();
    let user = Address::generate(&env);

    client.rate_contribution(&user, &5);
    let rep = client.get_user_reputation(&user);
    assert_eq!(rep.total_contributions, 1);
    assert_eq!(rep.contribution_quality, 5);
}

#[test]
fn test_average_rating() {
    let (env, client) = setup();
    let user = Address::generate(&env);

    client.rate_contribution(&user, &5);
    client.rate_contribution(&user, &3);

    let rep = client.get_user_reputation(&user);
    assert_eq!(rep.total_contributions, 2);
    assert_eq!(rep.contribution_quality, 4); // (5+3)/2
}

#[test]
fn test_zero_rating() {
    let (env, client) = setup();
    let user = Address::generate(&env);

    client.rate_contribution(&user, &0);
    assert_eq!(client.get_user_reputation(&user).contribution_quality, 0);
}

#[test]
#[should_panic(expected = "Rating must be between 0 and 5")]
fn test_rating_above_5_panics() {
    let (env, client) = setup();
    let user = Address::generate(&env);
    client.rate_contribution(&user, &6);
}

// ========== Isolation ==========

#[test]
fn test_reputation_isolated_between_users() {
    let (env, client) = setup();
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);

    client.update_participation(&user_a, &10);
    client.rate_contribution(&user_b, &5);

    assert_eq!(client.get_user_reputation(&user_a).participation_score, 10);
    assert_eq!(client.get_user_reputation(&user_a).total_contributions, 0);
    assert_eq!(client.get_user_reputation(&user_b).participation_score, 0);
    assert_eq!(client.get_user_reputation(&user_b).total_contributions, 1);
}
