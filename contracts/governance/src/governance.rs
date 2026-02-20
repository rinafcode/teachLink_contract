//! Core Governance Logic for the TeachLink Platform
//!
//! This module implements decentralized governance through proposal management,
//! voting, and execution. Token holders can participate in platform decisions
//! by creating proposals and voting with their token balance as voting power.
//!
//! # Enhanced Features
//!
//! - **Delegated Voting**: Token holders can delegate votes to representatives
//! - **Quadratic Voting**: Fair decision making with quadratic vote costs
//! - **Staking Amplification**: Staked tokens receive voting power bonuses
//! - **Analytics Integration**: All actions are tracked for governance health
//!
//! # Proposal Lifecycle
//!
//! 1. **Creation**: A token holder with sufficient balance creates a proposal
//! 2. **Voting**: Token holders vote during the voting period
//! 3. **Finalization**: After voting ends, the proposal is finalized as passed or failed
//! 4. **Execution**: Passed proposals can be executed after the execution delay
//! 5. **Dispute**: Outcomes can be disputed and appealed

use soroban_sdk::{token, Address, Bytes, Env};

use crate::analytics::Analytics;
use crate::delegation::DelegationManager;
use crate::events;
use crate::staking::Staking;
use crate::storage::{CONFIG, PROPOSALS, PROPOSAL_COUNT, VOTES};
use crate::types::{
    GovernanceConfig, Proposal, ProposalStatus, ProposalType, Vote, VoteDirection, VoteKey,
};

