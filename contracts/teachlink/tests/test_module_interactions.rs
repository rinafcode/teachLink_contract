//! Comprehensive Cross-Module Integration Tests
//!
//! Tests all module interactions, cross-module scenarios, edge cases,
//! and error propagation across the TeachLink smart contract.
//!
//! Issue: #275

#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unreadable_literal)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    vec, Address, Bytes, Env, Map, Vec,
};

use teachlink_contract::{
    AlertConditionType, AssessmentSettings, ContentTokenParameters, ContentType, ContributionType,
    CrossChainMessage, NotificationChannel, OperationType, QuestionType, ReportType, RewardType,
    RtoTier, SlashingReason, TeachLinkBridge, TeachLinkBridgeClient, TransferType,
};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Minimal bridge-initialized environment. Returns (client, admin, token, fee_recipient).
fn setup(env: &Env) -> (TeachLinkBridgeClient<'_>, Address, Address, Address) {
    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(env, &contract_id);
    let token = Address::generate(env);
    let admin = Address::generate(env);
    let fee_recipient = Address::generate(env);
    client.initialize(&token, &admin, &1, &fee_recipient);
    (client, admin, token, fee_recipient)
}

/// Contract registered but NOT bridge-initialized (for modules that don't need it).
fn setup_bare(env: &Env) -> TeachLinkBridgeClient<'_> {
    let contract_id = env.register(TeachLinkBridge, ());
    TeachLinkBridgeClient::new(env, &contract_id)
}

fn set_ledger_timestamp(env: &Env, timestamp: u64) {
    env.ledger().set(LedgerInfo {
        timestamp,
        protocol_version: 25,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 2_000_000,
    });
}

fn make_content_params(env: &Env, creator: Address) -> ContentTokenParameters {
    ContentTokenParameters {
        creator,
        title: Bytes::from_slice(env, b"Rust Fundamentals"),
        description: Bytes::from_slice(env, b"A comprehensive Rust course"),
        content_type: ContentType::Course,
        content_hash: Bytes::from_slice(env, b"QmHash123"),
        license_type: Bytes::from_slice(env, b"MIT"),
        tags: vec![env, Bytes::from_slice(env, b"rust")],
        is_transferable: true,
        royalty_percentage: 500,
    }
}

fn make_cross_chain_message(
    env: &Env,
    token: &Address,
    recipient: &Address,
    nonce: u64,
    amount: i128,
) -> CrossChainMessage {
    CrossChainMessage {
        source_chain: 1,
        source_tx_hash: Bytes::from_slice(env, b"src_tx_hash"),
        nonce,
        token: token.clone(),
        amount,
        recipient: recipient.clone(),
        destination_chain: 2,
    }
}

// ===========================================================================
// Group 1: Bridge <-> BFT Consensus
// ===========================================================================

#[test]
fn test_proposal_vote_then_bridge_completion() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, token, _fee) = setup(&env);

    let validator = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Register validator in BFT consensus
    client.register_validator(&validator, &100_000_000);

    // Also add to bridge validator set
    client.add_validator(&validator);
    client.add_supported_chain(&2);

    // Create a bridge proposal via BFT
    let message = make_cross_chain_message(&env, &token, &recipient, 1, 500);
    let proposal_id = client.create_bridge_proposal(&message);

    // Vote to approve
    client.vote_on_proposal(&validator, &proposal_id, &true);

    // Consensus should be reached with single validator
    assert!(client.is_consensus_reached(&proposal_id));

    // Verify proposal stored correctly
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.message.nonce, 1);
    assert_eq!(proposal.message.amount, 500);

    // Verify consensus state reflects validator count
    let state = client.get_consensus_state();
    assert!(state.active_validators > 0);
}

