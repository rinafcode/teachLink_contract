#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::too_many_lines)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, Bytes, Env,
};

use teachlink_contract::{
    BridgeError, BridgeProposal, ConsensusState, CrossChainMessage, ProposalStatus,
    TeachLinkBridge, TeachLinkBridgeClient, ValidatorInfo,
};

/// Minimum validator stake (mirrors MIN_VALIDATOR_STAKE in bft_consensus.rs)
const MIN_STAKE: i128 = 100_000_000;

fn make_message(env: &Env, token: &Address, recipient: &Address, nonce: u64) -> CrossChainMessage {
    CrossChainMessage {
        source_chain: 1,
        source_tx_hash: Bytes::from_slice(env, b"tx_hash_data"),
        nonce,
        token: token.clone(),
        amount: 1_000_000,
        recipient: recipient.clone(),
        destination_chain: 2,
    }
}

/// Register a contract and call `initialize`. Returns (client, token, admin).
fn setup(env: &Env) -> (TeachLinkBridgeClient, Address, Address) {
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(env, &contract_id);
    let token = Address::generate(env);
    let admin = Address::generate(env);
    let fee_recipient = Address::generate(env);
    client.initialize(&token, &admin, &3, &fee_recipient);
    (client, token, admin)
}

// ─── Initialisation ──────────────────────────────────────────────────────────

#[test]
fn test_consensus_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);

    // First initialisation succeeds
    client.initialize(&token, &admin, &3, &fee_recipient);

    // Second initialisation returns AlreadyInitialized
    let result = client.try_initialize(&token, &admin, &3, &fee_recipient);
    assert_eq!(result, Err(Ok(BridgeError::AlreadyInitialized)));
}

// ─── Validator registration ───────────────────────────────────────────────────

#[test]
fn test_validator_registration_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let validator = Address::generate(&env);
    client.register_validator(&validator, &(MIN_STAKE * 2));

    let info: ValidatorInfo = client.get_validator_info(&validator).unwrap();
    assert_eq!(info.address, validator);
    assert_eq!(info.stake, MIN_STAKE * 2);
    assert!(info.is_active);
}

#[test]
fn test_validator_registration_insufficient_stake() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let validator = Address::generate(&env);
    let result = client.try_register_validator(&validator, &(MIN_STAKE / 2));
    assert_eq!(result, Err(Ok(BridgeError::InsufficientStake)));
}

#[test]
fn test_validator_registration_duplicate() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let validator = Address::generate(&env);
    client.register_validator(&validator, &(MIN_STAKE * 2));

    // Registering again returns AlreadyInitialized (the error bft_consensus uses for duplicates)
    let result = client.try_register_validator(&validator, &(MIN_STAKE * 3));
    assert_eq!(result, Err(Ok(BridgeError::AlreadyInitialized)));
}

// ─── Proposal creation ───────────────────────────────────────────────────────

#[test]
fn test_proposal_creation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token, _) = setup(&env);

    let validator = Address::generate(&env);
    let recipient = Address::generate(&env);
    client.register_validator(&validator, &(MIN_STAKE * 2));

    let message = make_message(&env, &token, &recipient, 1);
    let proposal_id = client.create_bridge_proposal(&message);
    assert!(proposal_id > 0);

    let proposal: BridgeProposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.proposal_id, proposal_id);
    assert_eq!(proposal.status, ProposalStatus::Pending);
}

#[test]
fn test_multiple_proposals_have_sequential_ids() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token, _) = setup(&env);

    let validator = Address::generate(&env);
    let recipient = Address::generate(&env);
    client.register_validator(&validator, &(MIN_STAKE * 2));

    let id1 = client.create_bridge_proposal(&make_message(&env, &token, &recipient, 1));
    let id2 = client.create_bridge_proposal(&make_message(&env, &token, &recipient, 2));
    let id3 = client.create_bridge_proposal(&make_message(&env, &token, &recipient, 3));

    assert!(id1 < id2);
    assert!(id2 < id3);
}

// ─── Voting mechanism ────────────────────────────────────────────────────────

#[test]
fn test_voting_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token, _) = setup(&env);

    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let recipient = Address::generate(&env);
    client.register_validator(&validator1, &(MIN_STAKE * 2));
    client.register_validator(&validator2, &(MIN_STAKE * 3));

    let proposal_id = client.create_bridge_proposal(&make_message(&env, &token, &recipient, 1));

    // First vote succeeds
    client.vote_on_proposal(&validator1, &proposal_id, &true);

    // Proposal is still pending (needs more votes)
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.vote_count, 1);
}

