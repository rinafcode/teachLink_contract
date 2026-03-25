#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, ConversionError, Env, InvokeError};

use identity_registry::{
    IdentityRegistryContract, IdentityRegistryContractClient, IdentityRegistryError,
};

fn expect_contract_err<T, E: Copy + core::fmt::Debug>(
    res: Result<Result<T, ConversionError>, Result<E, InvokeError>>,
    context: &str,
) -> E {
    match res {
        Ok(Ok(_)) => panic!("{}: expected error", context),
        Ok(Err(err)) => panic!("{}: conversion error {:?}", context, err),
        Err(Ok(err)) => err,
        Err(Err(err)) => panic!("{}: invoke error {:?}", context, err),
    }
}

fn expect_contract_ok<T, E: core::fmt::Debug>(
    res: Result<Result<T, ConversionError>, Result<E, InvokeError>>,
    context: &str,
) -> T {
    match res {
        Ok(Ok(value)) => value,
        Ok(Err(err)) => panic!("{}: conversion error {:?}", context, err),
        Err(Ok(err)) => panic!("{}: contract error {:?}", context, err),
        Err(Err(err)) => panic!("{}: invoke error {:?}", context, err),
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

    let err = expect_contract_err(
        client.try_set_controller(&identity_id, &current, &new_controller),
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

    expect_contract_ok(
        client.try_create_did(&identity_id, &controller),
        "create_did",
    );

    let err = expect_contract_err(
        client.try_set_controller(&identity_id, &attacker, &attacker),
        "set_controller unauthorized",
    );
    assert_eq!(err, IdentityRegistryError::Unauthorized);
}
