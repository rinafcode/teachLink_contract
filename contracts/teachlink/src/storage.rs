use soroban_sdk::symbol_short;
use soroban_sdk::Symbol;

// Storage keys for the bridge contract
pub const TOKEN: Symbol = symbol_short!("token");
pub const VALIDATORS: Symbol = symbol_short!("validtor");
pub const MIN_VALIDATORS: Symbol = symbol_short!("min_valid");
pub const NONCE: Symbol = symbol_short!("nonce");
pub const BRIDGE_TXS: Symbol = symbol_short!("bridge_tx");
pub const SUPPORTED_CHAINS: Symbol = symbol_short!("chains");
pub const ADMIN: Symbol = symbol_short!("admin");
pub const FEE_RECIPIENT: Symbol = symbol_short!("fee_rcpt");
pub const BRIDGE_FEE: Symbol = symbol_short!("bridgefee");

// Storage keys for the rewards system
pub const REWARDS_ADMIN: Symbol = symbol_short!("rwd_admin");
pub const REWARD_POOL: Symbol = symbol_short!("rwd_pool");
pub const USER_REWARDS: Symbol = symbol_short!("usr_rwds");
pub const REWARD_RATES: Symbol = symbol_short!("rwd_rates");
pub const TOTAL_REWARDS_ISSUED: Symbol = symbol_short!("tot_rwds");
pub const ESCROW_COUNT: Symbol = symbol_short!("esc_ct");
pub const ESCROWS: Symbol = symbol_short!("escrows");
