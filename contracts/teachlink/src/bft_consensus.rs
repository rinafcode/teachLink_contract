//! Byzantine Fault Tolerant (BFT) Consensus Module
//!
//! This module implements a BFT consensus mechanism for bridge validators,
//! ensuring that the bridge can tolerate up to f faulty validators out of 3f+1 total validators.

use crate::errors::BridgeError;
use crate::events::{
    ProposalCreatedEvent, ProposalExecutedEvent, ProposalVotedEvent, ValidatorRegisteredEvent,
    ValidatorUnregisteredEvent,
};
use crate::storage::{
    BRIDGE_PROPOSALS, CONSENSUS_STATE, PROPOSAL_COUNTER, VALIDATORS, VALIDATOR_INFO,
    VALIDATOR_STAKES,
};
use crate::types::{
    BridgeProposal, ConsensusState, CrossChainMessage, ProposalStatus, ValidatorInfo,
};
use soroban_sdk::{Address, Env, Map, Vec};

/// Minimum stake required to become a validator
pub const MIN_VALIDATOR_STAKE: i128 = 100_000_000; // 100 tokens with 6 decimals

/// Proposal timeout in seconds (24 hours)
pub const PROPOSAL_TIMEOUT: u64 = 86_400;

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
        if env.storage().instance().has(&crate::storage::DataKey::Validator(validator.clone())) {
             if env.storage().instance().get::<_, bool>(&crate::storage::DataKey::Validator(validator.clone())).unwrap_or(false) {
                return Err(BridgeError::AlreadyInitialized);
             }
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

        // Store validator data
        env.storage().instance().set(&crate::storage::DataKey::Validator(validator.clone()), &true);
        env.storage().instance().set(&crate::storage::DataKey::ValidatorMetadata(validator.clone()), &validator_info);
        env.storage().instance().set(&crate::storage::DataKey::ValidatorStake(validator.clone()), &stake);

        // Maintain list
        let mut list: Vec<Address> = env.storage().instance().get(&crate::storage::VALIDATORS_LIST).unwrap_or_else(|| Vec::new(env));
        if !list.contains(&validator) {
            list.push_back(validator.clone());
            env.storage().instance().set(&crate::storage::VALIDATORS_LIST, &list);
        }

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
        if !env.storage().instance().get::<_, bool>(&crate::storage::DataKey::Validator(validator.clone())).unwrap_or(false) {
            return Err(BridgeError::InvalidValidatorSignature);
        }

        // Get stake
        let stake = env.storage().instance().get::<_, i128>(&crate::storage::DataKey::ValidatorStake(validator.clone())).unwrap_or(0);

        // Remove validator
        env.storage().instance().set(&crate::storage::DataKey::Validator(validator.clone()), &false);
        env.storage().instance().remove(&crate::storage::DataKey::ValidatorMetadata(validator.clone()));
        env.storage().instance().remove(&crate::storage::DataKey::ValidatorStake(validator.clone()));

        // Remove from list
        let mut list: Vec<Address> = env.storage().instance().get(&crate::storage::VALIDATORS_LIST).unwrap_or_else(|| Vec::new(env));
        if let Some(pos) = list.iter().position(|v| v == validator) {
            list.remove(pos as u32);
            env.storage().instance().set(&crate::storage::VALIDATORS_LIST, &list);
        }

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

    /// Create a new bridge proposal
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

        // Create proposal (votes field removed from struct)
        let proposal = BridgeProposal {
            proposal_id: proposal_counter,
            message: message.clone(),
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

        // Emit event
        ProposalCreatedEvent {
            proposal_id: proposal_counter,
            message,
            required_votes,
        }
        .publish(env);

        Ok(proposal_counter)
    }

    /// Vote on a bridge proposal
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
        if env.ledger().timestamp() > proposal.expires_at {
            proposal.status = ProposalStatus::Expired;
            proposals.set(proposal_id, proposal);
            env.storage().instance().set(&BRIDGE_PROPOSALS, &proposals);
            return Err(BridgeError::ProposalExpired);
        }

        // Check if validator already voted
        let vote_key = crate::storage::DataKey::ProposalVote(proposal_id, validator.clone());
        if env.storage().instance().has(&vote_key) {
            return Err(BridgeError::ProposalAlreadyVoted);
        }

        // Record vote
        env.storage().instance().set(&vote_key, &approve);
        if approve {
            proposal.vote_count += 1;
        }
        proposals.set(proposal_id, proposal.clone());
        env.storage().instance().set(&BRIDGE_PROPOSALS, &proposals);

        // Update validator activity
        Self::update_validator_activity(env, &validator)?;

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

    /// Update the consensus state based on current validators
    fn update_consensus_state(env: &Env) -> Result<(), BridgeError> {
        let validators: Vec<Address> = env
            .storage()
            .instance()
            .get(&crate::storage::VALIDATORS_LIST)
            .unwrap_or_else(|| Vec::new(env));

        let mut total_stake: i128 = 0;
        let mut active_validators: u32 = 0;

        for validator in validators.iter() {
            if Self::is_active_validator(env, &validator) {
                active_validators += 1;
                let stake = env.storage().instance().get(&crate::storage::DataKey::ValidatorStake(validator.clone())).unwrap_or(0);
                total_stake += stake;
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

        Ok(())
    }

    /// Check if an address is an active validator
    pub fn is_active_validator(env: &Env, address: &Address) -> bool {
        env.storage()
            .instance()
            .get(&crate::storage::DataKey::Validator(address.clone()))
            .unwrap_or(false)
    }

    /// Get validator info
    pub fn get_validator_info(env: &Env, validator: Address) -> Option<ValidatorInfo> {
        env.storage().instance().get(&crate::storage::DataKey::ValidatorMetadata(validator))
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
        let validators: Vec<Address> = env
            .storage()
            .instance()
            .get(&crate::storage::VALIDATORS_LIST)
            .unwrap_or_else(|| Vec::new(env));
        let mut active = Vec::new(env);
        for validator in validators.iter() {
            if Self::is_active_validator(env, &validator) {
                active.push_back(validator);
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
}
