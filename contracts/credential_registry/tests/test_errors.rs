#![cfg(test)]

use soroban_sdk::{
    testutils::Address as _, Address, Bytes, BytesN, ConversionError, Env, InvokeError,
};

use credential_registry::{
    CredentialRegistryContract, CredentialRegistryContractClient, CredentialRegistryError,
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
fn test_duplicate_issue() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CredentialRegistryContract, ());
    let client = CredentialRegistryContractClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let hash = BytesN::from_array(&env, &[1u8; 32]);
    let issuer_did = Bytes::from_slice(&env, b"did:issuer");
    let subject_did = Bytes::from_slice(&env, b"did:subject");
    let metadata = Bytes::from_slice(&env, b"ipfs://meta");

    expect_contract_ok(
        client.try_issue_credential(&hash, &issuer, &issuer_did, &subject_did, &metadata, &0),
        "initial issue",
    );

    let err = expect_contract_err(
        client.try_issue_credential(&hash, &issuer, &issuer_did, &subject_did, &metadata, &0),
        "duplicate issue",
    );
    assert_eq!(err, CredentialRegistryError::CredentialAlreadyExists);
}

#[test]
fn test_revoke_missing() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CredentialRegistryContract, ());
    let client = CredentialRegistryContractClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let hash = BytesN::from_array(&env, &[2u8; 32]);

    let err = expect_contract_err(
        client.try_revoke_credential(&hash, &issuer),
        "revoke missing",
    );
    assert_eq!(err, CredentialRegistryError::CredentialNotFound);
}
