#![allow(clippy::unreadable_literal)]

use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String};
use teachlink_contract::validation::{
    AddressValidator, BridgeValidator, BytesValidator, CrossChainValidator, NumberValidator,
    StringValidator, ValidationError,
};

// ── Amount validation ────────────────────────────────────────────────────────

#[test]
fn test_amount_valid() {
    assert!(NumberValidator::validate_amount(1).is_ok());
    assert!(NumberValidator::validate_amount(1_000_000).is_ok());
}

#[test]
fn test_amount_zero_rejected() {
    assert_eq!(
        NumberValidator::validate_amount(0),
        Err(ValidationError::InvalidAmountRange)
    );
}

#[test]
fn test_amount_negative_rejected() {
    assert_eq!(
        NumberValidator::validate_amount(-1),
        Err(ValidationError::InvalidAmountRange)
    );
}

#[test]
fn test_amount_overflow_rejected() {
    assert_eq!(
        NumberValidator::validate_amount(i128::MAX),
        Err(ValidationError::InvalidAmountRange)
    );
}

// ── Chain ID validation ──────────────────────────────────────────────────────

#[test]
fn test_chain_id_valid() {
    assert!(NumberValidator::validate_chain_id(1).is_ok());
    assert!(NumberValidator::validate_chain_id(999999).is_ok());
}

#[test]
fn test_chain_id_zero_rejected() {
    assert_eq!(
        NumberValidator::validate_chain_id(0),
        Err(ValidationError::InvalidChainId)
    );
}

#[test]
fn test_chain_id_too_large_rejected() {
    assert_eq!(
        NumberValidator::validate_chain_id(1_000_000),
        Err(ValidationError::InvalidChainId)
    );
}

// ── Address validation ───────────────────────────────────────────────────────

#[test]
fn test_address_valid() {
    let env = Env::default();
    let addr = Address::generate(&env);
    assert!(AddressValidator::validate(&env, &addr).is_ok());
}

#[test]
fn test_blacklisted_address_rejected() {
    let env = Env::default();
    let addr = Address::generate(&env);

    // Manually insert into blacklist
    let blacklist_key = soroban_sdk::symbol_short!("blacklist");
    let mut blacklist: soroban_sdk::Vec<Address> = soroban_sdk::Vec::new(&env);
    blacklist.push_back(addr.clone());
    env.storage().instance().set(&blacklist_key, &blacklist);

    assert_eq!(
        AddressValidator::validate(&env, &addr),
        Err(ValidationError::BlacklistedAddress)
    );
}

// ── Cross-chain address (bytes) validation ───────────────────────────────────

#[test]
fn test_cross_chain_address_valid() {
    let env = Env::default();
    let mut arr20 = [0u8; 20];
    arr20[0] = 1; // non-zero
    let addr = Bytes::from_array(&env, &arr20);
    assert!(BytesValidator::validate_cross_chain_address(&addr).is_ok());

    let mut arr32 = [0u8; 32];
    arr32[0] = 1;
    let addr32 = Bytes::from_array(&env, &arr32);
    assert!(BytesValidator::validate_cross_chain_address(&addr32).is_ok());
}

#[test]
fn test_cross_chain_address_all_zeros_rejected() {
    let env = Env::default();
    let addr = Bytes::from_array(&env, &[0u8; 20]);
    assert_eq!(
        BytesValidator::validate_cross_chain_address(&addr),
        Err(ValidationError::InvalidAddressFormat)
    );
}

#[test]
fn test_cross_chain_address_too_short_rejected() {
    let env = Env::default();
    let addr = Bytes::from_array(&env, &[0u8; 19]);
    assert_eq!(
        BytesValidator::validate_cross_chain_address(&addr),
        Err(ValidationError::InvalidBytesLength)
    );
}

#[test]
fn test_cross_chain_address_too_long_rejected() {
    let env = Env::default();
    let addr = Bytes::from_array(&env, &[0u8; 33]);
    assert_eq!(
        BytesValidator::validate_cross_chain_address(&addr),
        Err(ValidationError::InvalidBytesLength)
    );
}

// ── Payload validation ───────────────────────────────────────────────────────

#[test]
fn test_payload_valid() {
    let env = Env::default();
    let payload = Bytes::from_array(&env, &[1u8; 100]);
    assert!(BytesValidator::validate_payload(&payload).is_ok());
}

#[test]
fn test_payload_empty_rejected() {
    let env = Env::default();
    let payload = Bytes::new(&env);
    assert_eq!(
        BytesValidator::validate_payload(&payload),
        Err(ValidationError::InvalidCrossChainData)
    );
}

#[test]
fn test_payload_too_large_rejected() {
    let env = Env::default();
    // 4097 bytes exceeds MAX_PAYLOAD_SIZE (4096)
    let big: soroban_sdk::Vec<u8> = {
        let mut v = soroban_sdk::Vec::new(&env);
        for _ in 0..4097u32 {
            v.push_back(0u8);
        }
        v
    };
    let payload = Bytes::from_slice(&env, &{
        let mut arr = [0u8; 4097];
        arr
    });
    assert_eq!(
        BytesValidator::validate_payload(&payload),
        Err(ValidationError::InvalidBytesLength)
    );
}

// ── Cross-chain message validation ───────────────────────────────────────────

#[test]
fn test_cross_chain_message_valid() {
    let env = Env::default();
    let recipient = Address::generate(&env);
    assert!(
        CrossChainValidator::validate_cross_chain_message(&env, 1, 2, 1000, &recipient).is_ok()
    );
}

