use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Bytes, Env, String,
};

use governance_contract::{
    GovernanceContract, GovernanceContractClient, MockToken, MockTokenClient, ProposalStatus,
    ProposalType, VoteDirection,
};

// ========== Test Helper ==========

fn setup_governance() -> (
    Env,
    GovernanceContractClient<'static>,
    MockTokenClient<'static>,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    // Register contracts
    let governance_id = env.register(GovernanceContract, ());
    let governance_client = GovernanceContractClient::new(&env, &governance_id);

    let token_id = env.register(MockToken, ());
    let token_client = MockTokenClient::new(&env, &token_id);

    // Set up addresses
    let admin = Address::generate(&env);
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);

    // Initialize token
    let name = String::from_str(&env, "Governance Token");
    let symbol = String::from_str(&env, "GOV");
    token_client.initialize(&admin, &name, &symbol, &18);

    // Mint tokens
    token_client.mint(&voter1, &1000);
    token_client.mint(&voter2, &500);
    token_client.mint(&admin, &2000);

    // Initialize governance
    governance_client.initialize(
        &token_id, &admin, &100,  // proposal_threshold
        &500,  // quorum
        &3600, // voting_period (1 hour)
        &60,   // execution_delay (1 minute)
    );

    (env, governance_client, token_client, admin, voter1, voter2)
}

// ========== Tests ==========

#[test]
fn test_initialize_governance() {
    let (_env, governance_client, _token_client, admin, _voter1, _voter2) = setup_governance();

    let config = governance_client.get_config();
    assert_eq!(config.admin, admin);
    assert_eq!(config.proposal_threshold, 100);
    assert_eq!(config.quorum, 500);
    assert_eq!(config.voting_period, 3600);
    assert_eq!(config.execution_delay, 60);
}

#[test]
fn test_create_proposal() {
    let (env, governance_client, _token_client, admin, _voter1, _voter2) = setup_governance();

    let title = Bytes::from_slice(&env, b"Test Proposal");
    let description = Bytes::from_slice(&env, b"A test proposal for governance");

    let proposal_id = governance_client.create_proposal(
        &admin,
        &title,
        &description,
        &ProposalType::ParameterUpdate,
        &None,
    );

    assert_eq!(proposal_id, 1);

    let proposal = governance_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.id, 1);
    assert_eq!(proposal.proposer, admin);
    assert_eq!(proposal.status, ProposalStatus::Active);
    assert_eq!(proposal.for_votes, 0);
    assert_eq!(proposal.against_votes, 0);
}

#[test]
#[should_panic(expected = "Insufficient token balance to create proposal")]
fn test_create_proposal_insufficient_balance() {
    let (env, governance_client, _token_client, _admin, _voter1, _voter2) = setup_governance();

    // Create a new address with no tokens
    let poor_user = Address::generate(&env);

    let title = Bytes::from_slice(&env, b"Test Proposal");
    let description = Bytes::from_slice(&env, b"Should fail");

    governance_client.create_proposal(
        &poor_user,
        &title,
        &description,
        &ProposalType::ParameterUpdate,
        &None,
    );
}

#[test]
fn test_cast_vote() {
    let (env, governance_client, _token_client, admin, voter1, voter2) = setup_governance();

    // Create proposal
    let title = Bytes::from_slice(&env, b"Test Proposal");
    let description = Bytes::from_slice(&env, b"Vote test");
    let proposal_id = governance_client.create_proposal(
        &admin,
        &title,
        &description,
        &ProposalType::FeeChange,
        &None,
    );

    // Cast votes
    let power1 = governance_client.cast_vote(&proposal_id, &voter1, &VoteDirection::For);
    assert_eq!(power1, 1000);

    let power2 = governance_client.cast_vote(&proposal_id, &voter2, &VoteDirection::Against);
    assert_eq!(power2, 500);

    // Check proposal vote counts
    let proposal = governance_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.for_votes, 1000);
    assert_eq!(proposal.against_votes, 500);

    // Check has_voted
    assert!(governance_client.has_voted(&proposal_id, &voter1));
    assert!(governance_client.has_voted(&proposal_id, &voter2));
}

#[test]
#[should_panic(expected = "Already voted on this proposal")]
fn test_double_vote() {
    let (env, governance_client, _token_client, admin, voter1, _voter2) = setup_governance();

    let title = Bytes::from_slice(&env, b"Test Proposal");
    let description = Bytes::from_slice(&env, b"Double vote test");
    let proposal_id = governance_client.create_proposal(
        &admin,
        &title,
        &description,
        &ProposalType::ParameterUpdate,
        &None,
    );

    governance_client.cast_vote(&proposal_id, &voter1, &VoteDirection::For);
    governance_client.cast_vote(&proposal_id, &voter1, &VoteDirection::Against);
    // Should panic
}

