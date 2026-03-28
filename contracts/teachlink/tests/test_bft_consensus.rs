#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]

use soroban_sdk::{testutils::Address as _, testutils::Ledger, Address, Bytes, Env};
use teachlink_contract::{BridgeError, TeachLinkBridge, TeachLinkBridgeClient};

const MIN_STAKE: i128 = 100_000_000;

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

fn set_time(env: &Env, ts: u64) {
    env.ledger().with_mut(|l| l.timestamp = ts);
}

// ── Validator Registration ─────────────────────────────────────────

#[test]
fn test_register_validator() {
    let (env, client) = setup();
    let validator = Address::generate(&env);

    client.register_validator(&validator, &MIN_STAKE);

    let info = client.get_validator_info(&validator);
    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.stake, MIN_STAKE);
    assert!(info.is_active);
    assert_eq!(info.reputation_score, 100);
}

#[test]
fn test_register_validator_insufficient_stake_fails() {
    let (env, client) = setup();
    let validator = Address::generate(&env);

    let result = client.try_register_validator(&validator, &(MIN_STAKE - 1));
    assert_eq!(result, Err(Ok(BridgeError::InsufficientStake)));
}

#[test]
fn test_register_validator_already_registered_fails() {
    let (env, client) = setup();
    let validator = Address::generate(&env);

    client.register_validator(&validator, &MIN_STAKE);
    let result = client.try_register_validator(&validator, &MIN_STAKE);
    assert_eq!(result, Err(Ok(BridgeError::AlreadyInitialized)));
}

// ── Validator Unregistration ───────────────────────────────────────

#[test]
fn test_unregister_validator() {
    let (env, client) = setup();
    let validator = Address::generate(&env);

    client.register_validator(&validator, &MIN_STAKE);
    client.unregister_validator(&validator);

    let info = client.get_validator_info(&validator);
    assert!(info.is_none());
}

#[test]
fn test_unregister_nonexistent_validator_fails() {
    let (env, client) = setup();
    let validator = Address::generate(&env);

    let result = client.try_unregister_validator(&validator);
    assert_eq!(result, Err(Ok(BridgeError::InvalidValidatorSignature)));
}

// ── Consensus State ────────────────────────────────────────────────

#[test]
fn test_consensus_state_updates_on_registration() {
    let (env, client) = setup();

    let v1 = Address::generate(&env);
    let v2 = Address::generate(&env);
    let v3 = Address::generate(&env);

    client.register_validator(&v1, &MIN_STAKE);
    client.register_validator(&v2, &MIN_STAKE);
    client.register_validator(&v3, &MIN_STAKE);

    let state = client.get_consensus_state();
    assert_eq!(state.active_validators, 3);
    assert_eq!(state.total_stake, MIN_STAKE * 3);
    // BFT threshold for 3 validators: (2*3)/3 + 1 = 3
    assert_eq!(state.byzantine_threshold, 3);
}

// ── Proposal Creation ──────────────────────────────────────────────

#[test]
fn test_create_proposal() {
    let (env, client) = setup();
    set_time(&env, 1000);

    let v1 = Address::generate(&env);
    client.register_validator(&v1, &MIN_STAKE);

    let token = Address::generate(&env);
    let recipient = Address::generate(&env);
    let message = teachlink_contract::CrossChainMessage {
        source_chain: 1,
        source_tx_hash: Bytes::from_slice(&env, &[0xab; 32]),
        nonce: 1,
        token,
        amount: 1000,
        recipient,
        destination_chain: 2,
    };

    let proposal_id = client.create_bridge_proposal(&message);
    assert_eq!(proposal_id, 1);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.status, teachlink_contract::ProposalStatus::Pending);
    assert_eq!(proposal.vote_count, 0);
}

// ── Voting ─────────────────────────────────────────────────────────

#[test]
fn test_vote_on_proposal() {
    let (env, client) = setup();
    set_time(&env, 1000);

    let v1 = Address::generate(&env);
    client.register_validator(&v1, &MIN_STAKE);

    let token = Address::generate(&env);
    let recipient = Address::generate(&env);
    let message = teachlink_contract::CrossChainMessage {
        source_chain: 1,
        source_tx_hash: Bytes::from_slice(&env, &[0xab; 32]),
        nonce: 1,
        token,
        amount: 1000,
        recipient,
        destination_chain: 2,
    };

    let proposal_id = client.create_bridge_proposal(&message);
    client.vote_on_proposal(&v1, &proposal_id, &true);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.vote_count, 1);
}

