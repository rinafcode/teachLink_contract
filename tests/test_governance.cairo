use snforge_std::{
    declare, ContractClassTrait, DeclareResultTrait, start_cheat_caller_address,
    stop_cheat_caller_address, start_cheat_block_timestamp, stop_cheat_block_timestamp,
};
use starknet::{ContractAddress, contract_address_const};
use teachlink::interfaces::igovernance::{
    IGovernanceDispatcher, IGovernanceDispatcherTrait, GovernanceParameters,
};
use openzeppelin::token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherTrait};
use openzeppelin::access::ownable::interface::{IOwnableDispatcher, IOwnableDispatcherTrait};

const ADMIN: felt252 = 123;
const USER1: felt252 = 456;
const USER2: felt252 = 789;
const USER3: felt252 = 101112;

// Governance parameter constants
const VOTING_DELAY: u64 = 86400; // 1 day
const VOTING_PERIOD: u64 = 259200; // 3 days  
const PROPOSAL_THRESHOLD: u256 = 1000; // 1000 tokens to create proposal
const QUORUM_THRESHOLD: u256 = 10000; // 10000 tokens minimum participation
const EXECUTION_DELAY: u64 = 172800; // 2 days

fn ADMIN_ADDR() -> ContractAddress {
    ADMIN.try_into().unwrap()
}

fn USER1_ADDR() -> ContractAddress {
    USER1.try_into().unwrap()
}

fn USER2_ADDR() -> ContractAddress {
    USER2.try_into().unwrap()
}

fn USER3_ADDR() -> ContractAddress {
    USER3.try_into().unwrap()
}

const INITIAL_SUPPLY: u256 = 1000000_000000000000000000;
fn deploy_token() -> ContractAddress {
    let token_class = declare("TeachLinkToken").unwrap().contract_class();
    let mut constructor_calldata = array![];

    // Constructor args: name, symbol, total_supply
    let token_name: ByteArray = "TeachLink Token";
    let token_symbol: ByteArray = "TLT";

    token_name.serialize(ref constructor_calldata);
    token_symbol.serialize(ref constructor_calldata);
    INITIAL_SUPPLY.serialize(ref constructor_calldata); // 1M tokens

    let (token_address, _) = token_class.deploy(@constructor_calldata).unwrap();

    let token = IERC20Dispatcher { contract_address: token_address };
    let ownable_token = IOwnableDispatcher { contract_address: token_address };

    token.transfer(ADMIN_ADDR(), INITIAL_SUPPLY);

    ownable_token.transfer_ownership(ADMIN_ADDR());

    token_address
}

fn deploy_governance() -> ContractAddress {
    let governance_class = declare("TeachLinkGovernance").unwrap().contract_class();
    let constructor_calldata = array![];
    let (governance_address, _) = governance_class.deploy(@constructor_calldata).unwrap();

    let ownable_governance = IOwnableDispatcher { contract_address: governance_address };
    ownable_governance.transfer_ownership(ADMIN_ADDR());

    governance_address
}

fn setup_governance() -> (ContractAddress, ContractAddress) {
    let token_address = deploy_token();
    let governance_address = deploy_governance();

    let governance = IGovernanceDispatcher { contract_address: governance_address };

    let erc20_token = IERC20Dispatcher { contract_address: token_address };

    // Initialize governance with parameters
    let params = GovernanceParameters {
        voting_delay: VOTING_DELAY,
        voting_period: VOTING_PERIOD,
        proposal_threshold: PROPOSAL_THRESHOLD,
        quorum_threshold: QUORUM_THRESHOLD,
        execution_delay: EXECUTION_DELAY,
    };

    start_cheat_caller_address(governance_address, ADMIN_ADDR());
    governance.initialize(token_address, params);
    stop_cheat_caller_address(governance_address);

    // Distribute tokens to users
    start_cheat_caller_address(token_address, ADMIN_ADDR());
    erc20_token.transfer(USER1_ADDR(), 50000); // 50k tokens
    erc20_token.transfer(USER2_ADDR(), 30000); // 30k tokens  
    erc20_token.transfer(USER3_ADDR(), 20000); // 20k tokens
    stop_cheat_caller_address(token_address);

    (token_address, governance_address)
}

