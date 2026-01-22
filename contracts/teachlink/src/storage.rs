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
pub const ESCROW_COUNT: Symbol = symbol_short!("esc_ct");
pub const ESCROWS: Symbol = symbol_short!("escrows");
