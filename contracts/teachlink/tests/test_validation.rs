use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String, Vec};
use teachlink_contract::validation::{
    config, AddressValidator, BridgeValidator, BytesValidator, CrossChainValidator,
    EscrowValidator, NumberValidator, RewardsValidator, StringValidator, ValidationError,
};

#[test]
fn test_address_validation() {
    let env = Env::default();

    // Test valid address
    let valid_address = Address::generate(&env);
    assert!(AddressValidator::validate(&env, &valid_address).is_ok());

    // Test blacklist functionality (placeholder)
    // This would need actual blacklist data to test properly
}

#[test]
fn test_number_validation() {
    // Test valid amount
    assert!(NumberValidator::validate_amount(100).is_ok());
    assert!(NumberValidator::validate_amount(config::MAX_AMOUNT).is_ok());

    // Test invalid amounts
    assert!(NumberValidator::validate_amount(0).is_err());
    assert!(NumberValidator::validate_amount(-1).is_err());
    assert!(NumberValidator::validate_amount(config::MAX_AMOUNT + 1).is_err());

    // Test signer count validation
    assert!(NumberValidator::validate_signer_count(1).is_ok());
    assert!(NumberValidator::validate_signer_count(config::MAX_SIGNERS as usize).is_ok());

    assert!(NumberValidator::validate_signer_count(0).is_err());
    assert!(NumberValidator::validate_signer_count((config::MAX_SIGNERS + 1) as usize).is_err());

    // Test threshold validation
    assert!(NumberValidator::validate_threshold(1, 5).is_ok());
    assert!(NumberValidator::validate_threshold(5, 5).is_ok());

    assert!(NumberValidator::validate_threshold(0, 5).is_err());
    assert!(NumberValidator::validate_threshold(6, 5).is_err());

    // Test chain ID validation
    assert!(NumberValidator::validate_chain_id(1).is_ok());
    assert!(NumberValidator::validate_chain_id(config::MAX_CHAIN_ID).is_ok());

    assert!(NumberValidator::validate_chain_id(0).is_err());
    assert!(NumberValidator::validate_chain_id(config::MAX_CHAIN_ID + 1).is_err());

    // Test timeout validation
    assert!(NumberValidator::validate_timeout(config::MIN_TIMEOUT_SECONDS).is_ok());
    assert!(NumberValidator::validate_timeout(config::MAX_TIMEOUT_SECONDS).is_ok());

    assert!(NumberValidator::validate_timeout(config::MIN_TIMEOUT_SECONDS - 1).is_err());
    assert!(NumberValidator::validate_timeout(config::MAX_TIMEOUT_SECONDS + 1).is_err());
}

#[test]
fn test_string_validation() {
    let env = Env::default();

    // Test valid strings
    let valid_string = String::from_str(&env, "valid_string_123");
    assert!(StringValidator::validate(&valid_string, 50).is_ok());

    let alphanumeric = String::from_str(&env, "ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890");
    assert!(StringValidator::validate(&alphanumeric, 50).is_ok());

    let with_spaces = String::from_str(&env, "valid string with spaces");
    assert!(StringValidator::validate(&with_spaces, 50).is_ok());

    let with_punctuation = String::from_str(&env, "valid-string_with.punctuation!");
    assert!(StringValidator::validate(&with_punctuation, 50).is_ok());

    // Test invalid strings
    let empty_string = String::from_str(&env, "");
    assert!(StringValidator::validate(&empty_string, 50).is_err());

    let too_long = String::from_str(&env, "a".repeat(300).as_str());
    assert!(StringValidator::validate(&too_long, 50).is_err());

    // Test invalid characters
    let invalid_chars = String::from_str(&env, "invalid\x00\x01\x02");
    assert!(StringValidator::validate_characters(&invalid_chars).is_err());
}

#[test]
fn test_bytes_validation() {
    let env = Env::default();

    // Test valid cross-chain addresses (20-32 bytes)
    let valid_20_bytes = Bytes::from_array(&env, &[1u8; 20]);
    assert!(BytesValidator::validate_cross_chain_address(&valid_20_bytes).is_ok());

    let valid_32_bytes = Bytes::from_array(&env, &[1u8; 32]);
    assert!(BytesValidator::validate_cross_chain_address(&valid_32_bytes).is_ok());

    // Test invalid cross-chain addresses
    let too_short = Bytes::from_array(&env, &[1u8; 19]);
    assert!(BytesValidator::validate_cross_chain_address(&too_short).is_err());

    let too_long = Bytes::from_array(&env, &[1u8; 33]);
    assert!(BytesValidator::validate_cross_chain_address(&too_long).is_err());

    // Test general bytes validation
    assert!(BytesValidator::validate_length(&valid_20_bytes, 20, 32).is_ok());
    assert!(BytesValidator::validate_length(&valid_32_bytes, 20, 32).is_ok());

    assert!(BytesValidator::validate_length(&too_short, 20, 32).is_err());
    assert!(BytesValidator::validate_length(&too_long, 20, 32).is_err());
}

