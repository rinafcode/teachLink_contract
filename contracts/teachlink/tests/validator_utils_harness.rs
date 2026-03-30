use teachlink_contract::validator_utils;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Env;

#[test]
fn validator_utils_harness() {
    let env = Env::default();

    // Register the contract so storage calls inside validator_utils are
    // executed within a contract context.
    let contract_id = env.register(teachlink_contract::TeachLinkBridge, ());
    env.as_contract(&contract_id, || {
        let validator = soroban_sdk::Address::generate(&env);

        // Ensure starting state: no active validators
        let cs_before = validator_utils::compute_consensus_state(&env);
        assert_eq!(cs_before.active_validators, 0);

        // Register flag and add to list
        validator_utils::set_validator_flag(&env, &validator, true);
        validator_utils::add_validator_to_list(&env, &validator);

        let cs_after = validator_utils::compute_consensus_state(&env);
        assert_eq!(cs_after.active_validators, 1);

        // Unset flag and remove from list
        validator_utils::set_validator_flag(&env, &validator, false);
        validator_utils::remove_validator_from_list(&env, &validator);

        let cs_final = validator_utils::compute_consensus_state(&env);
        assert_eq!(cs_final.active_validators, 0);
    });
}
