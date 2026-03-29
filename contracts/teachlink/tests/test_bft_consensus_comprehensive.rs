#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::too_many_lines)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    vec, Address, Bytes, Env, Vec, Symbol, Map,
};

use teachlink_contract::{
    BftConsensusError, Proposal, ProposalType, Vote, TeachLinkBridge, TeachLinkBridgeClient,
    ValidatorInfo, ConsensusParameters,
};

fn create_consensus_params(
    env: &Env,
    min_stake: i128,
    voting_period: u64,
    execution_delay: u64,
) -> ConsensusParameters {
    ConsensusParameters {
        min_stake,
        voting_period,
        execution_delay,
        byzantine_threshold: 67, // 2/3 majority
    }
}

fn create_validator_info(env: &Env, stake: i128) -> ValidatorInfo {
    ValidatorInfo {
        validator: Address::generate(env),
        stake,
        is_active: true,
        voting_power: stake,
        last_vote_time: 0,
    }
}

#[test]
fn test_consensus_initialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let params = create_consensus_params(&env, 1000, 100, 50);

    // Test successful initialization
    client.initialize_consensus(&admin, &params);

    // Test double initialization
    let result = client.try_initialize_consensus(&admin, &params);
    assert_eq!(result.error(), Some(Ok(BftConsensusError::AlreadyInitialized)));
}

#[test]
fn test_validator_registration() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let params = create_consensus_params(&env, 1000, 100, 50);
    
    client.initialize_consensus(&admin, &params);

    // Test successful validator registration
    client.register_validator(&validator, &2000);

    // Test insufficient stake
    let validator2 = Address::generate(&env);
    let result = client.try_register_validator(&validator2, &500); // Less than min_stake
    assert_eq!(result.error(), Some(Ok(BftConsensusError::InsufficientStake)));

    // Test duplicate registration
    let result = client.try_register_validator(&validator, &3000);
    assert_eq!(result.error(), Some(Ok(BftConsensusError::ValidatorAlreadyRegistered)));
}

#[test]
fn test_proposal_creation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let params = create_consensus_params(&env, 1000, 100, 50);
    
    client.initialize_consensus(&admin, &params);
    client.register_validator(&proposer, &2000);

    // Test creating a parameter change proposal
    let proposal_data = Bytes::from_slice(&env, b"change_parameter");
    let proposal_id = client.create_proposal(&proposer, &ProposalType::ParameterChange, &proposal_data);
    assert!(proposal_id > 0);

    // Test unauthorized proposal creation
    let unauthorized = Address::generate(&env);
    let result = client.try_create_proposal(&unauthorized, &ProposalType::ParameterChange, &proposal_data);
    assert_eq!(result.error(), Some(Ok(BftConsensusError::ValidatorNotActive)));
}

#[test]
fn test_voting_mechanism() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let params = create_consensus_params(&env, 1000, 100, 50);
    
    client.initialize_consensus(&admin, &params);
    client.register_validator(&validator1, &2000);
    client.register_validator(&validator2, &3000);

    // Create proposal
    let proposal_data = Bytes::from_slice(&env, b"test_proposal");
    let proposal_id = client.create_proposal(&validator1, &ProposalType::ParameterChange, &proposal_data);

    // Test voting
    client.vote(&validator1, &proposal_id, &true); // Vote in favor

    // Test double voting
    let result = client.try_vote(&validator1, &proposal_id, &false);
    assert_eq!(result.error(), Some(Ok(BftConsensusError::ProposalAlreadyVoted)));

    // Test voting on non-existent proposal
    let result = client.try_vote(&validator2, &999999, &true);
    assert_eq!(result.error(), Some(Ok(BftConsensusError::ProposalNotFound)));
}

