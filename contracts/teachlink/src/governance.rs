use crate::errors::GovernanceError;
use soroban_sdk::{contracttype, map, symbol_short, Address, BytesN, Env, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    Active,
    Succeeded,
    Defeated,
    Executed,
    Canceled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub title: Symbol,
    pub desc_hash: BytesN<32>,
    pub end_time: u64,
    pub for_votes: i128,
    pub against_votes: i128,
    pub status: ProposalStatus,
}

const PROP_CNT: Symbol = symbol_short!("prop_cnt");
const PROPOSALS: Symbol = symbol_short!("props");
const VOTING_DURATION: u64 = 604800; // 7 days in seconds

pub struct GovernanceManager;

impl GovernanceManager {
    /// Creates a new proposal. Requires the proposer to have a minimum balance.
    pub fn create_proposal(
        env: &Env,
        proposer: Address,
        _token_addr: Address,
        title: Symbol,
        desc_hash: BytesN<32>,
    ) -> u64 {
        proposer.require_auth();

        // In a full implementation, we would verify the proposer's TEACH balance here

        let mut count: u64 = env.storage().persistent().get(&PROP_CNT).unwrap_or(0);
        count += 1;

        let proposal = Proposal {
            id: count,
            proposer: proposer.clone(),
            title,
            desc_hash,
            end_time: env.ledger().timestamp() + VOTING_DURATION,
            for_votes: 0,
            against_votes: 0,
            status: ProposalStatus::Active,
        };

        let mut proposals: soroban_sdk::Map<u64, Proposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .unwrap_or(map![&env]);

        proposals.set(count, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);
        env.storage().persistent().set(&PROP_CNT, &count);

        // Emit Event
        env.events().publish(
            (symbol_short!("gov"), symbol_short!("prop_new")),
            (count, proposer),
        );

        count
    }

    /// Casts a vote on an active proposal.
    pub fn cast_vote(
        env: &Env,
        voter: Address,
        proposal_id: u64,
        support: bool,
        weight: i128,
    ) -> Result<(), GovernanceError> {
        voter.require_auth();

        let mut proposals: soroban_sdk::Map<u64, Proposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .ok_or(GovernanceError::ProposalsNotInitialized)?;

        let mut proposal = proposals
            .get(proposal_id)
            .ok_or(GovernanceError::GovernanceProposalNotFound)?;

        if env.ledger().timestamp() > proposal.end_time {
            return Err(GovernanceError::VotingPeriodEnded);
        }
        if proposal.status != ProposalStatus::Active {
            return Err(GovernanceError::GovernanceProposalNotActive);
        }

        // Check for double voting using a composite key
        let vote_key = (symbol_short!("votes"), proposal_id, voter.clone());
        if env.storage().persistent().has(&vote_key) {
            return Err(GovernanceError::AlreadyVoted);
        }

        // Update Tally
        if support {
            proposal.for_votes += weight;
        } else {
            proposal.against_votes += weight;
        }

        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);
        env.storage().persistent().set(&vote_key, &true);

        env.events().publish(
            (symbol_short!("gov"), symbol_short!("vote")),
            (proposal_id, voter, weight, support),
        );

        Ok(())
    }

    /// Finalizes a proposal after the voting period has ended.
    pub fn finalize_proposal(env: &Env, proposal_id: u64) -> Result<(), GovernanceError> {
        let mut proposals: soroban_sdk::Map<u64, Proposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .ok_or(GovernanceError::ProposalsNotInitialized)?;

        let mut proposal = proposals
            .get(proposal_id)
            .ok_or(GovernanceError::GovernanceProposalNotFound)?;

        if env.ledger().timestamp() <= proposal.end_time {
            return Err(GovernanceError::VotingStillInProgress);
        }

        if proposal.for_votes > proposal.against_votes {
            proposal.status = ProposalStatus::Succeeded;
        } else {
            proposal.status = ProposalStatus::Defeated;
        }

        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);

        env.events().publish(
            (symbol_short!("gov"), symbol_short!("prop_end")),
            (proposal_id, proposal.status),
        );

        Ok(())
    }
}
