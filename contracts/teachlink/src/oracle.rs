//! Oracle price feed management.

use soroban_sdk::{symbol_short, Address, Env, Symbol, Vec};

use crate::{
    constants,
    errors::{handle_error, TeachLinkError},
    validation,
};

/// Update (or insert) a price entry from an authorized oracle.
pub fn update_oracle_price(
    env: &Env,
    asset: Symbol,
    price: i128,
    confidence: u32,
    oracle_signer: Address,
) {
    validation::require_initialized(env, true);
    validation::validate_price(env, &price);
    validation::validate_confidence(env, &confidence);

    let oracles_key = symbol_short!("oracles");
    let authorized: Vec<Address> = env
        .storage()
        .instance()
        .get(&oracles_key)
        .unwrap_or_else(|| Vec::new(env));

    if !authorized.iter().any(|o| o == oracle_signer) {
        handle_error(env, TeachLinkError::UnauthorizedOracle);
    }

    let prices_key = symbol_short!("prices");
    let mut prices: Vec<(Symbol, i128, u64, u32)> = env
        .storage()
        .instance()
        .get(&prices_key)
        .unwrap_or_else(|| Vec::new(env));

    let entry = (asset.clone(), price, env.ledger().timestamp(), confidence);

    let mut updated = false;
    for i in 0..prices.len() {
        if prices.get(i).unwrap().0 == asset {
            prices.set(i, entry.clone());
            updated = true;
            break;
        }
    }

    if !updated {
        if prices.len() >= constants::storage::MAX_ORACLE_PRICES {
            handle_error(env, TeachLinkError::InvalidPrice);
        }
        prices.push_back(entry);
    }

    env.storage().instance().set(&prices_key, &prices);
}
