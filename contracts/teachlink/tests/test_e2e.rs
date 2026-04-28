//! End-to-end tests for the TeachLink contract.
//!
//! Covers complete workflows, cross-module integrations, user scenarios,
//! and output validation.

#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::too_many_lines)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    vec, Address, Bytes, Env, Map, String, Vec,
};

use teachlink_contract::{
    AssessmentSettings, ContentTokenParameters, ContentType, ContractSemVer, ContributionType,
    QuestionType, TeachLinkBridge, TeachLinkBridgeClient,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup(env: &Env) -> TeachLinkBridgeClient<'_> {
    let id = env.register(TeachLinkBridge, ());
    TeachLinkBridgeClient::new(env, &id)
}

fn set_timestamp(env: &Env, ts: u64) {
    env.ledger().set(LedgerInfo {
        timestamp: ts,
        protocol_version: 25,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 2_000_000,
    });
}

fn bytes(env: &Env, s: &[u8]) -> Bytes {
    Bytes::from_slice(env, s)
}

// ---------------------------------------------------------------------------
// Workflow 1: Bridge initialisation → validator registration → chain config
// ---------------------------------------------------------------------------

#[test]
fn e2e_bridge_init_and_validator_workflow() {
    let env = Env::default();
    env.mock_all_auths();
    set_timestamp(&env, 1_000);

    let client = setup(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let validator = Address::generate(&env);

    // Initialize bridge
    client.initialize(&token, &admin, &1, &fee_recipient);

    // Verify initial state
    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_token(), token);
    assert_eq!(client.get_bridge_fee(), 0);
    assert_eq!(client.get_nonce(), 0);

    // Add validator and chain
    client.add_validator(&validator);
    client.add_supported_chain(&42);

    assert!(client.is_validator(&validator));
    assert!(client.is_chain_supported(&42));

    // Set fee and verify
    client.set_bridge_fee(&50);
    assert_eq!(client.get_bridge_fee(), 50);

    // Remove chain and verify
    client.remove_supported_chain(&42);
    assert!(!client.is_chain_supported(&42));
}

// ---------------------------------------------------------------------------
// Workflow 2: BFT consensus – register validators, create proposal, vote
// ---------------------------------------------------------------------------

#[test]
fn e2e_bft_consensus_workflow() {
    let env = Env::default();
    env.mock_all_auths();
    set_timestamp(&env, 2_000);

    let client = setup(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);

    client.initialize(&token, &admin, &2, &fee_recipient);

    let v1 = Address::generate(&env);
    let v2 = Address::generate(&env);

    // Register validators with stake
    client.register_validator(&v1, &1_000);
    client.register_validator(&v2, &2_000);

    let info1 = client.get_validator_info(&v1).unwrap();
    let info2 = client.get_validator_info(&v2).unwrap();
    assert_eq!(info1.stake, 1_000);
    assert_eq!(info2.stake, 2_000);

    // Consensus state reflects two validators
    let state = client.get_consensus_state();
    assert_eq!(state.active_validators, 2);

    // Create a bridge proposal
    let msg = teachlink_contract::CrossChainMessage {
        nonce: 1,
        source_chain: 1,
        destination_chain: 2,
        token: token.clone(),
        amount: 500,
        recipient: bytes(&env, b"0xrecipient"),
        timestamp: 2_000,
    };
    let proposal_id = client.create_bridge_proposal(&msg).unwrap();

    // Both validators vote
    client.vote_on_proposal(&v1, &proposal_id, &true);
    client.vote_on_proposal(&v2, &proposal_id, &true);

    assert!(client.is_consensus_reached(&proposal_id));

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.vote_count, 2);
}

// ---------------------------------------------------------------------------
// Workflow 3: Rewards – init, fund pool, issue, claim
// ---------------------------------------------------------------------------

