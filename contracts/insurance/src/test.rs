#![cfg(test)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::ignore_without_reason)]
#![allow(clippy::unused_unit)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::too_many_lines)]
#![allow(unused_variables)]

use super::*;
use crate::types::*;
use soroban_sdk::{
    testutils::{Address as _, Bytes as _, Ledger, LedgerInfo},
    vec, Address, Bytes, Env,
};

fn setup_test_env() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let user = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = Address::generate(&env);

    // Setup token
    let token_client = token::StellarAssetClient::new(&env, &token_address);
    token_client.initialize(&token_admin, &7, &"TestToken".into_val(&env), &"TEST".into_val(&env));
    token_client.mint(&admin, &1_000_000_000);
    token_client.mint(&user, &100_000_000);

    // Setup contract
    let contract_id = env.register(EnhancedInsurance, ());
    
    // Set ledger
    env.ledger().set(LedgerInfo {
        timestamp: 1000000,
        protocol_version: 25,
        sequence_number: 100,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1000,
        min_persistent_entry_ttl: 1000,
        max_entry_ttl: 2000000,
    });

    (env, admin, oracle, user, token_address)
}

#[test]
fn test_initialize_contract() {
    let (env, admin, oracle, _user, token_address) = setup_test_env();
    let client = EnhancedInsuranceClient::new(&env, &env.register(EnhancedInsurance, ()));

    // Initialize contract
    let result = client.try_initialize(&admin, &oracle, &token_address);
    assert!(result.is_ok());

    // Try to initialize again - should fail
    let result = client.try_initialize(&admin, &oracle, &token_address);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), 500); // AlreadyInitialized
}

#[test]
fn test_create_risk_profile() {
    let (env, admin, oracle, user, token_address) = setup_test_env();
    let client = EnhancedInsuranceClient::new(&env, &env.register(EnhancedInsurance, ()));
    client.initialize(&admin, &oracle, &token_address);

    let factors = RiskFactors {
        completion_rate: 85,
        reputation_score: 90,
        course_difficulty: 5,
        course_duration: 20,
        experience_level: 2,
        claim_frequency: 2,
        time_since_last_completion: 86400 * 30, // 30 days
    };

    let result = client.try_create_risk_profile(&user, &factors);
    assert!(result.is_ok());
    
    let profile_id = result.unwrap();
    assert_eq!(profile_id, 1);

    // Get the profile
    let profile = client.get_risk_profile(&user);
    assert!(profile.is_some());
    let profile = profile.unwrap();
    assert_eq!(profile.profile_id, profile_id);
    assert_eq!(profile.user, user);
    assert_eq!(profile.factors.completion_rate, 85);
    assert!(profile.risk_score <= 100);
}

#[test]
fn test_invalid_risk_factors() {
    let (env, admin, oracle, user, token_address) = setup_test_env();
    let client = EnhancedInsuranceClient::new(&env, &env.register(EnhancedInsurance, ()));
    client.initialize(&admin, &oracle, &token_address);

    // Invalid completion rate (> 100)
    let invalid_factors = RiskFactors {
        completion_rate: 150,
        reputation_score: 90,
        course_difficulty: 5,
        course_duration: 20,
        experience_level: 2,
        claim_frequency: 2,
        time_since_last_completion: 86400 * 30,
    };

    let result = client.try_create_risk_profile(&user, &invalid_factors);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), 505); // InvalidRiskFactors
}

