#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]

use soroban_sdk::{
    testutils::Address as _, Address, Bytes, ConversionError, Env, InvokeError, Vec,
};

use teachlink_contract::{
    BridgeError, ContentTokenParameters, ContentType, RewardsError, TeachLinkBridge,
    TeachLinkBridgeClient, TokenizationError,
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

fn mint_sample_token(env: &Env, client: &TeachLinkBridgeClient, creator: Address) -> u64 {
    let params = ContentTokenParameters {
        creator: creator.clone(),
        title: Bytes::from_slice(env, b"Title"),
        description: Bytes::from_slice(env, b"Desc"),
        content_type: ContentType::Material,
        content_hash: Bytes::from_slice(env, b"hash"),
        license_type: Bytes::from_slice(env, b"license"),
        tags: Vec::new(env),
        is_transferable: true,
        royalty_percentage: 0,
    };
    client.mint_content_token(&params)
}

#[test]
fn test_bridge_get_token_before_init() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let err = expect_contract_err(client.try_get_token(), "get_token before init");
    assert_eq!(err, BridgeError::NotInitialized);
}

#[test]
fn test_rewards_admin_before_init() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let err = expect_contract_err(
        client.try_get_rewards_admin(),
        "get_rewards_admin before init",
    );
    assert_eq!(err, RewardsError::MissingAdmin);
}

#[test]
fn test_tokenization_errors() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let attacker = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token_id = mint_sample_token(&env, &client, creator.clone());

    let err = expect_contract_err(
        client.try_transfer_content_token(&attacker, &recipient, &token_id, &None),
        "transfer by non-owner",
    );
    assert_eq!(err, TokenizationError::NotTokenOwner);

    let err = expect_contract_err(
        client.try_transfer_content_token(&creator, &recipient, &999u64, &None),
        "transfer missing token",
    );
    assert_eq!(err, TokenizationError::TokenNotFound);
}
