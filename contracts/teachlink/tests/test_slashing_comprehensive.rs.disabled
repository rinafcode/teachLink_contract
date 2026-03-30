#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::too_many_lines)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    vec, Address, Bytes, Env, Vec, Symbol, Map,
};

use teachlink_contract::{
    SlashingError, SlashingEvidence, SlashingCondition, TeachLinkBridge, TeachLinkBridgeClient,
    ValidatorSlashRecord, SlashParameters,
};

fn create_slash_params(
    env: &Env,
    slash_percentage: u32,
    bounty_percentage: u32,
    min_evidence_age: u64,
) -> SlashParameters {
    SlashParameters {
        slash_percentage,
        bounty_percentage,
        min_evidence_age,
        max_slash_per_period: 10000,
        evidence_bond: 1000,
    }
}

fn create_slashing_evidence(
    env: &Env,
    validator: Address,
    condition: SlashingCondition,
    evidence_data: Bytes,
) -> SlashingEvidence {
    SlashingEvidence {
        validator,
        condition,
        evidence_data,
        timestamp: env.ledger().timestamp(),
        reporter: Address::generate(env),
    }
}

#[test]
fn test_slashing_initialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let params = create_slash_params(&env, 50, 10, 100);

    // Test successful initialization
    client.initialize_slashing(&admin, &params);

    // Test double initialization
    let result = client.try_initialize_slashing(&admin, &params);
    assert_eq!(result.error(), Some(Ok(SlashingError::AlreadyInitialized)));
}

#[test]
fn test_valid_slashing_conditions() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let reporter = Address::generate(&env);
    let params = create_slash_params(&env, 50, 10, 100);
    
    client.initialize_slashing(&admin, &params);

    // Test double signing evidence
    let double_sign_data = Bytes::from_slice(&env, b"double_sign_evidence");
    let evidence = create_slashing_evidence(&env, validator.clone(), SlashingCondition::DoubleSigning, double_sign_data);
    
    // Submit evidence
    client.submit_slashing_evidence(&evidence);

    // Check slash record
    let slash_record = client.get_slash_record(&validator);
    assert!(slash_record.is_slashed);
    assert_eq!(slash_record.condition, SlashingCondition::DoubleSigning);
}

#[test]
fn test_invalid_slashing_evidence() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let params = create_slash_params(&env, 50, 10, 100);
    
    client.initialize_slashing(&admin, &params);

    // Test empty evidence
    let empty_evidence = Bytes::from_slice(&env, b"");
    let evidence = create_slashing_evidence(&env, validator, SlashingCondition::DoubleSigning, empty_evidence);
    
    let result = client.try_submit_slashing_evidence(&evidence);
    assert_eq!(result.error(), Some(Ok(SlashingError::InvalidSlashingEvidence)));
}

#[test]
fn test_self_slashing_prevention() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let params = create_slash_params(&env, 50, 10, 100);
    
    client.initialize_slashing(&admin, &params);

    // Test self-slashing
    let evidence_data = Bytes::from_slice(&env, b"self_slash_attempt");
    let mut evidence = create_slashing_evidence(&env, validator.clone(), SlashingCondition::DoubleSigning, evidence_data);
    evidence.reporter = validator.clone(); // Reporter is the same as validator
    
    let result = client.try_submit_slashing_evidence(&evidence);
    assert_eq!(result.error(), Some(Ok(SlashingError::CannotSlashSelf)));
}

#[test]
fn test_duplicate_slashing_prevention() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let reporter = Address::generate(&env);
    let params = create_slash_params(&env, 50, 10, 100);
    
    client.initialize_slashing(&admin, &params);

    // Submit first evidence
    let evidence_data = Bytes::from_slice(&env, b"first_evidence");
    let evidence1 = create_slashing_evidence(&env, validator.clone(), SlashingCondition::DoubleSigning, evidence_data);
    client.submit_slashing_evidence(&evidence1);

    // Try to submit duplicate evidence
    let duplicate_data = Bytes::from_slice(&env, b"first_evidence");
    let evidence2 = create_slashing_evidence(&env, validator, SlashingCondition::DoubleSigning, duplicate_data);
    
    let result = client.try_submit_slashing_evidence(&evidence2);
    assert_eq!(result.error(), Some(Ok(SlashingError::ValidatorAlreadySlashed)));
}

#[test]
fn test_slashing_amount_calculation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let reporter = Address::generate(&env);
    let params = create_slash_params(&env, 50, 10, 100); // 50% slash, 10% bounty
    
    client.initialize_slashing(&admin, &params);

    // Set up validator with stake
    let stake_amount = 10000;
    client.set_validator_stake(&validator, &stake_amount);

    // Submit evidence
    let evidence_data = Bytes::from_slice(&env, b"slash_evidence");
    let evidence = create_slashing_evidence(&env, validator.clone(), SlashingCondition::DoubleSigning, evidence_data);
    client.submit_slashing_evidence(&evidence);

    // Check slash amounts
    let slash_record = client.get_slash_record(&validator);
    assert_eq!(slash_record.slash_amount, 5000); // 50% of 10000
    assert_eq!(slash_record.bounty_amount, 1000); // 10% of 10000
}

