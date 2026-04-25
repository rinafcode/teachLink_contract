//! Integration tests for cross-contract interactions.
//!
//! Covers:
//! - Contract-to-contract calls (multi-module chained operations)
//! - Error propagation across module boundaries
//! - State consistency after multi-step interactions
//! - Event emission verification

#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unreadable_literal)]

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events},
    vec, Address, Bytes, Env, IntoVal, String,
};
use teachlink_contract::{
    ContentTokenParameters, ContentType, OperationType, TeachLinkBridge, TeachLinkBridgeClient,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup(env: &Env) -> (TeachLinkBridgeClient<'_>, Address, Address) {
    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(env, &contract_id);
    let token = Address::generate(env);
    let admin = Address::generate(env);
    let fee_recipient = Address::generate(env);
    client.initialize(&token, &admin, &1, &fee_recipient);
    (client, admin, token)
}

/// Setup with a real SAC token for reward pool operations that invoke token transfers.
fn setup_with_sac(
    env: &Env,
) -> (TeachLinkBridgeClient<'_>, Address, Address, Address, Address) {
    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(env, &contract_id);

    let token_admin = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token = sac.address();

    let admin = Address::generate(env);
    let fee_recipient = Address::generate(env);
    let rewards_admin = Address::generate(env);
    let funder = Address::generate(env);

    env.mock_all_auths();

    client.initialize(&token, &admin, &1, &fee_recipient);
    client.initialize_rewards(&token, &rewards_admin);

    // Mint tokens to funder and fund the pool
    env.invoke_contract::<()>(
        &token,
        &symbol_short!("mint"),
        vec![env, funder.clone().into_val(env), 10_000i128.into_val(env)],
    );
    client.fund_reward_pool(&funder, &5_000i128);

    (client, admin, token, rewards_admin, funder)
}

fn content_params(env: &Env, creator: &Address) -> ContentTokenParameters {
    ContentTokenParameters {
        creator: creator.clone(),
        title: Bytes::from_slice(env, b"Test Course"),
        description: Bytes::from_slice(env, b"A test course"),
        content_type: ContentType::Course,
        content_hash: Bytes::from_slice(env, b"QmHash"),
        license_type: Bytes::from_slice(env, b"MIT"),
        tags: vec![env, Bytes::from_slice(env, b"test")],
        is_transferable: true,
        royalty_percentage: 500,
    }
}

// ---------------------------------------------------------------------------
// 1. Contract-to-contract calls
//    Verifies that operations spanning multiple modules execute correctly
//    in sequence and each module sees the correct state.
// ---------------------------------------------------------------------------

#[test]
fn test_cross_module_tokenization_then_reputation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _token) = setup(&env);

    let creator = Address::generate(&env);

    // Tokenization module: mint a content token
    let token_id = client.mint_content_token(&content_params(&env, &creator));
    assert!(token_id > 0);

    // Reputation module: record course progress for the same user
    client.update_course_progress(&creator, &false); // started
    client.update_course_progress(&creator, &true);  // completed

    let rep = client.get_user_reputation(&creator);
    assert_eq!(rep.total_courses_completed, 1);

    // Tokenization state must still be intact after reputation calls
    let token = client.get_content_token(&token_id).expect("token must exist");
    assert_eq!(token.creator, creator);
}

#[test]
fn test_cross_module_rewards_then_credit_score() {
    let env = Env::default();
    let (client, _admin, _token, _rewards_admin, _funder) = setup_with_sac(&env);

    let user = Address::generate(&env);

    // Issue a reward to the user
    client.issue_reward(&user, &100i128, &String::from_str(&env, "course_completion"));

    let user_reward = client.get_user_rewards(&user).expect("reward must exist");
    assert_eq!(user_reward.pending, 100i128);

    // Credit score module: record a contribution for the same user
    client.record_contribution(
        &user,
        &teachlink_contract::ContributionType::Content,
        &Bytes::from_slice(&env, b"created a lesson"),
        &50u64,
    );

    let score = client.get_credit_score(&user);
    assert!(score > 0, "credit score should be positive after contribution");
}

#[test]
fn test_cross_module_bridge_then_audit() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _token) = setup(&env);

    // Bridge module: add a supported chain
    client.add_supported_chain(&42u32);
    assert!(client.is_chain_supported(&42u32));

    // Audit module: record an operation referencing the bridge action
    let record_id = client.create_audit_record(
        &OperationType::BridgeOut,
        &admin,
        &Bytes::from_slice(&env, b"added chain 42"),
        &Bytes::from_slice(&env, b"tx_hash_001"),
    );
    assert!(record_id > 0);

    let record = client.get_audit_record(&record_id).expect("audit record must exist");
    assert_eq!(record.operator, admin);
}

