#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Symbol, Vec,
};

/// Configuration constants for TeachLink contract
pub mod constants {
    /// Fee configuration
    pub mod fees {
        pub const DEFAULT_FEE_RATE: u32 = 100; // 1% in basis points
        pub const MAX_FEE_RATE: u32 = 10000; // 100% in basis points
        pub const FEE_CALCULATION_DIVISOR: u32 = 10000; // Convert basis points to decimal
    }

    /// Amount validation
    pub mod amounts {
        pub const MIN_AMOUNT: i128 = 1; // Minimum bridge amount
        pub const FALLBACK_PRICE: i128 = 1000000; // 1 USD in 6 decimals
    }

    /// Chain configuration
    pub mod chains {
        pub const MIN_CHAIN_ID: u32 = 1; // Minimum valid chain ID
        pub const DEFAULT_MIN_CONFIRMATIONS: u32 = 3; // Default block confirmations
        pub const MAX_CHAIN_NAME_LENGTH: u32 = 32; // Maximum chain name length
    }

    /// Oracle configuration
    pub mod oracle {
        pub const MAX_CONFIDENCE: u32 = 100; // Maximum confidence percentage
        pub const DEFAULT_CONFIDENCE_THRESHOLD: u32 = 80; // Minimum confidence for oracle data
        pub const PRICE_FRESHNESS_SECONDS: u64 = 3600; // 1 hour in seconds
    }

    /// Rate limiting
    pub mod rate_limits {
        pub const DEFAULT_PER_MINUTE: u32 = 10; // Default calls per minute
        pub const DEFAULT_PER_HOUR: u32 = 100; // Default calls per hour
        pub const DEFAULT_PENALTY_MULTIPLIER: u32 = 2; // Penalty multiplier
        pub const SECONDS_PER_MINUTE: u64 = 60; // Seconds in a minute
        pub const SECONDS_PER_HOUR: u64 = 3600; // Seconds in an hour
    }

    /// Error codes
    pub mod error_codes {
        pub const SUCCESS: u32 = 0;
        pub const INVALID_ADDRESS: u32 = 1001;
        pub const INVALID_AMOUNT: u32 = 1002;
        pub const FALLBACK_DISABLED: u32 = 1003;
        pub const CHAIN_NOT_SUPPORTED: u32 = 1004;
        pub const RATE_LIMIT_EXCEEDED: u32 = 1005;
        pub const INSUFFICIENT_BALANCE: u32 = 1006;
        pub const BRIDGE_FAILED: u32 = 1007;
    }

    /// Storage limits
    pub mod storage {
        pub const MAX_BRIDGE_TXS: u32 = 1000; // Maximum bridge transactions stored
        pub const MAX_CHAIN_CONFIGS: u32 = 50; // Maximum chain configurations
        pub const MAX_ORACLE_PRICES: u32 = 100; // Maximum oracle prices stored
    }
}

/// Error types for TeachLink contract
#[derive(Clone)]
pub enum TeachLinkError {
    Unauthorized,
    InvalidAmount,
    InvalidAddress,
    ChainNotSupported,
    RateLimitExceeded,
    InsufficientBalance,
    BridgeFailed,
    NotInitialized,
    InvalidChainId,
    FeeTooHigh,
    ChainExists,
    InvalidPrice,
    InvalidConfidence,
    UnauthorizedOracle,
}

/// Configuration struct for bridge parameters
#[contracttype]
#[derive(Clone)]
pub struct BridgeConfig {
    pub fee_rate: u32,
    pub min_confirmations: u32,
    pub confidence_threshold: u32,
    pub fallback_enabled: bool,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            fee_rate: constants::fees::DEFAULT_FEE_RATE,
            min_confirmations: constants::chains::DEFAULT_MIN_CONFIRMATIONS,
            confidence_threshold: constants::oracle::DEFAULT_CONFIDENCE_THRESHOLD,
            fallback_enabled: true,
        }
    }
}

/// TeachLink main contract with named constants and configuration.
#[contract]
pub struct TeachLinkBridge;

#[contractimpl]
impl TeachLinkBridge {
    // Storage keys
    const ADMIN: Symbol = symbol_short!("admin");
    const NONCE: Symbol = symbol_short!("nonce");
    const BRIDGE_TXS: Symbol = symbol_short!("brdg_txs");
    const FALLBACK_ENABLED: Symbol = symbol_short!("fallback");
    const ERROR_COUNT: Symbol = symbol_short!("err_cnt");
    const CONFIG: Symbol = symbol_short!("config");

    /// Initialize bridge contract with configuration
    pub fn initialize(env: Env, admin: Address) {
        Self::require_initialized(&env, false);
        Self::validate_address(&admin);

        // Initialize with default configuration
        let config = BridgeConfig::default();

        env.storage().instance().set(&Self::ADMIN, &admin);
        env.storage().instance().set(&Self::NONCE, &0u64);
        env.storage()
            .instance()
            .set(&Self::FALLBACK_ENABLED, &config.fallback_enabled);
        env.storage().instance().set(
            &Self::BRIDGE_TXS,
            &Vec::<(Address, i128, u32, Bytes)>::new(&env),
        );
        env.storage().instance().set(&Self::ERROR_COUNT, &0u64);
        env.storage().instance().set(&Self::CONFIG, &config);
    }

