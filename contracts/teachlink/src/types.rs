//! Shared data types used across the TeachLink contract.

use soroban_sdk::contracttype;

use crate::constants;

/// Runtime configuration for bridge parameters.
#[contracttype]
#[derive(Clone, Debug)]
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