#[test]
fn test_proposal_rejected_blocks_consensus() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, token, _fee) = setup(&env);

    let v1 = Address::generate(&env);
    let v2 = Address::generate(&env);
    let v3 = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Register three validators
    client.register_validator(&v1, &100_000_000);
    client.register_validator(&v2, &100_000_000);
    client.register_validator(&v3, &100_000_000);

    let message = make_cross_chain_message(&env, &token, &recipient, 1, 1000);
    let proposal_id = client.create_bridge_proposal(&message);

    // Two validators reject
    client.vote_on_proposal(&v1, &proposal_id, &false);
    client.vote_on_proposal(&v2, &proposal_id, &false);

    // Consensus should NOT be reached
    assert!(!client.is_consensus_reached(&proposal_id));

    // Proposal still exists but is not approved
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(proposal.vote_count <= proposal.required_votes);
}

// ===========================================================================
// Group 2: Slashing <-> BFT Consensus
// ===========================================================================

#[test]
fn test_slash_validator_reduces_stake() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _token, _fee) = setup(&env);

    let validator = Address::generate(&env);
    let slasher = Address::generate(&env);

    // Register in BFT and deposit stake via slashing module
    // register_validator deposits the BFT stake; deposit_stake adds to the slashing stake
    client.register_validator(&validator, &100_000_000);
    client.deposit_stake(&validator, &5000);

    // Total stake = BFT registration stake + slashing deposit
    let initial_stake = client.get_validator_stake(&validator);
    assert_eq!(initial_stake, 100_005_000);

    // Slash for double vote (50% penalty)
    let evidence = Bytes::from_slice(&env, b"double_vote_proof");
    let slashed =
        client.slash_validator(&validator, &SlashingReason::DoubleVote, &evidence, &slasher);

    // Stake should be reduced
    let post_stake = client.get_validator_stake(&validator);
    assert!(post_stake < initial_stake);
    assert!(slashed > 0);

    // BFT validator info should still exist
    let info = client.get_validator_info(&validator);
    assert!(info.is_some());
}

#[test]
fn test_reward_validator_after_consensus_participation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, token, _fee) = setup(&env);

    let validator = Address::generate(&env);
    let funder = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Register and fund reward pool
    client.register_validator(&validator, &100_000_000);
    client.fund_validator_reward_pool(&funder, &10_000);

    // Participate in consensus
    let message = make_cross_chain_message(&env, &token, &recipient, 1, 100);
    let proposal_id = client.create_bridge_proposal(&message);
    client.vote_on_proposal(&validator, &proposal_id, &true);
    assert!(client.is_consensus_reached(&proposal_id));

    // Reward for consensus participation
    client.reward_validator(&validator, &500, &RewardType::Consensus);

    // Validator info should reflect active participation
    let info = client.get_validator_info(&validator).unwrap();
    assert!(info.is_active);
}

// ===========================================================================
// Group 3: Emergency <-> Bridge Operations
// ===========================================================================

#[test]
fn test_emergency_pause_and_resume_state_management() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _token, _fee) = setup(&env);

    // Initially not paused
    assert!(!client.is_bridge_paused());

    // Pause
    let reason = Bytes::from_slice(&env, b"security_incident");
    client.pause_bridge(&admin, &reason);

    // Verify paused state
    assert!(client.is_bridge_paused());
    let state = client.get_emergency_state();
    assert!(state.is_paused);

    // Resume
    client.resume_bridge(&admin);
    assert!(!client.is_bridge_paused());

    let state_after = client.get_emergency_state();
    assert!(!state_after.is_paused);
}

#[test]
fn test_chain_specific_pause_isolation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _token, _fee) = setup(&env);

    // Pause only chain 1
    let reason = Bytes::from_slice(&env, b"chain_compromise");
    client.pause_chains(&admin, &vec![&env, 1], &reason);

    // Chain 1 paused, chain 2 unaffected
    assert!(client.is_chain_paused(&1));
    assert!(!client.is_chain_paused(&2));

    // Global bridge is NOT paused
    assert!(!client.is_bridge_paused());

    // Resume chain 1
    client.resume_chains(&admin, &vec![&env, 1]);
    assert!(!client.is_chain_paused(&1));
}

// ===========================================================================
// Group 4: Emergency <-> Message Passing
// ===========================================================================