#[test]
fn test_cross_chain_message_invalid_source_chain() {
    let env = Env::default();
    let recipient = Address::generate(&env);
    assert_eq!(
        CrossChainValidator::validate_cross_chain_message(&env, 0, 2, 1000, &recipient),
        Err(ValidationError::InvalidChainId)
    );
}

#[test]
fn test_cross_chain_message_invalid_dest_chain() {
    let env = Env::default();
    let recipient = Address::generate(&env);
    assert_eq!(
        CrossChainValidator::validate_cross_chain_message(&env, 1, 0, 1000, &recipient),
        Err(ValidationError::InvalidChainId)
    );
}

#[test]
fn test_cross_chain_message_invalid_amount() {
    let env = Env::default();
    let recipient = Address::generate(&env);
    assert_eq!(
        CrossChainValidator::validate_cross_chain_message(&env, 1, 2, 0, &recipient),
        Err(ValidationError::InvalidAmountRange)
    );
}

// ── String validation ────────────────────────────────────────────────────────

#[test]
fn test_string_valid() {
    let env = Env::default();
    let s = String::from_str(&env, "hello_world");
    assert!(StringValidator::validate(&s, 256).is_ok());
}

#[test]
fn test_string_too_long_rejected() {
    let env = Env::default();
    // 257 chars
    let long = "a".repeat(257);
    let s = String::from_str(&env, &long);
    assert_eq!(
        StringValidator::validate(&s, 256),
        Err(ValidationError::InvalidStringLength)
    );
}

#[test]
fn test_string_empty_rejected() {
    let env = Env::default();
    let s = String::from_str(&env, "");
    assert_eq!(
        StringValidator::validate_length(&s, 256),
        Err(ValidationError::InvalidStringLength)
    );
}

// ── Timeout validation ───────────────────────────────────────────────────────

#[test]
fn test_timeout_valid() {
    assert!(NumberValidator::validate_timeout(60).is_ok());
    assert!(NumberValidator::validate_timeout(86400).is_ok());
}

#[test]
fn test_timeout_too_short_rejected() {
    assert_eq!(
        NumberValidator::validate_timeout(59),
        Err(ValidationError::InvalidTimeout)
    );
}

#[test]
fn test_timeout_too_long_rejected() {
    assert_eq!(
        NumberValidator::validate_timeout(u64::MAX),
        Err(ValidationError::InvalidTimeout)
    );
}

// ── BridgeValidator ──────────────────────────────────────────────────────────

#[test]
fn test_bridge_out_valid() {
    let env = Env::default();
    let from = Address::generate(&env);
    let mut dest = [0u8; 20];
    dest[0] = 1;
    let dest_addr = Bytes::from_array(&env, &dest);
    assert!(BridgeValidator::validate_bridge_out(&env, &from, 1_000, 1, &dest_addr).is_ok());
}

#[test]
fn test_bridge_out_zero_amount_rejected() {
    let env = Env::default();
    let from = Address::generate(&env);
    let mut dest = [0u8; 20];
    dest[0] = 1;
    let dest_addr = Bytes::from_array(&env, &dest);
    let err = BridgeValidator::validate_bridge_out(&env, &from, 0, 1, &dest_addr).unwrap_err();
    assert_eq!(err, teachlink_contract::BridgeError::AmountMustBePositive);
}

#[test]
fn test_bridge_out_negative_amount_rejected() {
    let env = Env::default();
    let from = Address::generate(&env);
    let mut dest = [0u8; 20];
    dest[0] = 1;
    let dest_addr = Bytes::from_array(&env, &dest);
    let err = BridgeValidator::validate_bridge_out(&env, &from, -1, 1, &dest_addr).unwrap_err();
    assert_eq!(err, teachlink_contract::BridgeError::AmountMustBePositive);
}

#[test]
fn test_bridge_out_amount_exceeds_max_rejected() {
    let env = Env::default();
    let from = Address::generate(&env);
    let mut dest = [0u8; 20];
    dest[0] = 1;
    let dest_addr = Bytes::from_array(&env, &dest);
    let err =
        BridgeValidator::validate_bridge_out(&env, &from, i128::MAX, 1, &dest_addr).unwrap_err();
    assert_eq!(err, teachlink_contract::BridgeError::AmountMustBePositive);
}

#[test]
fn test_bridge_out_invalid_chain_id_rejected() {
    let env = Env::default();
    let from = Address::generate(&env);
    let mut dest = [0u8; 20];
    dest[0] = 1;
    let dest_addr = Bytes::from_array(&env, &dest);
    let err =
        BridgeValidator::validate_bridge_out(&env, &from, 1_000, 0, &dest_addr).unwrap_err();
    assert_eq!(
        err,
        teachlink_contract::BridgeError::DestinationChainNotSupported
    );
}

#[test]
fn test_bridge_out_zero_destination_address_rejected() {
    let env = Env::default();
    let from = Address::generate(&env);
    let dest_addr = Bytes::from_array(&env, &[0u8; 20]); // all zeros
    let err =
        BridgeValidator::validate_bridge_out(&env, &from, 1_000, 1, &dest_addr).unwrap_err();
    assert_eq!(err, teachlink_contract::BridgeError::InvalidInput);
}

#[test]
fn test_bridge_out_short_destination_address_rejected() {
    let env = Env::default();
    let from = Address::generate(&env);
    let dest_addr = Bytes::from_array(&env, &[1u8; 19]); // too short
    let err =
        BridgeValidator::validate_bridge_out(&env, &from, 1_000, 1, &dest_addr).unwrap_err();
    assert_eq!(err, teachlink_contract::BridgeError::InvalidInput);
}
