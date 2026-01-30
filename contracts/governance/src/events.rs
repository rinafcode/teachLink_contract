use soroban_sdk::{contractevent, Address, Bytes, Env};

use crate::types::{ProposalStatus, ProposalType, VoteDirection};

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalCreatedEvent {
    pub proposal_id: u64,
    pub proposer: Address,
    pub title: Bytes,
    pub proposal_type: ProposalType,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteCastEvent {
    pub proposal_id: u64,
    pub voter: Address,
    pub direction: VoteDirection,
    pub power: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalStatusChangedEvent {
    pub proposal_id: u64,
    pub old_status: ProposalStatus,
    pub new_status: ProposalStatus,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalExecutedEvent {
    pub proposal_id: u64,
    pub executor: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalCancelledEvent {
    pub proposal_id: u64,
    pub cancelled_by: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigUpdatedEvent {
    pub admin: Address,
}

/// Emitted when a new proposal is created
pub fn proposal_created(
    env: &Env,
    proposal_id: u64,
    proposer: &Address,
    title: &Bytes,
    proposal_type: &ProposalType,
) {
    ProposalCreatedEvent {
        proposal_id,
        proposer: proposer.clone(),
        title: title.clone(),
        proposal_type: proposal_type.clone(),
    }
    .publish(env);
}

/// Emitted when a vote is cast
pub fn vote_cast(
    env: &Env,
    proposal_id: u64,
    voter: &Address,
    direction: &VoteDirection,
    power: i128,
) {
    VoteCastEvent {
        proposal_id,
        voter: voter.clone(),
        direction: direction.clone(),
        power,
    }
    .publish(env);
}

/// Emitted when a proposal status changes
pub fn proposal_status_changed(
    env: &Env,
    proposal_id: u64,
    old_status: &ProposalStatus,
    new_status: &ProposalStatus,
) {
    ProposalStatusChangedEvent {
        proposal_id,
        old_status: old_status.clone(),
        new_status: new_status.clone(),
    }
    .publish(env);
}

/// Emitted when a proposal is executed
pub fn proposal_executed(env: &Env, proposal_id: u64, executor: &Address) {
    ProposalExecutedEvent {
        proposal_id,
        executor: executor.clone(),
    }
    .publish(env);
}

/// Emitted when a proposal is cancelled
pub fn proposal_cancelled(env: &Env, proposal_id: u64, cancelled_by: &Address) {
    ProposalCancelledEvent {
        proposal_id,
        cancelled_by: cancelled_by.clone(),
    }
    .publish(env);
}

/// Emitted when governance configuration is updated
pub fn config_updated(env: &Env, admin: &Address) {
    ConfigUpdatedEvent {
        admin: admin.clone(),
    }
    .publish(env);
}
