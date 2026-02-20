#![no_std]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![allow(deprecated)]

//! TeachLink Advanced Governance Contract
//!
//! Fully implementing Issue #96 Acceptance Criteria.

use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Vec};

mod analytics;
mod automation;
mod compliance;
mod cross_chain;
mod delegation;
mod disputes;
mod events;
mod governance;
pub mod mock_token;
mod quadratic;
mod simulation;
mod staking;
mod storage;
mod types;
mod insurance;

pub use types::*;

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    // 1-2. Initialization & Core
    pub fn initialize(env: Env, token: Address, admin: Address, threshold: i128, quorum: i128, period: u64, delay: u64) {
        governance::Governance::initialize(&env, token, admin, threshold, quorum, period, delay);
    }

    pub fn create_proposal(env: Env, proposer: Address, title: Bytes, desc: Bytes, p_type: ProposalType, data: Option<Bytes>) -> u64 {
        governance::Governance::create_proposal(&env, proposer, title, desc, p_type, data, false)
    }

    // 3. Liquid Democracy & Delegation
    pub fn delegate_vote(env: Env, delegator: Address, delegate: Address, expires_at: u64) {
        let config = governance::Governance::get_config(&env);
        delegation::DelegationManager::delegate(&env, &config, delegator, delegate, expires_at);
    }

    // 4. Quadratic Voting
    pub fn cast_quadratic_vote(env: Env, voter: Address, proposal_id: u64, num_votes: i128) -> i128 {
        voter.require_auth();
        let (total, _) = quadratic::QuadraticVoting::cast_quadratic_vote(&env, &voter, proposal_id, num_votes);
        total
    }

    // 5. Token Staking
    pub fn stake_tokens(env: Env, staker: Address, amount: i128) {
        let config = governance::Governance::get_config(&env);
        staking::Staking::stake(&env, &config.token, staker, amount);
    }

    // 6. Analytics & Participation
    pub fn get_analytics(env: Env) -> GovernanceAnalytics {
        analytics::Analytics::get_analytics(&env)
    }

    // 7. Cross-Chain Coordination
    pub fn register_chain(env: Env, admin: Address, id: Bytes, name: Bytes, weight: u32) {
        cross_chain::CrossChainGovernance::register_chain(&env, admin, id, name, weight);
    }

    // 8. Dispute Resolution & Appeals
    pub fn file_dispute(env: Env, caller: Address, proposal_id: u64, reason: Bytes) -> u64 {
        disputes::DisputeResolution::file_dispute(&env, caller, proposal_id, reason)
    }

    // 9. Insurance & Risk Mitigation
    pub fn assess_risk(env: Env, admin: Address, proposal_id: u64, level: insurance::RiskLevel, score: u32) {
        insurance::GovernanceInsurance::assess_risk(&env, admin, proposal_id, level, score);
    }

    // 10. Simulation & Prediction
    pub fn predict_outcome(env: Env, proposal_id: u64) -> (bool, u32, i128) {
        let config = governance::Governance::get_config(&env);
        simulation::Simulation::predict_outcome(&env, proposal_id, config.quorum)
    }

    // 11. Automation & Prioritization
    pub fn get_priority_queue(env: Env) -> Vec<automation::PriorityRecord> {
        automation::ProposalAutomation::get_prioritized_queue(&env)
    }

    // 12. Compliance & Reporting
    pub fn generate_compliance_report(env: Env, admin: Address, p: u64, v: u32, r: u32, h: Bytes) -> compliance::ComplianceReport {
        compliance::Compliance::generate_report(&env, admin, p, v, r, h)
    }
}