#[test]
fn test_cross_chain_validation() {
    let env = Env::default();

    let valid_chain_id = 1;
    let valid_address = Bytes::from_array(&env, &[1u8; 20]);
    let valid_amount = 1000i128;
    let valid_recipient = Address::generate(&env);

    // Test valid destination data
    assert!(
        CrossChainValidator::validate_destination_data(&env, valid_chain_id, &valid_address)
            .is_ok()
    );

    // Test invalid destination data
    assert!(CrossChainValidator::validate_destination_data(
        &env,
        0, // invalid chain ID
        &valid_address
    )
    .is_err());

    assert!(CrossChainValidator::validate_destination_data(
        &env,
        valid_chain_id,
        &Bytes::from_array(&env, &[1u8; 19]) // too short
    )
    .is_err());

    // Test valid cross-chain message
    assert!(CrossChainValidator::validate_cross_chain_message(
        &env,
        1, // source chain
        2, // destination chain
        valid_amount,
        &valid_recipient
    )
    .is_ok());

    // Test invalid cross-chain message
    assert!(CrossChainValidator::validate_cross_chain_message(
        &env,
        0, // invalid source chain
        2,
        valid_amount,
        &valid_recipient
    )
    .is_err());

    assert!(CrossChainValidator::validate_cross_chain_message(
        &env,
        1,
        0, // invalid destination chain
        valid_amount,
        &valid_recipient
    )
    .is_err());

    assert!(CrossChainValidator::validate_cross_chain_message(
        &env,
        1,
        2,
        0, // invalid amount
        &valid_recipient
    )
    .is_err());
}

#[test]
fn test_escrow_validation_edge_cases() {
    let env = Env::default();

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let token = Address::generate(&env);
    let arbitrator = Address::generate(&env);

    // Test duplicate signers
    let duplicate_signer = Address::generate(&env);
    let mut signers_with_duplicates = Vec::new(&env);
    signers_with_duplicates.push_back(duplicate_signer.clone());
    signers_with_duplicates.push_back(duplicate_signer.clone());

    assert!(EscrowValidator::check_duplicate_signers(&signers_with_duplicates).is_err());

    // Test valid unique signers
    let mut unique_signers = Vec::new(&env);
    unique_signers.push_back(Address::generate(&env));
    unique_signers.push_back(Address::generate(&env));

    assert!(EscrowValidator::check_duplicate_signers(&unique_signers).is_ok());

    // Test time validation
    let current_time = env.ledger().timestamp();
    let future_release = current_time + 1000;
    let future_refund = future_release + 1000;

    // This should pass - refund time after release time
    let result = EscrowValidator::validate_create_escrow(
        &env,
        &depositor,
        &beneficiary,
        &token,
        1000,
        &unique_signers,
        2,
        Some(future_release),
        Some(future_refund),
        &arbitrator,
    );
    assert!(result.is_ok());

    // This should fail - refund time before release time
    let result = EscrowValidator::validate_create_escrow(
        &env,
        &depositor,
        &beneficiary,
        &token,
        1000,
        &unique_signers,
        2,
        Some(future_refund),
        Some(future_release), // swapped
        &arbitrator,
    );
    assert!(result.is_err());
}

#[test]
fn test_escrow_release_conditions() {
    let env = Env::default();

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let signer = Address::generate(&env);
    let arbitrator = Address::generate(&env);

    let mut signers = Vec::new(&env);
    signers.push_back(signer.clone());

    // Create a test escrow
    let escrow = teachlink_contract::Escrow {
        id: 1,
        depositor: depositor.clone(),
        beneficiary: beneficiary.clone(),
        token: Address::generate(&env),
        amount: 1000,
        signers: signers.clone(),
        threshold: 1,
        approval_count: 1,
        release_time: None,
        refund_time: None,
        arbitrator,
        status: teachlink_contract::EscrowStatus::Pending,
        created_at: env.ledger().timestamp(),
        dispute_reason: None,
    };

    let current_time = env.ledger().timestamp();

    // Test authorized callers
    assert!(
        EscrowValidator::validate_release_conditions(&escrow, &depositor, current_time).is_ok()
    );
    assert!(
        EscrowValidator::validate_release_conditions(&escrow, &beneficiary, current_time).is_ok()
    );
    assert!(EscrowValidator::validate_release_conditions(&escrow, &signer, current_time).is_ok());

    // Test unauthorized caller
    let unauthorized = Address::generate(&env);
    assert!(
        EscrowValidator::validate_release_conditions(&escrow, &unauthorized, current_time).is_err()
    );

    // Test insufficient approvals
    let insufficient_escrow = teachlink_contract::Escrow {
        approval_count: 0,
        ..escrow.clone()
    };
    assert!(EscrowValidator::validate_release_conditions(
        &insufficient_escrow,
        &depositor,
        current_time
    )
    .is_err());

    // Test non-pending status
    let released_escrow = teachlink_contract::Escrow {
        status: teachlink_contract::EscrowStatus::Released,
        ..escrow.clone()
    };
    assert!(EscrowValidator::validate_release_conditions(
        &released_escrow,
        &depositor,
        current_time
    )
    .is_err());

    // Test release time not reached
    let future_time = current_time + 10000;
    let time_locked_escrow = teachlink_contract::Escrow {
        release_time: Some(future_time),
        ..escrow.clone()
    };
    assert!(EscrowValidator::validate_release_conditions(
        &time_locked_escrow,
        &depositor,
        current_time
    )
    .is_err());
}