#[test]
fn e2e_rewards_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    set_timestamp(&env, 3_000);

    let client = setup(&env);
    let token = Address::generate(&env);
    let rewards_admin = Address::generate(&env);
    let funder = Address::generate(&env);
    let learner = Address::generate(&env);

    // Initialize rewards subsystem
    client.initialize_rewards(&token, &rewards_admin);
    assert_eq!(client.get_rewards_admin(), rewards_admin);
    assert_eq!(client.get_reward_pool_balance(), 0);

    // Fund the pool
    client.fund_reward_pool(&funder, &10_000);
    assert_eq!(client.get_reward_pool_balance(), 10_000);

    // Set a reward rate for "course_completion"
    let reward_type = String::from_str(&env, "course_completion");
    client.set_reward_rate(&reward_type, &100, &true);

    let rate = client.get_reward_rate(&reward_type).unwrap();
    assert_eq!(rate.rate, 100);
    assert!(rate.enabled);

    // Issue reward to learner
    client.issue_reward(&learner, &100, &reward_type);

    let user_reward = client.get_user_rewards(&learner).unwrap();
    assert_eq!(user_reward.pending, 100);
    assert_eq!(client.get_total_rewards_issued(), 100);

    // Learner claims rewards
    client.claim_rewards(&learner);

    let after_claim = client.get_user_rewards(&learner).unwrap();
    assert_eq!(after_claim.pending, 0);
    assert_eq!(after_claim.claimed, 100);
}

// ---------------------------------------------------------------------------
// Workflow 4: Content tokenization + provenance tracking
// ---------------------------------------------------------------------------

#[test]
fn e2e_content_tokenization_and_provenance() {
    let env = Env::default();
    env.mock_all_auths();
    set_timestamp(&env, 4_000);

    let client = setup(&env);
    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);

    // Mint a content token
    let params = ContentTokenParameters {
        creator: creator.clone(),
        title: bytes(&env, b"Intro to Soroban"),
        description: bytes(&env, b"Learn Soroban smart contracts"),
        content_type: ContentType::Course,
        content_hash: bytes(&env, b"QmHash_soroban_101"),
        license_type: bytes(&env, b"MIT"),
        tags: vec![&env, bytes(&env, b"soroban"), bytes(&env, b"stellar")],
        is_transferable: true,
        royalty_percentage: 5,
    };

    let token_id = client.mint_content_token(&params);
    assert_eq!(token_id, 1);
    assert_eq!(client.get_content_token_count(), 1);

    // Validate token data
    let token = client.get_content_token(&token_id).unwrap();
    assert_eq!(token.metadata.creator, creator);
    assert!(client.is_content_token_owner(&token_id, &creator));

    // Verify provenance after mint
    let provenance = client.get_content_provenance(&token_id);
    assert_eq!(provenance.len(), 1);

    // Transfer to buyer
    client.transfer_content_token(&creator, &buyer, &token_id, &None);

    assert!(client.is_content_token_owner(&token_id, &buyer));
    assert!(!client.is_content_token_owner(&token_id, &creator));

    // Provenance now has 2 records (mint + transfer)
    let provenance_after = client.get_content_provenance(&token_id);
    assert_eq!(provenance_after.len(), 2);
    assert_eq!(client.get_content_transfer_count(&token_id), 1);

    // Verify chain integrity
    assert!(client.verify_content_chain(&token_id));

    // Buyer tokens list
    let buyer_tokens = client.get_owner_content_tokens(&buyer);
    assert_eq!(buyer_tokens.len(), 1);
    assert_eq!(buyer_tokens.get(0).unwrap(), token_id);
}

// ---------------------------------------------------------------------------
// Workflow 5: Assessment – create, add questions, submit, grade
// ---------------------------------------------------------------------------

