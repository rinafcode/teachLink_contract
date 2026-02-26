//! Quadratic Voting Module
//!
//! Implements quadratic voting where the cost of each additional vote
//! increases quadratically. This ensures fairer representation by
//! preventing wealthy participants from dominating decisions.
//!
//! # How Quadratic Voting Works
//!
//! - Each voter receives voting credits based on token balance
//! - The cost to cast N votes = N² credits
//! - 1 vote costs 1 credit, 2 votes cost 4 credits, 3 votes cost 9, etc.
//! - This allows voters to express intensity of preference
//!
//! # Example
//!
//! A voter with 100 credits can cast:
//! - 10 votes on one proposal (cost: 100 credits)
//! - 5 votes on proposal A (25) + 5 votes on proposal B (25) + 7 votes on C (49) = 99 credits
//! - Or distribute more broadly across many proposals

use soroban_sdk::{token, Address, Env};

use crate::events;
use crate::storage::QV_CREDITS;
use crate::types::{GovernanceConfig, QVCreditKey, QVCredits};

pub struct QuadraticVoting;

impl QuadraticVoting {
    /// Allocate quadratic voting credits to a voter for a specific proposal
    ///
    /// Credits are based on the voter's token balance. Each token = 1 credit.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment  
    /// * `config` - Governance configuration
    /// * `voter` - Address to allocate credits to
    /// * `proposal_id` - Proposal the credits are for
    ///
    /// # Returns
    /// The number of credits allocated
    pub fn allocate_credits(
        env: &Env,
        config: &GovernanceConfig,
        voter: &Address,
        proposal_id: u64,
    ) -> i128 {
        let token_client = token::Client::new(env, &config.token);
        let balance = token_client.balance(voter);

        let qv_key = QVCreditKey {
            voter: voter.clone(),
            proposal_id,
        };

        // Check if credits already allocated
        if env
            .storage()
            .persistent()
            .has(&(QV_CREDITS, qv_key.clone()))
        {
            let existing: QVCredits = env
                .storage()
                .persistent()
                .get(&(QV_CREDITS, qv_key))
                .unwrap();
            return existing.total_credits - existing.spent_credits;
        }

        let credits = QVCredits {
            total_credits: balance,
            spent_credits: 0,
            votes_purchased: 0,
        };

        env.storage()
            .persistent()
            .set(&(QV_CREDITS, qv_key), &credits);

        balance
    }

    /// Cast a quadratic vote
    ///
    /// Spends credits quadratically to cast votes. The cost for N total
    /// votes is N². If the voter already has M votes, the marginal cost
    /// to move to N votes is N² - M².
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `voter` - Address casting the vote
    /// * `proposal_id` - Proposal to vote on
    /// * `num_votes` - Number of additional votes to cast
    ///
    /// # Returns
    /// Tuple of (effective_votes, credits_spent)
    ///
    /// # Panics
    /// * If insufficient credits for the requested votes
    pub fn cast_quadratic_vote(
        env: &Env,
        voter: &Address,
        proposal_id: u64,
        num_votes: i128,
    ) -> (i128, i128) {
        let qv_key = QVCreditKey {
            voter: voter.clone(),
            proposal_id,
        };

        let mut credits: QVCredits = env
            .storage()
            .persistent()
            .get(&(QV_CREDITS, qv_key.clone()))
            .expect("ERR_NO_QV_CREDITS: No quadratic voting credits allocated");

        // Calculate quadratic cost
        let new_total_votes = credits.votes_purchased + num_votes;
        let new_total_cost = new_total_votes * new_total_votes;
        let current_cost = credits.votes_purchased * credits.votes_purchased;
        let marginal_cost = new_total_cost - current_cost;

        let remaining_credits = credits.total_credits - credits.spent_credits;
        assert!(
            remaining_credits >= marginal_cost,
            "ERR_INSUFFICIENT_QV_CREDITS: Not enough credits for quadratic vote"
        );

        credits.spent_credits += marginal_cost;
        credits.votes_purchased = new_total_votes;

        env.storage()
            .persistent()
            .set(&(QV_CREDITS, qv_key), &credits);

        events::quadratic_vote_cast(env, proposal_id, voter, num_votes, marginal_cost);

        (new_total_votes, marginal_cost)
    }

    /// Get the remaining credits for a voter on a proposal
    pub fn get_remaining_credits(env: &Env, voter: &Address, proposal_id: u64) -> i128 {
        let qv_key = QVCreditKey {
            voter: voter.clone(),
            proposal_id,
        };

        env.storage()
            .persistent()
            .get::<_, QVCredits>(&(QV_CREDITS, qv_key))
            .map(|c| c.total_credits - c.spent_credits)
            .unwrap_or(0)
    }

    /// Get the quadratic voting record for a voter on a proposal
    pub fn get_qv_credits(env: &Env, voter: &Address, proposal_id: u64) -> Option<QVCredits> {
        let qv_key = QVCreditKey {
            voter: voter.clone(),
            proposal_id,
        };

        env.storage().persistent().get(&(QV_CREDITS, qv_key))
    }

    /// Calculate the cost for a given number of votes
    ///
    /// Pure utility function: cost = num_votes²
    pub fn calculate_cost(num_votes: i128) -> i128 {
        num_votes * num_votes
    }

    /// Calculate the maximum number of votes purchasable with given credits
    ///
    /// max_votes = floor(sqrt(credits))
    pub fn max_votes_for_credits(credits: i128) -> i128 {
        if credits <= 0 {
            return 0;
        }
        // Integer square root approximation
        let mut x = credits;
        let mut y = (x + 1) / 2;
        while y < x {
            x = y;
            y = (x + credits / x) / 2;
        }
        x
    }
}
