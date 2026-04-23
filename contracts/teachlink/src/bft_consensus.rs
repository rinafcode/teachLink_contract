use crate::errors::BridgeError;
use crate::events::{
    ProposalCreatedEvent, ProposalExecutedEvent, ProposalVotedEvent,
    ValidatorRegisteredEvent, ValidatorUnregisteredEvent,
};
use crate::storage::{
    BRIDGE_PROPOSALS, CONSENSUS_STATE, PROPOSAL_COUNTER,
    VALIDATORS, VALIDATOR_INFO, VALIDATOR_STAKES,
};
use crate::types::{
    BridgeProposal, ConsensusState, CrossChainMessage,
    ProposalStatus, ValidatorInfo,
};
use soroban_sdk::{Address, Env, Map, Vec};

pub const MIN_VALIDATOR_STAKE: i128 = 100_000_000;
pub const MAX_VALIDATOR_STAKE: i128 = 1_000_000_000;
pub const ROTATION_PERIOD: u64 = 2_592_000;
pub const MIN_REPUTATION_SCORE: u32 = 50;
pub const PROPOSAL_TIMEOUT: u64 = 86_400; // ✅ FIXED

pub struct BFTConsensus;

impl BFTConsensus {

    pub fn register_validator(
        env: &Env,
        validator: Address,
        stake: i128,
    ) -> Result<(), BridgeError> {
        validator.require_auth();

        if stake < MIN_VALIDATOR_STAKE {
            return Err(BridgeError::InsufficientStake);
        }

        if stake > MAX_VALIDATOR_STAKE {
            return Err(BridgeError::StakeTooHigh);
        }

        let validators: Map<Address, bool> =
            env.storage().instance().get(&VALIDATORS)
            .unwrap_or_else(|| Map::new(env));

        if validators.get(validator.clone()).unwrap_or(false) {
            return Err(BridgeError::AlreadyInitialized);
        }

        let now = env.ledger().timestamp();

        let info = ValidatorInfo {
            address: validator.clone(),
            stake,
            reputation_score: 100,
            is_active: true,
            joined_at: now,
            last_activity: now,
            total_validations: 0,
            missed_validations: 0,
            slashed_amount: 0,
        };

        let mut infos: Map<Address, ValidatorInfo> =
            env.storage().instance().get(&VALIDATOR_INFO)
            .unwrap_or_else(|| Map::new(env));
        infos.set(validator.clone(), info);
        env.storage().instance().set(&VALIDATOR_INFO, &infos);

        let mut stakes: Map<Address, i128> =
            env.storage().instance().get(&VALIDATOR_STAKES)
            .unwrap_or_else(|| Map::new(env));
        stakes.set(validator.clone(), stake);
        env.storage().instance().set(&VALIDATOR_STAKES, &stakes);

        let mut validators = validators;
        validators.set(validator.clone(), true);
        env.storage().instance().set(&VALIDATORS, &validators);

        Self::update_consensus_state(env)?;

        ValidatorRegisteredEvent {
            validator,
            stake,
            joined_at: now,
        }.publish(env);

        Ok(())
    }

    pub fn create_proposal(
        env: &Env,
        message: CrossChainMessage
    ) -> Result<u64, BridgeError> {

        let mut counter: u64 =
            env.storage().instance().get(&PROPOSAL_COUNTER)
            .unwrap_or(0);

        counter += 1;

        let state = Self::get_consensus_state(env);

        let proposal = BridgeProposal {
            proposal_id: counter,
            message: message.clone(),
            votes: Map::new(env),
            vote_count: 0,
            required_votes: state.byzantine_threshold,
            status: ProposalStatus::Pending,
            created_at: env.ledger().timestamp(),
            expires_at: env.ledger().timestamp() + PROPOSAL_TIMEOUT,
        };

        let mut proposals: Map<u64, BridgeProposal> =
            env.storage().instance().get(&BRIDGE_PROPOSALS)
            .unwrap_or_else(|| Map::new(env));

        proposals.set(counter, proposal);

        env.storage().instance().set(&BRIDGE_PROPOSALS, &proposals);
        env.storage().instance().set(&PROPOSAL_COUNTER, &counter);

        ProposalCreatedEvent {
            proposal_id: counter,
            message,
            required_votes: state.byzantine_threshold,
        }.publish(env);

        Ok(counter)
    }

    fn update_consensus_state(env: &Env) -> Result<(), BridgeError> {
        let validators: Map<Address, bool> =
            env.storage().instance().get(&VALIDATORS)
            .unwrap_or_else(|| Map::new(env));

        let stakes: Map<Address, i128> =
            env.storage().instance().get(&VALIDATOR_STAKES)
            .unwrap_or_else(|| Map::new(env));

        let mut total_stake = 0;
        let mut active = 0;

        for (addr, is_active) in validators.iter() {
            if is_active {
                active += 1;
                if let Some(stake) = stakes.get(addr) {
                    total_stake += stake;
                }
            }
        }

        let threshold = if active > 0 {
            ((2 * active) / 3) + 1
        } else {
            1
        };

        let state = ConsensusState {
            total_stake,
            active_validators: active,
            byzantine_threshold: threshold,
            last_consensus_round: env.ledger().timestamp(),
        };

        env.storage().instance().set(&CONSENSUS_STATE, &state);

        Ok(())
    }

    pub fn get_consensus_state(env: &Env) -> ConsensusState {
        env.storage().instance().get(&CONSENSUS_STATE)
        .unwrap_or(ConsensusState {
            total_stake: 0,
            active_validators: 0,
            byzantine_threshold: 1,
            last_consensus_round: 0,
        })
    }

    /// ✅ FIXED rotation (no sort, no type issues)
    pub fn rotate_validators(env: &Env) -> Result<(), BridgeError> {
        let now = env.ledger().timestamp();
        let state = Self::get_consensus_state(env);

        if now - state.last_consensus_round < ROTATION_PERIOD {
            return Ok(());
        }

        let infos: Map<Address, ValidatorInfo> =
            env.storage().instance().get(&VALIDATOR_INFO)
            .unwrap_or_else(|| Map::new(env));

        let mut validators: Map<Address, bool> = Map::new(env);

        let mut count: u32 = 0;
        let max_validators: u32 = 100;

        for (addr, info) in infos.iter() {
            if count >= max_validators {
                break;
            }

            if info.reputation_score >= MIN_REPUTATION_SCORE &&
               info.stake >= MIN_VALIDATOR_STAKE {

                validators.set(addr, true);
                count += 1;
            }
        }

        env.storage().instance().set(&VALIDATORS, &validators);

        Self::update_consensus_state(env)?;

        Ok(())
    }
}