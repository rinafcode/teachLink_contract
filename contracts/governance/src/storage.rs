use soroban_sdk::{symbol_short, Symbol};

// Storage keys for the governance contract

/// Governance configuration
pub const CONFIG: Symbol = symbol_short!("config");

/// Proposal count (for generating IDs)
pub const PROPOSAL_COUNT: Symbol = symbol_short!("prop_cnt");

/// Proposals storage prefix
pub const PROPOSALS: Symbol = symbol_short!("proposal");

/// Votes storage prefix
pub const VOTES: Symbol = symbol_short!("votes");

/// Admin address
pub const ADMIN: Symbol = symbol_short!("admin");

/// Governance token address
pub const TOKEN: Symbol = symbol_short!("token");