#[test]
fn test_bridge_validation_edge_cases() {
    let env = Env::default();

    let from = Address::generate(&env);
    let valid_amount = 1000i128;
    let valid_chain_id = 1;
    let valid_address = Bytes::from_array(&env, &[1u8; 20]);

    // Test valid bridge out parameters
    assert!(BridgeValidator::validate_bridge_out(
        &env,
        &from,
        valid_amount,
        valid_chain_id,
        &valid_address
    )
    .is_ok());

    // Test edge cases for amounts
    assert!(BridgeValidator::validate_bridge_out(
        &env,
        &from,
        config::MIN_AMOUNT,
        valid_chain_id,
        &valid_address
    )
    .is_ok());

    assert!(BridgeValidator::validate_bridge_out(
        &env,
        &from,
        0, // invalid
        valid_chain_id,
        &valid_address
    )
    .is_err());

    assert!(BridgeValidator::validate_bridge_out(
        &env,
        &from,
        -1, // invalid
        valid_chain_id,
        &valid_address
    )
    .is_err());

    // Test edge cases for chain IDs
    assert!(BridgeValidator::validate_bridge_out(
        &env,
        &from,
        valid_amount,
        config::MIN_CHAIN_ID,
        &valid_address
    )
    .is_ok());

    assert!(BridgeValidator::validate_bridge_out(
        &env,
        &from,
        valid_amount,
        0, // invalid
        &valid_address
    )
    .is_err());

    // Test edge cases for address lengths
    let min_address = Bytes::from_array(&env, &[1u8; 20]);
    let max_address = Bytes::from_array(&env, &[1u8; 32]);

    assert!(BridgeValidator::validate_bridge_out(
        &env,
        &from,
        valid_amount,
        valid_chain_id,
        &min_address
    )
    .is_ok());

    assert!(BridgeValidator::validate_bridge_out(
        &env,
        &from,
        valid_amount,
        valid_chain_id,
        &max_address
    )
    .is_ok());

    let too_short = Bytes::from_array(&env, &[1u8; 19]);
    assert!(BridgeValidator::validate_bridge_out(
        &env,
        &from,
        valid_amount,
        valid_chain_id,
        &too_short
    )
    .is_err());

    let too_long = Bytes::from_array(&env, &[1u8; 33]);
    assert!(BridgeValidator::validate_bridge_out(
        &env,
        &from,
        valid_amount,
        valid_chain_id,
        &too_long
    )
    .is_err());
}

#[test]
fn test_bridge_completion_validation() {
    let env = Env::default();

    let recipient = Address::generate(&env);
    let token = Address::generate(&env);

    let message = teachlink_contract::CrossChainMessage {
        source_chain: 1,
        source_tx_hash: Bytes::from_array(&env, &[1u8; 32]),
        nonce: 1,
        token: token.clone(),
        amount: 1000,
        recipient: recipient.clone(),
        destination_chain: 2,
    };

    let validator = Address::generate(&env);
    let mut validators = Vec::new(&env);
    validators.push_back(validator.clone());

    // Test valid completion
    assert!(BridgeValidator::validate_bridge_completion(&env, &message, &validators, 1).is_ok());

    // Test insufficient validators
    assert!(BridgeValidator::validate_bridge_completion(
        &env,
        &message,
        &validators,
        2 // require 2 but only have 1
    )
    .is_err());

    // Test invalid message data
    let invalid_message = teachlink_contract::CrossChainMessage {
        source_chain: 0, // invalid
        ..message.clone()
    };
    assert!(
        BridgeValidator::validate_bridge_completion(&env, &invalid_message, &validators, 1)
            .is_err()
    );
}

