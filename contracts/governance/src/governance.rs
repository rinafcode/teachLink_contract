//! Core Governance Logic for the TeachLink Platform
//!
//! This module implements decentralized governance through proposal management,
//! voting, and execution. Token holders can participate in platform decisions
//! by creating proposals and voting with their token balance as voting power.
//!
//! # Proposal Lifecycle
//!
//! 1. **Creation**: A token holder with sufficient balance creates a proposal
//! 2. **Voting**: Token holders vote during the voting period
//! 3. **Finalization**: After voting ends, the proposal is finalized as passed or failed
//! 4. **Execution**: Passed proposals can be executed after the execution delay
//!
//! # Governance Parameters
//!
//! - `proposal_threshold`: Minimum token balance to create proposals
//! - `quorum`: Minimum total votes required for a valid decision
//! - `voting_period`: Duration of the voting window (in seconds)
//! - `execution_delay`: Waiting period before executing passed proposals
//!
//! # Example
//!
//! ```ignore
//! // Create a proposal
//! let proposal_id = Governance::create_proposal(
//!     &env, proposer, title, description, ProposalType::FeeChange, None
//! );
//!
//! // Vote on the proposal
//! Governance::cast_vote(&env, proposal_id, voter, VoteDirection::For);
//!
//! // After voting period ends, finalize
//! Governance::finalize_proposal(&env, proposal_id);
//!
//! // After execution delay, execute
//! Governance::execute_proposal(&env, proposal_id, executor);
//! ```

use soroban_sdk::{token, Address, Bytes, Env};

use crate::events;
use crate::storage::{CONFIG, PROPOSALS, PROPOSAL_COUNT, VOTES};
use crate::types::{
    GovernanceConfig, Proposal, ProposalStatus, ProposalType, Vote, VoteDirection, VoteKey,
};

/// Governance contract implementation.
///
/// Provides on-chain governance for the TeachLink platform through
/// token-weighted voting on proposals.
pub struct Governance;

impl Governance {
    /// Initialize the governance contract.
    ///
    /// Sets up the governance system with the specified configuration.
    /// This function can only be called once.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `token` - Address of the governance token (used for voting power)
    /// * `admin` - Address with administrative privileges
    /// * `proposal_threshold` - Minimum token balance to create proposals
    /// * `quorum` - Minimum total votes required for valid decisions
    /// * `voting_period` - Duration of voting in seconds
    /// * `execution_delay` - Delay before executing passed proposals in seconds
    ///
    /// # Panics
    /// * If the contract is already initialized
    pub fn initialize(
        env: &Env,
        token: Address,
        admin: Address,
        proposal_threshold: i128,
        quorum: i128,
        voting_period: u64,
        execution_delay: u64,
    ) {
        if env.storage().instance().has(&CONFIG) {
            panic!("Already initialized");
        }

        let config = GovernanceConfig {
            token,
            admin,
            proposal_threshold,
            quorum,
            voting_period,
            execution_delay,
        };

        env.storage().instance().set(&CONFIG, &config);
        env.storage().instance().set(&PROPOSAL_COUNT, &0u64);
    }

    /// Get the current governance configuration.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// The current governance configuration parameters.
    ///
    /// # Panics
    /// * If the contract is not initialized
    pub fn get_config(env: &Env) -> GovernanceConfig {
        env.storage()
            .instance()
            .get(&CONFIG)
            .expect("Not initialized")
    }

    /// Get the admin address.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// The current admin address.
    pub fn get_admin(env: &Env) -> Address {
        Self::get_config(env).admin
    }

    /// Get the governance token address.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// The governance token address used for voting power.
    pub fn get_token(env: &Env) -> Address {
        Self::get_config(env).token
    }