#[test]
fn e2e_assessment_full_workflow() {
    let env = Env::default();
    env.mock_all_auths();
    set_timestamp(&env, 5_000);

    let client = setup(&env);
    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    // Add questions to the pool
    let q1 = client.add_assessment_question(
        &creator,
        &QuestionType::MultipleChoice,
        &bytes(&env, b"What is a smart contract?"),
        &20,
        &3,
        &bytes(&env, b"Self-executing code on blockchain"),
        &Map::new(&env),
    );
    let q2 = client.add_assessment_question(
        &creator,
        &QuestionType::MultipleChoice,
        &bytes(&env, b"What language is Soroban written in?"),
        &20,
        &2,
        &bytes(&env, b"Rust"),
        &Map::new(&env),
    );

    // Create assessment
    let mut questions = Vec::new(&env);
    questions.push_back(q1);
    questions.push_back(q2);

    let assessment_id = client.create_assessment(
        &creator,
        &bytes(&env, b"Soroban Basics"),
        &bytes(&env, b"Fundamentals of Soroban development"),
        &questions,
        &AssessmentSettings {
            time_limit: 3_600,
            passing_score: 30,
            is_adaptive: false,
            allow_retakes: false,
            proctoring_enabled: false,
        },
    );
    assert_eq!(assessment_id, 1);

    let assessment = client.get_assessment(&assessment_id).unwrap();
    assert_eq!(assessment.creator, creator);

    // Student submits – answers q1 correctly, q2 incorrectly
    let mut answers = Map::new(&env);
    answers.set(q1, bytes(&env, b"Self-executing code on blockchain"));
    answers.set(q2, bytes(&env, b"Python")); // wrong

    let score = client.submit_assessment(&student, &assessment_id, &answers, &Vec::new(&env));
    assert_eq!(score, 20); // only q1 correct

    let submission = client
        .get_assessment_submission(&student, &assessment_id)
        .unwrap();
    assert_eq!(submission.score, 20);
    assert_eq!(submission.max_score, 40);
    assert!(submission.is_graded);
    // score 20 < passing_score 30 → not passed
    assert!(submission.score < 30);
}

// ---------------------------------------------------------------------------
// Workflow 6: Reputation + credit score – user learning journey
// ---------------------------------------------------------------------------