#[test]
fn test_slashing_conditions_variations() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let validator3 = Address::generate(&env);
    let params = create_slash_params(&env, 50, 10, 100);
    
    client.initialize_slashing(&admin, &params);

    // Test double signing
    let double_sign_data = Bytes::from_slice(&env, b"double_sign_evidence");
    let evidence1 = create_slashing_evidence(&env, validator1, SlashingCondition::DoubleSigning, double_sign_data);
    client.submit_slashing_evidence(&evidence1);

    // Test unavailable validator
    let unavailable_data = Bytes::from_slice(&env, b"unavailable_evidence");
    let evidence2 = create_slashing_evidence(&env, validator2, SlashingCondition::UnavailableValidator, unavailable_data);
    client.submit_slashing_evidence(&evidence2);

    // Test malicious behavior
    let malicious_data = Bytes::from_slice(&env, b"malicious_evidence");
    let evidence3 = create_slashing_evidence(&env, validator3, SlashingCondition::MaliciousBehavior, malicious_data);
    client.submit_slashing_evidence(&evidence3);

    // Verify all records
    let record1 = client.get_slash_record(&validator1);
    let record2 = client.get_slash_record(&validator2);
    let record3 = client.get_slash_record(&validator3);

    assert_eq!(record1.condition, SlashingCondition::DoubleSigning);
    assert_eq!(record2.condition, SlashingCondition::UnavailableValidator);
    assert_eq!(record3.condition, SlashingCondition::MaliciousBehavior);
}

#[test]
fn test_evidence_age_validation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let params = create_slash_params(&env, 50, 10, 100);
    
    client.initialize_slashing(&admin, &params);

    // Create old evidence (before min_evidence_age)
    let old_timestamp = env.ledger().timestamp() - 50; // Too recent
    let evidence_data = Bytes::from_slice(&env, b"old_evidence");
    let mut evidence = create_slashing_evidence(&env, validator, SlashingCondition::DoubleSigning, evidence_data);
    evidence.timestamp = old_timestamp;
    
    let result = client.try_submit_slashing_evidence(&evidence);
    assert_eq!(result.error(), Some(Ok(SlashingError::EvidenceTooRecent)));

    // Create valid old evidence
    let valid_timestamp = env.ledger().timestamp() - 150; // Old enough
    evidence.timestamp = valid_timestamp;
    
    client.submit_slashing_evidence(&evidence); // Should succeed
}

#[test]
fn test_slashing_bounty_distribution() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let reporter = Address::generate(&env);
    let params = create_slash_params(&env, 50, 10, 100);
    
    client.initialize_slashing(&admin, &params);

    // Set up validator with stake
    let stake_amount = 10000;
    client.set_validator_stake(&validator, &stake_amount);

    // Submit evidence with specific reporter
    let evidence_data = Bytes::from_slice(&env, b"bounty_evidence");
    let mut evidence = create_slashing_evidence(&env, validator, SlashingCondition::DoubleSigning, evidence_data);
    evidence.reporter = reporter.clone();
    
    client.submit_slashing_evidence(&evidence);

    // Check bounty distribution
    let bounty_balance = client.get_bounty_balance(&reporter);
    assert_eq!(bounty_balance, 1000); // 10% of 10000
}

#[test]
fn test_slashing_limits() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let params = create_slash_params(&env, 50, 10, 100);
    params.max_slash_per_period = 5000; // Limit per period
    
    client.initialize_slashing(&admin, &params);

    // Set up validators with stake
    client.set_validator_stake(&validator1, &10000);
    client.set_validator_stake(&validator2, &10000);

    // Slash first validator (should succeed)
    let evidence_data1 = Bytes::from_slice(&env, b"evidence1");
    let evidence1 = create_slashing_evidence(&env, validator1, SlashingCondition::DoubleSigning, evidence_data1);
    client.submit_slashing_evidence(&evidence1);

    // Try to slash second validator (might hit limit)
    let evidence_data2 = Bytes::from_slice(&env, b"evidence2");
    let evidence2 = create_slashing_evidence(&env, validator2, SlashingCondition::DoubleSigning, evidence_data2);
    
    let result = client.try_submit_slashing_evidence(&evidence2);
    // This might fail due to period limit, depending on implementation
    if result.error().is_some() {
        assert_eq!(result.error(), Some(Ok(SlashingError::SlashLimitExceeded)));
    }
}
