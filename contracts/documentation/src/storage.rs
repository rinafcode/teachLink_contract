//! Storage key definitions for the documentation contract.

use soroban_sdk::contracttype;

/// Documentation contract storage keys
#[contracttype]
pub enum DocKey {
    ArticleCount,
    FaqCount,
    Version,
}
