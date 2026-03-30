use crate::storage::{VALIDATORS_LIST, DataKey};
use crate::types::ConsensusState;
use soroban_sdk::{Address, Env, Vec};

/// Lightweight utilities for validator management to avoid duplicated logic
/// across modules (bridge, bft_consensus, slashing, etc.).
pub fn set_validator_flag(env: &Env, validator: &Address, active: bool) {
    env.storage()
        .instance()
        .set(&DataKey::Validator(validator.clone()), &active);
}

pub fn add_validator_to_list(env: &Env, validator: &Address) {
    let mut list: Vec<Address> = env
        .storage()
        .instance()
        .get(&VALIDATORS_LIST)
        .unwrap_or_else(|| Vec::new(env));
    if !list.contains(validator) {
        list.push_back(validator.clone());
        env.storage().instance().set(&VALIDATORS_LIST, &list);
    }
}

pub fn remove_validator_from_list(env: &Env, validator: &Address) {
    let mut list: Vec<Address> = env
        .storage()
        .instance()
        .get(&VALIDATORS_LIST)
        .unwrap_or_else(|| Vec::new(env));
    // iterator yields Address by value; compare against the dereferenced
    // `validator` to match types
    if let Some(pos) = list.iter().position(|v| v == *validator) {
        list.remove(pos as u32);
        env.storage().instance().set(&VALIDATORS_LIST, &list);
    }
}

/// Returns the computed consensus state from current validators and stakes.
/// This function is pure in terms of computation (reads storage) and returns
/// a ConsensusState. Callers may persist it as needed.
pub fn compute_consensus_state(env: &Env) -> ConsensusState {
    let validators: Vec<Address> = env
        .storage()
        .instance()
        .get(&VALIDATORS_LIST)
        .unwrap_or_else(|| Vec::new(env));

    let mut total_stake: i128 = 0;
    let mut active_validators: u32 = 0;

    for validator in validators.iter() {
        let is_active = env
            .storage()
            .instance()
            .get::<_, bool>(&DataKey::Validator(validator.clone()))
            .unwrap_or(false);
        if is_active {
            active_validators += 1;
            let stake = env
                .storage()
                .instance()
                .get::<_, i128>(&DataKey::ValidatorStake(validator.clone()))
                .unwrap_or(0);
            total_stake += stake;
        }
    }

    let byzantine_threshold = if active_validators > 0 {
        ((2 * active_validators) / 3) + 1
    } else {
        1
    };

    ConsensusState {
        total_stake,
        active_validators,
        byzantine_threshold,
        last_consensus_round: env.ledger().timestamp(),
    }
}

// =========================
// Unit tests for utilities
// =========================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ConsensusState;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Env, Vec};

    #[test]
    fn set_and_remove_validator_flag_and_list() {
        let env = Env::default();
        let validator = Address::generate(&env);

        // start empty
        let list: Vec<Address> = env
            .storage()
            .instance()
            .get(&VALIDATORS_LIST)
            .unwrap_or_else(|| Vec::new(&env));
        assert_eq!(list.len(), 0);

        // set active and add to list
        set_validator_flag(&env, &validator, true);
        add_validator_to_list(&env, &validator);

        let flag: bool = env
            .storage()
            .instance()
            .get(&DataKey::Validator(validator.clone()))
            .unwrap_or(false);
        assert!(flag, "validator flag should be true");

        let list: Vec<Address> = env
            .storage()
            .instance()
            .get(&VALIDATORS_LIST)
            .unwrap_or_else(|| Vec::new(&env));
        assert_eq!(list.len(), 1);

        // remove
        set_validator_flag(&env, &validator, false);
        remove_validator_from_list(&env, &validator);

        let flag2: bool = env
            .storage()
            .instance()
            .get(&DataKey::Validator(validator.clone()))
            .unwrap_or(false);
        assert!(!flag2, "validator flag should be false");

        let list2: Vec<Address> = env
            .storage()
            .instance()
            .get(&VALIDATORS_LIST)
            .unwrap_or_else(|| Vec::new(&env));
        assert_eq!(list2.len(), 0);
    }

    #[test]
    fn compute_consensus_state_counts_active_validators_and_stake() {
        let env = Env::default();
        let v1 = Address::generate(&env);
        let v2 = Address::generate(&env);

        // prepare stakes and flags
        env.storage().instance().set(&DataKey::ValidatorStake(v1.clone()), &100i128);
        env.storage().instance().set(&DataKey::ValidatorStake(v2.clone()), &50i128);

        set_validator_flag(&env, &v1, true);
        set_validator_flag(&env, &v2, false);

        add_validator_to_list(&env, &v1);
        add_validator_to_list(&env, &v2);

        let cs: ConsensusState = compute_consensus_state(&env);
        assert_eq!(cs.active_validators, 1);
        assert_eq!(cs.total_stake, 100i128);
        assert!(cs.byzantine_threshold >= 1);
    }
}
