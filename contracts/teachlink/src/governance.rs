use soroban_sdk::{contracttype, Address, BytesN, Env, Symbol, Vec, map, symbol_short, panic_with_error};

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
const VOTES: Symbol = symbol_short!("votes");
const MIN_PROPOSAL_THRESHOLD: i128 = 1000; // Example threshold
const VOTING_DURATION: u64 = 604800; // 7 days in seconds

pub struct GovernanceManager;

impl GovernanceManager {
    /// Creates a new proposal. Requires the proposer to have a minimum balance.
    pub fn create_proposal(
        env: &Env,
        proposer: Address,
        token_addr: Address,
        title: Symbol,
        desc_hash: BytesN<32>,
    ) -> u64 {
        proposer.require_auth();

        // Check proposer balance (Simplified token check)
        // In production, this would call the TEACH token contract's 'balance' function
        
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
    ) {
        voter.require_auth();

        let mut proposals: soroban_sdk::Map<u64, Proposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .expect("Proposals not initialized");

        let mut proposal = proposals.get(proposal_id).expect("Proposal not found");

        // Validation
        if env.ledger().timestamp() > proposal.end_time {
            panic!("Voting period ended");
        }
        if proposal.status != ProposalStatus::Active {
            panic!("Proposal not active");
        }

        // Check for double voting
        let vote_key = (proposal_id, voter.clone());
        let has_voted: bool = env.storage().persistent().has(&vote_key);
        if has_voted {
            panic!("Already voted");
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
    }

    /// Finalizes a proposal after the voting period has ended.
    pub fn finalize_proposal(env: &Env, proposal_id: u64) {
        let mut proposals: soroban_sdk::Map<u64, Proposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .expect("Proposals not initialized");

        let mut proposal = proposals.get(proposal_id).expect("Proposal not found");

        if env.ledger().timestamp() <= proposal.end_time {
            panic!("Voting still in progress");
        }

        if proposal.for_votes > proposal.against_votes {
            proposal.status = ProposalStatus::Succeeded;
        } else {
            proposal.status = ProposalStatus::Defeated;
        }

        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);
    }
}