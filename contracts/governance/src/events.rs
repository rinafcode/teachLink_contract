#![allow(deprecated)]

use soroban_sdk::{contractevent, Address, Bytes, Env};

use crate::types::{ProposalStatus, ProposalType, VoteDirection};

// ========== Core Governance Events ==========

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

// ========== Delegation Events ==========

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegationCreatedEvent {
    pub delegator: Address,
    pub delegate: Address,
    pub expires_at: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegationRevokedEvent {
    pub delegator: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegatedVoteCastEvent {
    pub proposal_id: u64,
    pub delegate: Address,
    pub own_power: i128,
    pub delegated_power: i128,
}

// ========== Quadratic Voting Events ==========

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuadraticVoteCastEvent {
    pub proposal_id: u64,
    pub voter: Address,
    pub votes: i128,
    pub credits_spent: i128,
}

// ========== Staking Events ==========

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokensStakedEvent {
    pub staker: Address,
    pub amount: i128,
    pub power_bonus: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokensUnstakedEvent {
    pub staker: Address,
    pub amount: i128,
}

// ========== Dispute Events ==========

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeFiledEvent {
    pub dispute_id: u64,
    pub proposal_id: u64,
    pub disputant: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeResolvedEvent {
    pub dispute_id: u64,
    pub resolver: Address,
    pub upheld: bool,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppealFiledEvent {
    pub dispute_id: u64,
    pub appellant: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppealResolvedEvent {
    pub dispute_id: u64,
    pub resolver: Address,
    pub granted: bool,
}

// ========== Event Emission Functions ==========

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

// ========== Delegation Event Functions ==========

/// Emitted when a delegation is created
pub fn delegation_created(env: &Env, delegator: &Address, delegate: &Address, expires_at: u64) {
    DelegationCreatedEvent {
        delegator: delegator.clone(),
        delegate: delegate.clone(),
        expires_at,
    }
    .publish(env);
}

/// Emitted when a delegation is revoked
pub fn delegation_revoked(env: &Env, delegator: &Address) {
    DelegationRevokedEvent {
        delegator: delegator.clone(),
    }
    .publish(env);
}

/// Emitted when a delegate casts a vote including delegated power
pub fn delegated_vote_cast(
    env: &Env,
    proposal_id: u64,
    delegate: &Address,
    own_power: i128,
    delegated_power: i128,
) {
    DelegatedVoteCastEvent {
        proposal_id,
        delegate: delegate.clone(),
        own_power,
        delegated_power,
    }
    .publish(env);
}

// ========== Quadratic Voting Event Functions ==========

/// Emitted when a quadratic vote is cast
pub fn quadratic_vote_cast(
    env: &Env,
    proposal_id: u64,
    voter: &Address,
    votes: i128,
    credits_spent: i128,
) {
    QuadraticVoteCastEvent {
        proposal_id,
        voter: voter.clone(),
        votes,
        credits_spent,
    }
    .publish(env);
}

// ========== Staking Event Functions ==========

/// Emitted when tokens are staked
pub fn tokens_staked(env: &Env, staker: &Address, amount: i128, power_bonus: i128) {
    TokensStakedEvent {
        staker: staker.clone(),
        amount,
        power_bonus,
    }
    .publish(env);
}

/// Emitted when tokens are unstaked
pub fn tokens_unstaked(env: &Env, staker: &Address, amount: i128) {
    TokensUnstakedEvent {
        staker: staker.clone(),
        amount,
    }
    .publish(env);
}

// ========== Dispute Event Functions ==========

/// Emitted when a dispute is filed
pub fn dispute_filed(env: &Env, dispute_id: u64, proposal_id: u64, disputant: &Address) {
    DisputeFiledEvent {
        dispute_id,
        proposal_id,
        disputant: disputant.clone(),
    }
    .publish(env);
}

/// Emitted when a dispute is resolved
pub fn dispute_resolved(env: &Env, dispute_id: u64, resolver: &Address, upheld: bool) {
    DisputeResolvedEvent {
        dispute_id,
        resolver: resolver.clone(),
        upheld,
    }
    .publish(env);
}

/// Emitted when an appeal is filed
pub fn appeal_filed(env: &Env, dispute_id: u64, appellant: &Address) {
    AppealFiledEvent {
        dispute_id,
        appellant: appellant.clone(),
    }
    .publish(env);
}

/// Emitted when an appeal is resolved
pub fn appeal_resolved(env: &Env, dispute_id: u64, resolver: &Address, granted: bool) {
    AppealResolvedEvent {
        dispute_id,
        resolver: resolver.clone(),
        granted,
    }
    .publish(env);
}
