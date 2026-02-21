#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::too_many_lines)]
#![allow(unused_variables)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    vec, Address, Bytes, Env, Vec,
};

use teachlink_contract::{
    ContentTokenParameters, ContentType, TeachLinkBridge, TeachLinkBridgeClient, TransferType,
};

fn create_params(
    _env: &Env,
    creator: Address,
    title: Bytes,
    description: Bytes,
    content_type: ContentType,
    content_hash: Bytes,
    license_type: Bytes,
    tags: Vec<Bytes>,
    is_transferable: bool,
    royalty_percentage: u32,
) -> ContentTokenParameters {
    ContentTokenParameters {
        creator,
        title,
        description,
        content_type,
        content_hash,
        license_type,
        tags,
        is_transferable,
        royalty_percentage,
    }
}

#[test]
fn test_mint_content_token() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TeachLinkBridge, ());
    let creator = Address::generate(&env);

    // Set ledger timestamp
    env.ledger().set(LedgerInfo {
        timestamp: 1000,
        protocol_version: 25,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 2000000,
    });

    let title = Bytes::from_slice(&env, b"Introduction to Rust");
    let description = Bytes::from_slice(&env, b"A comprehensive course on Rust programming");
    let content_hash = Bytes::from_slice(&env, b"QmHash123456789");
    let license_type = Bytes::from_slice(&env, b"MIT");
    let tags = vec![
        &env,
        Bytes::from_slice(&env, b"rust"),
        Bytes::from_slice(&env, b"programming"),
    ];

    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let params = create_params(
        &env,
        creator.clone(),
        title.clone(),
        description.clone(),
        ContentType::Course,
        content_hash.clone(),
        license_type.clone(),
        tags.clone(),
        true,
        500u32, // 5% royalty
    );
    let token_id = client.mint_content_token(&params);

    assert_eq!(token_id, 1u64);

    // Verify token exists
    let token = client.get_content_token(&token_id).unwrap();

    assert_eq!(token.token_id, 1u64);
    assert_eq!(token.owner, creator);
    assert_eq!(token.metadata.title, title);
    assert_eq!(token.metadata.description, description);
    assert_eq!(token.metadata.content_type, ContentType::Course);
    assert_eq!(token.metadata.creator, creator);
    assert!(token.is_transferable);
    assert_eq!(token.royalty_percentage, 500u32);

    // Verify ownership
    let owner = client.get_content_token_owner(&token_id).unwrap();
    assert_eq!(owner, creator);

    // Verify creator owns the token
    assert!(client.is_content_token_owner(&token_id, &creator));

    // TODO: Re-enable when provenance module is implemented
    // Verify provenance
    // let provenance = client.get_content_provenance(&token_id);
    // assert_eq!(provenance.len(), 1u32);
    // let first_record = provenance.get(0).unwrap();
    // assert_eq!(first_record.transfer_type, TransferType::Mint);
    // assert_eq!(first_record.to, creator);
}

#[test]
fn test_transfer_content_token() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TeachLinkBridge, ());
    let creator = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    env.ledger().set(LedgerInfo {
        timestamp: 1000,
        protocol_version: 25,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 2000000,
    });

    // Mint a token
    let title = Bytes::from_slice(&env, b"Test Course");
    let description = Bytes::from_slice(&env, b"Test Description");
    let content_hash = Bytes::from_slice(&env, b"QmHash");
    let license_type = Bytes::from_slice(&env, b"MIT");
    let tags = vec![&env];

    let params = create_params(
        &env,
        creator.clone(),
        title,
        description,
        ContentType::Course,
        content_hash,
        license_type,
        tags,
        true,
        0u32,
    );
    let token_id = client.mint_content_token(&params);

    // Transfer token
    env.ledger().set(LedgerInfo {
        timestamp: 2000,
        protocol_version: 25,
        sequence_number: 11,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 2000000,
    });

    let notes = Bytes::from_slice(&env, b"Transfer to new owner");
    client.transfer_content_token(&creator, &new_owner, &token_id, &Some(notes.clone()));

    // Verify new ownership
    let owner = client.get_content_token_owner(&token_id).unwrap();
    assert_eq!(owner, new_owner);

    assert!(client.is_content_token_owner(&token_id, &new_owner));
    assert!(!client.is_content_token_owner(&token_id, &creator));

    // TODO: Re-enable when provenance module is implemented
    // Verify provenance
    // let provenance = client.get_content_provenance(&token_id);
    // assert_eq!(provenance.len(), 2u32);

    // let mint_record = provenance.get(0).unwrap();
    // assert_eq!(mint_record.transfer_type, TransferType::Mint);

    // let transfer_record = provenance.get(1).unwrap();
    // assert_eq!(transfer_record.transfer_type, TransferType::Transfer);
    // assert_eq!(transfer_record.from, Some(creator));
    // assert_eq!(transfer_record.to, new_owner);
}

