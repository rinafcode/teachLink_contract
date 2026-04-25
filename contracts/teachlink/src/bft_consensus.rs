//! Byzantine Fault Tolerant (BFT) Consensus Module
//!
//! This module implements a BFT consensus mechanism for bridge validators,
//! ensuring that the bridge can tolerate up to f faulty validators out of 3f+1 total validators.
//!
//! # BFT Threshold Algorithm
//!
//! The Byzantine threshold (minimum votes required to approve a proposal) is
//! computed as:
//!
//! ```text
//! byzantine_threshold = floor(2 * n / 3) + 1
//! ```
//!
//! where `n` is the number of active validators.  This satisfies the classic
//! BFT requirement: a quorum of ⌈2n/3⌉ guarantees safety even when up to
//! ⌊n/3⌋ validators are Byzantine (malicious or offline).
//!
//! Example: with 10 validators, threshold = (2*10/3)+1 = 7.  An attacker
//! controlling 3 validators cannot reach quorum alone.
//!
//! # Proposal Lifecycle
//!
//! ```text
//! create_proposal → Pending
//!     ↓ (votes accumulate)
//! vote_count >= byzantine_threshold → execute_proposal → Approved
//!     ↓ (timeout reached)
//! expires_at exceeded → Expired
//! ```
//!
//! # Expiry Dual-Signal
//!
//! Proposals store both a wall-clock `expires_at` timestamp and a
//! `PROPOSAL_EXPIRES_SEQ` ledger-sequence deadline.  Expiry is triggered if
//! *either* signal indicates the deadline has passed.  This guards against
//! networks where `ledger().timestamp()` is frozen or unreliable.
//!
//! # Validator Rotation
//!
//! Every `ROTATION_EPOCH_ROUNDS` consensus rounds, validators with
//! `reputation_score < MIN_ACTIVE_REPUTATION` are rotated out of the active
//! set.  Active voters receive a reputation boost to reward participation.
//!
//! # Spec Reference
//! See `contracts/documentation/COLLABORATION.md` §Consensus for the
//! governance rationale behind the rotation policy.

use crate::errors::BridgeError;
use crate::events::{
    ProposalCreatedEvent, ProposalExecutedEvent, ProposalVotedEvent, ValidatorRegisteredEvent,
    ValidatorUnregisteredEvent,
};
use crate::storage::{
    StorageKey, BRIDGE_PROPOSALS, CONSENSUS_STATE, PROPOSAL_COUNTER, PROPOSAL_EXPIRES_SEQ,
    VALIDATORS, VALIDATOR_ACTIVITY_SEQ, VALIDATOR_INFO, VALIDATOR_STAKES,
};
use crate::types::{
    BridgeProposal, ConsensusState, CrossChainMessage, ProposalStatus, ValidatorInfo,
};
use soroban_sdk::{Address, Env, Map, Vec};

/// Minimum stake required to become a validator
pub const MIN_VALIDATOR_STAKE: i128 = 100_000_000; // 100 tokens with 6 decimals

/// Proposal timeout in seconds (24 hours)
pub const PROPOSAL_TIMEOUT: u64 = 86_400;

/// Number of consensus rounds per rotation epoch.
/// After this many rounds, the active validator set is re-evaluated and
/// low-reputation validators may be rotated out.
pub const ROTATION_EPOCH_ROUNDS: u64 = 100;

/// Minimum reputation score required to remain in the active validator set.
/// Validators below this threshold are rotated out during epoch transitions.
pub const MIN_ACTIVE_REPUTATION: u32 = 40;

/// BFT Consensus Manager
pub struct BFTConsensus;

impl BFTConsensus {
    /// Register a new validator with stake
    pub fn register_validator(
        env: &Env,
        validator: Address,
        stake: i128,
    ) -> Result<(), BridgeError> {
        validator.require_auth();

        if stake < MIN_VALIDATOR_STAKE {
            return Err(BridgeError::InsufficientStake);
        }

        // Check if already registered
        let validators: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&VALIDATORS)
            .unwrap_or_else(|| Map::new(env));
        if validators.get(validator.clone()).unwrap_or(false) {
            return Err(BridgeError::AlreadyInitialized);
        }

