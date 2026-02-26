use soroban_sdk::{symbol_short, Symbol};

// ========== Core Governance Storage Keys ==========

/// Governance configuration
pub const CONFIG: Symbol = symbol_short!("config");

/// Proposal count (for generating IDs)
pub const PROPOSAL_COUNT: Symbol = symbol_short!("prop_cnt");

/// Proposals storage prefix
pub const PROPOSALS: Symbol = symbol_short!("proposal");

/// Votes storage prefix
pub const VOTES: Symbol = symbol_short!("votes");

/// Admin address
#[allow(dead_code)]
pub const ADMIN: Symbol = symbol_short!("admin");

/// Governance token address
#[allow(dead_code)]
pub const TOKEN: Symbol = symbol_short!("token");

// ========== Delegation Storage Keys ==========

/// Delegation mappings (delegator -> delegate)
pub const DELEGATIONS: Symbol = symbol_short!("deleg");

/// Delegated power accumulator (delegate -> total delegated power)
pub const DELEG_PWR: Symbol = symbol_short!("del_pwr");

/// Delegation chain depth tracker
pub const DELEG_DEPTH: Symbol = symbol_short!("del_dep");

// ========== Quadratic Voting Storage Keys ==========

/// Quadratic voting credits per voter per proposal
pub const QV_CREDITS: Symbol = symbol_short!("qv_cred");

/// Quadratic voting enabled flag per proposal
pub const QV_ENABLED: Symbol = symbol_short!("qv_on");

// ========== Staking Storage Keys ==========

/// Staking records (staker -> StakeInfo)
pub const STAKES: Symbol = symbol_short!("stakes");

/// Total staked amount
pub const TOTAL_STAKED: Symbol = symbol_short!("tot_stkd");

/// Staking configuration
pub const STAKE_CONFIG: Symbol = symbol_short!("stk_cfg");

// ========== Analytics Storage Keys ==========

/// Participation record per address
pub const PARTICIPATION: Symbol = symbol_short!("particip");

/// Global governance analytics
pub const ANALYTICS: Symbol = symbol_short!("analytics");

/// Proposal analytics per proposal
pub const PROP_STATS: Symbol = symbol_short!("pr_stats");

// ========== Dispute Resolution Storage Keys ==========

/// Disputes storage prefix
pub const DISPUTES: Symbol = symbol_short!("disputes");

/// Dispute count
pub const DISPUTE_COUNT: Symbol = symbol_short!("disp_cnt");

/// Appeals storage prefix
pub const APPEALS: Symbol = symbol_short!("appeals");

// ========== Simulation Storage Keys ==========

/// Simulation snapshots
pub const SIMULATIONS: Symbol = symbol_short!("sim");

/// Simulation count
pub const SIM_COUNT: Symbol = symbol_short!("sim_cnt");