#[test]
fn test_double_voting_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token, _) = setup(&env);

    // Register 2 validators so byzantine_threshold = (2*2)/3+1 = 2,
    // meaning a single vote does not immediately approve the proposal.
    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let recipient = Address::generate(&env);
    client.register_validator(&validator1, &(MIN_STAKE * 2));
    client.register_validator(&validator2, &(MIN_STAKE * 2));

    let proposal_id = client.create_bridge_proposal(&make_message(&env, &token, &recipient, 1));
    client.vote_on_proposal(&validator1, &proposal_id, &true);

    // Voting again with the same validator should be rejected
    let result = client.try_vote_on_proposal(&validator1, &proposal_id, &false);
    assert_eq!(result, Err(Ok(BridgeError::ProposalAlreadyVoted)));
}

#[test]
fn test_vote_on_nonexistent_proposal() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let validator = Address::generate(&env);
    client.register_validator(&validator, &(MIN_STAKE * 2));

    let result = client.try_vote_on_proposal(&validator, &999_999, &true);
    assert_eq!(result, Err(Ok(BridgeError::ProposalNotFound)));
}

#[test]
fn test_inactive_validator_cannot_vote() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token, _) = setup(&env);

    let validator = Address::generate(&env);
    let non_validator = Address::generate(&env);
    let recipient = Address::generate(&env);
    client.register_validator(&validator, &(MIN_STAKE * 2));

    let proposal_id = client.create_bridge_proposal(&make_message(&env, &token, &recipient, 1));

    let result = client.try_vote_on_proposal(&non_validator, &proposal_id, &true);
    assert_eq!(result, Err(Ok(BridgeError::ValidatorNotActive)));
}

// ─── Consensus reached ───────────────────────────────────────────────────────

#[test]
fn test_proposal_reaches_consensus() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token, _) = setup(&env);

    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let validator3 = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.register_validator(&validator1, &(MIN_STAKE * 2));
    client.register_validator(&validator2, &(MIN_STAKE * 2));
    client.register_validator(&validator3, &(MIN_STAKE * 2));

    // byzantine_threshold for 3 validators = (2*3)/3 + 1 = 3
    let proposal_id = client.create_bridge_proposal(&make_message(&env, &token, &recipient, 1));

    client.vote_on_proposal(&validator1, &proposal_id, &true);
    client.vote_on_proposal(&validator2, &proposal_id, &true);
    client.vote_on_proposal(&validator3, &proposal_id, &true);

    assert!(client.is_consensus_reached(&proposal_id));
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Approved);
}

// ─── Proposal timeout ────────────────────────────────────────────────────────

#[test]
fn test_proposal_timeout() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token, _) = setup(&env);

    let validator = Address::generate(&env);
    let recipient = Address::generate(&env);
    client.register_validator(&validator, &(MIN_STAKE * 2));

    let proposal_id = client.create_bridge_proposal(&make_message(&env, &token, &recipient, 1));

    // Advance ledger past PROPOSAL_TIMEOUT (86_400 seconds)
    env.ledger().set(LedgerInfo {
        timestamp: 86_401,
        protocol_version: 25,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3_110_400,
    });

    let result = client.try_vote_on_proposal(&validator, &proposal_id, &true);
    assert_eq!(result, Err(Ok(BridgeError::ProposalExpired)));
}

// ─── Consensus state ─────────────────────────────────────────────────────────

#[test]
fn test_consensus_state_reflects_registered_validators() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    client.register_validator(&validator1, &(MIN_STAKE * 2));
    client.register_validator(&validator2, &(MIN_STAKE * 2));

    let state: ConsensusState = client.get_consensus_state();
    assert_eq!(state.active_validators, 2);
    assert_eq!(state.total_stake, MIN_STAKE * 4);
    assert!(state.byzantine_threshold > 0);
}

// ─── Unregister validator ────────────────────────────────────────────────────

#[test]
fn test_unregister_validator() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let validator = Address::generate(&env);
    client.register_validator(&validator, &(MIN_STAKE * 2));

    assert_eq!(client.get_consensus_state().active_validators, 1);

    client.unregister_validator(&validator);

    assert_eq!(client.get_consensus_state().active_validators, 0);
}
