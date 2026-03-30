#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]

use soroban_sdk::{testutils::Address as _, Address, Bytes, Env};
use teachlink_contract::{ContributionType, TeachLinkBridge, TeachLinkBridgeClient};

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

// ========== Credit Score: Course Completion ==========

#[test]
fn test_record_course_completion_awards_points() {
    let (env, client) = setup();
    let user = Address::generate(&env);

    client.record_course_completion(&user, &1, &100);

    assert_eq!(client.get_credit_score(&user), 100);
    let courses = client.get_user_courses(&user);
    assert_eq!(courses.len(), 1);
    assert_eq!(courses.get(0).unwrap(), 1);
}

#[test]
fn test_multiple_course_completions_accumulate() {
    let (env, client) = setup();
    let user = Address::generate(&env);

    client.record_course_completion(&user, &1, &100);
    client.record_course_completion(&user, &2, &200);

    assert_eq!(client.get_credit_score(&user), 300);
    assert_eq!(client.get_user_courses(&user).len(), 2);
}

#[test]
fn test_duplicate_course_completion_ignored() {
    let (env, client) = setup();
    let user = Address::generate(&env);

    client.record_course_completion(&user, &1, &100);
    client.record_course_completion(&user, &1, &100); // duplicate

    assert_eq!(client.get_credit_score(&user), 100);
    assert_eq!(client.get_user_courses(&user).len(), 1);
}

// ========== Credit Score: Contributions ==========

#[test]
fn test_record_contribution_awards_points() {
    let (env, client) = setup();
    let user = Address::generate(&env);
    let desc = Bytes::from_slice(&env, b"tutorial");

    client.record_contribution(&user, &ContributionType::Content, &desc, &50);

    assert_eq!(client.get_credit_score(&user), 50);
    let contribs = client.get_user_contributions(&user);
    assert_eq!(contribs.len(), 1);
}

#[test]
fn test_contributions_accumulate_with_courses() {
    let (env, client) = setup();
    let user = Address::generate(&env);
    let desc = Bytes::from_slice(&env, b"code");

    client.record_course_completion(&user, &1, &100);
    client.record_contribution(&user, &ContributionType::Code, &desc, &50);

    assert_eq!(client.get_credit_score(&user), 150);
}

// ========== Credit Score: View Functions ==========

#[test]
fn test_initial_score_is_zero() {
    let (env, client) = setup();
    let user = Address::generate(&env);
    assert_eq!(client.get_credit_score(&user), 0);
}

#[test]
fn test_initial_courses_empty() {
    let (env, client) = setup();
    let user = Address::generate(&env);
    assert_eq!(client.get_user_courses(&user).len(), 0);
}

#[test]
fn test_initial_contributions_empty() {
    let (env, client) = setup();
    let user = Address::generate(&env);
    assert_eq!(client.get_user_contributions(&user).len(), 0);
}

// ========== Credit Score: Isolation Between Users ==========

#[test]
fn test_scores_isolated_between_users() {
    let (env, client) = setup();
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);

    client.record_course_completion(&user_a, &1, &100);
    client.record_course_completion(&user_b, &2, &200);

    assert_eq!(client.get_credit_score(&user_a), 100);
    assert_eq!(client.get_credit_score(&user_b), 200);
    assert_eq!(client.get_user_courses(&user_a).len(), 1);
    assert_eq!(client.get_user_courses(&user_b).len(), 1);
}