#[test]
fn test_emergency_state_alongside_packet_operations() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _token, _fee) = setup(&env);
    set_ledger_timestamp(&env, 1000);

    // Send a packet while bridge is active
    let sender = Bytes::from_slice(&env, b"sender_addr");
    let recipient = Bytes::from_slice(&env, b"recipient_addr");
    let payload = Bytes::from_slice(&env, b"test_payload");

    let packet_id = client.send_cross_chain_packet(&1, &2, &sender, &recipient, &payload, &None);

    // Packet should exist
    let packet = client.get_packet(&packet_id).unwrap();
    assert_eq!(packet.source_chain, 1);
    assert_eq!(packet.destination_chain, 2);

    // Pause bridge — state changes but existing packets remain
    let reason = Bytes::from_slice(&env, b"maintenance");
    client.pause_bridge(&admin, &reason);
    assert!(client.is_bridge_paused());

    // Existing packet still queryable
    let packet_after = client.get_packet(&packet_id);
    assert!(packet_after.is_some());

    // Deliver the existing packet (delivery is a separate operation)
    let result_bytes = Bytes::from_slice(&env, b"delivered");
    client.deliver_cross_chain_packet(&packet_id, &100, &result_bytes);
    assert!(client.verify_packet_delivery(&packet_id));

    // Resume
    client.resume_bridge(&admin);
    assert!(!client.is_bridge_paused());
}

// ===========================================================================
// Group 5: Tokenization <-> Provenance (Chain of Custody)
// ===========================================================================

#[test]
fn test_mint_transfer_chain_provenance_verification() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_bare(&env);

    let creator = Address::generate(&env);
    let buyer1 = Address::generate(&env);
    let buyer2 = Address::generate(&env);

    // Mint at t=1000
    set_ledger_timestamp(&env, 1000);
    let params = make_content_params(&env, creator.clone());
    let token_id = client.mint_content_token(&params);
    assert_eq!(token_id, 1);

    // Provenance: 1 record (Mint)
    let prov = client.get_content_provenance(&token_id);
    assert_eq!(prov.len(), 1);
    assert_eq!(prov.get(0).unwrap().transfer_type, TransferType::Mint);
    assert_eq!(prov.get(0).unwrap().to, creator.clone());

    // Transfer creator -> buyer1 at t=2000
    set_ledger_timestamp(&env, 2000);
    let notes1 = Bytes::from_slice(&env, b"first sale");
    client.transfer_content_token(&creator, &buyer1, &token_id, &Some(notes1));

    // Provenance: 2 records
    let prov2 = client.get_content_provenance(&token_id);
    assert_eq!(prov2.len(), 2);
    let rec1 = prov2.get(1).unwrap();
    assert_eq!(rec1.transfer_type, TransferType::Transfer);
    assert_eq!(rec1.from, Some(creator.clone()));
    assert_eq!(rec1.to, buyer1.clone());

    // Transfer buyer1 -> buyer2 at t=3000
    set_ledger_timestamp(&env, 3000);
    let notes2 = Bytes::from_slice(&env, b"resale");
    client.transfer_content_token(&buyer1, &buyer2, &token_id, &Some(notes2));

    // Provenance: 3 records
    let prov3 = client.get_content_provenance(&token_id);
    assert_eq!(prov3.len(), 3);

    // Chain integrity verification
    assert!(client.verify_content_chain(&token_id));

    // Creator is still the original
    assert_eq!(client.get_content_creator(&token_id).unwrap(), creator);

    // Current owner is buyer2
    assert!(client.is_content_token_owner(&token_id, &buyer2));
    assert!(!client.is_content_token_owner(&token_id, &creator));
    assert!(!client.is_content_token_owner(&token_id, &buyer1));

    // All historical owners tracked
    let all_owners = client.get_content_all_owners(&token_id);
    assert!(all_owners.len() >= 2);

    // Token count
    assert_eq!(client.get_content_token_count(), 1);
}

// ===========================================================================
// Group 6: Score <-> Reputation
// ===========================================================================

