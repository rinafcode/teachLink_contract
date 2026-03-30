//! Compile-time configuration constants for the TeachLink contract.

/// Fee configuration
pub mod fees {
    pub const DEFAULT_FEE_RATE: u32 = 100; // 1% in basis points
    pub const MAX_FEE_RATE: u32 = 10000; // 100% in basis points
    pub const FEE_CALCULATION_DIVISOR: u32 = 10000;
}

/// Amount validation
pub mod amounts {
    pub const MIN_AMOUNT: i128 = 1;
    pub const FALLBACK_PRICE: i128 = 1_000_000; // 1 USD in 6 decimals
}

/// Chain configuration
pub mod chains {
    pub const MIN_CHAIN_ID: u32 = 1;
    pub const DEFAULT_MIN_CONFIRMATIONS: u32 = 3;
    pub const MAX_CHAIN_NAME_LENGTH: u32 = 32;
}

/// Oracle configuration
pub mod oracle {
    pub const MAX_CONFIDENCE: u32 = 100;
    pub const DEFAULT_CONFIDENCE_THRESHOLD: u32 = 80;
    pub const PRICE_FRESHNESS_SECONDS: u64 = 3600;
}

/// Storage limits
pub mod storage {
    pub const MAX_BRIDGE_TXS: u32 = 1000;
    pub const MAX_CHAIN_CONFIGS: u32 = 50;
    pub const MAX_ORACLE_PRICES: u32 = 100;
}
