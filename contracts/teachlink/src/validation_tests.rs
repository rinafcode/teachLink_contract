//! Comprehensive input validation tests
//!
//! Covers: address format, amount bounds, chain ID (range + registry),
//! parameter sanitization, cross-chain data, and escrow/bridge/rewards validators.

#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String};

    use crate::validation::{
        config, AddressValidator, BridgeValidator, BytesValidator, CrossChainValidator,
        EscrowValidator, InputSanitizer, NumberValidator, RewardsValidator, StringValidator,
        ValidationError,
    };

    // ── helpers ──────────────────────────────────────────────────────────────

    fn make_address_bytes(env: &Env, len: u32, fill: u8) -> Bytes {
        let mut b = Bytes::new(env);
        for _ in 0..len {
            b.push_back(fill);
        }
        b
    }

    fn valid_dest(env: &Env) -> Bytes {
        make_address_bytes(env, 20, 0xAB)
    }

    // ── AddressValidator ─────────────────────────────────────────────────────

    #[test]
    fn address_format_valid() {
        let env = Env::default();
        let addr = Address::generate(&env);
        assert!(AddressValidator::validate_format(&env, &addr).is_ok());
    }

    #[test]
    fn address_not_blacklisted_by_default() {
        let env = Env::default();
        let addr = Address::generate(&env);
        assert!(AddressValidator::check_blacklist(&env, &addr).is_ok());
    }

    #[test]
    fn address_blacklisted_returns_error() {
        let env = Env::default();
        let addr = Address::generate(&env);
        // Manually insert into blacklist
        let key = soroban_sdk::symbol_short!("blacklist");
        let mut list: soroban_sdk::Vec<Address> = soroban_sdk::Vec::new(&env);
        list.push_back(addr.clone());
        env.storage().instance().set(&key, &list);
        assert_eq!(
            AddressValidator::check_blacklist(&env, &addr),
            Err(ValidationError::BlacklistedAddress)
        );
    }

    // ── NumberValidator – amount ─────────────────────────────────────────────

    #[test]
    fn amount_zero_is_invalid() {
        assert_eq!(
            NumberValidator::validate_amount(0),
            Err(ValidationError::InvalidAmountRange)
        );
    }

    #[test]
    fn amount_negative_is_invalid() {
        assert_eq!(
            NumberValidator::validate_amount(-1),
            Err(ValidationError::InvalidAmountRange)
        );
    }

    #[test]
    fn amount_min_is_valid() {
        assert!(NumberValidator::validate_amount(config::MIN_AMOUNT).is_ok());
    }

    #[test]
    fn amount_max_is_valid() {
        assert!(NumberValidator::validate_amount(config::MAX_AMOUNT).is_ok());
    }

    #[test]
    fn amount_over_max_is_invalid() {
        assert_eq!(
            NumberValidator::validate_amount(config::MAX_AMOUNT + 1),
            Err(ValidationError::InvalidAmountRange)
        );
    }

    // ── NumberValidator – chain ID ───────────────────────────────────────────

    #[test]
    fn chain_id_zero_is_invalid() {
        assert_eq!(
            NumberValidator::validate_chain_id(0),
            Err(ValidationError::InvalidChainId)
        );
    }

    #[test]
    fn chain_id_min_is_valid() {
        assert!(NumberValidator::validate_chain_id(config::MIN_CHAIN_ID).is_ok());
    }

    #[test]
    fn chain_id_max_is_valid() {
        assert!(NumberValidator::validate_chain_id(config::MAX_CHAIN_ID).is_ok());
    }

    #[test]
    fn chain_id_over_max_is_invalid() {
        assert_eq!(
            NumberValidator::validate_chain_id(config::MAX_CHAIN_ID + 1),
            Err(ValidationError::InvalidChainId)
        );
    }

    // ── BytesValidator ───────────────────────────────────────────────────────

    #[test]
    fn cross_chain_address_too_short_is_invalid() {
        let env = Env::default();
        let b = make_address_bytes(&env, 19, 0x01);
        assert_eq!(
            BytesValidator::validate_cross_chain_address(&b),
            Err(ValidationError::InvalidBytesLength)
        );
    }

    #[test]
    fn cross_chain_address_too_long_is_invalid() {
        let env = Env::default();
        let b = make_address_bytes(&env, 33, 0x01);
        assert_eq!(
            BytesValidator::validate_cross_chain_address(&b),
            Err(ValidationError::InvalidBytesLength)
        );
    }

    #[test]
    fn cross_chain_address_all_zeros_is_invalid() {
        let env = Env::default();
        let b = make_address_bytes(&env, 20, 0x00);
        assert_eq!(
            BytesValidator::validate_cross_chain_address(&b),
            Err(ValidationError::InvalidAddressFormat)
        );
    }

    #[test]
    fn cross_chain_address_valid_20_bytes() {
        let env = Env::default();
        assert!(BytesValidator::validate_cross_chain_address(&valid_dest(&env)).is_ok());
    }

    #[test]
    fn cross_chain_address_valid_32_bytes() {
        let env = Env::default();
        let b = make_address_bytes(&env, 32, 0xFF);
        assert!(BytesValidator::validate_cross_chain_address(&b).is_ok());
    }

    #[test]
    fn payload_empty_is_invalid() {
        let env = Env::default();
        let b = Bytes::new(&env);
        assert_eq!(
            BytesValidator::validate_payload(&b),
            Err(ValidationError::InvalidCrossChainData)
        );
    }

    #[test]
    fn payload_over_limit_is_invalid() {
        let env = Env::default();
        let b = make_address_bytes(&env, config::MAX_PAYLOAD_SIZE + 1, 0x01);
        assert_eq!(
            BytesValidator::validate_payload(&b),
            Err(ValidationError::InvalidBytesLength)
        );
    }

    #[test]
    fn payload_at_limit_is_valid() {
        let env = Env::default();
        let b = make_address_bytes(&env, config::MAX_PAYLOAD_SIZE, 0x01);
        assert!(BytesValidator::validate_payload(&b).is_ok());
    }

    // ── StringValidator ──────────────────────────────────────────────────────

    #[test]
    fn string_empty_is_invalid() {
        let env = Env::default();
        let s = String::from_str(&env, "");
        assert_eq!(
            StringValidator::validate_length(&s, 256),
            Err(ValidationError::InvalidStringLength)
        );
    }

    #[test]
    fn string_over_max_length_is_invalid() {
        let env = Env::default();
        // 5-char string with max_length = 4
        let s = String::from_str(&env, "hello");
        assert_eq!(
            StringValidator::validate_length(&s, 4),
            Err(ValidationError::InvalidStringLength)
        );
    }

    #[test]
    fn string_valid_length() {
        let env = Env::default();
        let s = String::from_str(&env, "reward");
        assert!(StringValidator::validate_length(&s, 256).is_ok());
    }

    // ── InputSanitizer ───────────────────────────────────────────────────────

    #[test]
    fn sanitize_amount_zero_rejected() {
        assert_eq!(
            InputSanitizer::sanitize_amount(0),
            Err(ValidationError::InvalidAmountRange)
        );
    }

    #[test]
    fn sanitize_amount_negative_rejected() {
        assert_eq!(
            InputSanitizer::sanitize_amount(-100),
            Err(ValidationError::InvalidAmountRange)
        );
    }

    #[test]
    fn sanitize_amount_over_bridge_max_rejected() {
        assert_eq!(
            InputSanitizer::sanitize_amount(config::MAX_BRIDGE_AMOUNT + 1),
            Err(ValidationError::InvalidAmountRange)
        );
    }

    #[test]
    fn sanitize_amount_valid_passthrough() {
        let v = 1_000_000i128;
        assert_eq!(InputSanitizer::sanitize_amount(v), Ok(v));
    }

    #[test]
    fn sanitize_chain_id_zero_rejected() {
        assert_eq!(
            InputSanitizer::sanitize_chain_id(0),
            Err(ValidationError::InvalidChainId)
        );
    }

    #[test]
    fn sanitize_chain_id_valid_passthrough() {
        assert_eq!(InputSanitizer::sanitize_chain_id(1), Ok(1));
    }

    #[test]
    fn sanitize_destination_address_null_rejected() {
        let env = Env::default();
        let b = make_address_bytes(&env, 20, 0x00);
        assert!(InputSanitizer::sanitize_destination_address(&b).is_err());
    }

    #[test]
    fn sanitize_destination_address_valid() {
        let env = Env::default();
        assert!(InputSanitizer::sanitize_destination_address(&valid_dest(&env)).is_ok());
    }

    // ── CrossChainValidator ──────────────────────────────────────────────────

    #[test]
    fn cross_chain_message_valid() {
        let env = Env::default();
        let recipient = Address::generate(&env);
        assert!(
            CrossChainValidator::validate_cross_chain_message(&env, 1, 2, 1_000, &recipient)
                .is_ok()
        );
    }

    #[test]
    fn cross_chain_message_invalid_source_chain() {
        let env = Env::default();
        let recipient = Address::generate(&env);
        assert_eq!(
            CrossChainValidator::validate_cross_chain_message(&env, 0, 2, 1_000, &recipient),
            Err(ValidationError::InvalidChainId)
        );
    }

    #[test]
    fn cross_chain_message_invalid_amount() {
        let env = Env::default();
        let recipient = Address::generate(&env);
        assert_eq!(
            CrossChainValidator::validate_cross_chain_message(&env, 1, 2, 0, &recipient),
            Err(ValidationError::InvalidAmountRange)
        );
    }

    #[test]
    fn cross_chain_destination_data_valid() {
        let env = Env::default();
        assert!(CrossChainValidator::validate_destination_data(&env, 1, &valid_dest(&env)).is_ok());
    }

    #[test]
    fn cross_chain_destination_data_invalid_chain() {
        let env = Env::default();
        assert_eq!(
            CrossChainValidator::validate_destination_data(&env, 0, &valid_dest(&env)),
            Err(ValidationError::InvalidChainId)
        );
    }

    // ── BridgeValidator – chain registry check ───────────────────────────────

    #[test]
    fn bridge_out_rejects_unregistered_chain() {
        let env = Env::default();
        let from = Address::generate(&env);
        // Chain 42 is not in the supported-chains registry → must be rejected
        let result =
            BridgeValidator::validate_bridge_out(&env, &from, 1_000, 42, &valid_dest(&env));
        assert_eq!(
            result,
            Err(crate::errors::BridgeError::DestinationChainNotSupported)
        );
    }

    #[test]
    fn bridge_out_accepts_registered_chain() {
        let env = Env::default();
        let from = Address::generate(&env);
        // Register chain 1
        let mut chains: soroban_sdk::Map<u32, bool> = soroban_sdk::Map::new(&env);
        chains.set(1u32, true);
        env.storage()
            .instance()
            .set(&crate::storage::SUPPORTED_CHAINS, &chains);

        assert!(
            BridgeValidator::validate_bridge_out(&env, &from, 1_000, 1, &valid_dest(&env)).is_ok()
        );
    }

    #[test]
    fn bridge_out_rejects_zero_amount() {
        let env = Env::default();
        let from = Address::generate(&env);
        let mut chains: soroban_sdk::Map<u32, bool> = soroban_sdk::Map::new(&env);
        chains.set(1u32, true);
        env.storage()
            .instance()
            .set(&crate::storage::SUPPORTED_CHAINS, &chains);

        assert_eq!(
            BridgeValidator::validate_bridge_out(&env, &from, 0, 1, &valid_dest(&env)),
            Err(crate::errors::BridgeError::AmountMustBePositive)
        );
    }

    #[test]
    fn bridge_out_rejects_null_destination_address() {
        let env = Env::default();
        let from = Address::generate(&env);
        let mut chains: soroban_sdk::Map<u32, bool> = soroban_sdk::Map::new(&env);
        chains.set(1u32, true);
        env.storage()
            .instance()
            .set(&crate::storage::SUPPORTED_CHAINS, &chains);

        let null_addr = make_address_bytes(&env, 20, 0x00);
        assert_eq!(
            BridgeValidator::validate_bridge_out(&env, &from, 1_000, 1, &null_addr),
            Err(crate::errors::BridgeError::InvalidInput)
        );
    }

    #[test]
    fn bridge_out_rejects_chain_id_zero() {
        let env = Env::default();
        let from = Address::generate(&env);
        assert_eq!(
            BridgeValidator::validate_bridge_out(&env, &from, 1_000, 0, &valid_dest(&env)),
            Err(crate::errors::BridgeError::DestinationChainNotSupported)
        );
    }

    // ── RewardsValidator ─────────────────────────────────────────────────────

    #[test]
    fn rewards_valid_issuance() {
        let env = Env::default();
        let recipient = Address::generate(&env);
        let reward_type = String::from_str(&env, "completion");
        assert!(
            RewardsValidator::validate_reward_issuance(&env, &recipient, 500, &reward_type).is_ok()
        );
    }

    #[test]
    fn rewards_zero_amount_rejected() {
        let env = Env::default();
        let recipient = Address::generate(&env);
        let reward_type = String::from_str(&env, "completion");
        assert!(
            RewardsValidator::validate_reward_issuance(&env, &recipient, 0, &reward_type).is_err()
        );
    }

    // ── AtomicSwap address / amount / timelock validation ────────────────────

    #[test]
    fn atomic_swap_rejects_zero_initiator_amount() {
        use crate::atomic_swap::AtomicSwapManager;
        let env = Env::default();
        env.mock_all_auths();
        let initiator = Address::generate(&env);
        let counterparty = Address::generate(&env);
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);
        let hashlock = make_address_bytes(&env, 32, 0xAB);
        let result = AtomicSwapManager::initiate_swap(
            &env,
            initiator,
            token_a,
            0,
            counterparty,
            token_b,
            1_000,
            hashlock,
            crate::atomic_swap::MIN_TIMELOCK,
        );
        assert_eq!(
            result,
            Err(crate::errors::BridgeError::AmountMustBePositive)
        );
    }

    #[test]
    fn atomic_swap_rejects_zero_counterparty_amount() {
        use crate::atomic_swap::AtomicSwapManager;
        let env = Env::default();
        env.mock_all_auths();
        let initiator = Address::generate(&env);
        let counterparty = Address::generate(&env);
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);
        let hashlock = make_address_bytes(&env, 32, 0xAB);
        let result = AtomicSwapManager::initiate_swap(
            &env,
            initiator,
            token_a,
            1_000,
            counterparty,
            token_b,
            0,
            hashlock,
            crate::atomic_swap::MIN_TIMELOCK,
        );
        assert_eq!(
            result,
            Err(crate::errors::BridgeError::AmountMustBePositive)
        );
    }

    #[test]
    fn atomic_swap_rejects_timelock_below_min() {
        use crate::atomic_swap::AtomicSwapManager;
        let env = Env::default();
        env.mock_all_auths();
        let initiator = Address::generate(&env);
        let counterparty = Address::generate(&env);
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);
        let hashlock = make_address_bytes(&env, 32, 0xAB);
        let result = AtomicSwapManager::initiate_swap(
            &env,
            initiator,
            token_a,
            1_000,
            counterparty,
            token_b,
            1_000,
            hashlock,
            crate::atomic_swap::MIN_TIMELOCK - 1,
        );
        assert_eq!(result, Err(crate::errors::BridgeError::InvalidInput));
    }

    #[test]
    fn atomic_swap_rejects_timelock_above_max() {
        use crate::atomic_swap::AtomicSwapManager;
        let env = Env::default();
        env.mock_all_auths();
        let initiator = Address::generate(&env);
        let counterparty = Address::generate(&env);
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);
        let hashlock = make_address_bytes(&env, 32, 0xAB);
        let result = AtomicSwapManager::initiate_swap(
            &env,
            initiator,
            token_a,
            1_000,
            counterparty,
            token_b,
            1_000,
            hashlock,
            crate::atomic_swap::MAX_TIMELOCK + 1,
        );
        assert_eq!(result, Err(crate::errors::BridgeError::InvalidInput));
    }

    #[test]
    fn atomic_swap_rejects_same_initiator_and_counterparty() {
        use crate::atomic_swap::AtomicSwapManager;
        let env = Env::default();
        env.mock_all_auths();
        let party = Address::generate(&env);
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);
        let hashlock = make_address_bytes(&env, 32, 0xAB);
        let result = AtomicSwapManager::initiate_swap(
            &env,
            party.clone(),
            token_a,
            1_000,
            party,
            token_b,
            1_000,
            hashlock,
            crate::atomic_swap::MIN_TIMELOCK,
        );
        assert_eq!(result, Err(crate::errors::BridgeError::InvalidInput));
    }

    #[test]
    fn atomic_swap_rejects_invalid_hashlock_length() {
        use crate::atomic_swap::AtomicSwapManager;
        let env = Env::default();
        env.mock_all_auths();
        let initiator = Address::generate(&env);
        let counterparty = Address::generate(&env);
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);
        // 16 bytes instead of required 32
        let bad_hashlock = make_address_bytes(&env, 16, 0xAB);
        let result = AtomicSwapManager::initiate_swap(
            &env,
            initiator,
            token_a,
            1_000,
            counterparty,
            token_b,
            1_000,
            bad_hashlock,
            crate::atomic_swap::MIN_TIMELOCK,
        );
        assert_eq!(result, Err(crate::errors::BridgeError::InvalidHashlock));
    }

    // ── EscrowValidator ──────────────────────────────────────────────────────

    #[test]
    fn escrow_rejects_zero_amount() {
        use crate::types::EscrowSigner;
        let env = Env::default();
        let depositor = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        let signer = EscrowSigner {
            address: Address::generate(&env),
            weight: 1,
        };
        let mut signers = soroban_sdk::Vec::new(&env);
        signers.push_back(signer);
        let result = EscrowValidator::validate_create_escrow(
            &env,
            &depositor,
            &beneficiary,
            &token,
            0,
            &signers,
            1,
            None,
            None,
            &arbitrator,
        );
        assert_eq!(
            result,
            Err(crate::errors::EscrowError::AmountMustBePositive)
        );
    }

    #[test]
    fn escrow_rejects_empty_signers() {
        use crate::types::EscrowSigner;
        let env = Env::default();
        let depositor = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        let signers: soroban_sdk::Vec<EscrowSigner> = soroban_sdk::Vec::new(&env);
        let result = EscrowValidator::validate_create_escrow(
            &env,
            &depositor,
            &beneficiary,
            &token,
            1_000,
            &signers,
            1,
            None,
            None,
            &arbitrator,
        );
        assert_eq!(
            result,
            Err(crate::errors::EscrowError::AtLeastOneSignerRequired)
        );
    }

    #[test]
    fn escrow_rejects_threshold_exceeding_total_weight() {
        use crate::types::EscrowSigner;
        let env = Env::default();
        let depositor = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        let signer = EscrowSigner {
            address: Address::generate(&env),
            weight: 1,
        };
        let mut signers = soroban_sdk::Vec::new(&env);
        signers.push_back(signer);
        // threshold 5 > total weight 1
        let result = EscrowValidator::validate_create_escrow(
            &env,
            &depositor,
            &beneficiary,
            &token,
            1_000,
            &signers,
            5,
            None,
            None,
            &arbitrator,
        );
        assert_eq!(
            result,
            Err(crate::errors::EscrowError::InvalidSignerThreshold)
        );
    }

    #[test]
    fn escrow_rejects_duplicate_signers() {
        use crate::types::EscrowSigner;
        let env = Env::default();
        let depositor = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        let dup_addr = Address::generate(&env);
        let signer_a = EscrowSigner {
            address: dup_addr.clone(),
            weight: 1,
        };
        let signer_b = EscrowSigner {
            address: dup_addr,
            weight: 1,
        };
        let mut signers = soroban_sdk::Vec::new(&env);
        signers.push_back(signer_a);
        signers.push_back(signer_b);
        let result = EscrowValidator::validate_create_escrow(
            &env,
            &depositor,
            &beneficiary,
            &token,
            1_000,
            &signers,
            1,
            None,
            None,
            &arbitrator,
        );
        assert_eq!(result, Err(crate::errors::EscrowError::DuplicateSigner));
    }

    #[test]
    fn escrow_rejects_refund_before_release() {
        use crate::types::EscrowSigner;
        let env = Env::default();
        let depositor = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        let signer = EscrowSigner {
            address: Address::generate(&env),
            weight: 1,
        };
        let mut signers = soroban_sdk::Vec::new(&env);
        signers.push_back(signer);
        // refund_time (100) <= release_time (200) → invalid
        let result = EscrowValidator::validate_create_escrow(
            &env,
            &depositor,
            &beneficiary,
            &token,
            1_000,
            &signers,
            1,
            Some(200),
            Some(100),
            &arbitrator,
        );
        assert_eq!(
            result,
            Err(crate::errors::EscrowError::RefundTimeMustBeAfterReleaseTime)
        );
    }

    #[test]
    fn escrow_valid_parameters_accepted() {
        use crate::types::EscrowSigner;
        let env = Env::default();
        let depositor = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        let signer = EscrowSigner {
            address: Address::generate(&env),
            weight: 1,
        };
        let mut signers = soroban_sdk::Vec::new(&env);
        signers.push_back(signer);
        let result = EscrowValidator::validate_create_escrow(
            &env,
            &depositor,
            &beneficiary,
            &token,
            1_000,
            &signers,
            1,
            Some(100),
            Some(200),
            &arbitrator,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_rbac_admin_has_all_roles() {
        use crate::access_control::AccessControlManager;
        use crate::storage::ADMIN;
        use crate::types::AccessRole;
        let env = Env::default();
        let admin = Address::generate(&env);

        env.storage().instance().set(&ADMIN, &admin);

        assert!(AccessControlManager::has_role(
            &env,
            &admin,
            AccessRole::BridgeOperator
        ));
        assert!(AccessControlManager::has_role(
            &env,
            &admin,
            AccessRole::EmergencyManager
        ));
    }

    #[test]
    fn test_rbac_unauthorized_fails() {
        use crate::access_control::AccessControlManager;
        use crate::errors::BridgeError;
        use crate::types::AccessRole;
        let env = Env::default();
        let user = Address::generate(&env);

        let result = AccessControlManager::check_role(&env, &user, AccessRole::BridgeOperator);
        assert_eq!(result, Err(BridgeError::Unauthorized));
    }

    #[test]
    fn test_rbac_grant_role() {
        use crate::access_control::AccessControlManager;
        use crate::storage::ADMIN;
        use crate::types::AccessRole;
        let env = Env::default();
        let admin = Address::generate(&env);
        let user = Address::generate(&env);

        env.storage().instance().set(&ADMIN, &admin);
        admin.require_auth();

        assert!(AccessControlManager::grant_role(
            &env,
            admin,
            user.clone(),
            AccessRole::BridgeOperator
        )
        .is_ok());
        assert!(AccessControlManager::has_role(
            &env,
            &user,
            AccessRole::BridgeOperator
        ));
    }
}
