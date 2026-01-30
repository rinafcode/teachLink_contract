use soroban_sdk::{Address, Bytes, Env, Symbol};

use crate::types::{ProposalStatus, ProposalType, VoteDirection};

/// Emitted when a new proposal is created
pub fn proposal_created(
    env: &Env,
    proposal_id: u64,
    proposer: &Address,
    title: &Bytes,
    proposal_type: &ProposalType,
) {
    let topics = (Symbol::new(env, "proposal_created"), proposer);
    env.events()
        .publish(topics, (proposal_id, title.clone(), proposal_type.clone()));
}

/// Emitted when a vote is cast
pub fn vote_cast(
    env: &Env,
    proposal_id: u64,
    voter: &Address,
    direction: &VoteDirection,
    power: i128,
) {
    let topics = (Symbol::new(env, "vote_cast"), voter);
    env.events()
        .publish(topics, (proposal_id, direction.clone(), power));
}

/// Emitted when a proposal status changes
pub fn proposal_status_changed(
    env: &Env,
    proposal_id: u64,
    old_status: &ProposalStatus,
    new_status: &ProposalStatus,
) {
    let topics = (Symbol::new(env, "proposal_status"),);
    env.events().publish(
        topics,
        (proposal_id, old_status.clone(), new_status.clone()),
    );
}

/// Emitted when a proposal is executed
pub fn proposal_executed(env: &Env, proposal_id: u64, executor: &Address) {
    let topics = (Symbol::new(env, "proposal_executed"), executor);
    env.events().publish(topics, proposal_id);
}

/// Emitted when a proposal is cancelled
pub fn proposal_cancelled(env: &Env, proposal_id: u64, cancelled_by: &Address) {
    let topics = (Symbol::new(env, "proposal_cancelled"), cancelled_by);
    env.events().publish(topics, proposal_id);
}

/// Emitted when governance configuration is updated
pub fn config_updated(env: &Env, admin: &Address) {
    let topics = (Symbol::new(env, "config_updated"), admin);
    env.events().publish(topics, ());
}