#[test]
#[should_panic(expected = "Caller is not the owner")]
fn test_transfer_not_owner() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TeachLinkBridge, ());
    let creator = Address::generate(&env);
    let attacker = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    env.ledger().set(LedgerInfo {
        timestamp: 1000,
        protocol_version: 25,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 2000000,
    });

    // Mint a token
    let title = Bytes::from_slice(&env, b"Test Course");
    let description = Bytes::from_slice(&env, b"Test Description");
    let content_hash = Bytes::from_slice(&env, b"QmHash");
    let license_type = Bytes::from_slice(&env, b"MIT");
    let tags = vec![&env];

    let params = create_params(
        &env,
        creator.clone(),
        title,
        description,
        ContentType::Course,
        content_hash,
        license_type,
        tags,
        true,
        0u32,
    );
    let token_id = client.mint_content_token(&params);

    // Try to transfer as non-owner (should fail)
    client.transfer_content_token(&attacker, &new_owner, &token_id, &None);
}

#[test]
#[should_panic(expected = "Token is not transferable")]
fn test_transfer_non_transferable() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TeachLinkBridge, ());
    let creator = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    env.ledger().set(LedgerInfo {
        timestamp: 1000,
        protocol_version: 25,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 2000000,
    });

    // Mint a non-transferable token
    let title = Bytes::from_slice(&env, b"Test Course");
    let description = Bytes::from_slice(&env, b"Test Description");
    let content_hash = Bytes::from_slice(&env, b"QmHash");
    let license_type = Bytes::from_slice(&env, b"MIT");
    let tags = vec![&env];

    let params = create_params(
        &env,
        creator.clone(),
        title,
        description,
        ContentType::Course,
        content_hash,
        license_type,
        tags,
        false, // Not transferable
        0u32,
    );
    let token_id = client.mint_content_token(&params);

    // Try to transfer (should fail)
    client.transfer_content_token(&creator, &new_owner, &token_id, &None);
}

#[test]
fn test_get_owner_tokens() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TeachLinkBridge, ());
    let creator = Address::generate(&env);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    env.ledger().set(LedgerInfo {
        timestamp: 1000,
        protocol_version: 25,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 2000000,
    });

    let title = Bytes::from_slice(&env, b"Course");
    let description = Bytes::from_slice(&env, b"Description");
    let content_hash = Bytes::from_slice(&env, b"QmHash");
    let license_type = Bytes::from_slice(&env, b"MIT");
    let tags = vec![&env];

    // Mint multiple tokens
    let params1 = create_params(
        &env,
        creator.clone(),
        title.clone(),
        description.clone(),
        ContentType::Course,
        content_hash.clone(),
        license_type.clone(),
        tags.clone(),
        true,
        0u32,
    );
    let token1 = client.mint_content_token(&params1);

    let params2 = create_params(
        &env,
        creator.clone(),
        title.clone(),
        description.clone(),
        ContentType::Material,
        content_hash.clone(),
        license_type.clone(),
        tags.clone(),
        true,
        0u32,
    );
    let token2 = client.mint_content_token(&params2);

    // Get owner's tokens
    let owner_tokens = client.get_owner_content_tokens(&creator);
    assert_eq!(owner_tokens.len(), 2u32);
    assert_eq!(owner_tokens.get(0).unwrap(), token1);
    assert_eq!(owner_tokens.get(1).unwrap(), token2);
}