#[test]
fn test_purchase_policy() {
    let (env, admin, oracle, user, token_address) = setup_test_env();
    let client = EnhancedInsuranceClient::new(&env, &env.register(EnhancedInsurance, ()));
    let token_client = token::Client::new(&env, &token_address);
    
    client.initialize(&admin, &oracle, &token_address);

    // Create risk profile first
    let factors = RiskFactors {
        completion_rate: 85,
        reputation_score: 90,
        course_difficulty: 5,
        course_duration: 20,
        experience_level: 2,
        claim_frequency: 2,
        time_since_last_completion: 86400 * 30,
    };
    client.create_risk_profile(&user, &factors);

    // Check initial balance
    let initial_balance = token_client.balance(&user);
    let contract_balance = token_client.balance(&client.address);

    // Purchase policy
    let coverage_amount = 10000;
    let result = client.try_purchase_policy(&user, &101, &coverage_amount);
    assert!(result.is_ok());
    
    let policy_id = result.unwrap();
    assert_eq!(policy_id, 1);

    // Check balances after purchase
    let final_balance = token_client.balance(&user);
    let final_contract_balance = token_client.balance(&client.address);
    
    assert!(final_balance < initial_balance);
    assert!(final_contract_balance > contract_balance);

    // Get policy
    let policy = client.get_policy(&policy_id);
    assert!(policy.is_some());
    let policy = policy.unwrap();
    assert_eq!(policy.policy_id, policy_id);
    assert_eq!(policy.holder, user);
    assert_eq!(policy.course_id, 101);
    assert_eq!(policy.coverage_amount, coverage_amount);
    assert_eq!(policy.status, PolicyStatus::Active);
}

#[test]
fn test_file_claim() {
    let (env, admin, oracle, user, token_address) = setup_test_env();
    let client = EnhancedInsuranceClient::new(&env, &env.register(EnhancedInsurance, ()));
    let token_client = token::Client::new(&env, &token_address);
    
    client.initialize(&admin, &oracle, &token_address);

    // Setup: create profile and purchase policy
    let factors = RiskFactors {
        completion_rate: 85,
        reputation_score: 90,
        course_difficulty: 5,
        course_duration: 20,
        experience_level: 2,
        claim_frequency: 2,
        time_since_last_completion: 86400 * 30,
    };
    client.create_risk_profile(&user, &factors);
    
    let policy_id = client.purchase_policy(&user, &101, &10000);

    // File claim
    let evidence = Bytes::from_slice(&env, &[1u8; 32]);
    let reason = Bytes::from_slice(&env, b"Course completion failed due to technical issues");
    
    let result = client.try_file_claim(&user, &policy_id, &evidence, &reason);
    assert!(result.is_ok());
    
    let claim_id = result.unwrap();
    assert_eq!(claim_id, 1);

    // Get claim
    let claim = client.get_claim(&claim_id);
    assert!(claim.is_some());
    let claim = claim.unwrap();
    assert_eq!(claim.claim_id, claim_id);
    assert_eq!(claim.policy_id, policy_id);
    assert_eq!(claim.status, ClaimStatus::AiVerified); // High confidence
    assert_eq!(claim.ai_confidence, 75);
    assert_eq!(claim.evidence, evidence);
}

#[test]
fn test_parametric_trigger() {
    let (env, admin, oracle, user, token_address) = setup_test_env();
    let client = EnhancedInsuranceClient::new(&env, &env.register(EnhancedInsurance, ()));
    let token_client = token::Client::new(&env, &token_address);
    
    client.initialize(&admin, &oracle, &token_address);

    // Create parametric trigger
    let trigger_id = client.create_parametric_trigger(
        &admin,
        &101,
        &LearningMetric::CompletionPercentage,
        &80, // Threshold: 80%
        &5000, // Payout amount
    );

    // Check initial balances
    let user_balance = token_client.balance(&user);
    let contract_balance = token_client.balance(&client.address);

    // Execute trigger (completion < 80% should trigger)
    let result = client.try_execute_trigger(&trigger_id, &user, &75);
    assert!(result.is_ok());

    // Check balances after payout
    let final_user_balance = token_client.balance(&user);
    let final_contract_balance = token_client.balance(&client.address);
    
    assert_eq!(final_user_balance, user_balance + 5000);
    assert_eq!(final_contract_balance, contract_balance - 5000);

    // Try to execute again - should fail as trigger is deactivated
    let result = client.try_execute_trigger(&trigger_id, &user, &70);
    assert!(result.is_err());
}