#[test]
fn test_rewards_validation_edge_cases() {
    let env = Env::default();

    let recipient = Address::generate(&env);
    let valid_amount = 1000i128;
    let valid_reward_type = String::from_str(&env, "course_completion");

    // Test valid reward issuance
    assert!(RewardsValidator::validate_reward_issuance(
        &env,
        &recipient,
        valid_amount,
        &valid_reward_type
    )
    .is_ok());

    // Test edge cases for amounts
    assert!(RewardsValidator::validate_reward_issuance(
        &env,
        &recipient,
        config::MIN_AMOUNT,
        &valid_reward_type
    )
    .is_ok());

    assert!(RewardsValidator::validate_reward_issuance(
        &env,
        &recipient,
        0, // invalid
        &valid_reward_type
    )
    .is_err());

    assert!(RewardsValidator::validate_reward_issuance(
        &env,
        &recipient,
        -1, // invalid
        &valid_reward_type
    )
    .is_err());

    // Test edge cases for reward type strings
    let max_length_string = String::from_str(
        &env,
        "a".repeat(config::MAX_STRING_LENGTH as usize).as_str(),
    );
    assert!(RewardsValidator::validate_reward_issuance(
        &env,
        &recipient,
        valid_amount,
        &max_length_string
    )
    .is_ok());

    let too_long_string = String::from_str(
        &env,
        "a".repeat((config::MAX_STRING_LENGTH + 1) as usize)
            .as_str(),
    );
    assert!(RewardsValidator::validate_reward_issuance(
        &env,
        &recipient,
        valid_amount,
        &too_long_string
    )
    .is_err());

    let empty_string = String::from_str(&env, "");
    assert!(RewardsValidator::validate_reward_issuance(
        &env,
        &recipient,
        valid_amount,
        &empty_string
    )
    .is_err());

    // Test pool funding validation
    let funder = Address::generate(&env);
    assert!(RewardsValidator::validate_pool_funding(&env, &funder, valid_amount).is_ok());

    assert!(RewardsValidator::validate_pool_funding(
        &env,
        &funder,
        0 // invalid
    )
    .is_err());
}

#[test]
fn test_attack_vectors() {
    let env = Env::default();

    // Test overflow attacks
    let max_amount = i128::MAX;
    assert!(NumberValidator::validate_amount(max_amount).is_err());

    // Test very large numbers that might cause issues
    let large_but_valid = config::MAX_AMOUNT;
    assert!(NumberValidator::validate_amount(large_but_valid).is_ok());

    // Test string injection attacks
    let injection_attempts = vec![
        String::from_str(&env, "'; DROP TABLE users; --"),
        String::from_str(&env, "<script>alert('xss')</script>"),
        String::from_str(&env, "../../etc/passwd"),
        String::from_str(&env, "\x00\x01\x02\x03\x04"),
    ];

    for injection in injection_attempts {
        assert!(StringValidator::validate_characters(&injection).is_err());
    }

    // Test boundary conditions
    assert!(NumberValidator::validate_signer_count(config::MAX_SIGNERS as usize).is_ok());
    assert!(NumberValidator::validate_signer_count((config::MAX_SIGNERS + 1) as usize).is_err());

    assert!(NumberValidator::validate_chain_id(config::MAX_CHAIN_ID).is_ok());
    assert!(NumberValidator::validate_chain_id(config::MAX_CHAIN_ID + 1).is_err());

    // Test time-based attacks
    let current_time = env.ledger().timestamp();

    // Test with maximum timeout
    assert!(NumberValidator::validate_timeout(config::MAX_TIMEOUT_SECONDS).is_ok());
    assert!(NumberValidator::validate_timeout(config::MAX_TIMEOUT_SECONDS + 1).is_err());

    // Test with minimum timeout
    assert!(NumberValidator::validate_timeout(config::MIN_TIMEOUT_SECONDS).is_ok());
    assert!(NumberValidator::validate_timeout(config::MIN_TIMEOUT_SECONDS - 1).is_err());
}

#[test]
fn test_config_constants() {
    // Verify that configuration constants are reasonable
    assert!(config::MIN_AMOUNT > 0);
    assert!(config::MAX_AMOUNT > config::MIN_AMOUNT);
    assert!(config::MIN_SIGNERS > 0);
    assert!(config::MAX_SIGNERS > config::MIN_SIGNERS);
    assert!(config::MIN_THRESHOLD > 0);
    assert!(config::MAX_STRING_LENGTH > 0);
    assert!(config::MIN_CHAIN_ID > 0);
    assert!(config::MAX_CHAIN_ID > config::MIN_CHAIN_ID);
    assert!(config::MIN_TIMEOUT_SECONDS > 0);
    assert!(config::MAX_TIMEOUT_SECONDS > config::MIN_TIMEOUT_SECONDS);
}