#[test]
fn test_governance_initialization() {
    let (_, governance_address) = setup_governance();
    let governance = IGovernanceDispatcher { contract_address: governance_address };

    let params = governance.get_governance_parameters();
    assert!(params.voting_delay == VOTING_DELAY, "Incorrect voting delay");
    assert!(params.voting_period == VOTING_PERIOD, "Incorrect voting period");
    assert!(params.proposal_threshold == PROPOSAL_THRESHOLD, "Incorrect proposal threshold");
    assert!(params.quorum_threshold == QUORUM_THRESHOLD, "Incorrect quorum threshold");
    assert!(params.execution_delay == EXECUTION_DELAY, "Incorrect execution delay");

    assert!(governance.get_proposal_count() == 0, "Initial proposal count should be 0");
}

#[test]
fn test_create_proposal() {
    let (_, governance_address) = setup_governance();
    let governance = IGovernanceDispatcher { contract_address: governance_address };

    // USER1 has enough tokens to create proposal
    start_cheat_caller_address(governance_address, USER1_ADDR());
    start_cheat_block_timestamp(governance_address, 1000000);

    let proposal_id = governance
        .create_proposal(
            "Test Proposal",
            "This is a test proposal",
            contract_address_const::<0>(),
            array![].span(),
            0,
        );

    stop_cheat_block_timestamp(governance_address);
    stop_cheat_caller_address(governance_address);

    assert!(proposal_id == 1, "First proposal should have ID 1");
    assert!(governance.get_proposal_count() == 1, "Proposal count should be 1");

    let proposal = governance.get_proposal(proposal_id);
    assert!(proposal.proposer == USER1_ADDR(), "Incorrect proposer");
    assert!(proposal.title == "Test Proposal", "Incorrect title");
    assert!(proposal.executed == false, "Proposal should not be executed");
    assert!(proposal.canceled == false, "Proposal should not be canceled");
}

#[test]
#[should_panic(
    expected: (
        0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3,
        0x0,
        0x496e73756666696369656e7420766f74696e6720706f776572,
        0x19,
    ),
)]
fn test_create_proposal_insufficient_tokens() {
    let (token_address, governance_address) = setup_governance();
    let governance = IGovernanceDispatcher { contract_address: governance_address };
    let erc20_token = IERC20Dispatcher { contract_address: token_address };

    // Transfer away most tokens from USER3 so they don't have enough
    start_cheat_caller_address(token_address, USER3_ADDR());
    erc20_token.transfer(USER1_ADDR(), 19500); // Leave only 500 tokens
    stop_cheat_caller_address(token_address);

    // USER3 now has insufficient tokens
    start_cheat_caller_address(governance_address, USER3_ADDR());
    governance
        .create_proposal(
            "Test Proposal", "This should fail", contract_address_const::<0>(), array![].span(), 0,
        );
    stop_cheat_caller_address(governance_address);
}

#[test]
fn test_voting_lifecycle() {
    let (_, governance_address) = setup_governance();
    let governance = IGovernanceDispatcher { contract_address: governance_address };

    // Create proposal
    start_cheat_caller_address(governance_address, USER1_ADDR());
    start_cheat_block_timestamp(governance_address, 1000000);

    let proposal_id = governance
        .create_proposal(
            "Test Proposal",
            "This is a test proposal",
            contract_address_const::<0>(),
            array![].span(),
            0,
        );

    stop_cheat_caller_address(governance_address);

    // Check proposal state - should be PENDING (0)
    let state = governance.get_proposal_state(proposal_id);
    assert!(state == 0, "Proposal should be PENDING");

    // Fast forward past voting delay
    start_cheat_block_timestamp(governance_address, 1000000 + VOTING_DELAY + 1);

    // Check proposal state - should be ACTIVE (1)
    let state = governance.get_proposal_state(proposal_id);
    assert!(state == 1, "Proposal should be ACTIVE");

    // Vote FOR with USER1 (50k tokens)
    start_cheat_caller_address(governance_address, USER1_ADDR());
    governance.cast_vote(proposal_id, 1, "I support this proposal");
    stop_cheat_caller_address(governance_address);

    // Vote AGAINST with USER2 (30k tokens)
    start_cheat_caller_address(governance_address, USER2_ADDR());
    governance.cast_vote(proposal_id, 0, "I oppose this proposal");
    stop_cheat_caller_address(governance_address);

    // Verify votes were recorded
    assert!(governance.has_voted(proposal_id, USER1_ADDR()), "USER1 should have voted");
    assert!(governance.has_voted(proposal_id, USER2_ADDR()), "USER2 should have voted");
    assert!(!governance.has_voted(proposal_id, USER3_ADDR()), "USER3 should not have voted");

    let proposal = governance.get_proposal(proposal_id);
    assert!(proposal.votes_for == 50000, "Incorrect votes for");
    assert!(proposal.votes_against == 30000, "Incorrect votes against");

    // Fast forward past voting period
    start_cheat_block_timestamp(governance_address, 1000000 + VOTING_DELAY + VOTING_PERIOD + 1);

    // Check proposal state - should be SUCCEEDED (4) since for > against and quorum met
    let state = governance.get_proposal_state(proposal_id);
    assert!(state == 4, "Proposal should be SUCCEEDED");

    stop_cheat_block_timestamp(governance_address);
}

