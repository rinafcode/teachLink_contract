#![no_std]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]

//! TeachLink Governance Contract
//!
//! A decentralized governance system allowing token holders to vote on
//! platform changes, fee structures, and new feature implementations.

use soroban_sdk::{contract, contractimpl, Address, Bytes, Env};

mod events;
mod governance;
pub mod mock_token;
mod storage;
mod types;

pub use mock_token::{MockToken, MockTokenClient};
pub use types::{
    GovernanceConfig, GovernanceError, Proposal, ProposalStatus, ProposalType, Vote, VoteDirection,
    VoteKey,
};

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    // ========== Initialization ==========

    /// Initialize the governance contract
    ///
    /// # Arguments
    /// * `token` - Address of the governance token
    /// * `admin` - Admin address for privileged operations
    /// * `proposal_threshold` - Minimum tokens required to create a proposal
    /// * `quorum` - Minimum total votes required for a proposal to pass
    /// * `voting_period` - Duration of voting period in seconds (e.g., 604800 for 7 days)
    /// * `execution_delay` - Delay before executing passed proposals in seconds
    pub fn initialize(
        env: Env,
        token: Address,
        admin: Address,
        proposal_threshold: i128,
        quorum: i128,
        voting_period: u64,
        execution_delay: u64,
    ) {
        governance::Governance::initialize(
            &env,
            token,
            admin,
            proposal_threshold,
            quorum,
            voting_period,
            execution_delay,
        );
    }

    // ========== Proposal Management ==========

    /// Create a new governance proposal
    ///
    /// Requires the proposer to hold at least `proposal_threshold` tokens.
    /// Voting starts immediately upon proposal creation.
    pub fn create_proposal(
        env: Env,
        proposer: Address,
        title: Bytes,
        description: Bytes,
        proposal_type: ProposalType,
        execution_data: Option<Bytes>,
    ) -> u64 {
        governance::Governance::create_proposal(
            &env,
            proposer,
            title,
            description,
            proposal_type,
            execution_data,
        )
    }

    /// Cast a vote on an active proposal
    ///
    /// Voting power is equal to the voter's token balance at time of voting.
    /// Each address can only vote once per proposal.
    pub fn cast_vote(env: Env, proposal_id: u64, voter: Address, direction: VoteDirection) -> i128 {
        governance::Governance::cast_vote(&env, proposal_id, voter, direction)
    }

    /// Finalize a proposal after voting ends
    ///
    /// Updates the proposal status to Passed or Failed based on votes and quorum.
    pub fn finalize_proposal(env: Env, proposal_id: u64) {
        governance::Governance::finalize_proposal(&env, proposal_id);
    }

    /// Execute a passed proposal
    ///
    /// Can be called by anyone after the execution delay has passed.
    pub fn execute_proposal(env: Env, proposal_id: u64, executor: Address) {
        governance::Governance::execute_proposal(&env, proposal_id, executor);
    }

    /// Cancel a proposal
    ///
    /// - Proposer can cancel during voting period
    /// - Admin can cancel anytime (except executed proposals)
    pub fn cancel_proposal(env: Env, proposal_id: u64, caller: Address) {
        governance::Governance::cancel_proposal(&env, proposal_id, caller);
    }

    // ========== Admin Functions ==========

    /// Update governance configuration (admin only)
    pub fn update_config(
        env: Env,
        new_proposal_threshold: Option<i128>,
        new_quorum: Option<i128>,
        new_voting_period: Option<u64>,
        new_execution_delay: Option<u64>,
    ) {
        governance::Governance::update_config(
            &env,
            new_proposal_threshold,
            new_quorum,
            new_voting_period,
            new_execution_delay,
        );
    }

    /// Transfer admin role to a new address (admin only)
    pub fn transfer_admin(env: Env, new_admin: Address) {
        governance::Governance::transfer_admin(&env, new_admin);
    }

    // ========== View Functions ==========

    /// Get the governance configuration
    pub fn get_config(env: Env) -> GovernanceConfig {
        governance::Governance::get_config(&env)
    }

    /// Get a proposal by ID
    pub fn get_proposal(env: Env, proposal_id: u64) -> Option<Proposal> {
        governance::Governance::get_proposal(&env, proposal_id)
    }

    /// Get a vote record
    pub fn get_vote(env: Env, proposal_id: u64, voter: Address) -> Option<Vote> {
        governance::Governance::get_vote(&env, proposal_id, voter)
    }

    /// Check if an address has voted on a proposal
    pub fn has_voted(env: Env, proposal_id: u64, voter: Address) -> bool {
        governance::Governance::has_voted(&env, proposal_id, voter)
    }

    /// Get the current proposal count
    pub fn get_proposal_count(env: Env) -> u64 {
        governance::Governance::get_proposal_count(&env)
    }

    /// Get the admin address
    pub fn get_admin(env: Env) -> Address {
        governance::Governance::get_admin(&env)
    }

    /// Get the governance token address
    pub fn get_token(env: Env) -> Address {
        governance::Governance::get_token(&env)
    }
}