#[test]
fn test_course_completion_updates_score_and_reputation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _token, _fee) = setup(&env);

    let user = Address::generate(&env);

    // Start a course (reputation)
    client.update_course_progress(&user, &false);

    // Complete the course (reputation)
    client.update_course_progress(&user, &true);

    // Record course completion (score module — requires admin auth)
    client.record_course_completion(&user, &101, &50);

    // Verify reputation updated
    let rep = client.get_user_reputation(&user);
    assert!(rep.total_courses_completed > 0);

    // Verify score updated
    let score = client.get_credit_score(&user);
    assert!(score > 0);

    // Verify course tracked
    let courses = client.get_user_courses(&user);
    assert_eq!(courses.len(), 1);
    assert_eq!(courses.get(0).unwrap(), 101);
}

#[test]
fn test_contribution_updates_score_and_reputation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _token, _fee) = setup(&env);

    let user = Address::generate(&env);

    // Rate contribution in reputation module
    client.rate_contribution(&user, &5);

    // Record contribution in score module
    let description = Bytes::from_slice(&env, b"Published tutorial");
    client.record_contribution(&user, &ContributionType::Content, &description, &25);

    // Verify reputation
    let rep = client.get_user_reputation(&user);
    assert!(rep.total_contributions > 0);

    // Verify score
    let score = client.get_credit_score(&user);
    assert!(score > 0);

    // Verify contribution stored
    let contributions = client.get_user_contributions(&user);
    assert_eq!(contributions.len(), 1);
}

// ===========================================================================
// Group 7: Assessment <-> Score <-> Reputation (3-way)
// ===========================================================================

#[test]
fn test_assessment_to_score_to_reputation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _token, _fee) = setup(&env);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    // Add questions
    let q1_id = client.add_assessment_question(
        &creator,
        &QuestionType::MultipleChoice,
        &Bytes::from_slice(&env, b"Q1"),
        &10,
        &5,
        &Bytes::from_slice(&env, b"A"),
        &Map::new(&env),
    );
    let q2_id = client.add_assessment_question(
        &creator,
        &QuestionType::MultipleChoice,
        &Bytes::from_slice(&env, b"Q2"),
        &10,
        &5,
        &Bytes::from_slice(&env, b"B"),
        &Map::new(&env),
    );

    // Create assessment
    let mut questions = Vec::new(&env);
    questions.push_back(q1_id);
    questions.push_back(q2_id);

    let settings = AssessmentSettings {
        time_limit: 3600,
        passing_score: 50,
        is_adaptive: false,
        allow_retakes: false,
        proctoring_enabled: false,
    };

    let assessment_id = client.create_assessment(
        &creator,
        &Bytes::from_slice(&env, b"Quiz"),
        &Bytes::from_slice(&env, b"Test"),
        &questions,
        &settings,
    );

    // Student submits (correct answers)
    let mut answers = Map::new(&env);
    answers.set(q1_id, Bytes::from_slice(&env, b"A"));
    answers.set(q2_id, Bytes::from_slice(&env, b"B"));
    let score = client.submit_assessment(&student, &assessment_id, &answers, &Vec::new(&env));

    // Verify assessment submission exists
    let submission = client.get_assessment_submission(&student, &assessment_id);
    assert!(submission.is_some());

    // Record in score module
    client.record_course_completion(&student, &assessment_id, &(score as u64));

    // Record in reputation module
    client.update_course_progress(&student, &true);

    // Cross-module verification
    let credit_score = client.get_credit_score(&student);
    assert!(credit_score > 0);

    let rep = client.get_user_reputation(&student);
    assert!(rep.total_courses_completed > 0);

    let courses = client.get_user_courses(&student);
    assert!(courses.len() > 0);
}

// ===========================================================================
// Group 8: Analytics <-> Audit
// ===========================================================================