    /// Create a new governance proposal.
    ///
    /// Creates a proposal that immediately enters the active voting state.
    /// The proposer must hold at least `proposal_threshold` tokens.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `proposer` - Address creating the proposal (must authorize)
    /// * `title` - Short descriptive title for the proposal
    /// * `description` - Detailed description of the proposal
    /// * `proposal_type` - Category of the proposal
    /// * `execution_data` - Optional data for proposal execution
    ///
    /// # Returns
    /// The unique proposal ID.
    ///
    /// # Authorization
    /// Requires authorization from `proposer`.
    ///
    /// # Panics
    /// * If proposer's token balance is below `proposal_threshold`
    ///
    /// # Events
    /// Emits a `proposal_created` event.
    pub fn create_proposal(
        env: &Env,
        proposer: Address,
        title: Bytes,
        description: Bytes,
        proposal_type: ProposalType,
        execution_data: Option<Bytes>,
    ) -> u64 {
        proposer.require_auth();

        let config = Self::get_config(env);

        // Check proposer has enough tokens
        let token_client = token::Client::new(env, &config.token);
        let balance = token_client.balance(&proposer);
        if balance < config.proposal_threshold {
            panic!("Insufficient token balance to create proposal");
        }

        // Generate proposal ID
        let mut proposal_count: u64 = env.storage().instance().get(&PROPOSAL_COUNT).unwrap_or(0);
        proposal_count += 1;

        let now = env.ledger().timestamp();
        let voting_start = now; // Voting starts immediately
        let voting_end = voting_start + config.voting_period;

        let proposal = Proposal {
            id: proposal_count,
            proposer: proposer.clone(),
            title: title.clone(),
            description,
            proposal_type: proposal_type.clone(),
            status: ProposalStatus::Active, // Active immediately
            created_at: now,
            voting_start,
            voting_end,
            for_votes: 0,
            against_votes: 0,
            abstain_votes: 0,
            execution_data,
        };

        // Store proposal
        env.storage()
            .persistent()
            .set(&(PROPOSALS, proposal_count), &proposal);
        env.storage()
            .instance()
            .set(&PROPOSAL_COUNT, &proposal_count);

        // Emit event
        events::proposal_created(env, proposal_count, &proposer, &title, &proposal_type);

        proposal_count
    }

    /// Cast a vote on an active proposal.
    ///
    /// Records a vote with the voter's token balance as voting power.
    /// Each address can only vote once per proposal.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `proposal_id` - ID of the proposal to vote on
    /// * `voter` - Address casting the vote (must authorize)
    /// * `direction` - Vote direction (For, Against, or Abstain)
    ///
    /// # Returns
    /// The voting power used (voter's token balance).
    ///
    /// # Authorization
    /// Requires authorization from `voter`.
    ///
    /// # Panics
    /// * If the proposal does not exist
    /// * If the proposal is not in Active status
    /// * If the voting period has not started or has ended
    /// * If the voter has already voted on this proposal
    /// * If the voter has no voting power (zero token balance)
    ///
    /// # Events
    /// Emits a `vote_cast` event.
    pub fn cast_vote(
        env: &Env,
        proposal_id: u64,
        voter: Address,
        direction: VoteDirection,
    ) -> i128 {
        voter.require_auth();

        let config = Self::get_config(env);

        // Get proposal
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&(PROPOSALS, proposal_id))
            .expect("Proposal not found");

        // Check proposal is active
        if proposal.status != ProposalStatus::Active {
            panic!("Proposal is not active");
        }

        // Check voting period
        let now = env.ledger().timestamp();
        if now < proposal.voting_start || now > proposal.voting_end {
            panic!("Voting period not active");
        }

        // Check if already voted
        let vote_key = VoteKey {
            proposal_id,
            voter: voter.clone(),
        };
        if env.storage().persistent().has(&(VOTES, vote_key.clone())) {
            panic!("Already voted on this proposal");
        }

        // Get voting power (token balance)
        let token_client = token::Client::new(env, &config.token);
        let power = token_client.balance(&voter);
        if power <= 0 {
            panic!("No voting power");
        }

        // Record vote
        let vote = Vote {
            voter: voter.clone(),
            proposal_id,
            power,
            direction: direction.clone(),
            timestamp: now,
        };
        env.storage().persistent().set(&(VOTES, vote_key), &vote);

        // Update proposal vote counts
        match direction {
            VoteDirection::For => proposal.for_votes += power,
            VoteDirection::Against => proposal.against_votes += power,
            VoteDirection::Abstain => proposal.abstain_votes += power,
        }