#[test]
fn test_cross_module_reputation_then_tokenization() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _token) = setup(&env);

    let user = Address::generate(&env);

    // Reputation first
    client.update_participation(&user, &10u32);
    let rep = client.get_user_reputation(&user);
    assert_eq!(rep.participation_score, 10);

    // Then tokenization
    let token_id = client.mint_content_token(&content_params(&env, &user));
    assert!(token_id > 0);

    // Reputation must be unchanged after minting
    let rep_after = client.get_user_reputation(&user);
    assert_eq!(rep_after.participation_score, 10);
}

// ---------------------------------------------------------------------------
// 2. Error propagation
//    Verifies that errors from one module surface correctly to the caller
//    and do not corrupt state in other modules.
// ---------------------------------------------------------------------------

#[test]
fn test_error_rewards_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    // Do NOT call initialize_rewards — issuing a reward should fail
    let user = Address::generate(&env);
    let result = client.try_issue_reward(&user, &100i128, &String::from_str(&env, "test"));
    assert!(result.is_err(), "should fail when rewards not initialized");
}

#[test]
fn test_error_bridge_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let from = Address::generate(&env);
    let result = client.try_bridge_out(
        &from,
        &100i128,
        &1u32,
        &Bytes::from_slice(&env, b"dest"),
    );
    assert!(result.is_err(), "bridge_out should fail when not initialized");
}

#[test]
fn test_error_bridge_unsupported_chain_does_not_corrupt_reputation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _token) = setup(&env);

    let user = Address::generate(&env);

    // Record reputation state
    client.update_participation(&user, &10u32);
    let rep_before = client.get_user_reputation(&user);
    assert_eq!(rep_before.participation_score, 10);

    // Attempt a failing bridge_out (unsupported chain)
    let result = client.try_bridge_out(
        &user,
        &100i128,
        &999u32, // unsupported chain
        &Bytes::from_slice(&env, b"dest"),
    );
    assert!(result.is_err(), "bridge_out to unsupported chain should fail");

    // Reputation state must be unchanged after the failed bridge call
    let rep_after = client.get_user_reputation(&user);
    assert_eq!(rep_after.participation_score, rep_before.participation_score);
}

#[test]
fn test_error_rewards_double_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, token) = setup(&env);

    let rewards_admin = Address::generate(&env);
    client.initialize_rewards(&token, &rewards_admin);

    // Second initialization must return AlreadyInitialized
    let result = client.try_initialize_rewards(&token, &rewards_admin);
    assert!(result.is_err(), "double initialize_rewards should fail");
}

#[test]
fn test_error_bridge_double_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, token) = setup(&env);

    let admin2 = Address::generate(&env);
    let fee2 = Address::generate(&env);
    let result = client.try_initialize(&token, &admin2, &1u32, &fee2);
    assert!(result.is_err(), "double bridge initialize should fail");
}

// ---------------------------------------------------------------------------
// 3. State consistency
//    Verifies that multi-step interactions leave the contract in a coherent
//    state (no partial writes, counters match, ownership is correct).
// ---------------------------------------------------------------------------

#[test]
fn test_state_token_transfer_updates_ownership() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _token) = setup(&env);

    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);

    let token_id = client.mint_content_token(&content_params(&env, &creator));

    assert!(client.is_content_token_owner(&token_id, &creator));
    assert!(!client.is_content_token_owner(&token_id, &buyer));

    client.transfer_content_token(&creator, &buyer, &token_id, &None);

    // Ownership must be updated atomically
    assert!(!client.is_content_token_owner(&token_id, &creator));
    assert!(client.is_content_token_owner(&token_id, &buyer));

    let owner = client.get_content_token_owner(&token_id).expect("owner must exist");
    assert_eq!(owner, buyer);
}

#[test]
fn test_state_reward_pool_after_issue_and_claim() {
    let env = Env::default();
    let (client, _admin, token, _rewards_admin, _funder) = setup_with_sac(&env);

    let user = Address::generate(&env);

    // Mint tokens to user so the claim transfer succeeds
    env.invoke_contract::<()>(
        &token,
        &symbol_short!("mint"),
        vec![&env, user.clone().into_val(&env), 0i128.into_val(&env)],
    );

    let pool_before = client.get_reward_pool_balance();

    client.issue_reward(&user, &200i128, &String::from_str(&env, "participation"));

    // Pool balance unchanged until claim (issue only reserves, deducts at claim)
    assert_eq!(client.get_reward_pool_balance(), pool_before);
    assert_eq!(client.get_total_rewards_issued(), 200i128);

    let user_reward = client.get_user_rewards(&user).expect("reward record must exist");
    assert_eq!(user_reward.pending, 200i128);

    // Claim rewards
    client.claim_rewards(&user);

    let user_reward_after = client.get_user_rewards(&user).expect("reward record must exist");
    assert_eq!(user_reward_after.pending, 0i128, "pending should be zero after claim");
    assert_eq!(user_reward_after.claimed, 200i128);

    // Pool balance decreases after claim
    assert_eq!(client.get_reward_pool_balance(), pool_before - 200);
}