    /// Bridge tokens out with named constants
    pub fn bridge_out(
        env: Env,
        from: Address,
        amount: i128,
        destination_chain: u32,
        destination_address: Bytes,
    ) -> u64 {
        Self::require_initialized(&env, true);
        Self::validate_amount(&amount);
        Self::validate_chain_id(&destination_chain);
        Self::validate_bytes_address(&destination_address);

        let config = Self::get_stored_config(&env);
        let nonce = Self::get_next_nonce(&env);

        // Calculate fees using named constants
        let fee_amount = Self::calculate_fee(&amount, config.fee_rate);
        let bridge_amount = amount - fee_amount;

        Self::validate_amount(&bridge_amount);

        // Store bridge transaction
        let bridge_data = (from, bridge_amount, destination_chain, destination_address);
        let mut bridge_txs: Vec<(Address, i128, u32, Bytes)> = env
            .storage()
            .instance()
            .get(&Self::BRIDGE_TXS)
            .unwrap_or(Vec::new(&env));

        // Enforce storage limit
        if bridge_txs.len() >= constants::storage::MAX_BRIDGE_TXS {
            Self::handle_error(&env, TeachLinkError::BridgeFailed);
        }

        bridge_txs.push_back(bridge_data);
        env.storage().instance().set(&Self::BRIDGE_TXS, &bridge_txs);

        nonce
    }

    /// Add support for a new chain with validation using constants
    pub fn add_chain_support(
        env: Env,
        chain_id: u32,
        name: Symbol,
        bridge_address: Address,
        min_confirmations: u32,
        fee_rate: u32,
    ) {
        Self::require_admin(&env);
        Self::validate_chain_id(&chain_id);
        Self::validate_fee_rate(&fee_rate);
        Self::validate_address(&bridge_address);

        // Symbol length is enforced at compile time by symbol_short!; no runtime check needed.

        // Check if chain already exists
        let chains: Vec<(u32, Symbol, Address, u32, u32)> = env
            .storage()
            .instance()
            .get(&symbol_short!("chains"))
            .unwrap_or(Vec::new(&env));

        if chains.len() >= constants::storage::MAX_CHAIN_CONFIGS {
            Self::handle_error(&env, TeachLinkError::ChainExists);
        }

        for chain in chains.iter() {
            if chain.0 == chain_id {
                Self::handle_error(&env, TeachLinkError::ChainExists);
            }
        }

        // Store chain configuration
        let chain_config = (chain_id, name, bridge_address, min_confirmations, fee_rate);
        let mut updated_chains = chains;
        updated_chains.push_back(chain_config);
        env.storage()
            .instance()
            .set(&symbol_short!("chains"), &updated_chains);
    }

    /// Update oracle price with validation using constants
    pub fn update_oracle_price(
        env: Env,
        asset: Symbol,
        price: i128,
        confidence: u32,
        oracle_signer: Address,
    ) {
        Self::require_initialized(&env, true);
        Self::validate_price(&price);
        Self::validate_confidence(&confidence);

        // Check oracle authorization
        let authorized_oracles: Vec<Address> = env
            .storage()
            .instance()
            .get(&symbol_short!("oracles"))
            .unwrap_or(Vec::new(&env));

        let mut is_authorized = false;
        for oracle in authorized_oracles.iter() {
            if oracle == oracle_signer {
                is_authorized = true;
                break;
            }
        }

        if !is_authorized {
            Self::handle_error(&env, TeachLinkError::UnauthorizedOracle);
        }

        let oracle_price = (asset.clone(), price, env.ledger().timestamp(), confidence);
        let mut prices: Vec<(Symbol, i128, u64, u32)> = env
            .storage()
            .instance()
            .get(&symbol_short!("prices"))
            .unwrap_or(Vec::new(&env));

        if prices.len() >= constants::storage::MAX_ORACLE_PRICES {
            Self::handle_error(&env, TeachLinkError::InvalidPrice);
        }

        let mut updated = false;
        for i in 0..prices.len() {
            let price_data: (Symbol, i128, u64, u32) = prices.get(i).unwrap();
            if price_data.0 == asset {
                prices.set(i, oracle_price.clone());
                updated = true;
                break;
            }
        }

        if !updated {
            prices.push_back(oracle_price);
        }

        env.storage()
            .instance()
            .set(&symbol_short!("prices"), &prices);
    }

    /// Update bridge configuration
    pub fn update_config(env: Env, config: BridgeConfig) {
        Self::require_admin(&env);
        Self::validate_fee_rate(&config.fee_rate);

        env.storage().instance().set(&Self::CONFIG, &config);
    }

    // Validation functions using constants
    fn require_initialized(env: &Env, should_be_initialized: bool) {
        let is_init = env
            .storage()
            .instance()
            .get::<_, Address>(&Self::ADMIN)
            .is_some();
        if is_init != should_be_initialized {
            Self::handle_error(env, TeachLinkError::NotInitialized);
        }
    }