        // Create validator info
        let validator_info = ValidatorInfo {
            address: validator.clone(),
            stake,
            reputation_score: 100, // Start with perfect reputation
            is_active: true,
            joined_at: env.ledger().timestamp(),
            last_activity: env.ledger().timestamp(),
            total_validations: 0,
            missed_validations: 0,
            slashed_amount: 0,
        };

        // Store validator info
        let mut validator_infos: Map<Address, ValidatorInfo> = env
            .storage()
            .instance()
            .get(&VALIDATOR_INFO)
            .unwrap_or_else(|| Map::new(env));
        validator_infos.set(validator.clone(), validator_info);
        env.storage()
            .instance()
            .set(&VALIDATOR_INFO, &validator_infos);

        // Sequence-based activity fallback
        let mut activity_seq: Map<Address, u32> = env
            .storage()
            .instance()
            .get(&VALIDATOR_ACTIVITY_SEQ)
            .unwrap_or_else(|| Map::new(env));
        activity_seq.set(validator.clone(), env.ledger().sequence());
        env.storage()
            .instance()
            .set(&VALIDATOR_ACTIVITY_SEQ, &activity_seq);

        // Store stake
        let mut stakes: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&VALIDATOR_STAKES)
            .unwrap_or_else(|| Map::new(env));
        stakes.set(validator.clone(), stake);
        env.storage().instance().set(&VALIDATOR_STAKES, &stakes);

        // Add to validators list
        let mut validators = validators;
        validators.set(validator.clone(), true);
        env.storage().instance().set(&VALIDATORS, &validators);

        // Update consensus state
        Self::update_consensus_state(env)?;

        // Emit event
        ValidatorRegisteredEvent {
            validator: validator.clone(),
            stake,
            joined_at: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    /// Unregister a validator and return stake
    pub fn unregister_validator(env: &Env, validator: Address) -> Result<(), BridgeError> {
        validator.require_auth();

        // Check if validator exists
        let validators: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&VALIDATORS)
            .unwrap_or_else(|| Map::new(env));
        if !validators.get(validator.clone()).unwrap_or(false) {
            return Err(BridgeError::InvalidValidatorSignature);
        }

        // Get stake
        let stakes: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&VALIDATOR_STAKES)
            .unwrap_or_else(|| Map::new(env));
        let stake = stakes.get(validator.clone()).unwrap_or(0);

        // Remove validator
        let mut validators = validators;
        validators.set(validator.clone(), false);
        env.storage().instance().set(&VALIDATORS, &validators);

        // Remove validator info
        let mut validator_infos: Map<Address, ValidatorInfo> = env
            .storage()
            .instance()
            .get(&VALIDATOR_INFO)
            .unwrap_or_else(|| Map::new(env));
        validator_infos.remove(validator.clone());
        env.storage()
            .instance()
            .set(&VALIDATOR_INFO, &validator_infos);

        // Remove stake
        let mut stakes = stakes;
        stakes.remove(validator.clone());
        env.storage().instance().set(&VALIDATOR_STAKES, &stakes);

        // Update consensus state
        Self::update_consensus_state(env)?;

        // Emit event
        ValidatorUnregisteredEvent {
            validator: validator.clone(),
            unstaked_amount: stake,
            left_at: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    /// Create a new bridge proposal.
    ///
    /// Proposals represent a cross-chain message that validators must reach
    /// consensus on before the bridge releases funds on the destination chain.
    ///
    /// # Expiry Dual-Signal
    ///
    /// Two independent expiry signals are stored:
    /// - `expires_at`: wall-clock timestamp (`now + PROPOSAL_TIMEOUT`).
    /// - `PROPOSAL_EXPIRES_SEQ`: ledger sequence deadline, computed via
    ///   `ledger_time::seconds_to_ledger_delta` as a fallback for networks
    ///   where `timestamp()` is unreliable.
    ///
    /// A proposal is considered expired if *either* signal is exceeded.
    ///
    /// # TODO
    /// - Add a proposer field so off-chain indexers can attribute proposals
    ///   to specific relayers for analytics and accountability.
    pub fn create_proposal(env: &Env, message: CrossChainMessage) -> Result<u64, BridgeError> {
        // Get proposal counter
        let mut proposal_counter: u64 = env
            .storage()
            .instance()
            .get(&PROPOSAL_COUNTER)
            .unwrap_or(0u64);
        proposal_counter += 1;

        // Get consensus state for required votes
        let consensus_state: ConsensusState = env
            .storage()
            .instance()
            .get(&CONSENSUS_STATE)
            .unwrap_or(ConsensusState {
                total_stake: 0,
                active_validators: 0,
                byzantine_threshold: 1,
                last_consensus_round: 0,
            });

        let required_votes = consensus_state.byzantine_threshold;

        // Create proposal
        let proposal = BridgeProposal {
            proposal_id: proposal_counter,
            message: message.clone(),
            votes: Map::new(env),
            vote_count: 0,
            required_votes,
            status: ProposalStatus::Pending,
            created_at: env.ledger().timestamp(),
            expires_at: env.ledger().timestamp() + PROPOSAL_TIMEOUT,
        };

        // Store proposal
        let mut proposals: Map<u64, BridgeProposal> = env
            .storage()
            .instance()
            .get(&BRIDGE_PROPOSALS)
            .unwrap_or_else(|| Map::new(env));
        proposals.set(proposal_counter, proposal);
        env.storage().instance().set(&BRIDGE_PROPOSALS, &proposals);
        env.storage()
            .instance()
            .set(&PROPOSAL_COUNTER, &proposal_counter);

        // Store sequence-based expiry fallback.
        let expires_seq =
            env.ledger()
                .sequence()
                .saturating_add(crate::ledger_time::seconds_to_ledger_delta(
                    PROPOSAL_TIMEOUT,
                ));
        let mut proposal_expires_seq: Map<u64, u32> = env
            .storage()
            .instance()
            .get(&PROPOSAL_EXPIRES_SEQ)
            .unwrap_or_else(|| Map::new(env));
        proposal_expires_seq.set(proposal_counter, expires_seq);
        env.storage()
            .instance()
            .set(&PROPOSAL_EXPIRES_SEQ, &proposal_expires_seq);

        // Emit event
        ProposalCreatedEvent {
            proposal_id: proposal_counter,
            message,
            required_votes,
        }
        .publish(env);

        Ok(proposal_counter)
    }

    /// Vote on a bridge proposal.
    ///
    /// # Voting Algorithm
    ///
    /// 1. Verify the caller is an active validator (registered and not rotated out).
    /// 2. Check proposal expiry using the dual-signal approach (timestamp + sequence).
    /// 3. Reject duplicate votes (one vote per validator per proposal).
    /// 4. Record the vote; increment `vote_count` only for approvals.
    /// 5. Update the validator's `last_activity` timestamp and ledger sequence
    ///    to prevent false inactivity slashing.
    /// 6. Boost the validator's reputation score for participating.
    /// 7. If `vote_count >= required_votes`, immediately execute the proposal
    ///    (mark as Approved and emit `ProposalExecutedEvent`).
    ///
    /// # Note on Rejection Votes
    ///
    /// Rejection votes are recorded in `proposal.votes` but do not increment
    /// `vote_count`.  A proposal can only be approved, not explicitly rejected —
    /// it expires if it fails to accumulate enough approvals within the timeout.
    ///
    /// # TODO
    /// - Implement explicit rejection: if `reject_count > n - byzantine_threshold`,
    ///   mark the proposal as `Rejected` early to free storage.
    pub fn vote_on_proposal(
        env: &Env,
        validator: Address,
        proposal_id: u64,
        approve: bool,
    ) -> Result<(), BridgeError> {
        validator.require_auth();

        // Check if validator is active
        if !Self::is_active_validator(env, &validator) {
            return Err(BridgeError::ValidatorNotActive);
        }

        // Get proposal
        let mut proposals: Map<u64, BridgeProposal> = env
            .storage()
            .instance()
            .get(&BRIDGE_PROPOSALS)
            .unwrap_or_else(|| Map::new(env));
        let mut proposal = proposals
            .get(proposal_id)
            .ok_or(BridgeError::ProposalNotFound)?;

        // Check if proposal is still pending
        if proposal.status != ProposalStatus::Pending {
            return Err(BridgeError::ProposalExpired);
        }

        // Check if proposal has expired
        let mut expired = env.ledger().timestamp() > proposal.expires_at;
        if !expired {
            let proposal_expires_seq: Map<u64, u32> = env
                .storage()
                .instance()
                .get(&PROPOSAL_EXPIRES_SEQ)
                .unwrap_or_else(|| Map::new(env));
            if let Some(expires_seq) = proposal_expires_seq.get(proposal_id) {
                expired = env.ledger().sequence() > expires_seq;
            }
        }
        if expired {
            proposal.status = ProposalStatus::Expired;
            proposals.set(proposal_id, proposal);
            env.storage().instance().set(&BRIDGE_PROPOSALS, &proposals);
            return Err(BridgeError::ProposalExpired);
        }

        // Check if validator already voted
        if proposal.votes.get(validator.clone()).is_some() {
            return Err(BridgeError::ProposalAlreadyVoted);
        }

        // Record vote
        proposal.votes.set(validator.clone(), approve);
        if approve {
            proposal.vote_count += 1;
        }
        proposals.set(proposal_id, proposal.clone());
        env.storage().instance().set(&BRIDGE_PROPOSALS, &proposals);

        // Update validator activity and boost reputation for participation
        Self::update_validator_activity(env, &validator)?;
        Self::boost_reputation(env, &validator);

        // Check if proposal has reached consensus
        if proposal.vote_count >= proposal.required_votes {
            Self::execute_proposal(env, proposal_id)?;
        }

        // Emit event
        ProposalVotedEvent {
            proposal_id,
            validator: validator.clone(),
            vote: approve,
            vote_count: proposal.vote_count,
        }
        .publish(env);

        Ok(())
    }

    /// Execute a proposal that has reached consensus
    fn execute_proposal(env: &Env, proposal_id: u64) -> Result<(), BridgeError> {
        let mut proposals: Map<u64, BridgeProposal> = env
            .storage()
            .instance()
            .get(&BRIDGE_PROPOSALS)
            .unwrap_or_else(|| Map::new(env));
        let mut proposal = proposals
            .get(proposal_id)
            .ok_or(BridgeError::ProposalNotFound)?;

        // Mark as approved
        proposal.status = ProposalStatus::Approved;
        proposals.set(proposal_id, proposal.clone());
        env.storage().instance().set(&BRIDGE_PROPOSALS, &proposals);

        // Update consensus state
        let mut consensus_state: ConsensusState = env
            .storage()
            .instance()
            .get(&CONSENSUS_STATE)
            .unwrap_or(ConsensusState {
                total_stake: 0,
                active_validators: 0,
                byzantine_threshold: 1,
                last_consensus_round: 0,
            });
        consensus_state.last_consensus_round = env.ledger().timestamp();
        env.storage()
            .instance()
            .set(&CONSENSUS_STATE, &consensus_state);

        // Emit event
        ProposalExecutedEvent {
            proposal_id,
            status: ProposalStatus::Approved,
            executed_at: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    /// Update the consensus state based on current validators.
    ///
    /// # Algorithm
    ///
    /// Iterates all registered validators, summing stake and counting active
    /// entries.  Then computes the Byzantine threshold:
    ///
    /// ```text
    /// byzantine_threshold = floor(2 * active_validators / 3) + 1
    /// ```
    ///
    /// This is the minimum number of approving votes required for a proposal
    /// to reach consensus.  The formula satisfies BFT safety: with `n = 3f+1`
    /// validators, `2f+1` votes are needed, tolerating `f` Byzantine nodes.
    ///
    /// Called after every validator registration or unregistration to keep the
    /// threshold in sync with the current validator set size.
    ///
    /// # TODO
    /// - Weight the threshold by stake rather than validator count to make
    ///   Sybil attacks more expensive (stake-weighted BFT).
    fn update_consensus_state(env: &Env) -> Result<(), BridgeError> {
        let validators: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&VALIDATORS)
            .unwrap_or_else(|| Map::new(env));
        let stakes: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&VALIDATOR_STAKES)
            .unwrap_or_else(|| Map::new(env));

        let mut total_stake: i128 = 0;
        let mut active_validators: u32 = 0;

        for (validator, is_active) in validators.iter() {
            if is_active {
                active_validators += 1;
                if let Some(stake) = stakes.get(validator.clone()) {
                    total_stake += stake;
                }
            }
        }

        // Byzantine threshold: 2f+1 where n = 3f+1
        // For n validators, we need ceil(2n/3) + 1 for BFT
        let byzantine_threshold = if active_validators > 0 {
            ((2 * active_validators) / 3) + 1
        } else {
            1
        };

        let consensus_state = ConsensusState {
            total_stake,
            active_validators,
            byzantine_threshold,
            last_consensus_round: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&CONSENSUS_STATE, &consensus_state);

        Ok(())
    }

    /// Update validator activity timestamp
    fn update_validator_activity(env: &Env, validator: &Address) -> Result<(), BridgeError> {
        let mut validator_infos: Map<Address, ValidatorInfo> = env
            .storage()
            .instance()
            .get(&VALIDATOR_INFO)
            .unwrap_or_else(|| Map::new(env));

        if let Some(mut info) = validator_infos.get(validator.clone()) {
            info.last_activity = env.ledger().timestamp();
            info.total_validations += 1;
            validator_infos.set(validator.clone(), info);
            env.storage()
                .instance()
                .set(&VALIDATOR_INFO, &validator_infos);
        }

        let mut activity_seq: Map<Address, u32> = env
            .storage()
            .instance()
            .get(&VALIDATOR_ACTIVITY_SEQ)
            .unwrap_or_else(|| Map::new(env));
        activity_seq.set(validator.clone(), env.ledger().sequence());
        env.storage()
            .instance()
            .set(&VALIDATOR_ACTIVITY_SEQ, &activity_seq);

        Ok(())
    }

    /// Check if an address is an active validator
    pub fn is_active_validator(env: &Env, address: &Address) -> bool {
        let validators: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&VALIDATORS)
            .unwrap_or_else(|| Map::new(env));
        validators.get(address.clone()).unwrap_or(false)
    }

    /// Get validator info
    pub fn get_validator_info(env: &Env, validator: Address) -> Option<ValidatorInfo> {
        let validator_infos: Map<Address, ValidatorInfo> = env
            .storage()
            .instance()
            .get(&VALIDATOR_INFO)
            .unwrap_or_else(|| Map::new(env));
        validator_infos.get(validator)
    }

    /// Get proposal by ID
    pub fn get_proposal(env: &Env, proposal_id: u64) -> Option<BridgeProposal> {
        let proposals: Map<u64, BridgeProposal> = env
            .storage()
            .instance()
            .get(&BRIDGE_PROPOSALS)
            .unwrap_or_else(|| Map::new(env));
        proposals.get(proposal_id)
    }

    /// Get consensus state
    pub fn get_consensus_state(env: &Env) -> ConsensusState {
        env.storage()
            .instance()
            .get(&CONSENSUS_STATE)
            .unwrap_or(ConsensusState {
                total_stake: 0,
                active_validators: 0,
                byzantine_threshold: 1,
                last_consensus_round: 0,
            })
    }

    /// Get all active validators
    pub fn get_active_validators(env: &Env) -> Vec<Address> {
        let validators: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&VALIDATORS)
            .unwrap_or_else(|| Map::new(env));
        let mut active = Vec::new(env);
        for (validator, is_active) in validators.iter() {
            if is_active {
                active.push_back(validator.clone());
            }
        }
        active
    }

    /// Check if BFT consensus is reached for a proposal
    pub fn is_consensus_reached(env: &Env, proposal_id: u64) -> bool {
        if let Some(proposal) = Self::get_proposal(env, proposal_id) {
            proposal.vote_count >= proposal.required_votes
        } else {
            false
        }
    }

    /// Rotate validators: deactivate those below `MIN_ACTIVE_REPUTATION` or with
    /// insufficient stake, and update the consensus state.
    ///
    /// This should be called at epoch boundaries (every `ROTATION_EPOCH_ROUNDS`
    /// consensus rounds) to prevent long-term validator collusion by ensuring
    /// only validators with good standing remain active.
    pub fn rotate_validators(env: &Env) -> Result<u32, BridgeError> {
        let mut validators: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&VALIDATORS)
            .unwrap_or_else(|| Map::new(env));

        let validator_infos: Map<Address, ValidatorInfo> = env
            .storage()
            .instance()
            .get(&VALIDATOR_INFO)
            .unwrap_or_else(|| Map::new(env));

        let stakes: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&VALIDATOR_STAKES)
            .unwrap_or_else(|| Map::new(env));

        let mut rotated_out: u32 = 0;

        // Collect addresses to deactivate (can't mutate map while iterating)
        let mut to_deactivate: Vec<Address> = Vec::new(env);
        for (addr, is_active) in validators.iter() {
            if !is_active {
                continue;
            }
            let should_deactivate = if let Some(info) = validator_infos.get(addr.clone()) {
                info.reputation_score < MIN_ACTIVE_REPUTATION
                    || stakes.get(addr.clone()).unwrap_or(0) < MIN_VALIDATOR_STAKE
            } else {
                // No info record — deactivate
                true
            };
            if should_deactivate {
                to_deactivate.push_back(addr);
            }
        }

        for addr in to_deactivate.iter() {
            validators.set(addr.clone(), false);
            rotated_out += 1;
        }

        env.storage().instance().set(&VALIDATORS, &validators);

        // Record the rotation epoch in namespaced storage (issue #242)
        let current_epoch: u64 = env
            .storage()
            .instance()
            .get(&StorageKey::ValidatorRotationEpoch)
            .unwrap_or(0u64);
        env.storage()
            .instance()
            .set(&StorageKey::ValidatorRotationEpoch, &(current_epoch + 1));

        Self::update_consensus_state(env)?;
        Ok(rotated_out)
    }

    /// Check whether the current consensus round has crossed an epoch boundary
    /// and trigger rotation if so.
    pub fn maybe_rotate(env: &Env) -> Result<bool, BridgeError> {
        let state = Self::get_consensus_state(env);
        if state.last_consensus_round > 0 && state.last_consensus_round % ROTATION_EPOCH_ROUNDS == 0
        {
            Self::rotate_validators(env)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Boost a validator's reputation score for participating in consensus.
    /// Called internally after a successful vote.
    fn boost_reputation(env: &Env, validator: &Address) {
        let mut validator_infos: Map<Address, ValidatorInfo> = env
            .storage()
            .instance()
            .get(&VALIDATOR_INFO)
            .unwrap_or_else(|| Map::new(env));
        if let Some(mut info) = validator_infos.get(validator.clone()) {
            // Cap at 100
            info.reputation_score = info.reputation_score.saturating_add(1).min(100);
            validator_infos.set(validator.clone(), info);
            env.storage()
                .instance()
                .set(&VALIDATOR_INFO, &validator_infos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MIN_VALIDATOR_STAKE, PROPOSAL_TIMEOUT};
    use crate::errors::BridgeError;
    use crate::storage::PROPOSAL_EXPIRES_SEQ;
    use crate::types::CrossChainMessage;
    use crate::TeachLinkBridge;
    use crate::TeachLinkBridgeClient;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::{Bytes, Env, Map};

    fn set_ledger(env: &Env, timestamp: u64, sequence: u32) {
        env.ledger().with_mut(|li| {
            li.timestamp = timestamp;
            li.sequence_number = sequence;
        });
    }

    #[test]
    fn proposal_expiry_uses_sequence_fallback() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());

        let client = TeachLinkBridgeClient::new(&env, &contract_id);

        set_ledger(&env, 1_000, 1);

        let validator = soroban_sdk::Address::generate(&env);
        assert_eq!(
            client.try_register_validator(&validator, &MIN_VALIDATOR_STAKE),
            Ok(Ok(()))
        );

        let msg = CrossChainMessage {
            source_chain: 1,
            source_tx_hash: Bytes::from_slice(&env, &[0x11; 32]),
            nonce: 1,
            token: soroban_sdk::Address::generate(&env),
            amount: 1,
            recipient: soroban_sdk::Address::generate(&env),
            destination_chain: 2,
        };
        let proposal_id = client.try_create_bridge_proposal(&msg).unwrap().unwrap();

        // Ensure the sequence-based expiry is stored.
        let deadline = env.as_contract(&contract_id, || {
            let expires_seq: Map<u64, u32> =
                env.storage().instance().get(&PROPOSAL_EXPIRES_SEQ).unwrap();
            expires_seq.get(proposal_id).unwrap()
        });

        // Keep timestamp constant but move sequence beyond the deadline.
        // With timestamp-only logic, this would still be pending.
        set_ledger(&env, 1_000, deadline.saturating_add(1));
        let r = client.try_vote_on_proposal(&validator, &proposal_id, &true);
        assert_eq!(r, Err(Ok(BridgeError::ProposalExpired)));

        // Sanity check constant is used (guards against accidental removal).
        assert!(PROPOSAL_TIMEOUT > 0);
    }

    #[test]
    fn rotate_validators_removes_low_reputation() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());
        set_ledger(&env, 1_000, 1);

        let validator = soroban_sdk::Address::generate(&env);
        let client = TeachLinkBridgeClient::new(&env, &contract_id);
        client.register_validator(&validator, &MIN_VALIDATOR_STAKE);

        // Manually drop reputation below threshold
        env.as_contract(&contract_id, || {
            use crate::storage::VALIDATOR_INFO;
            use crate::types::ValidatorInfo;
            let mut infos: Map<soroban_sdk::Address, ValidatorInfo> =
                env.storage().instance().get(&VALIDATOR_INFO).unwrap();
            let mut info = infos.get(validator.clone()).unwrap();
            info.reputation_score = 10; // below MIN_ACTIVE_REPUTATION (40)
            infos.set(validator.clone(), info);
            env.storage().instance().set(&VALIDATOR_INFO, &infos);
        });

        env.as_contract(&contract_id, || {
            use crate::bft_consensus::BFTConsensus;
            let rotated = BFTConsensus::rotate_validators(&env).unwrap();
            assert_eq!(rotated, 1);
            assert!(!BFTConsensus::is_active_validator(&env, &validator));
        });
    }

    #[test]
    fn voting_boosts_reputation() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());
        set_ledger(&env, 1_000, 1);

        let validator = soroban_sdk::Address::generate(&env);
        let client = TeachLinkBridgeClient::new(&env, &contract_id);
        client.register_validator(&validator, &MIN_VALIDATOR_STAKE);

        // Lower reputation below max so the boost is observable (validators start at 100)
        env.as_contract(&contract_id, || {
            use crate::storage::VALIDATOR_INFO;
            use crate::types::ValidatorInfo;
            let mut infos: Map<soroban_sdk::Address, ValidatorInfo> =
                env.storage().instance().get(&VALIDATOR_INFO).unwrap();
            let mut info = infos.get(validator.clone()).unwrap();
            info.reputation_score = 90;
            infos.set(validator.clone(), info);
            env.storage().instance().set(&VALIDATOR_INFO, &infos);
        });

        let msg = CrossChainMessage {
            source_chain: 1,
            source_tx_hash: soroban_sdk::Bytes::from_slice(&env, &[0xAB; 32]),
            nonce: 1,
            token: soroban_sdk::Address::generate(&env),
            amount: 1,
            recipient: soroban_sdk::Address::generate(&env),
            destination_chain: 2,
        };
        let proposal_id = client.create_bridge_proposal(&msg);
        client.vote_on_proposal(&validator, &proposal_id, &true);

        let after_rep = env.as_contract(&contract_id, || {
            use crate::storage::VALIDATOR_INFO;
            use crate::types::ValidatorInfo;
            let infos: Map<soroban_sdk::Address, ValidatorInfo> =
                env.storage().instance().get(&VALIDATOR_INFO).unwrap();
            infos.get(validator.clone()).unwrap().reputation_score
        });

        assert!(after_rep > 90, "reputation should increase after voting");
    }
}