#[test]
fn test_delegation() {
    let (_token_address, governance_address) = setup_governance();
    let governance = IGovernanceDispatcher { contract_address: governance_address };

    // Initially, users should be their own delegates
    assert!(governance.get_delegate(USER1_ADDR()) == USER1_ADDR(), "USER1 should delegate to self");
    assert!(governance.get_delegate(USER2_ADDR()) == USER2_ADDR(), "USER2 should delegate to self");

    // USER2 delegates to USER1
    start_cheat_caller_address(governance_address, USER2_ADDR());
    governance.delegate(USER1_ADDR());
    stop_cheat_caller_address(governance_address);

    // Check delegation
    assert!(
        governance.get_delegate(USER2_ADDR()) == USER1_ADDR(), "USER2 should delegate to USER1",
    );

    let delegation = governance.get_delegation(USER2_ADDR());
    assert!(delegation.delegator == USER2_ADDR(), "Incorrect delegator");
    assert!(delegation.delegate == USER1_ADDR(), "Incorrect delegate");

    // USER2 undelegates
    start_cheat_caller_address(governance_address, USER2_ADDR());
    governance.undelegate();
    stop_cheat_caller_address(governance_address);

    assert!(
        governance.get_delegate(USER2_ADDR()) == USER2_ADDR(),
        "USER2 should delegate to self after undelegate",
    );
}

#[test]
fn test_proposal_cancellation() {
    let (_token_address, governance_address) = setup_governance();
    let governance = IGovernanceDispatcher { contract_address: governance_address };

    // Create proposal
    start_cheat_caller_address(governance_address, USER1_ADDR());
    start_cheat_block_timestamp(governance_address, 1000000);

    let proposal_id = governance
        .create_proposal(
            "Test Proposal",
            "This is a test proposal",
            contract_address_const::<0>(),
            array![].span(),
            0,
        );

    // Proposer can cancel
    governance.cancel_proposal(proposal_id);
    stop_cheat_caller_address(governance_address);

    // Check proposal state - should be CANCELED (2)
    let state = governance.get_proposal_state(proposal_id);
    assert!(state == 2, "Proposal should be CANCELED");

    let proposal = governance.get_proposal(proposal_id);
    assert!(proposal.canceled == true, "Proposal should be marked as canceled");

    stop_cheat_block_timestamp(governance_address);
}

#[test]
#[should_panic(
    expected: (
        0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3,
        0x0,
        0x556e617574686f72697a656420746f2063616e63656c,
        0x16,
    ),
)]
fn test_unauthorized_cancellation() {
    let (_token_address, governance_address) = setup_governance();
    let governance = IGovernanceDispatcher { contract_address: governance_address };

    // Create proposal
    start_cheat_caller_address(governance_address, USER1_ADDR());
    start_cheat_block_timestamp(governance_address, 1000000);

    let proposal_id = governance
        .create_proposal(
            "Test Proposal",
            "This is a test proposal",
            contract_address_const::<0>(),
            array![].span(),
            0,
        );

    stop_cheat_caller_address(governance_address);

    // USER2 tries to cancel USER1's proposal - should fail
    start_cheat_caller_address(governance_address, USER2_ADDR());
    governance.cancel_proposal(proposal_id);
    stop_cheat_caller_address(governance_address);

    stop_cheat_block_timestamp(governance_address);
}

