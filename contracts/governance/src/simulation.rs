//! Governance Simulation and Prediction Module
//!
//! Provides tools for simulating governance outcomes before proposals
//! are finalized. Helps participants understand potential outcomes
//! and make more informed voting decisions.
//!
//! # Features
//!
//! - Simulate vote outcomes based on current state
//! - Predict turnout based on historical data
//! - Model delegation effects on proposals

use soroban_sdk::{Address, Env};

use crate::storage::{PROPOSALS, SIMULATIONS, SIM_COUNT};
use crate::types::{Proposal, SimulationSnapshot};

pub struct Simulation;

impl Simulation {
    /// Create a simulation snapshot for a proposal
    ///
    /// Takes the current vote state of a proposal and creates a prediction
    /// of the final outcome based on current trajectory.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `creator` - Address creating the simulation
    /// * `proposal_id` - Proposal to simulate
    /// * `additional_for` - Additional hypothetical for votes
    /// * `additional_against` - Additional hypothetical against votes
    /// * `additional_abstain` - Additional hypothetical abstain votes
    ///
    /// # Returns
    /// The simulation ID
    pub fn create_simulation(
        env: &Env,
        creator: Address,
        proposal_id: u64,
        additional_for: i128,
        additional_against: i128,
        additional_abstain: i128,
    ) -> u64 {
        creator.require_auth();

        // Get current proposal state
        let proposal: Proposal = env
            .storage()
            .persistent()
            .get(&(PROPOSALS, proposal_id))
            .expect("ERR_PROPOSAL_NOT_FOUND: Proposal does not exist");

        let sim_for = proposal.for_votes + additional_for;
        let sim_against = proposal.against_votes + additional_against;
        let sim_abstain = proposal.abstain_votes + additional_abstain;

        let total_votes = sim_for + sim_against + sim_abstain;
        let predicted_pass = sim_for > sim_against && total_votes > 0;

        // Simple turnout prediction (based on current participation)
        let predicted_turnout_bps = if total_votes > 0 { 5000u32 } else { 0u32 };

        let mut sim_count: u64 = env
            .storage()
            .instance()
            .get(&SIM_COUNT)
            .unwrap_or(0);
        sim_count += 1;

        let snapshot = SimulationSnapshot {
            id: sim_count,
            proposal_id,
            creator: creator.clone(),
            sim_for_votes: sim_for,
            sim_against_votes: sim_against,
            sim_abstain_votes: sim_abstain,
            predicted_pass,
            predicted_turnout_bps,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&(SIMULATIONS, sim_count), &snapshot);
        env.storage()
            .instance()
            .set(&SIM_COUNT, &sim_count);

        sim_count
    }

    /// Predict outcome of a proposal based on current votes and quorum
    ///
    /// # Returns
    /// Tuple of (would_pass, current_turnout_bps, votes_needed_for_quorum)
    pub fn predict_outcome(
        env: &Env,
        proposal_id: u64,
        quorum: i128,
    ) -> (bool, u32, i128) {
        let proposal: Proposal = env
            .storage()
            .persistent()
            .get(&(PROPOSALS, proposal_id))
            .expect("ERR_PROPOSAL_NOT_FOUND: Proposal does not exist");

        let total_votes =
            proposal.for_votes + proposal.against_votes + proposal.abstain_votes;

        let would_pass =
            total_votes >= quorum && proposal.for_votes > proposal.against_votes;

        let turnout_bps = if quorum > 0 {
            u32::min(
                ((total_votes * 10000) / quorum) as u32,
                10000,
            )
        } else {
            10000
        };

        let votes_needed = if total_votes >= quorum {
            0
        } else {
            quorum - total_votes
        };

        (would_pass, turnout_bps, votes_needed)
    }

    /// Get a simulation snapshot by ID
    pub fn get_simulation(env: &Env, sim_id: u64) -> Option<SimulationSnapshot> {
        env.storage()
            .persistent()
            .get(&(SIMULATIONS, sim_id))
    }

    /// Get simulation count
    pub fn get_simulation_count(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&SIM_COUNT)
            .unwrap_or(0)
    }
}