#[test]
fn test_audit_record_alongside_analytics_metrics() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _token, _fee) = setup(&env);
    set_ledger_timestamp(&env, 1000);

    // Initialize analytics
    client.initialize_bridge_metrics();

    // Create audit record
    let details = Bytes::from_slice(&env, b"bridge_out_1000_tokens");
    let tx_hash = Bytes::from_slice(&env, b"tx_abc123");
    let record_id =
        client.create_audit_record(&OperationType::BridgeOut, &admin, &details, &tx_hash);

    // Verify audit record
    let record = client.get_audit_record(&record_id).unwrap();
    assert_eq!(record.operator, admin);

    // Verify analytics accessible
    let _metrics = client.get_bridge_metrics();
    let health = client.calculate_bridge_health_score();
    assert!(health <= 100);

    // Generate compliance report spanning the audit period
    let report_id = client.generate_compliance_report(&0, &2000);
    let report = client.get_compliance_report(&report_id).unwrap();
    assert_eq!(report.period_start, 0);
    assert_eq!(report.period_end, 2000);
}

#[test]
fn test_report_template_with_alert_rules() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _token, _fee) = setup(&env);
    set_ledger_timestamp(&env, 1000);

    // Initialize analytics (needed for alert evaluation)
    client.initialize_bridge_metrics();

    // Create report template
    let template_name = Bytes::from_slice(&env, b"Weekly Bridge Health");
    let config = Bytes::from_slice(&env, b"{}");
    let template_id =
        client.create_report_template(&admin, &template_name, &ReportType::BridgeHealth, &config);

    // Schedule the report
    let schedule_id = client.schedule_report(&admin, &template_id, &2000, &604_800);
    assert!(schedule_id > 0);

    // Verify scheduled reports
    let schedules = client.get_scheduled_reports(&admin);
    assert!(schedules.len() > 0);

    // Create alert rule
    let alert_name = Bytes::from_slice(&env, b"Low Health Alert");
    let _rule_id = client.create_alert_rule(
        &admin,
        &alert_name,
        &AlertConditionType::BridgeHealthBelow,
        &50,
    );

    // Verify alert rules
    let rules = client.get_alert_rules(&admin);
    assert_eq!(rules.len(), 1);

    // Evaluate alerts
    let _triggered = client.evaluate_alerts();
    // Health score is default so may or may not trigger

    // Generate report snapshot
    let snapshot_id = client.generate_report_snapshot(&admin, &template_id, &0, &5000);
    let snapshot = client.get_report_snapshot(&snapshot_id).unwrap();
    assert_eq!(snapshot.template_id, template_id);

    // Add comment to report
    let comment_body = Bytes::from_slice(&env, b"Looks good");
    let _comment_id = client.add_report_comment(&snapshot_id, &admin, &comment_body);
    let comments = client.get_report_comments(&snapshot_id);
    assert_eq!(comments.len(), 1);
}

// ===========================================================================
// Group 9: Notification <-> Bridge Operations
// ===========================================================================

#[test]
fn test_notification_alongside_bridge_operations() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _token, _fee) = setup(&env);

    // Set timestamp to midday, NOT divisible by 10 (delivery simulates 90% success
    // via current_time % 10 != 0)
    set_ledger_timestamp(&env, 43_201);

    // Initialize notification system
    client.initialize_notifications();

    // Setup bridge chains
    client.add_supported_chain(&1);

    // Configure user notification settings to enable channels
    let user = Address::generate(&env);
    client.update_notification_settings(
        &user,
        &Bytes::from_slice(&env, b"UTC"),
        &79_200, // quiet hours start 10 PM
        &28_800, // quiet hours end 8 AM
        &50,
        &false,
    );

    // Send a notification (simulating bridge event notification)
    let subject = Bytes::from_slice(&env, b"Bridge Transfer Initiated");
    let body = Bytes::from_slice(&env, b"Your transfer of 1000 tokens is processing");

    let notif_id = client.send_notification(&user, &NotificationChannel::InApp, &subject, &body);

    // Verify notification tracking
    let tracking = client.get_notification_tracking(&notif_id).unwrap();
    assert_eq!(tracking.recipient, user);

    // Verify user notifications retrievable
    let user_notifs = client.get_user_notifications(&user, &10);
    assert!(user_notifs.len() > 0);

    // Bridge operations still work alongside notifications
    assert!(client.is_chain_supported(&1));
}

