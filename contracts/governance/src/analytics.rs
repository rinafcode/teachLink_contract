//! Governance Analytics and Participation Tracking Module
//!
//! Tracks participation metrics, voter engagement, and governance health.
//! Provides data for governance dashboards and compliance reporting.
//!
//! # Metrics Tracked
//!
//! - Per-address: votes cast, proposals created, power used, delegation activity
//! - Global: total proposals, total votes, average turnout, pass rate

use soroban_sdk::{Address, Env};

use crate::storage::{ANALYTICS, PARTICIPATION};
use crate::types::{GovernanceAnalytics, ParticipationRecord};

pub struct Analytics;

impl Analytics {
    /// Record a vote participation event
    ///
    /// Updates both individual and global analytics
    pub fn record_vote(env: &Env, voter: &Address, power_used: i128) {
        // Update individual participation
        let mut record = Self::get_participation(env, voter).unwrap_or(ParticipationRecord {
            participant: voter.clone(),
            proposals_voted: 0,
            proposals_created: 0,
            total_power_used: 0,
            delegation_count: 0,
            last_active: 0,
            participation_score: 0,
        });

        record.proposals_voted += 1;
        record.total_power_used += power_used;
        record.last_active = env.ledger().timestamp();
        record.participation_score = Self::calculate_score(&record);

        env.storage()
            .persistent()
            .set(&(PARTICIPATION, voter.clone()), &record);

        // Update global analytics
        let mut analytics = Self::get_analytics(env);
        analytics.total_votes_cast += 1;
        analytics.last_updated = env.ledger().timestamp();

        env.storage().instance().set(&ANALYTICS, &analytics);
    }

    /// Record a proposal creation event
    pub fn record_proposal_created(env: &Env, proposer: &Address) {
        // Update individual
        let mut record = Self::get_participation(env, proposer).unwrap_or(ParticipationRecord {
            participant: proposer.clone(),
            proposals_voted: 0,
            proposals_created: 0,
            total_power_used: 0,
            delegation_count: 0,
            last_active: 0,
            participation_score: 0,
        });

        record.proposals_created += 1;
        record.last_active = env.ledger().timestamp();
        record.participation_score = Self::calculate_score(&record);

        env.storage()
            .persistent()
            .set(&(PARTICIPATION, proposer.clone()), &record);

        // Update global
        let mut analytics = Self::get_analytics(env);
        analytics.total_proposals += 1;
        analytics.last_updated = env.ledger().timestamp();

        env.storage().instance().set(&ANALYTICS, &analytics);
    }

    /// Record a proposal finalization (passed or failed)
    pub fn record_proposal_finalized(env: &Env, passed: bool) {
        let mut analytics = Self::get_analytics(env);

        if passed {
            analytics.proposals_passed += 1;
        } else {
            analytics.proposals_failed += 1;
        }

        analytics.last_updated = env.ledger().timestamp();
        env.storage().instance().set(&ANALYTICS, &analytics);
    }

    /// Record a delegation event
    pub fn record_delegation(env: &Env, delegate: &Address) {
        let mut record = Self::get_participation(env, delegate).unwrap_or(ParticipationRecord {
            participant: delegate.clone(),
            proposals_voted: 0,
            proposals_created: 0,
            total_power_used: 0,
            delegation_count: 0,
            last_active: 0,
            participation_score: 0,
        });

        record.delegation_count += 1;
        record.last_active = env.ledger().timestamp();
        record.participation_score = Self::calculate_score(&record);

        env.storage()
            .persistent()
            .set(&(PARTICIPATION, delegate.clone()), &record);

        // Update global analytics
        let mut analytics = Self::get_analytics(env);
        analytics.active_delegations += 1;
        analytics.last_updated = env.ledger().timestamp();

        env.storage().instance().set(&ANALYTICS, &analytics);
    }

    /// Record staking event
    pub fn record_staking(env: &Env, amount: i128) {
        let mut analytics = Self::get_analytics(env);
        analytics.total_staked += amount;
        analytics.last_updated = env.ledger().timestamp();

        env.storage().instance().set(&ANALYTICS, &analytics);
    }

    /// Record unstaking event
    pub fn record_unstaking(env: &Env, amount: i128) {
        let mut analytics = Self::get_analytics(env);
        analytics.total_staked = if analytics.total_staked > amount {
            analytics.total_staked - amount
        } else {
            0
        };
        analytics.last_updated = env.ledger().timestamp();

        env.storage().instance().set(&ANALYTICS, &analytics);
    }

    /// Get participation record for an address
    pub fn get_participation(env: &Env, participant: &Address) -> Option<ParticipationRecord> {
        env.storage()
            .persistent()
            .get(&(PARTICIPATION, participant.clone()))
    }

    /// Get global governance analytics
    pub fn get_analytics(env: &Env) -> GovernanceAnalytics {
        env.storage()
            .instance()
            .get(&ANALYTICS)
            .unwrap_or(GovernanceAnalytics {
                total_proposals: 0,
                total_votes_cast: 0,
                unique_voters: 0,
                avg_turnout_bps: 0,
                active_delegations: 0,
                total_staked: 0,
                proposals_passed: 0,
                proposals_failed: 0,
                last_updated: 0,
            })
    }

    /// Calculate participation score (0-10000 basis points)
    ///
    /// Score is based on:
    /// - Number of proposals voted on (40% weight)
    /// - Number of proposals created (30% weight)
    /// - Delegation activity (20% weight)
    /// - Recency bonus (10% weight)
    fn calculate_score(record: &ParticipationRecord) -> u32 {
        let vote_score = u32::min(record.proposals_voted * 400, 4000);
        let create_score = u32::min(record.proposals_created * 1000, 3000);
        let delegation_score = u32::min(record.delegation_count * 500, 2000);

        // Recency is hard to compute without context, give base score
        let recency_score: u32 = if record.last_active > 0 { 1000 } else { 0 };

        u32::min(
            vote_score + create_score + delegation_score + recency_score,
            10000,
        )
    }

    /// Update global turnout average after a proposal is finalized
    pub fn update_turnout(env: &Env, _total_supply: i128, _votes_in_proposal: i128) {
        let mut analytics = Self::get_analytics(env);

        // Simple running average for now
        if analytics.total_proposals > 0 {
            let total_decided = analytics.proposals_passed + analytics.proposals_failed;
            if total_decided > 0 {
                // Approximate turnout tracking
                analytics.avg_turnout_bps = u32::min(
                    (analytics.total_votes_cast as u32 * 10000) / (total_decided as u32 * 10),
                    10000,
                );
            }
        }

        analytics.last_updated = env.ledger().timestamp();
        env.storage().instance().set(&ANALYTICS, &analytics);
    }
}