#[test]
fn test_vote_non_validator_fails() {
    let (env, client) = setup();
    set_time(&env, 1000);

    let v1 = Address::generate(&env);
    client.register_validator(&v1, &MIN_STAKE);

    let token = Address::generate(&env);
    let recipient = Address::generate(&env);
    let message = teachlink_contract::CrossChainMessage {
        source_chain: 1,
        source_tx_hash: Bytes::from_slice(&env, &[0xab; 32]),
        nonce: 1,
        token,
        amount: 1000,
        recipient,
        destination_chain: 2,
    };

    let proposal_id = client.create_bridge_proposal(&message);

    let stranger = Address::generate(&env);
    let result = client.try_vote_on_proposal(&stranger, &proposal_id, &true);
    assert_eq!(result, Err(Ok(BridgeError::ValidatorNotActive)));
}

#[test]
fn test_double_vote_fails() {
    let (env, client) = setup();
    set_time(&env, 1000);

    // Need multiple validators so first vote doesn't auto-approve
    let v1 = Address::generate(&env);
    let v2 = Address::generate(&env);
    let v3 = Address::generate(&env);
    client.register_validator(&v1, &MIN_STAKE);
    client.register_validator(&v2, &MIN_STAKE);
    client.register_validator(&v3, &MIN_STAKE);

    let token = Address::generate(&env);
    let recipient = Address::generate(&env);
    let message = teachlink_contract::CrossChainMessage {
        source_chain: 1,
        source_tx_hash: Bytes::from_slice(&env, &[0xab; 32]),
        nonce: 1,
        token,
        amount: 1000,
        recipient,
        destination_chain: 2,
    };

    let proposal_id = client.create_bridge_proposal(&message);
    client.vote_on_proposal(&v1, &proposal_id, &true);

    let result = client.try_vote_on_proposal(&v1, &proposal_id, &true);
    assert_eq!(result, Err(Ok(BridgeError::ProposalAlreadyVoted)));
}

#[test]
fn test_vote_on_nonexistent_proposal_fails() {
    let (env, client) = setup();
    let v1 = Address::generate(&env);
    client.register_validator(&v1, &MIN_STAKE);

    let result = client.try_vote_on_proposal(&v1, &999, &true);
    assert_eq!(result, Err(Ok(BridgeError::ProposalNotFound)));
}

#[test]
fn test_vote_on_expired_proposal_fails() {
    let (env, client) = setup();
    set_time(&env, 1000);

    let v1 = Address::generate(&env);
    client.register_validator(&v1, &MIN_STAKE);

    let token = Address::generate(&env);
    let recipient = Address::generate(&env);
    let message = teachlink_contract::CrossChainMessage {
        source_chain: 1,
        source_tx_hash: Bytes::from_slice(&env, &[0xab; 32]),
        nonce: 1,
        token,
        amount: 1000,
        recipient,
        destination_chain: 2,
    };

    let proposal_id = client.create_bridge_proposal(&message);

    // Advance past 24h timeout
    set_time(&env, 1000 + 86_401);
    let result = client.try_vote_on_proposal(&v1, &proposal_id, &true);
    assert_eq!(result, Err(Ok(BridgeError::ProposalExpired)));
}

// ── Consensus Reached ──────────────────────────────────────────────

#[test]
fn test_consensus_reached_executes_proposal() {
    let (env, client) = setup();
    set_time(&env, 1000);

    // Register 1 validator → threshold = (2*1)/3 + 1 = 1
    let v1 = Address::generate(&env);
    client.register_validator(&v1, &MIN_STAKE);

    let token = Address::generate(&env);
    let recipient = Address::generate(&env);
    let message = teachlink_contract::CrossChainMessage {
        source_chain: 1,
        source_tx_hash: Bytes::from_slice(&env, &[0xab; 32]),
        nonce: 1,
        token,
        amount: 1000,
        recipient,
        destination_chain: 2,
    };

    let proposal_id = client.create_bridge_proposal(&message);
    client.vote_on_proposal(&v1, &proposal_id, &true);

    assert!(client.is_consensus_reached(&proposal_id));

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(
        proposal.status,
        teachlink_contract::ProposalStatus::Approved
    );
}

#[test]
fn test_consensus_not_reached_without_enough_votes() {
    let (env, client) = setup();
    set_time(&env, 1000);

    // Register 3 validators → threshold = (2*3)/3 + 1 = 3
    let v1 = Address::generate(&env);
    let v2 = Address::generate(&env);
    let v3 = Address::generate(&env);
    client.register_validator(&v1, &MIN_STAKE);
    client.register_validator(&v2, &MIN_STAKE);
    client.register_validator(&v3, &MIN_STAKE);

    let token = Address::generate(&env);
    let recipient = Address::generate(&env);
    let message = teachlink_contract::CrossChainMessage {
        source_chain: 1,
        source_tx_hash: Bytes::from_slice(&env, &[0xab; 32]),
        nonce: 1,
        token,
        amount: 1000,
        recipient,
        destination_chain: 2,
    };

    let proposal_id = client.create_bridge_proposal(&message);
    client.vote_on_proposal(&v1, &proposal_id, &true);

    assert!(!client.is_consensus_reached(&proposal_id));
}