        env.storage()
            .persistent()
            .set(&(PROPOSALS, proposal_id), &proposal);

        // Emit event
        events::vote_cast(env, proposal_id, &voter, &direction, power);

        power
    }

    /// Finalize a proposal after the voting period ends.
    ///
    /// Determines whether the proposal passed or failed based on
    /// quorum requirements and vote counts.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `proposal_id` - ID of the proposal to finalize
    ///
    /// # Panics
    /// * If the proposal does not exist
    /// * If the proposal is not in Active status
    /// * If the voting period has not ended
    ///
    /// # Events
    /// Emits a `proposal_status_changed` event.
    pub fn finalize_proposal(env: &Env, proposal_id: u64) {
        let config = Self::get_config(env);

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&(PROPOSALS, proposal_id))
            .expect("Proposal not found");

        // Check proposal is still active
        if proposal.status != ProposalStatus::Active {
            panic!("Proposal is not active");
        }

        // Check voting period has ended
        let now = env.ledger().timestamp();
        if now <= proposal.voting_end {
            panic!("Voting period not ended");
        }

        let old_status = proposal.status.clone();

        // Calculate total votes
        let total_votes = proposal.for_votes + proposal.against_votes + proposal.abstain_votes;

        // Check quorum and majority
        if total_votes >= config.quorum && proposal.for_votes > proposal.against_votes {
            proposal.status = ProposalStatus::Passed;
        } else {
            proposal.status = ProposalStatus::Failed;
        }

        env.storage()
            .persistent()
            .set(&(PROPOSALS, proposal_id), &proposal);

        events::proposal_status_changed(env, proposal_id, &old_status, &proposal.status);
    }

    /// Execute a passed proposal.
    ///
    /// Marks a proposal as executed after the execution delay has passed.
    /// Actual execution logic depends on the proposal type.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `proposal_id` - ID of the proposal to execute
    /// * `executor` - Address triggering execution (must authorize)
    ///
    /// # Authorization
    /// Requires authorization from `executor`.
    ///
    /// # Panics
    /// * If the proposal does not exist
    /// * If the proposal has not passed
    /// * If the execution delay has not elapsed
    ///
    /// # Events
    /// Emits `proposal_status_changed` and `proposal_executed` events.
    pub fn execute_proposal(env: &Env, proposal_id: u64, executor: Address) {
        executor.require_auth();

        let config = Self::get_config(env);

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&(PROPOSALS, proposal_id))
            .expect("Proposal not found");

        // Check proposal has passed
        if proposal.status != ProposalStatus::Passed {
            panic!("Proposal has not passed");
        }

        // Check execution delay has passed
        let now = env.ledger().timestamp();
        if now < proposal.voting_end + config.execution_delay {
            panic!("Execution delay not met");
        }

        let old_status = proposal.status.clone();
        proposal.status = ProposalStatus::Executed;

        env.storage()
            .persistent()
            .set(&(PROPOSALS, proposal_id), &proposal);

        events::proposal_status_changed(env, proposal_id, &old_status, &proposal.status);
        events::proposal_executed(env, proposal_id, &executor);
    }

    /// Cancel a proposal.
    ///
    /// The proposer can cancel during the voting period.
    /// The admin can cancel at any time (except executed proposals).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `proposal_id` - ID of the proposal to cancel
    /// * `caller` - Address requesting cancellation (must authorize)
    ///
    /// # Authorization
    /// Requires authorization from `caller` (must be proposer or admin).
    ///
    /// # Panics
    /// * If the proposal does not exist
    /// * If caller is neither proposer nor admin
    /// * If proposer attempts to cancel after voting ends
    /// * If the proposal has already been executed
    ///
    /// # Events
    /// Emits `proposal_status_changed` and `proposal_cancelled` events.
    pub fn cancel_proposal(env: &Env, proposal_id: u64, caller: Address) {
        caller.require_auth();

        let config = Self::get_config(env);

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&(PROPOSALS, proposal_id))
            .expect("Proposal not found");

        // Check if cancellable
        let is_admin = caller == config.admin;
        let is_proposer = caller == proposal.proposer;
        let now = env.ledger().timestamp();
        let voting_ended = now > proposal.voting_end;

        if !is_admin && !is_proposer {
            panic!("Only proposer or admin can cancel");
        }

        if !is_admin && voting_ended {
            panic!("Proposer can only cancel during voting period");
        }

        // Cannot cancel executed proposals
        if proposal.status == ProposalStatus::Executed {
            panic!("Cannot cancel executed proposal");
        }

        let old_status = proposal.status.clone();
        proposal.status = ProposalStatus::Cancelled;

        env.storage()
            .persistent()
            .set(&(PROPOSALS, proposal_id), &proposal);

        events::proposal_status_changed(env, proposal_id, &old_status, &proposal.status);
        events::proposal_cancelled(env, proposal_id, &caller);
    }

    /// Update governance configuration.
    ///
    /// Allows the admin to modify governance parameters.
    /// Only provided values are updated; `None` values are ignored.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `new_proposal_threshold` - New minimum tokens for proposals (optional)
    /// * `new_quorum` - New quorum requirement (optional)
    /// * `new_voting_period` - New voting duration in seconds (optional)
    /// * `new_execution_delay` - New execution delay in seconds (optional)
    ///
    /// # Authorization
    /// Requires authorization from the admin address.
    ///
    /// # Events
    /// Emits a `config_updated` event.
    pub fn update_config(
        env: &Env,
        new_proposal_threshold: Option<i128>,
        new_quorum: Option<i128>,
        new_voting_period: Option<u64>,
        new_execution_delay: Option<u64>,
    ) {
        let mut config = Self::get_config(env);
        config.admin.require_auth();

        if let Some(threshold) = new_proposal_threshold {
            config.proposal_threshold = threshold;
        }
        if let Some(quorum) = new_quorum {
            config.quorum = quorum;
        }
        if let Some(period) = new_voting_period {
            config.voting_period = period;
        }
        if let Some(delay) = new_execution_delay {
            config.execution_delay = delay;
        }

        env.storage().instance().set(&CONFIG, &config);
        events::config_updated(env, &config.admin);
    }

    /// Transfer admin role to a new address.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `new_admin` - Address to receive admin privileges
    ///
    /// # Authorization
    /// Requires authorization from the current admin.
    pub fn transfer_admin(env: &Env, new_admin: Address) {
        let mut config = Self::get_config(env);
        config.admin.require_auth();

        config.admin = new_admin;
        env.storage().instance().set(&CONFIG, &config);
    }

    // ========== View Functions ==========

    /// Get a proposal by its ID.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `proposal_id` - ID of the proposal to retrieve
    ///
    /// # Returns
    /// The proposal if it exists, `None` otherwise.
    pub fn get_proposal(env: &Env, proposal_id: u64) -> Option<Proposal> {
        env.storage().persistent().get(&(PROPOSALS, proposal_id))
    }

    /// Get a vote record by proposal ID and voter address.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `proposal_id` - ID of the proposal
    /// * `voter` - Address of the voter
    ///
    /// # Returns
    /// The vote record if it exists, `None` otherwise.
    pub fn get_vote(env: &Env, proposal_id: u64, voter: Address) -> Option<Vote> {
        let vote_key = VoteKey { proposal_id, voter };
        env.storage().persistent().get(&(VOTES, vote_key))
    }

    /// Check if an address has voted on a proposal.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `proposal_id` - ID of the proposal
    /// * `voter` - Address to check
    ///
    /// # Returns
    /// `true` if the address has voted, `false` otherwise.
    pub fn has_voted(env: &Env, proposal_id: u64, voter: Address) -> bool {
        let vote_key = VoteKey { proposal_id, voter };
        env.storage().persistent().has(&(VOTES, vote_key))
    }

    /// Get the total number of proposals created.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// The current proposal count.
    pub fn get_proposal_count(env: &Env) -> u64 {
        env.storage().instance().get(&PROPOSAL_COUNT).unwrap_or(0)
    }
}