#[test]
fn e2e_user_learning_journey() {
    let env = Env::default();
    env.mock_all_auths();
    set_timestamp(&env, 6_000);

    let client = setup(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(&token, &admin, &1, &Address::generate(&env));

    let learner = Address::generate(&env);

    // Initial state – zero scores
    assert_eq!(client.get_credit_score(&learner), 0);
    let rep = client.get_user_reputation(&learner);
    assert_eq!(rep.participation_score, 0);

    // Learner participates
    client.update_participation(&learner, &50);
    let rep = client.get_user_reputation(&learner);
    assert_eq!(rep.participation_score, 50);

    // Learner starts a course
    client.update_course_progress(&learner, &false);
    let rep = client.get_user_reputation(&learner);
    assert_eq!(rep.total_courses_started, 1);

    // Learner completes the course (credit score)
    client.record_course_completion(&learner, &101, &200);
    assert_eq!(client.get_credit_score(&learner), 200);

    // Completing again for same course should not double-count
    client.record_course_completion(&learner, &101, &200);
    assert_eq!(client.get_credit_score(&learner), 200);

    // Learner completes a second course
    client.record_course_completion(&learner, &102, &150);
    assert_eq!(client.get_credit_score(&learner), 350);

    // Learner makes a contribution
    client.record_contribution(
        &learner,
        &ContributionType::ContentCreation,
        &bytes(&env, b"Created tutorial"),
        &100,
    );
    assert_eq!(client.get_credit_score(&learner), 450);

    // Verify courses list
    let courses = client.get_user_courses(&learner);
    assert_eq!(courses.len(), 2);

    // Verify contributions list
    let contribs = client.get_user_contributions(&learner);
    assert_eq!(contribs.len(), 1);
}

// ---------------------------------------------------------------------------
// Workflow 7: Emergency pause / resume integration
// ---------------------------------------------------------------------------

#[test]
fn e2e_emergency_pause_resume() {
    let env = Env::default();
    env.mock_all_auths();
    set_timestamp(&env, 7_000);

    let client = setup(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);

    client.initialize(&token, &admin, &1, &fee_recipient);

    // Bridge is not paused initially
    assert!(!client.is_bridge_paused());

    // Pause the bridge
    client.pause_bridge(&admin, &bytes(&env, b"Security incident"));
    assert!(client.is_bridge_paused());

    let state = client.get_emergency_state();
    assert!(state.is_paused);

    // Resume the bridge
    client.resume_bridge(&admin);
    assert!(!client.is_bridge_paused());

    // Pause specific chains
    let chains = vec![&env, 1u32, 2u32];
    client.pause_chains(&admin, &chains, &bytes(&env, b"Maintenance"));
    assert!(client.is_chain_paused(&1));
    assert!(client.is_chain_paused(&2));
    assert!(!client.is_chain_paused(&3));

    // Resume chains
    client.resume_chains(&admin, &chains);
    assert!(!client.is_chain_paused(&1));
    assert!(!client.is_chain_paused(&2));
}

// ---------------------------------------------------------------------------
// Workflow 8: Atomic swap – initiate and accept
// ---------------------------------------------------------------------------

#[test]
fn e2e_atomic_swap_workflow() {
    let env = Env::default();
    env.mock_all_auths();
    set_timestamp(&env, 8_000);

    let client = setup(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(&token, &admin, &1, &Address::generate(&env));

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);

    let hashlock = bytes(&env, b"sha256_hashlock_value");
    let timelock = 8_000 + 3_600; // 1 hour from now

    // Alice initiates swap
    let swap_id = client
        .initiate_atomic_swap(
            &alice,
            &token_a,
            &1_000,
            &bob,
            &token_b,
            &2_000,
            &hashlock,
            &timelock,
        )
        .unwrap();

    let swap = client.get_atomic_swap(&swap_id).unwrap();
    assert_eq!(swap.initiator, alice);
    assert_eq!(swap.counterparty, bob);
    assert_eq!(swap.initiator_amount, 1_000);
    assert_eq!(swap.counterparty_amount, 2_000);

    // Active swaps list includes this swap
    let active = client.get_active_atomic_swaps();
    assert!(active.contains(swap_id));

    // Bob accepts with preimage
    let preimage = bytes(&env, b"secret_preimage_value");
    client.accept_atomic_swap(&swap_id, &bob, &preimage).unwrap();

    let completed = client.get_atomic_swap(&swap_id).unwrap();
    assert_eq!(completed.status, teachlink_contract::SwapStatus::Completed);
}

// ---------------------------------------------------------------------------
// Workflow 9: Audit trail – create records and compliance report
// ---------------------------------------------------------------------------

#[test]
fn e2e_audit_and_compliance_workflow() {
    let env = Env::default();
    env.mock_all_auths();
    set_timestamp(&env, 9_000);

    let client = setup(&env);
    let operator = Address::generate(&env);

    // Create audit records
    let record_id = client
        .create_audit_record(
            &teachlink_contract::OperationType::BridgeOut,
            &operator,
            &bytes(&env, b"Bridge out 500 tokens to chain 2"),
            &bytes(&env, b"0xdeadbeef"),
        )
        .unwrap();
    assert_eq!(record_id, 1);

    let record = client.get_audit_record(&record_id).unwrap();
    assert_eq!(record.operator, operator);

    // Second record
    client
        .create_audit_record(
            &teachlink_contract::OperationType::BridgeIn,
            &operator,
            &bytes(&env, b"Bridge in 300 tokens from chain 3"),
            &bytes(&env, b"0xcafebabe"),
        )
        .unwrap();

    // Generate compliance report covering the period
    let report_id = client
        .generate_compliance_report(&0, &10_000)
        .unwrap();
    assert_eq!(report_id, 1);

    let report = client.get_compliance_report(&report_id).unwrap();
    assert_eq!(report.period_start, 0);
    assert_eq!(report.period_end, 10_000);
    assert!(report.total_transactions >= 2);
}

// ---------------------------------------------------------------------------
// Workflow 10: Interface versioning – set, query, compatibility check
// ---------------------------------------------------------------------------

#[test]
fn e2e_interface_versioning_workflow() {
    let env = Env::default();
    env.mock_all_auths();
    set_timestamp(&env, 10_000);

    let client = setup(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(&token, &admin, &1, &Address::generate(&env));

    // Read initial version (set during initialize)
    let status = client.get_interface_version_status();
    let current = client.get_interface_version();
    let min_compat = client.get_min_compat_interface_version();

    assert_eq!(status.current, current);
    assert_eq!(status.minimum_compatible, min_compat);

    // Update versions
    let new_current = ContractSemVer { major: 2, minor: 0, patch: 0 };
    let new_min = ContractSemVer { major: 1, minor: 5, patch: 0 };
    client.set_interface_version(&new_current, &new_min).unwrap();

    assert_eq!(client.get_interface_version(), new_current);
    assert_eq!(client.get_min_compat_interface_version(), new_min);

    // Compatible client version
    let compatible = ContractSemVer { major: 1, minor: 5, patch: 0 };
    assert!(client.is_interface_compatible(&compatible));

    // Incompatible client version (too old)
    let incompatible = ContractSemVer { major: 1, minor: 0, patch: 0 };
    assert!(!client.is_interface_compatible(&incompatible));
}

// ---------------------------------------------------------------------------
// Workflow 11: Multi-token content minting – output validation
// ---------------------------------------------------------------------------

#[test]
fn e2e_multi_content_token_output_validation() {
    let env = Env::default();
    env.mock_all_auths();
    set_timestamp(&env, 11_000);

    let client = setup(&env);
    let creator = Address::generate(&env);

    let types = [
        ContentType::Course,
        ContentType::Video,
        ContentType::Article,
        ContentType::Quiz,
    ];

    for (i, ct) in types.iter().enumerate() {
        let params = ContentTokenParameters {
            creator: creator.clone(),
            title: bytes(&env, format!("Content {i}").as_bytes()),
            description: bytes(&env, b"desc"),
            content_type: ct.clone(),
            content_hash: bytes(&env, format!("hash_{i}").as_bytes()),
            license_type: bytes(&env, b"MIT"),
            tags: Vec::new(&env),
            is_transferable: true,
            royalty_percentage: 0,
        };
        let id = client.mint_content_token(&params);
        assert_eq!(id, (i as u64) + 1);
    }

    assert_eq!(client.get_content_token_count(), 4);

    let creator_tokens = client.get_owner_content_tokens(&creator);
    assert_eq!(creator_tokens.len(), 4);
}

// ---------------------------------------------------------------------------
// Workflow 12: Backup and disaster recovery
// ---------------------------------------------------------------------------

#[test]
fn e2e_backup_and_recovery_workflow() {
    let env = Env::default();
    env.mock_all_auths();
    set_timestamp(&env, 12_000);

    let client = setup(&env);
    let operator = Address::generate(&env);

    // Create a backup
    let backup_id = client
        .create_backup(
            &operator,
            &bytes(&env, b"sha256_integrity_hash"),
            &teachlink_contract::RtoTier::Tier1,
            &0,
        )
        .unwrap();
    assert_eq!(backup_id, 1);

    let manifest = client.get_backup_manifest(&backup_id).unwrap();
    assert_eq!(manifest.created_by, operator);

    // Verify backup integrity
    let valid = client
        .verify_backup(&backup_id, &operator, &bytes(&env, b"sha256_integrity_hash"))
        .unwrap();
    assert!(valid);

    // Wrong hash should fail verification
    let invalid = client
        .verify_backup(&backup_id, &operator, &bytes(&env, b"wrong_hash"))
        .unwrap();
    assert!(!invalid);

    // Schedule a backup
    let schedule_id = client
        .schedule_backup(
            &operator,
            &(12_000 + 86_400),
            &86_400,
            &teachlink_contract::RtoTier::Tier2,
        )
        .unwrap();
    assert_eq!(schedule_id, 1);

    let schedules = client.get_scheduled_backups(&operator);
    assert_eq!(schedules.len(), 1);

    // Record a recovery
    let recovery_id = client
        .record_recovery(&backup_id, &operator, &120, &true)
        .unwrap();
    assert_eq!(recovery_id, 1);

    let records = client.get_recovery_records(&10);
    assert_eq!(records.len(), 1);
    assert!(records.get(0).unwrap().success);
}