#[test]
fn test_proposal_execution() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let validator3 = Address::generate(&env);
    let params = create_consensus_params(&env, 1000, 100, 1); // Short execution delay for testing
    
    client.initialize_consensus(&admin, &params);
    client.register_validator(&validator1, &2000);
    client.register_validator(&validator2, &2000);
    client.register_validator(&validator3, &2000);

    // Create proposal
    let proposal_data = Bytes::from_slice(&env, b"test_proposal");
    let proposal_id = client.create_proposal(&validator1, &ProposalType::ParameterChange, &proposal_data);

    // Vote to reach threshold
    client.vote(&validator1, &proposal_id, &true);
    client.vote(&validator2, &proposal_id, &true);

    // Fast forward time past execution delay
    env.ledger().set(LedgerInfo {
        timestamp: 150,
        protocol_version: 20,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Execute proposal
    client.execute_proposal(&proposal_id);

    // Test executing already executed proposal
    let result = client.try_execute_proposal(&proposal_id);
    assert_eq!(result.error(), Some(Ok(BftConsensusError::ProposalAlreadyExecuted)));
}

#[test]
fn test_byzantine_fault_detection() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let params = create_consensus_params(&env, 1000, 100, 50);
    
    client.initialize_consensus(&admin, &params);
    client.register_validator(&validator1, &2000);
    client.register_validator(&validator2, &2000);

    // Test reporting byzantine behavior
    let evidence = Bytes::from_slice(&env, b"byzantine_evidence");
    client.report_byzantine_behavior(&validator1, &validator2, &evidence);

    // Check if validator2 is now inactive
    let validator_info = client.get_validator_info(&validator2);
    assert!(!validator_info.is_active);
}

#[test]
fn test_consensus_parameters_update() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let params = create_consensus_params(&env, 1000, 100, 50);
    
    client.initialize_consensus(&admin, &params);
    client.register_validator(&validator1, &2000);
    client.register_validator(&validator2, &2000);

    // Create parameter update proposal
    let new_params = create_consensus_params(&env, 1500, 200, 75);
    let proposal_data = Bytes::from_slice(&env, &new_params.serialize().to_vec());
    let proposal_id = client.create_proposal(&validator1, &ProposalType::ParameterChange, &proposal_data);

    // Vote and execute
    client.vote(&validator1, &proposal_id, &true);
    client.vote(&validator2, &proposal_id, &true);

    // Fast forward and execute
    env.ledger().set(LedgerInfo {
        timestamp: 150,
        protocol_version: 20,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    client.execute_proposal(&proposal_id);

    // Verify parameters updated
    let updated_params = client.get_consensus_parameters();
    assert_eq!(updated_params.min_stake, 1500);
    assert_eq!(updated_params.voting_period, 200);
}

#[test]
fn test_validator_stake_management() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let params = create_consensus_params(&env, 1000, 100, 50);
    
    client.initialize_consensus(&admin, &params);
    client.register_validator(&validator, &2000);

    // Test increasing stake
    client.increase_stake(&validator, &1000);
    let validator_info = client.get_validator_info(&validator);
    assert_eq!(validator_info.stake, 3000);

    // Test decreasing stake below minimum
    let result = client.try_decrease_stake(&validator, &2500); // Would leave 500 < min_stake
    assert_eq!(result.error(), Some(Ok(BftConsensusError::InsufficientStake)));

    // Test valid decrease
    client.decrease_stake(&validator, &1500);
    let validator_info = client.get_validator_info(&validator);
    assert_eq!(validator_info.stake, 1500);
}

#[test]
fn test_proposal_timeout() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let params = create_consensus_params(&env, 1000, 100, 50);
    
    client.initialize_consensus(&admin, &params);
    client.register_validator(&validator, &2000);

    // Create proposal
    let proposal_data = Bytes::from_slice(&env, b"test_proposal");
    let proposal_id = client.create_proposal(&validator, &ProposalType::ParameterChange, &proposal_data);

    // Fast forward past voting period
    env.ledger().set(LedgerInfo {
        timestamp: 200,
        protocol_version: 20,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Try to vote on expired proposal
    let result = client.try_vote(&validator, &proposal_id, &true);
    assert_eq!(result.error(), Some(Ok(BftConsensusError::ProposalExpired)));
}