/// Governance contract implementation.
///
/// Provides on-chain governance for the TeachLink platform through
/// token-weighted voting on proposals with delegation, quadratic voting,
/// and staking amplification support.
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
    /// * `proposal_threshold` - Minimum token balance to create proposals (must be >= 0)
    /// * `quorum` - Minimum total votes required for valid decisions (must be >= 0)
    /// * `voting_period` - Duration of voting in seconds (must be > 0)
    /// * `execution_delay` - Delay before executing passed proposals in seconds
    ///
    /// # Panics
    /// * If the contract is already initialized
    /// * If voting_period is 0
    /// * If proposal_threshold or quorum are negative
    pub fn initialize(
        env: &Env,
        token: Address,
        admin: Address,
        proposal_threshold: i128,
        quorum: i128,
        voting_period: u64,
        execution_delay: u64,
    ) {
        assert!(
            !env.storage().instance().has(&CONFIG),
            "ERR_ALREADY_INITIALIZED: Contract is already initialized"
        );

        // Validate configuration parameters
        assert!(
            proposal_threshold >= 0 && quorum >= 0,
            "ERR_INVALID_CONFIG: Governance parameters must not be negative"
        );

        assert!(
            voting_period != 0,
            "ERR_INVALID_CONFIG: Voting period must be greater than 0"
        );

        let config = GovernanceConfig {
            token,
            admin,
            proposal_threshold,
            quorum,
            voting_period,
            execution_delay,
            max_delegation_depth: 3,
            quadratic_voting_enabled: false,
            staking_multiplier: 10000, // 1x default
        };

        env.storage().instance().set(&CONFIG, &config);
        env.storage().instance().set(&PROPOSAL_COUNT, &0u64);
    }

    /// Get the current governance configuration.
    pub fn get_config(env: &Env) -> GovernanceConfig {
        env.storage()
            .instance()
            .get(&CONFIG)
            .expect("ERR_NOT_INITIALIZED: Contract not initialized")
    }

    /// Get the admin address.
    pub fn get_admin(env: &Env) -> Address {
        Self::get_config(env).admin
    }

    /// Get the governance token address.
    pub fn get_token(env: &Env) -> Address {
        Self::get_config(env).token
    }

    /// Create a new governance proposal.
    ///
    /// Creates a proposal that immediately enters the active voting state.
    /// The proposer must hold at least `proposal_threshold` tokens
    /// (including staked tokens).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `proposer` - Address creating the proposal (must authorize)
    /// * `title` - Short descriptive title for the proposal (must not be empty)
    /// * `description` - Detailed description of the proposal (must not be empty)
    /// * `proposal_type` - Category of the proposal
    /// * `execution_data` - Optional data for proposal execution
    /// * `enable_quadratic` - Whether to enable quadratic voting for this proposal
    ///
    /// # Returns
    /// The unique proposal ID.
    pub fn create_proposal(
        env: &Env,
        proposer: Address,
        title: Bytes,
        description: Bytes,
        proposal_type: ProposalType,
        execution_data: Option<Bytes>,
        enable_quadratic: bool,
    ) -> u64 {
        proposer.require_auth();

        // Validate input parameters
        assert!(
            !title.is_empty(),
            "ERR_EMPTY_TITLE: Proposal title cannot be empty"
        );

        assert!(
            !description.is_empty(),
            "ERR_EMPTY_DESCRIPTION: Proposal description cannot be empty"
        );

        let config = Self::get_config(env);

        // Check proposer has enough tokens (balance + staked)
        let token_client = token::Client::new(env, &config.token);
        let balance = token_client.balance(&proposer);
        let staking_bonus = Staking::get_staking_bonus(env, &proposer);
        let effective_balance = balance + staking_bonus;

        assert!(
            effective_balance >= config.proposal_threshold,
            "ERR_INSUFFICIENT_BALANCE: Proposer balance below threshold"
        );

        // Check if quadratic voting is allowed
        let qv_enabled = enable_quadratic && config.quadratic_voting_enabled;

        // Generate proposal ID
        let mut proposal_count: u64 = env.storage().instance().get(&PROPOSAL_COUNT).unwrap_or(0);
        proposal_count += 1;

        let now = env.ledger().timestamp();
        let voting_start = now;
        let voting_end = voting_start + config.voting_period;

        let proposal = Proposal {
            id: proposal_count,
            proposer: proposer.clone(),
            title: title.clone(),
            description,
            proposal_type: proposal_type.clone(),
            status: ProposalStatus::Active,
            created_at: now,
            voting_start,
            voting_end,
            for_votes: 0,
            against_votes: 0,
            abstain_votes: 0,
            execution_data,
            quadratic_voting: qv_enabled,
            voter_count: 0,
        };

        // Store proposal
        env.storage()
            .persistent()
            .set(&(PROPOSALS, proposal_count), &proposal);
        env.storage()
            .instance()
            .set(&PROPOSAL_COUNT, &proposal_count);

        // Track analytics
        Analytics::record_proposal_created(env, &proposer);

        // Emit event
        events::proposal_created(env, proposal_count, &proposer, &title, &proposal_type);

        proposal_count
    }

    /// Cast a vote on an active proposal with delegation support.
    ///
    /// Records a vote with the voter's token balance as voting power,
    /// plus any delegated power they have received. If the voter has
    /// staked tokens, they receive an amplified voting power bonus.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `proposal_id` - ID of the proposal to vote on
    /// * `voter` - Address casting the vote (must authorize)
    /// * `direction` - Vote direction (For, Against, or Abstain)
    ///
    /// # Returns
    /// The total voting power used (own + delegated + staking bonus).
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
            .expect("ERR_PROPOSAL_NOT_FOUND: Proposal does not exist");

        // Check proposal is active
        assert!(
            proposal.status == ProposalStatus::Active,
            "ERR_INVALID_STATUS: Proposal is not in active status"
        );

        // Check voting period
        let now = env.ledger().timestamp();
        assert!(
            now >= proposal.voting_start && now <= proposal.voting_end,
            "ERR_VOTING_PERIOD_INACTIVE: Voting period is not active"
        );

        // Check if already voted
        let vote_key = VoteKey {
            proposal_id,
            voter: voter.clone(),
        };
        assert!(
            !env.storage().persistent().has(&(VOTES, vote_key.clone())),
            "ERR_ALREADY_VOTED: Address has already voted on this proposal"
        );

        // Calculate total voting power: own tokens + delegated power + staking bonus
        let token_client = token::Client::new(env, &config.token);
        let own_power = token_client.balance(&voter);

        // Get delegated power
        let delegated_power = DelegationManager::get_delegated_power(env, &voter);

        // Get staking bonus
        let staking_bonus = Staking::get_staking_bonus(env, &voter);

        let total_power = own_power + delegated_power + staking_bonus;

        assert!(
            total_power > 0,
            "ERR_NO_VOTING_POWER: Address has no voting power"
        );

        // Record vote
        let vote = Vote {
            voter: voter.clone(),
            proposal_id,
            power: total_power,
            direction: direction.clone(),
            timestamp: now,
            includes_delegated: delegated_power > 0,
            delegated_power,
        };
        env.storage().persistent().set(&(VOTES, vote_key), &vote);

        // Update proposal vote counts
        match direction {
            VoteDirection::For => proposal.for_votes += total_power,
            VoteDirection::Against => proposal.against_votes += total_power,
            VoteDirection::Abstain => proposal.abstain_votes += total_power,
        }
        proposal.voter_count += 1;

        env.storage()
            .persistent()
            .set(&(PROPOSALS, proposal_id), &proposal);

        // Track analytics
        Analytics::record_vote(env, &voter, total_power);

        // Emit events
        events::vote_cast(env, proposal_id, &voter, &direction, total_power);

        if delegated_power > 0 {
            events::delegated_vote_cast(env, proposal_id, &voter, own_power, delegated_power);
        }

        total_power
    }

    /// Finalize a proposal after the voting period ends.
    ///
    /// Determines whether the proposal passed or failed based on
    /// quorum requirements and vote counts.
    pub fn finalize_proposal(env: &Env, proposal_id: u64) {
        let config = Self::get_config(env);

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&(PROPOSALS, proposal_id))
            .expect("ERR_PROPOSAL_NOT_FOUND: Proposal does not exist");

        // Check proposal is still active
        assert!(
            proposal.status == ProposalStatus::Active,
            "ERR_INVALID_STATUS: Proposal is not in active status"
        );

        // Check voting period has ended
        let now = env.ledger().timestamp();
        assert!(
            now > proposal.voting_end,
            "ERR_VOTING_PERIOD_ACTIVE: Voting period has not ended yet"
        );

        let old_status = proposal.status.clone();

        // Calculate total votes
        let total_votes = proposal.for_votes + proposal.against_votes + proposal.abstain_votes;

        // Check quorum and majority
        let passed = total_votes >= config.quorum && proposal.for_votes > proposal.against_votes;

        if passed {
            proposal.status = ProposalStatus::Passed;
        } else {
            proposal.status = ProposalStatus::Failed;
        }

        env.storage()
            .persistent()
            .set(&(PROPOSALS, proposal_id), &proposal);

        // Track analytics
        Analytics::record_proposal_finalized(env, passed);

        events::proposal_status_changed(env, proposal_id, &old_status, &proposal.status);
    }

    /// Execute a passed proposal.
    pub fn execute_proposal(env: &Env, proposal_id: u64, executor: Address) {
        executor.require_auth();

        let config = Self::get_config(env);

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&(PROPOSALS, proposal_id))
            .expect("ERR_PROPOSAL_NOT_FOUND: Proposal does not exist");

        // Check proposal has passed
        assert!(
            proposal.status == ProposalStatus::Passed,
            "ERR_INVALID_STATUS: Proposal has not passed"
        );

        // Check execution delay has passed
        let now = env.ledger().timestamp();
        assert!(
            now >= proposal.voting_end + config.execution_delay,
            "ERR_EXECUTION_DELAY_NOT_MET: Execution delay period has not passed"
        );

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
    pub fn cancel_proposal(env: &Env, proposal_id: u64, caller: Address) {
        caller.require_auth();

        let config = Self::get_config(env);

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&(PROPOSALS, proposal_id))
            .expect("ERR_PROPOSAL_NOT_FOUND: Proposal does not exist");

        // Check if cancellable
        let is_admin = caller == config.admin;
        let is_proposer = caller == proposal.proposer;
        let now = env.ledger().timestamp();
        let voting_ended = now > proposal.voting_end;

        assert!(
            is_admin || is_proposer,
            "ERR_UNAUTHORIZED: Only proposer or admin can cancel"
        );

        assert!(
            is_admin || !voting_ended,
            "ERR_VOTING_ENDED: Proposer can only cancel during voting period"
        );

        // Cannot cancel executed proposals
        assert!(
            proposal.status != ProposalStatus::Executed,
            "ERR_INVALID_STATUS: Cannot cancel executed proposal"
        );

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
            assert!(
                threshold >= 0,
                "ERR_INVALID_CONFIG: Proposal threshold must not be negative"
            );
            config.proposal_threshold = threshold;
        }

        if let Some(quorum) = new_quorum {
            assert!(
                quorum >= 0,
                "ERR_INVALID_CONFIG: Quorum must not be negative"
            );
            config.quorum = quorum;
        }

        if let Some(period) = new_voting_period {
            assert!(
                period != 0,
                "ERR_INVALID_CONFIG: Voting period must be greater than 0"
            );
            config.voting_period = period;
        }

        if let Some(delay) = new_execution_delay {
            config.execution_delay = delay;
        }

        env.storage().instance().set(&CONFIG, &config);
        events::config_updated(env, &config.admin);
    }

    /// Update advanced governance settings (admin only)
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `max_delegation_depth` - New max delegation chain depth
    /// * `quadratic_voting_enabled` - Enable/disable quadratic voting globally
    /// * `staking_multiplier` - New staking power multiplier (basis points)
    pub fn update_advanced_config(
        env: &Env,
        max_delegation_depth: Option<u32>,
        quadratic_voting_enabled: Option<bool>,
        staking_multiplier: Option<u32>,
    ) {
        let mut config = Self::get_config(env);
        config.admin.require_auth();

        if let Some(depth) = max_delegation_depth {
            assert!(
                depth > 0 && depth <= 10,
                "ERR_INVALID_CONFIG: Delegation depth must be between 1 and 10"
            );
            config.max_delegation_depth = depth;
        }

        if let Some(qv_enabled) = quadratic_voting_enabled {
            config.quadratic_voting_enabled = qv_enabled;
        }

        if let Some(multiplier) = staking_multiplier {
            assert!(
                multiplier >= 10000,
                "ERR_INVALID_CONFIG: Staking multiplier must be at least 10000 (1x)"
            );
            config.staking_multiplier = multiplier;
        }

        env.storage().instance().set(&CONFIG, &config);
        events::config_updated(env, &config.admin);
    }

    /// Transfer admin role to a new address.
    pub fn transfer_admin(env: &Env, new_admin: Address) {
        let mut config = Self::get_config(env);
        config.admin.require_auth();

        config.admin = new_admin;
        env.storage().instance().set(&CONFIG, &config);
    }

    // ========== View Functions ==========

    /// Get a proposal by its ID.
    pub fn get_proposal(env: &Env, proposal_id: u64) -> Option<Proposal> {
        env.storage().persistent().get(&(PROPOSALS, proposal_id))
    }

    /// Get a vote record by proposal ID and voter address.
    pub fn get_vote(env: &Env, proposal_id: u64, voter: Address) -> Option<Vote> {
        let vote_key = VoteKey { proposal_id, voter };
        env.storage().persistent().get(&(VOTES, vote_key))
    }

    /// Check if an address has voted on a proposal.
    pub fn has_voted(env: &Env, proposal_id: u64, voter: Address) -> bool {
        let vote_key = VoteKey { proposal_id, voter };
        env.storage().persistent().has(&(VOTES, vote_key))
    }

    /// Get the total number of proposals created.
    pub fn get_proposal_count(env: &Env) -> u64 {
        env.storage().instance().get(&PROPOSAL_COUNT).unwrap_or(0)
    }
}
