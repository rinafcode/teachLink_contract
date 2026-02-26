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
mod insurance;
pub mod mock_token;
mod quadratic;
mod simulation;
mod staking;
mod storage;
mod types;

pub use mock_token::*;
pub use types::*;

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    // 1-2. Initialization & Core
    pub fn initialize(
        env: Env,
        token: Address,
        admin: Address,
        threshold: i128,
        quorum: i128,
        period: u64,
        delay: u64,
    ) {
        governance::Governance::initialize(&env, token, admin, threshold, quorum, period, delay);
    }

    pub fn get_config(env: Env) -> GovernanceConfig {
        governance::Governance::get_config(&env)
    }

    pub fn get_admin(env: Env) -> Address {
        governance::Governance::get_admin(&env)
    }

    pub fn get_token(env: Env) -> Address {
        governance::Governance::get_token(&env)
    }

    pub fn create_proposal(
        env: Env,
        proposer: Address,
        title: Bytes,
        desc: Bytes,
        p_type: ProposalType,
        data: Option<Bytes>,
    ) -> u64 {
        governance::Governance::create_proposal(&env, proposer, title, desc, p_type, data, false)
    }

    pub fn get_proposal(env: Env, proposal_id: u64) -> Option<Proposal> {
        governance::Governance::get_proposal(&env, proposal_id)
    }

    pub fn cast_vote(env: Env, proposal_id: u64, voter: Address, direction: VoteDirection) -> i128 {
        governance::Governance::cast_vote(&env, proposal_id, voter, direction)
    }

    pub fn get_vote(env: Env, proposal_id: u64, voter: Address) -> Option<Vote> {
        governance::Governance::get_vote(&env, proposal_id, voter)
    }

    pub fn has_voted(env: Env, proposal_id: u64, voter: Address) -> bool {
        governance::Governance::has_voted(&env, proposal_id, voter)
    }

    pub fn finalize_proposal(env: Env, proposal_id: u64) {
        governance::Governance::finalize_proposal(&env, proposal_id);
    }

    pub fn execute_proposal(env: Env, proposal_id: u64, executor: Address) {
        governance::Governance::execute_proposal(&env, proposal_id, executor);
    }

    pub fn cancel_proposal(env: Env, proposal_id: u64, caller: Address) {
        governance::Governance::cancel_proposal(&env, proposal_id, caller);
    }

    pub fn update_config(
        env: Env,
        new_threshold: Option<i128>,
        new_quorum: Option<i128>,
        new_period: Option<u64>,
        new_delay: Option<u64>,
    ) {
        governance::Governance::update_config(
            &env,
            new_threshold,
            new_quorum,
            new_period,
            new_delay,
        );
    }

    pub fn update_advanced_config(
        env: Env,
        depth: Option<u32>,
        qv_enabled: Option<bool>,
        multiplier: Option<u32>,
    ) {
        governance::Governance::update_advanced_config(&env, depth, qv_enabled, multiplier);
    }

    pub fn transfer_admin(env: Env, new_admin: Address) {
        governance::Governance::transfer_admin(&env, new_admin);
    }

    pub fn get_proposal_count(env: Env) -> u64 {
        governance::Governance::get_proposal_count(&env)
    }

    // 3. Liquid Democracy & Delegation
    pub fn delegate_vote(env: Env, delegator: Address, delegate: Address, expires_at: u64) {
        let config = governance::Governance::get_config(&env);
        delegation::DelegationManager::delegate(&env, &config, delegator, delegate, expires_at);
    }

    pub fn revoke_delegation(env: Env, delegator: Address) {
        let config = governance::Governance::get_config(&env);
        delegation::DelegationManager::revoke_delegation(&env, &config, delegator);
    }

    pub fn get_effective_delegate(env: Env, delegator: Address) -> Address {
        let config = governance::Governance::get_config(&env);
        delegation::DelegationManager::get_effective_delegate(
            &env,
            &delegator,
            config.max_delegation_depth,
        )
    }

    pub fn get_total_voting_power(env: Env, voter: Address) -> i128 {
        let config = governance::Governance::get_config(&env);
        let (_, _, mut total) =
            delegation::DelegationManager::get_total_voting_power(&env, &config, &voter);

        let staking_bonus = staking::Staking::get_staking_bonus(&env, &voter);
        total += staking_bonus;

        total
    }

    pub fn get_delegation(env: Env, delegator: Address) -> Option<Delegation> {
        delegation::DelegationManager::get_delegation(&env, &delegator)
    }

    pub fn has_delegated(env: Env, delegator: Address) -> bool {
        delegation::DelegationManager::has_delegated(&env, &delegator)
    }

    pub fn get_delegated_power(env: Env, delegate: Address) -> i128 {
        delegation::DelegationManager::get_delegated_power(&env, &delegate)
    }

    // 4. Quadratic Voting
    pub fn create_proposal_with_qv(
        env: Env,
        proposer: Address,
        title: Bytes,
        desc: Bytes,
        p_type: ProposalType,
        data: Option<Bytes>,
    ) -> u64 {
        governance::Governance::create_proposal(&env, proposer, title, desc, p_type, data, true)
    }

    pub fn allocate_qv_credits(env: Env, voter: Address, proposal_id: u64) -> i128 {
        let config = governance::Governance::get_config(&env);
        quadratic::QuadraticVoting::allocate_credits(&env, &config, &voter, proposal_id)
    }

    pub fn cast_quadratic_vote(
        env: Env,
        voter: Address,
        proposal_id: u64,
        num_votes: i128,
    ) -> i128 {
        voter.require_auth();
        let (total, _) =
            quadratic::QuadraticVoting::cast_quadratic_vote(&env, &voter, proposal_id, num_votes);
        total
    }

    pub fn get_qv_remaining(env: Env, voter: Address, proposal_id: u64) -> i128 {
        quadratic::QuadraticVoting::get_remaining_credits(&env, &voter, proposal_id)
    }

    pub fn calculate_qv_cost(_env: Env, num_votes: i128) -> i128 {
        quadratic::QuadraticVoting::calculate_cost(num_votes)
    }

    pub fn get_qv_credits(env: Env, voter: Address, proposal_id: u64) -> Option<QVCredits> {
        quadratic::QuadraticVoting::get_qv_credits(&env, &voter, proposal_id)
    }

    // 5. Token Staking
    pub fn initialize_staking(
        env: Env,
        admin: Address,
        min_stake: i128,
        lock_period: u64,
        multiplier: u32,
    ) {
        staking::Staking::initialize_staking(&env, admin, min_stake, lock_period, multiplier);
    }

    pub fn stake_tokens(env: Env, staker: Address, amount: i128) {
        let config = governance::Governance::get_config(&env);
        staking::Staking::stake(&env, &config.token, staker, amount);
    }

    pub fn unstake_tokens(env: Env, staker: Address, amount: i128) {
        let config = governance::Governance::get_config(&env);
        staking::Staking::unstake(&env, &config.token, staker, amount);
    }

    pub fn get_staking_config(env: Env) -> Option<StakingConfig> {
        staking::Staking::get_staking_config(&env)
    }

    pub fn get_stake(env: Env, staker: Address) -> Option<StakeInfo> {
        staking::Staking::get_stake(&env, &staker)
    }

    pub fn get_total_staked(env: Env) -> i128 {
        staking::Staking::get_total_staked(&env)
    }

    pub fn is_stake_unlocked(env: Env, staker: Address) -> bool {
        staking::Staking::is_unlocked(&env, &staker)
    }

    // 6. Analytics & Participation
    pub fn get_analytics(env: Env) -> GovernanceAnalytics {
        analytics::Analytics::get_analytics(&env)
    }

    pub fn get_participation(env: Env, participant: Address) -> Option<ParticipationRecord> {
        analytics::Analytics::get_participation(&env, &participant)
    }

    // 7. Cross-Chain Coordination
    pub fn register_chain(env: Env, admin: Address, id: Bytes, name: Bytes, weight: u32) {
        cross_chain::CrossChainGovernance::register_chain(&env, admin, id, name, weight);
    }

    // 8. Dispute Resolution & Appeals
    pub fn file_dispute(env: Env, caller: Address, proposal_id: u64, reason: Bytes) -> u64 {
        disputes::DisputeResolution::file_dispute(&env, caller, proposal_id, reason)
    }

    pub fn resolve_dispute(
        env: Env,
        dispute_id: u64,
        resolver: Address,
        upheld: bool,
        resolution: Bytes,
    ) {
        disputes::DisputeResolution::resolve_dispute(
            &env, dispute_id, resolver, upheld, resolution,
        );
    }

    pub fn get_dispute(env: Env, dispute_id: u64) -> Option<Dispute> {
        disputes::DisputeResolution::get_dispute(&env, dispute_id)
    }

    pub fn file_appeal(env: Env, dispute_id: u64, appellant: Address, reason: Bytes) {
        disputes::DisputeResolution::file_appeal(&env, dispute_id, appellant, reason);
    }

    pub fn get_appeal(env: Env, dispute_id: u64) -> Option<Appeal> {
        disputes::DisputeResolution::get_appeal(&env, dispute_id)
    }

    pub fn resolve_appeal(env: Env, dispute_id: u64, admin: Address, granted: bool) {
        disputes::DisputeResolution::resolve_appeal(&env, dispute_id, admin, granted);
    }

    pub fn get_dispute_count(env: Env) -> u64 {
        disputes::DisputeResolution::get_dispute_count(&env)
    }

    // 9. Insurance & Risk Mitigation
    pub fn assess_risk(
        env: Env,
        admin: Address,
        proposal_id: u64,
        level: insurance::RiskLevel,
        score: u32,
    ) {
        insurance::GovernanceInsurance::assess_risk(&env, admin, proposal_id, level, score);
    }

    // 10. Simulation & Prediction
    pub fn create_simulation(
        env: Env,
        creator: Address,
        proposal_id: u64,
        sim_for: i128,
        sim_against: i128,
        sim_abstain: i128,
    ) -> u64 {
        simulation::Simulation::create_simulation(
            &env,
            creator,
            proposal_id,
            sim_for,
            sim_against,
            sim_abstain,
        )
    }

    pub fn get_simulation(env: Env, simulation_id: u64) -> Option<SimulationSnapshot> {
        simulation::Simulation::get_simulation(&env, simulation_id)
    }

    pub fn predict_outcome(env: Env, proposal_id: u64) -> (bool, u32, i128) {
        let config = governance::Governance::get_config(&env);
        simulation::Simulation::predict_outcome(&env, proposal_id, config.quorum)
    }

    // 11. Automation & Prioritization
    pub fn get_priority_queue(env: Env) -> Vec<automation::PriorityRecord> {
        automation::ProposalAutomation::get_prioritized_queue(&env)
    }

    // 12. Compliance & Reporting
    pub fn generate_compliance_report(
        env: Env,
        admin: Address,
        p: u64,
        v: u32,
        r: u32,
        h: Bytes,
    ) -> compliance::ComplianceReport {
        compliance::Compliance::generate_report(&env, admin, p, v, r, h)
    }
}