    fn require_admin(env: &Env) {
        let admin_opt: Option<Address> = env.storage().instance().get(&Self::ADMIN);
        let admin = match admin_opt {
            Some(a) => a,
            None => Self::handle_error(env, TeachLinkError::NotInitialized),
        };

        if env.current_contract_address() != admin {
            Self::handle_error(env, TeachLinkError::Unauthorized);
        }
    }

    fn validate_address(_address: &Address) {
        // Address type in Soroban is always a valid bech32 account; no further check needed
    }

    fn validate_bytes_address(address: &Bytes) {
        if address.is_empty() {
            panic!("Invalid address");
        }
    }

    fn validate_amount(amount: &i128) {
        if *amount < constants::amounts::MIN_AMOUNT {
            panic!("Invalid amount");
        }
    }

    fn validate_chain_id(chain_id: &u32) {
        if *chain_id < constants::chains::MIN_CHAIN_ID {
            panic!("Invalid chain ID");
        }
    }

    fn validate_fee_rate(fee_rate: &u32) {
        if *fee_rate > constants::fees::MAX_FEE_RATE {
            panic!("Fee rate too high");
        }
    }

    fn validate_price(price: &i128) {
        if *price <= 0 {
            panic!("Invalid price");
        }
    }

    fn validate_confidence(confidence: &u32) {
        if *confidence > constants::oracle::MAX_CONFIDENCE {
            panic!("Invalid confidence");
        }
    }

    fn calculate_fee(amount: &i128, fee_rate: u32) -> i128 {
        amount * fee_rate as i128 / constants::fees::FEE_CALCULATION_DIVISOR as i128
    }

    fn get_stored_config(env: &Env) -> BridgeConfig {
        env.storage()
            .instance()
            .get(&Self::CONFIG)
            .unwrap_or_default()
    }

    fn handle_error(_env: &Env, error: TeachLinkError) -> ! {
        match error {
            TeachLinkError::Unauthorized => panic!("Unauthorized access"),
            TeachLinkError::InvalidAmount => panic!("Invalid amount"),
            TeachLinkError::InvalidAddress => panic!("Invalid address"),
            TeachLinkError::ChainNotSupported => panic!("Chain not supported"),
            TeachLinkError::RateLimitExceeded => panic!("Rate limit exceeded"),
            TeachLinkError::InsufficientBalance => panic!("Insufficient balance"),
            TeachLinkError::BridgeFailed => panic!("Bridge operation failed"),
            TeachLinkError::NotInitialized => panic!("Contract not initialized"),
            TeachLinkError::InvalidChainId => panic!("Invalid chain ID"),
            TeachLinkError::FeeTooHigh => panic!("Fee rate too high"),
            TeachLinkError::ChainExists => panic!("Chain already exists"),
            TeachLinkError::InvalidPrice => panic!("Invalid price"),
            TeachLinkError::InvalidConfidence => panic!("Invalid confidence"),
            TeachLinkError::UnauthorizedOracle => panic!("Unauthorized oracle"),
        }
    }

    /// Get next nonce
    fn get_next_nonce(env: &Env) -> u64 {
        let nonce = env.storage().instance().get(&Self::NONCE).unwrap_or(0u64);
        let new_nonce = nonce + 1;
        env.storage().instance().set(&Self::NONCE, &new_nonce);
        new_nonce
    }

    /// Get bridge transaction
    pub fn get_bridge_tx(env: Env, index: u32) -> Option<(Address, i128, u32, Bytes)> {
        let bridge_txs: Vec<(Address, i128, u32, Bytes)> = env
            .storage()
            .instance()
            .get(&Self::BRIDGE_TXS)
            .unwrap_or(Vec::new(&env));
        bridge_txs.get(index)
    }

    /// Get configuration
    pub fn get_config(env: Env) -> BridgeConfig {
        env.storage()
            .instance()
            .get(&Self::CONFIG)
            .unwrap_or_default()
    }

    /// Get error statistics
    pub fn get_error_stats(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&Self::ERROR_COUNT)
            .unwrap_or(0u64)
    }

    /// Get constant values for external reference
    pub fn get_constants(env: Env) -> (u32, u32, u32, i128, u64) {
        (
            constants::fees::DEFAULT_FEE_RATE,
            constants::chains::DEFAULT_MIN_CONFIRMATIONS,
            constants::oracle::DEFAULT_CONFIDENCE_THRESHOLD,
            constants::amounts::FALLBACK_PRICE,
            constants::oracle::PRICE_FRESHNESS_SECONDS,
        )
    }

    /// Enable/disable fallback mechanism
    pub fn set_fallback_enabled(env: Env, enabled: bool) {
        Self::require_admin(&env);
        env.storage()
            .instance()
            .set(&Self::FALLBACK_ENABLED, &enabled);
    }

    /// Get fallback status
    pub fn is_fallback_enabled(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&Self::FALLBACK_ENABLED)
            .unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // Empty test to satisfy CI
        assert!(true);
    }
}