#[test]
fn test_state_reputation_accumulates_correctly() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _token) = setup(&env);

    let user = Address::generate(&env);

    client.update_participation(&user, &5u32);
    client.update_participation(&user, &3u32);

    let rep = client.get_user_reputation(&user);
    assert_eq!(rep.participation_score, 8, "participation scores should accumulate");

    client.update_course_progress(&user, &false); // start 1
    client.update_course_progress(&user, &false); // start 2
    client.update_course_progress(&user, &true);  // complete 1

    let rep2 = client.get_user_reputation(&user);
    assert_eq!(rep2.total_courses_started, 2);
    assert_eq!(rep2.total_courses_completed, 1);
}

#[test]
fn test_state_multiple_tokens_are_independent() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _token) = setup(&env);

    let creator_a = Address::generate(&env);
    let creator_b = Address::generate(&env);

    let id_a = client.mint_content_token(&content_params(&env, &creator_a));
    let id_b = client.mint_content_token(&content_params(&env, &creator_b));

    assert_ne!(id_a, id_b, "token IDs must be unique");

    // Transfer token A; token B must be unaffected
    let new_owner = Address::generate(&env);
    client.transfer_content_token(&creator_a, &new_owner, &id_a, &None);

    assert!(client.is_content_token_owner(&id_b, &creator_b), "token B ownership unchanged");
    assert!(client.is_content_token_owner(&id_a, &new_owner), "token A transferred");
}

// ---------------------------------------------------------------------------
// 4. Event emission
//    Verifies that key cross-module operations emit the expected events.
// ---------------------------------------------------------------------------

#[test]
fn test_event_reward_pool_funded() {
    let env = Env::default();
    let (client, _admin, _token, _rewards_admin, _funder) = setup_with_sac(&env);
    // fund_reward_pool was already called in setup_with_sac
    let events = env.events().all();
    assert!(!events.is_empty(), "at least one event should be emitted after funding");
}

#[test]
fn test_event_reward_issued() {
    let env = Env::default();
    let (client, _admin, _token, _rewards_admin, _funder) = setup_with_sac(&env);

    let user = Address::generate(&env);
    client.issue_reward(&user, &100i128, &String::from_str(&env, "course_completion"));

    let events = env.events().all();
    assert!(!events.is_empty(), "reward issued event should be emitted");
}

#[test]
fn test_event_content_token_minted() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _token) = setup(&env);

    let creator = Address::generate(&env);
    client.mint_content_token(&content_params(&env, &creator));

    let events = env.events().all();
    assert!(!events.is_empty(), "content minted event should be emitted");
}

#[test]
fn test_event_validator_added() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _token) = setup(&env);

    let validator = Address::generate(&env);
    client.add_validator(&validator);

    let events = env.events().all();
    assert!(!events.is_empty(), "validator added event should be emitted");
}

#[test]
fn test_event_audit_record_created() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _token) = setup(&env);

    client.create_audit_record(
        &OperationType::BridgeOut,
        &admin,
        &Bytes::from_slice(&env, b"details"),
        &Bytes::from_slice(&env, b"tx_hash"),
    );

    let events = env.events().all();
    assert!(!events.is_empty(), "audit record created event should be emitted");
}

#[test]
fn test_event_multiple_modules_emit_events() {
    let env = Env::default();
    let (client, admin, _token, _rewards_admin, _funder) = setup_with_sac(&env);

    let user = Address::generate(&env);

    // Trigger events across three modules
    client.mint_content_token(&content_params(&env, &user));
    client.update_participation(&user, &5u32);
    client.create_audit_record(
        &OperationType::ConfigUpdate,
        &admin,
        &Bytes::from_slice(&env, b"multi-module test"),
        &Bytes::from_slice(&env, b"tx_multi"),
    );

    let events = env.events().all();
    // At minimum: RewardPoolFunded (setup) + ContentMinted + ParticipationUpdated + AuditRecordCreated
    assert!(
        events.len() >= 4,
        "expected at least 4 events across modules, got {}",
        events.len()
    );
}