#[test]
fn test_create_pool() {
    let (env, admin, oracle, _user, token_address) = setup_test_env();
    let client = EnhancedInsuranceClient::new(&env, &env.register(EnhancedInsurance, ()));
    
    client.initialize(&admin, &oracle, &token_address);

    let pool_id = client.create_pool(
        &admin,
        &Bytes::from_slice(&env, b"Learning Insurance Pool"),
        &8000, // 80% target utilization
        &1500, // 15% risk reserve
    );

    assert_eq!(pool_id, 1);

    // Get pool
    let pool = client.get_pool(&pool_id);
    assert!(pool.is_some());
    let pool = pool.unwrap();
    assert_eq!(pool.pool_id, pool_id);
    assert_eq!(pool.name, "Learning Insurance Pool");
    assert_eq!(pool.target_utilization, 8000);
    assert_eq!(pool.risk_reserve_ratio, 1500);
    assert_eq!(pool.status, PoolStatus::Active);

    // Check active pools
    let active_pools = client.get_active_pools();
    assert_eq!(active_pools.len(), 1);
    assert_eq!(active_pools.get(0), pool_id);
}

#[test]
fn test_add_reinsurance_partner() {
    let (env, admin, oracle, user, token_address) = setup_test_env();
    let client = EnhancedInsuranceClient::new(&env, &env.register(EnhancedInsurance, ()));
    
    client.initialize(&admin, &oracle, &token_address);
    
    let pool_id = client.create_pool(
        &admin,
        &Bytes::from_slice(&env, b"Test Pool"),
        &8000,
        &1500,
    );

    // Add reinsurance partner
    let result = client.try_add_reinsurance_partner(&admin, &pool_id, &user, &2000); // 20%
    assert!(result.is_ok());

    // Check pool has partner
    let pool = client.get_pool(&pool_id).unwrap();
    assert_eq!(pool.reinsurance_partners.len(), 2); // user + contract address
    assert_eq!(pool.reinsurance_partners.get(1), user);
}

#[test]
fn test_create_proposal() {
    let (env, admin, oracle, user, token_address) = setup_test_env();
    let client = EnhancedInsuranceClient::new(&env, &env.register(EnhancedInsurance, ()));
    let token_client = token::Client::new(&env, &token_address);
    
    client.initialize(&admin, &oracle, &token_address);

    // User needs tokens to create proposal
    token_client.transfer(&admin, &user, &2000);

    let proposal_id = client.create_proposal(
        &user,
        &Bytes::from_slice(&env, b"Increase Premium Rate"),
        &Bytes::from_slice(&env, b"Proposal to increase base premium rate to 1.5%"),
        &ProposalType::PremiumRate,
        &150, // 1.5% in basis points
    );

    assert_eq!(proposal_id, 1);

    // Get proposal
    let proposal = client.get_proposal(&proposal_id);
    assert!(proposal.is_some());
    let proposal = proposal.unwrap();
    assert_eq!(proposal.proposal_id, proposal_id);
    assert_eq!(proposal.title, "Increase Premium Rate");
    assert_eq!(proposal.proposal_type, ProposalType::PremiumRate);
    assert_eq!(proposal.new_value, 150);
    assert_eq!(proposal.status, ProposalStatus::Active);
}

#[test]
fn test_voting_process() {
    let (env, admin, oracle, user, token_address) = setup_test_env();
    let client = EnhancedInsuranceClient::new(&env, &env.register(EnhancedInsurance, ()));
    let token_client = token::Client::new(&env, &token_address);
    
    client.initialize(&admin, &oracle, &token_address);

    // Create proposal
    token_client.transfer(&admin, &user, &2000);
    let proposal_id = client.create_proposal(
        &user,
        &Bytes::from_slice(&env, b"Test Proposal"),
        &Bytes::from_slice(&env, b"Test description"),
        &ProposalType::PremiumRate,
        &150,
    );

    // Vote for proposal
    let result = client.try_vote(&user, &proposal_id, &true);
    assert!(result.is_ok());

    // Try to vote again - should fail
    let result = client.try_vote(&user, &proposal_id, &false);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), 528); // AlreadyVoted

    // Check proposal vote counts
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.votes_for, 1);
    assert_eq!(proposal.votes_against, 0);
}