#[test]
#[should_panic(
    expected: (
        0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3,
        0x0,
        0x416c726561647920766f746564,
        0xd,
    ),
)]
fn test_double_voting() {
    let (_token_address, governance_address) = setup_governance();
    let governance = IGovernanceDispatcher { contract_address: governance_address };

    // Create proposal and make it active
    start_cheat_caller_address(governance_address, USER1_ADDR());
    start_cheat_block_timestamp(governance_address, 1000000);

    let proposal_id = governance
        .create_proposal(
            "Test Proposal",
            "This is a test proposal",
            contract_address_const::<0>(),
            array![].span(),
            0,
        );

    stop_cheat_caller_address(governance_address);

    // Fast forward to active period
    start_cheat_block_timestamp(governance_address, 1000000 + VOTING_DELAY + 1);

    // Vote once
    start_cheat_caller_address(governance_address, USER1_ADDR());
    governance.cast_vote(proposal_id, 1, "First vote");

    // Try to vote again - should fail
    governance.cast_vote(proposal_id, 0, "Second vote");
    stop_cheat_caller_address(governance_address);

    stop_cheat_block_timestamp(governance_address);
}

#[test]
fn test_quorum_not_met() {
    let (_token_address, governance_address) = setup_governance();
    let governance = IGovernanceDispatcher { contract_address: governance_address };

    // Create proposal
    start_cheat_caller_address(governance_address, USER1_ADDR());
    start_cheat_block_timestamp(governance_address, 1000000);

    let proposal_id = governance
        .create_proposal(
            "Test Proposal",
            "This is a test proposal",
            contract_address_const::<0>(),
            array![].span(),
            0,
        );

    stop_cheat_caller_address(governance_address);

    // Fast forward to active period
    start_cheat_block_timestamp(governance_address, 1000000 + VOTING_DELAY + 1);

    // Only USER3 votes (20k tokens) but votes AGAINST - this should make quorum but fail due to
    // opposition
    start_cheat_caller_address(governance_address, USER3_ADDR());
    governance.cast_vote(proposal_id, 0, "Against"); // Vote against instead of for
    stop_cheat_caller_address(governance_address);

    // Fast forward past voting period
    start_cheat_block_timestamp(governance_address, 1000000 + VOTING_DELAY + VOTING_PERIOD + 1);

    // Should be DEFEATED (3) due to more against votes than for votes
    let state = governance.get_proposal_state(proposal_id);
    assert!(state == 3, "Proposal should be DEFEATED due to more against votes");

    stop_cheat_block_timestamp(governance_address);
}

#[test]
fn test_quorum_actually_not_met() {
    let (token_address, governance_address) = setup_governance();
    let governance = IGovernanceDispatcher { contract_address: governance_address };
    let erc20_token = IERC20Dispatcher { contract_address: token_address };

    // Transfer away most tokens from USER3 so they have less than quorum threshold
    start_cheat_caller_address(token_address, USER3_ADDR());
    erc20_token.transfer(USER1_ADDR(), 15000); // Transfer 15k, leaving only 5k tokens
    stop_cheat_caller_address(token_address);

    // Create proposal
    start_cheat_caller_address(governance_address, USER1_ADDR());
    start_cheat_block_timestamp(governance_address, 1000000);

    let proposal_id = governance
        .create_proposal(
            "Test Proposal",
            "This is a test proposal",
            contract_address_const::<0>(),
            array![].span(),
            0,
        );

    stop_cheat_caller_address(governance_address);

    // Fast forward to active period
    start_cheat_block_timestamp(governance_address, 1000000 + VOTING_DELAY + 1);

    // Only USER3 votes (5k tokens) - not enough for quorum (10k needed)
    start_cheat_caller_address(governance_address, USER3_ADDR());
    governance.cast_vote(proposal_id, 1, "Support");
    stop_cheat_caller_address(governance_address);

    // Fast forward past voting period
    start_cheat_block_timestamp(governance_address, 1000000 + VOTING_DELAY + VOTING_PERIOD + 1);

    // Should be DEFEATED (3) due to insufficient participation (5k < 10k quorum)
    let state = governance.get_proposal_state(proposal_id);
    assert!(state == 3, "Proposal should be DEFEATED due to insufficient quorum");

    stop_cheat_block_timestamp(governance_address);
}