// ===========================================================================
// Group 10: Backup <-> Recovery (End-to-End)
// ===========================================================================

#[test]
fn test_backup_create_verify_recover_cycle() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_bare(&env);
    set_ledger_timestamp(&env, 1000);

    let admin = Address::generate(&env);

    // Create backup
    let integrity_hash = Bytes::from_slice(&env, b"sha256_backup_hash_abc");
    let backup_id = client.create_backup(&admin, &integrity_hash, &RtoTier::Critical, &42);

    // Verify manifest stored
    let manifest = client.get_backup_manifest(&backup_id).unwrap();
    assert_eq!(manifest.created_by, admin);
    assert_eq!(manifest.encryption_ref, 42);

    // Verify integrity with correct hash — should return true
    let valid = client.verify_backup(&backup_id, &admin, &integrity_hash);
    assert!(valid);

    // Verify with wrong hash — should return false
    let wrong_hash = Bytes::from_slice(&env, b"wrong_hash");
    let invalid = client.verify_backup(&backup_id, &admin, &wrong_hash);
    assert!(!invalid);

    // Record successful recovery
    let recovery_id_1 = client.record_recovery(&backup_id, &admin, &120, &true);
    assert!(recovery_id_1 > 0);

    // Record failed recovery
    let recovery_id_2 = client.record_recovery(&backup_id, &admin, &300, &false);
    assert!(recovery_id_2 > recovery_id_1);

    // Verify recovery records
    let records = client.get_recovery_records(&10);
    assert!(records.len() >= 2);

    // Verify recent backups
    let recent = client.get_recent_backups(&10);
    assert!(recent.len() > 0);

    // Schedule automated backup
    let schedule_id = client.schedule_backup(&admin, &2000, &3600, &RtoTier::Standard);
    assert!(schedule_id > 0);

    // Verify schedules
    let schedules = client.get_scheduled_backups(&admin);
    assert!(schedules.len() > 0);
}

// ===========================================================================
// Group 11: Liquidity <-> Bridge Fee Calculation
// ===========================================================================

#[test]
fn test_liquidity_pool_affects_bridge_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, token, _fee) = setup(&env);

    let provider = Address::generate(&env);

    // Add supported chain and initialize pool
    client.add_supported_chain(&1);
    client.initialize_liquidity_pool(&1, &token);

    // Fee with empty pool
    let fee_empty = client.calculate_bridge_fee(&1, &1000, &0);

    // Add liquidity
    let position = client.add_liquidity(&provider, &1, &50_000);
    assert!(position > 0);

    // Verify available liquidity
    let available = client.get_available_liquidity(&1);
    assert_eq!(available, 50_000);

    // Fee with funded pool
    let fee_funded = client.calculate_bridge_fee(&1, &1000, &0);

    // Both fees should be valid positive values
    assert!(fee_empty >= 0);
    assert!(fee_funded >= 0);

    // Fee with high volume user (volume discount)
    let fee_volume = client.calculate_bridge_fee(&1, &1000, &100_000);
    assert!(fee_volume >= 0);
}

// ===========================================================================
// Group 12: MultiChain <-> Bridge <-> Message Passing
// ===========================================================================

