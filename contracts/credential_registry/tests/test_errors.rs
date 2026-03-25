#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env};

use credential_registry_contract::{
    CredentialRegistryContract, CredentialRegistryContractClient, CredentialRegistryError,
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

    expect_ok(
        client.issue_credential(&hash, &issuer, &issuer_did, &subject_did, &metadata, &0),
        "initial issue",
    );

    let err = expect_err(
        client.issue_credential(&hash, &issuer, &issuer_did, &subject_did, &metadata, &0),
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

    let err = expect_err(
        client.revoke_credential(&hash, &issuer),
        "revoke missing",
    );
    assert_eq!(err, CredentialRegistryError::CredentialNotFound);
}