#[test]
fn test_update_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TeachLinkBridge, ());
    let creator = Address::generate(&env);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    env.ledger().set(LedgerInfo {
        timestamp: 1000,
        protocol_version: 25,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 2000000,
    });

    let title = Bytes::from_slice(&env, b"Original Title");
    let description = Bytes::from_slice(&env, b"Original Description");
    let content_hash = Bytes::from_slice(&env, b"QmHash");
    let license_type = Bytes::from_slice(&env, b"MIT");
    let tags = vec![&env];

    let params = create_params(
        &env,
        creator.clone(),
        title,
        description,
        ContentType::Course,
        content_hash,
        license_type,
        tags,
        true,
        0u32,
    );
    let token_id = client.mint_content_token(&params);

    // Update metadata
    env.ledger().set(LedgerInfo {
        timestamp: 2000,
        protocol_version: 25,
        sequence_number: 11,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 2000000,
    });

    let new_title = Bytes::from_slice(&env, b"Updated Title");
    let new_description = Bytes::from_slice(&env, b"Updated Description");
    let new_tags = vec![
        &env,
        Bytes::from_slice(&env, b"tag1"),
        Bytes::from_slice(&env, b"tag2"),
    ];

    client.update_content_metadata(
        &creator,
        &token_id,
        &Some(new_title.clone()),
        &Some(new_description.clone()),
        &Some(new_tags.clone()),
    );

    // Verify updates
    let token = client.get_content_token(&token_id).unwrap();
    assert_eq!(token.metadata.title, new_title);
    assert_eq!(token.metadata.description, new_description);
    assert_eq!(token.metadata.tags, new_tags);
    assert_eq!(token.metadata.updated_at, 2000u64);
}

#[test]
fn test_verify_provenance_chain() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TeachLinkBridge, ());
    let creator = Address::generate(&env);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);
    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);

    env.ledger().set(LedgerInfo {
        timestamp: 1000,
        protocol_version: 25,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 2000000,
    });

    // Mint token
    let title = Bytes::from_slice(&env, b"Test Course");
    let description = Bytes::from_slice(&env, b"Test Description");
    let content_hash = Bytes::from_slice(&env, b"QmHash");
    let license_type = Bytes::from_slice(&env, b"MIT");
    let tags = vec![&env];

    let params = create_params(
        &env,
        creator.clone(),
        title,
        description,
        ContentType::Course,
        content_hash,
        license_type,
        tags,
        true,
        0u32,
    );
    let token_id = client.mint_content_token(&params);

    // Transfer multiple times
    env.ledger().set(LedgerInfo {
        timestamp: 2000,
        protocol_version: 25,
        sequence_number: 11,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 2000000,
    });

    client.transfer_content_token(&creator, &owner1, &token_id, &None);

    env.ledger().set(LedgerInfo {
        timestamp: 3000,
        protocol_version: 25,
        sequence_number: 12,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 2000000,
    });

    client.transfer_content_token(&owner1, &owner2, &token_id, &None);

    // TODO: Re-enable when provenance module is implemented
    // Verify chain integrity
    // let is_valid = client.verify_content_chain(&token_id);
    // assert!(is_valid);

    // Verify creator
    let creator_addr = client.get_content_creator(&token_id).unwrap();
    assert_eq!(creator_addr, creator);
}

#[test]
fn test_get_token_count() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TeachLinkBridge, ());
    let creator = Address::generate(&env);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    env.ledger().set(LedgerInfo {
        timestamp: 1000,
        protocol_version: 25,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 2000000,
    });

    let title = Bytes::from_slice(&env, b"Course");
    let description = Bytes::from_slice(&env, b"Description");
    let content_hash = Bytes::from_slice(&env, b"QmHash");
    let license_type = Bytes::from_slice(&env, b"MIT");
    let tags = vec![&env];

    // Initially zero
    let count = client.get_content_token_count();
    assert_eq!(count, 0u64);

    // Mint tokens
    let params1 = create_params(
        &env,
        creator.clone(),
        title.clone(),
        description.clone(),
        ContentType::Course,
        content_hash.clone(),
        license_type.clone(),
        tags.clone(),
        true,
        0u32,
    );
    client.mint_content_token(&params1);

    let params2 = create_params(
        &env,
        creator.clone(),
        title.clone(),
        description.clone(),
        ContentType::Material,
        content_hash.clone(),
        license_type.clone(),
        tags.clone(),
        true,
        0u32,
    );
    client.mint_content_token(&params2);

    let count = client.get_content_token_count();
    assert_eq!(count, 2u64);
}