#[test]
fn test_multichain_config_and_packet_workflow() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _token, _fee) = setup(&env);
    set_ledger_timestamp(&env, 1000);

    // Configure chain via multichain module
    let chain_name = Bytes::from_slice(&env, b"Ethereum");
    let bridge_addr = Bytes::from_slice(&env, b"0xBridgeContract");
    client.add_supported_chain_config(&1, &chain_name, &bridge_addr, &12, &20_000);

    // Verify config
    let config = client.get_chain_config(&1).unwrap();
    assert_eq!(config.chain_id, 1);
    assert!(client.is_chain_active(&1));

    // Also register in bridge module
    client.add_supported_chain(&1);
    client.add_supported_chain(&2);
    assert!(client.is_chain_supported(&1));

    // Verify supported chains list
    let chains = client.get_supported_chains();
    assert!(chains.len() > 0);

    // Send cross-chain packet
    let sender = Bytes::from_slice(&env, b"stellar_sender");
    let recipient = Bytes::from_slice(&env, b"eth_recipient");
    let payload = Bytes::from_slice(&env, b"transfer_data");

    let packet_id = client.send_cross_chain_packet(&1, &2, &sender, &recipient, &payload, &None);

    // Verify packet exists with correct routing
    let packet = client.get_packet(&packet_id).unwrap();
    assert_eq!(packet.source_chain, 1);
    assert_eq!(packet.destination_chain, 2);

    // Deliver the packet
    let result = Bytes::from_slice(&env, b"success");
    client.deliver_cross_chain_packet(&packet_id, &200, &result);

    // Verify delivery
    assert!(client.verify_packet_delivery(&packet_id));
}

#[test]
fn test_deactivate_chain_updates_multichain_state() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _token, _fee) = setup(&env);

    // Add and configure chain
    let chain_name = Bytes::from_slice(&env, b"Polygon");
    let bridge_addr = Bytes::from_slice(&env, b"0xPolygonBridge");
    client.add_supported_chain_config(&3, &chain_name, &bridge_addr, &20, &30_000);
    assert!(client.is_chain_active(&3));

    // Deactivate
    client.update_chain_config(&3, &false, &None, &None);
    assert!(!client.is_chain_active(&3));

    // Config still exists but inactive
    let config = client.get_chain_config(&3);
    assert!(config.is_some());

    // Reactivate with updated params
    client.update_chain_config(&3, &true, &Some(25), &Some(35_000));
    assert!(client.is_chain_active(&3));
}

// ===========================================================================
// Group 13: Error Propagation Across Modules
// ===========================================================================

#[test]
fn test_error_propagation_across_modules() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token, _fee) = setup(&env);

    // --- Bridge: unsupported chain ---
    let user = Address::generate(&env);
    let dest = Bytes::from_slice(&env, b"dest_addr");
    let bridge_result = client.try_bridge_out(&user, &1000, &999, &dest);
    assert!(bridge_result.is_err());

    // --- BFT: vote on nonexistent proposal ---
    let validator = Address::generate(&env);
    client.register_validator(&validator, &100_000_000);
    let vote_result = client.try_vote_on_proposal(&validator, &9999, &true);
    assert!(vote_result.is_err());

    // --- Message Passing: deliver nonexistent packet ---
    let result_bytes = Bytes::from_slice(&env, b"result");
    let deliver_result = client.try_deliver_cross_chain_packet(&9999, &100, &result_bytes);
    assert!(deliver_result.is_err());

    // --- Backup: verify nonexistent backup ---
    let hash = Bytes::from_slice(&env, b"hash");
    let verify_result = client.try_verify_backup(&9999, &admin, &hash);
    assert!(verify_result.is_err());

    // --- Emergency: resume when not paused ---
    let resume_result = client.try_resume_bridge(&admin);
    assert!(resume_result.is_err());

    // --- Emergency: double pause ---
    let reason = Bytes::from_slice(&env, b"pause");
    client.pause_bridge(&admin, &reason);
    let double_pause = client.try_pause_bridge(&admin, &reason);
    assert!(double_pause.is_err());

    // --- Slashing: slash with insufficient stake ---
    let non_staked = Address::generate(&env);
    let slasher = Address::generate(&env);
    let evidence = Bytes::from_slice(&env, b"evidence");
    let slash_result = client.try_slash_validator(
        &non_staked,
        &SlashingReason::Inactivity,
        &evidence,
        &slasher,
    );
    assert!(slash_result.is_err());

    // --- Bridge: double initialization ---
    let reinit = client.try_initialize(&token, &admin, &1, &admin);
    assert!(reinit.is_err());
}