#[test]
fn test_create_insurance_token() {
    let (env, admin, oracle, user, token_address) = setup_test_env();
    let client = EnhancedInsuranceClient::new(&env, &env.register(EnhancedInsurance, ()));
    
    client.initialize(&admin, &oracle, &token_address);
    
    let pool_id = client.create_pool(
        &admin,
        &Bytes::from_slice(&env, b"Test Pool"),
        &8000,
        &1500,
    );

    let token_id = client.create_insurance_token(
        &admin,
        &pool_id,
        &Bytes::from_slice(&env, b"Insurance Pool Token"),
        &Bytes::from_slice(&env, b"IPT"),
        &1000000,
    );

    assert_eq!(token_id, 1);

    // Get token
    let token = client.get_insurance_token(&token_id);
    assert!(token.is_some());
    let token = token.unwrap();
    assert_eq!(token.token_id, token_id);
    assert_eq!(token.pool_id, pool_id);
    assert_eq!(token.name, "Insurance Pool Token");
    assert_eq!(token.symbol, "IPT");
    assert_eq!(token.total_supply, 1000000);
    assert_eq!(token.holder, admin);

    // Check balance
    let balance = client.get_token_balance(&admin, &token_id);
    assert_eq!(balance, 1000000);
}

#[test]
fn test_token_transfer() {
    let (env, admin, oracle, user, token_address) = setup_test_env();
    let client = EnhancedInsuranceClient::new(&env, &env.register(EnhancedInsurance, ()));
    
    client.initialize(&admin, &oracle, &token_address);
    
    let pool_id = client.create_pool(&admin, &Bytes::from_slice(&env, b"Test Pool"), &8000, &1500);
    let token_id = client.create_insurance_token(&admin, &pool_id, &Bytes::from_slice(&env, b"IPT"), &Bytes::from_slice(&env, b"IPT"), &1000000);

    // Transfer tokens
    let result = client.try_transfer_tokens(&admin, &user, &token_id, &100000);
    assert!(result.is_ok());

    // Check balances
    let admin_balance = client.get_token_balance(&admin, &token_id);
    let user_balance = client.get_token_balance(&user, &token_id);
    
    assert_eq!(admin_balance, 900000);
    assert_eq!(user_balance, 100000);
}

#[test]
fn test_compliance_report() {
    let (env, admin, oracle, _user, token_address) = setup_test_env();
    let client = EnhancedInsuranceClient::new(&env, &env.register(EnhancedInsurance, ()));
    
    client.initialize(&admin, &oracle, &token_address);

    let report_id = client.generate_compliance_report(&admin, &30); // 30 days
    assert_eq!(report_id, 1);

    // Get report
    let report = client.get_compliance_report(&report_id);
    assert!(report.is_some());
    let report = report.unwrap();
    assert_eq!(report.report_id, report_id);
    assert_eq!(report.total_policies, 287);
    assert_eq!(report.claims_paid, 35);
    assert_eq!(report.loss_ratio, 12200); // 122%
}

#[test]
fn test_risk_multiplier_calculation() {
    let (env, admin, oracle, _user, token_address) = setup_test_env();
    let client = EnhancedInsuranceClient::new(&env, &env.register(EnhancedInsurance, ()));
    
    client.initialize(&admin, &oracle, &token_address);

    // Test low risk (0-30) -> 1.0x multiplier
    let multiplier = client.get_risk_multiplier(&15).unwrap();
    assert_eq!(multiplier, 10000);

    // Test medium risk (31-60) -> 1.5x multiplier
    let multiplier = client.get_risk_multiplier(&45).unwrap();
    assert_eq!(multiplier, 15000);

    // Test high risk (61-100) -> 3.0x multiplier
    let multiplier = client.get_risk_multiplier(&80).unwrap();
    assert_eq!(multiplier, 30000);

    // Test invalid risk score
    let result = client.try_get_risk_multiplier(&150);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), 540); // RiskScoreOutOfRange
}

#[test]
fn test_governance_parameters() {
    let (env, admin, oracle, _user, token_address) = setup_test_env();
    let client = EnhancedInsuranceClient::new(&env, &env.register(EnhancedInsurance, ()));
    
    client.initialize(&admin, &oracle, &token_address);

    let params = client.get_governance_parameters();
    assert_eq!(params.quorum_percentage, 5000); // 50%
    assert_eq!(params.voting_period_days, 7);
    assert_eq!(params.proposal_threshold, 1000);
}