#[test]
fn test_finalize_and_execute_proposal() {
    let (env, governance_client, _token_client, admin, _voter1, _voter2) = setup_governance();

    let title = Bytes::from_slice(&env, b"Execute Test");
    let description = Bytes::from_slice(&env, b"Test execution");
    let proposal_id = governance_client.create_proposal(
        &admin,
        &title,
        &description,
        &ProposalType::FeatureToggle,
        &None,
    );

    // Vote for with quorum (admin has 2000)
    governance_client.cast_vote(&proposal_id, &admin, &VoteDirection::For);

    // Advance time past voting period
    env.ledger().with_mut(|li| {
        li.timestamp += 3700; // Past 1 hour voting period
    });

    // Finalize
    governance_client.finalize_proposal(&proposal_id);

    let proposal = governance_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Passed);

    // Advance time past execution delay
    env.ledger().with_mut(|li| {
        li.timestamp += 100; // Past 1 minute execution delay
    });

    // Execute
    governance_client.execute_proposal(&proposal_id, &admin);

    let proposal = governance_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Executed);
}

#[test]
fn test_cancel_proposal() {
    let (env, governance_client, _token_client, admin, _voter1, _voter2) = setup_governance();

    let title = Bytes::from_slice(&env, b"Cancel Test");
    let description = Bytes::from_slice(&env, b"Test cancellation");
    let proposal_id = governance_client.create_proposal(
        &admin,
        &title,
        &description,
        &ProposalType::Custom,
        &None,
    );

    governance_client.cancel_proposal(&proposal_id, &admin);

    let proposal = governance_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Cancelled);
}

#[test]
fn test_update_config() {
    let (_env, governance_client, _token_client, _admin, _voter1, _voter2) = setup_governance();

    governance_client.update_config(&Some(200), &Some(1000), &Some(7200), &Some(120));

    let config = governance_client.get_config();
    assert_eq!(config.proposal_threshold, 200);
    assert_eq!(config.quorum, 1000);
    assert_eq!(config.voting_period, 7200);
    assert_eq!(config.execution_delay, 120);
}

#[test]
fn test_proposal_passes_with_quorum() {
    let (env, governance_client, _token_client, admin, _voter1, voter2) = setup_governance();

    let title = Bytes::from_slice(&env, b"Quorum Test");
    let description = Bytes::from_slice(&env, b"Should pass with quorum");
    let proposal_id = governance_client.create_proposal(
        &admin,
        &title,
        &description,
        &ProposalType::ParameterUpdate,
        &None,
    );

    // voter2 votes (500 tokens = quorum threshold)
    governance_client.cast_vote(&proposal_id, &voter2, &VoteDirection::For);

    // Advance time past voting period
    env.ledger().with_mut(|li| {
        li.timestamp += 3700;
    });

    // Finalize - should pass since 500 >= 500 quorum and all for
    governance_client.finalize_proposal(&proposal_id);

    let proposal = governance_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Passed);
}

#[test]
fn test_proposal_fails_majority_against() {
    let (env, governance_client, _token_client, admin, voter1, voter2) = setup_governance();

    let title = Bytes::from_slice(&env, b"Rejected");
    let description = Bytes::from_slice(&env, b"Should be rejected");
    let proposal_id = governance_client.create_proposal(
        &admin,
        &title,
        &description,
        &ProposalType::ParameterUpdate,
        &None,
    );

    // voter1 (1000) votes against, voter2 (500) votes for
    governance_client.cast_vote(&proposal_id, &voter1, &VoteDirection::Against);
    governance_client.cast_vote(&proposal_id, &voter2, &VoteDirection::For);

    // Advance time
    env.ledger().with_mut(|li| {
        li.timestamp += 3700;
    });

    governance_client.finalize_proposal(&proposal_id);

    let proposal = governance_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Failed);
}

#[test]
fn test_mock_token_operations() {
    let (env, _governance_client, token_client, admin, voter1, voter2) = setup_governance();

    // Check initial balances
    assert_eq!(token_client.balance(&admin), 2000);
    assert_eq!(token_client.balance(&voter1), 1000);
    assert_eq!(token_client.balance(&voter2), 500);

    // Test transfer
    token_client.transfer(&voter1, &voter2, &200);
    assert_eq!(token_client.balance(&voter1), 800);
    assert_eq!(token_client.balance(&voter2), 700);

    // Test metadata
    assert_eq!(
        token_client.name(),
        String::from_str(&env, "Governance Token")
    );
    assert_eq!(token_client.symbol(), String::from_str(&env, "GOV"));
    assert_eq!(token_client.decimals(), 18);
    assert_eq!(token_client.total_supply(), 3500);
}
