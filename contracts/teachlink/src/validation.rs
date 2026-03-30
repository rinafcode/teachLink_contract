//! Input validation helpers used by contract entry points.

use soroban_sdk::{Address, Bytes, Env};

use crate::{
    constants,
    errors::{handle_error, TeachLinkError},
    storage::ADMIN,
};

pub fn require_initialized(env: &Env, should_be: bool) {
    let is_init = env.storage().instance().get::<_, Address>(&ADMIN).is_some();
    if is_init != should_be {
        handle_error(env, TeachLinkError::NotInitialized);
    }
}

pub fn require_admin(env: &Env) {
    let admin: Address = env
        .storage()
        .instance()
        .get(&ADMIN)
        .unwrap_or_else(|| handle_error(env, TeachLinkError::NotInitialized));

    if env.current_contract_address() != admin {
        handle_error(env, TeachLinkError::Unauthorized);
    }
}

pub fn validate_bytes_address(env: &Env, address: &Bytes) {
    if address.len() == 0 {
        handle_error(env, TeachLinkError::InvalidAddress);
    }
}

pub fn validate_amount(env: &Env, amount: &i128) {
    if *amount < constants::amounts::MIN_AMOUNT {
        handle_error(env, TeachLinkError::InvalidAmount);
    }
}

pub fn validate_chain_id(env: &Env, chain_id: &u32) {
    if *chain_id < constants::chains::MIN_CHAIN_ID {
        handle_error(env, TeachLinkError::InvalidChainId);
    }
}

pub fn validate_fee_rate(env: &Env, fee_rate: &u32) {
    if *fee_rate > constants::fees::MAX_FEE_RATE {
        handle_error(env, TeachLinkError::FeeTooHigh);
    }
}

pub fn validate_price(env: &Env, price: &i128) {
    if *price <= 0 {
        handle_error(env, TeachLinkError::InvalidPrice);
    }
}

pub fn validate_confidence(env: &Env, confidence: &u32) {
    if *confidence > constants::oracle::MAX_CONFIDENCE {
        handle_error(env, TeachLinkError::InvalidConfidence);
    }
}
