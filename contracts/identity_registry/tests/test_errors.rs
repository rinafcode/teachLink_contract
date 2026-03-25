#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env};

use identity_registry_contract::{
    IdentityRegistryContract, IdentityRegistryContractClient, IdentityRegistryError,
};

fn expect_err<T, E: core::fmt::Debug>(res: Result<T, E>, context: &str) -> E {
    match res {
        Ok(_) => panic!("{}: expected error", context),
        Err(err) => err,
    }
}

fn expect_ok<T, E: core::fmt::Debug>(res: Result<T, E>, context: &str) -> T {
    match res {
        Ok(value) => value,
        Err(err) => panic!("{}: {:?}", context, err),
    }
}

#[test]
fn test_set_controller_missing() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(IdentityRegistryContract, ());
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let identity_id = BytesN::from_array(&env, &[1u8; 32]);
    let current = Address::generate(&env);
    let new_controller = Address::generate(&env);

    let err = expect_err(
        client.set_controller(&identity_id, &current, &new_controller),
        "set_controller missing",
    );
    assert_eq!(err, IdentityRegistryError::DidNotFound);
}

#[test]
fn test_set_controller_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(IdentityRegistryContract, ());
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let identity_id = BytesN::from_array(&env, &[2u8; 32]);
    let controller = Address::generate(&env);
    let attacker = Address::generate(&env);

    expect_ok(
        client.create_did(&identity_id, &controller),
        "create_did",
    );

    let err = expect_err(
        client.set_controller(&identity_id, &attacker, &attacker),
        "set_controller unauthorized",
    );
    assert_eq!(err, IdentityRegistryError::Unauthorized);
}
