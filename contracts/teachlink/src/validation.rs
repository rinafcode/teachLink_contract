use crate::errors::EscrowError;
use crate::types::EscrowSigner;
use soroban_sdk::{Address, Bytes, Env, String, Vec};

/// Validation configuration constants
pub mod config {
    pub const MIN_AMOUNT: i128 = 1;
    pub const MAX_AMOUNT: i128 = i128::MAX / 2; // Prevent overflow
    pub const MIN_SIGNERS: u32 = 1;
    pub const MAX_SIGNERS: u32 = 100;
    pub const MIN_THRESHOLD: u32 = 1;
    pub const MAX_STRING_LENGTH: u32 = 256;
    pub const MIN_CHAIN_ID: u32 = 1;
    pub const MAX_CHAIN_ID: u32 = 999999;
    pub const MAX_ESCROW_DESCRIPTION_LENGTH: u32 = 1000;
    pub const MIN_TIMEOUT_SECONDS: u64 = 60; // 1 minute minimum
    pub const MAX_TIMEOUT_SECONDS: u64 = 31536000 * 10; // 10 years maximum
    pub const MAX_PAYLOAD_SIZE: u32 = 4096; // 4 KB max packet payload
    /// Bridge-specific amount bounds
    pub const MIN_BRIDGE_AMOUNT: i128 = 1;
    pub const MAX_BRIDGE_AMOUNT: i128 = 1_000_000_000_000_000_000; // 1e18
}

/// Validation errors
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ValidationError {
    InvalidAddressFormat,
    BlacklistedAddress,
    InvalidAmountRange,
    InvalidSignerCount,
    InvalidThreshold,
    InvalidStringLength,
    InvalidChainId,
    InvalidTimeout,
    EmptySignersList,
    DuplicateSigners,
    InvalidBytesLength,
    InvalidCrossChainData,
    /// String contains a character outside the allowed set.
    InvalidCharacters,
}

/// Result type for validation operations
pub type ValidationResult<T> = core::result::Result<T, ValidationError>;

/// Address validation utilities
pub struct AddressValidator;

impl AddressValidator {
    /// Validates address format and basic constraints
    pub fn validate_format(_env: &Env, _address: &Address) -> ValidationResult<()> {
        // In Soroban, Address format is validated at the SDK level
        // Additional validation can be added here if needed
        // For now, we'll just check that it's not a zero address
        Ok(())
    }

    /// Checks if address is blacklisted (placeholder for future implementation)
    pub fn check_blacklist(env: &Env, address: &Address) -> ValidationResult<()> {
        let blacklist_key = soroban_sdk::symbol_short!("blacklist");
        let blacklist: Vec<Address> = env
            .storage()
            .instance()
            .get(&blacklist_key)
            .unwrap_or_else(|| Vec::new(env));

        if blacklist.contains(address) {
            return Err(ValidationError::BlacklistedAddress);
        }
        Ok(())
    }

    /// Comprehensive address validation
    pub fn validate(env: &Env, address: &Address) -> ValidationResult<()> {
        Self::validate_format(env, address)?;
        Self::check_blacklist(env, address)?;
        Ok(())
    }
}

/// Numerical validation utilities
pub struct NumberValidator;

impl NumberValidator {
    /// Validates amount within allowed range
    pub fn validate_amount(amount: i128) -> ValidationResult<()> {
        if amount < config::MIN_AMOUNT {
            return Err(ValidationError::InvalidAmountRange);
        }
        if amount > config::MAX_AMOUNT {
            return Err(ValidationError::InvalidAmountRange);
        }
        Ok(())
    }

    /// Validates signer count
    #[allow(clippy::cast_possible_truncation)]
    pub fn validate_signer_count(count: usize) -> ValidationResult<()> {
        if count == 0 {
            return Err(ValidationError::EmptySignersList);
        }
        if (count as u32) < config::MIN_SIGNERS {
            return Err(ValidationError::InvalidSignerCount);
        }
        if (count as u32) > config::MAX_SIGNERS {
            return Err(ValidationError::InvalidSignerCount);
        }
        Ok(())
    }

    /// Validates threshold against signer count
    pub fn validate_threshold(threshold: u32, signer_count: u32) -> ValidationResult<()> {
        if threshold < config::MIN_THRESHOLD {
            return Err(ValidationError::InvalidThreshold);
        }
        if threshold > signer_count {
            return Err(ValidationError::InvalidThreshold);
        }
        Ok(())
    }

    /// Validates chain ID
    pub fn validate_chain_id(chain_id: u32) -> ValidationResult<()> {
        if !(config::MIN_CHAIN_ID..=config::MAX_CHAIN_ID).contains(&chain_id) {
            return Err(ValidationError::InvalidChainId);
        }
        Ok(())
    }

    /// Validates timeout duration
    pub fn validate_timeout(timeout_seconds: u64) -> ValidationResult<()> {
        if timeout_seconds < config::MIN_TIMEOUT_SECONDS {
            return Err(ValidationError::InvalidTimeout);
        }
        if timeout_seconds > config::MAX_TIMEOUT_SECONDS {
            return Err(ValidationError::InvalidTimeout);
        }
        Ok(())
    }
}

/// String validation utilities
pub struct StringValidator;

impl StringValidator {
    /// Validates string length
    pub fn validate_length(string: &String, max_length: u32) -> ValidationResult<()> {
        if string.is_empty() {
            return Err(ValidationError::InvalidStringLength);
        }
        if string.len() > max_length {
            return Err(ValidationError::InvalidStringLength);
        }
        Ok(())
    }

    /// Validates string contains only allowed characters (alphanumeric + safe punctuation).
    pub fn validate_characters(string: &String) -> ValidationResult<()> {
        let string_bytes = string.to_bytes();
        for byte in string_bytes.iter() {
            let ch = byte as char;
            if !ch.is_alphanumeric()
                && !ch.is_whitespace()
                && !matches!(
                    ch,
                    '-' | '_'
                        | '.'
                        | ','
                        | '!'
                        | '?'
                        | '@'
                        | '#'
                        | '$'
                        | '%'
                        | '&'
                        | '*'
                        | '+'
                        | '='
                        | ':'
                )
            {
                return Err(ValidationError::InvalidCharacters);
            }
        }
        Ok(())
    }

    /// Comprehensive string validation (length + character set).
    pub fn validate(string: &String, max_length: u32) -> ValidationResult<()> {
        Self::validate_length(string, max_length)?;
        Self::validate_characters(string)?;
        Ok(())
    }

    /// Validate after stripping ASCII whitespace from both ends.
    ///
    /// Returns `InvalidStringLength` if the trimmed result is empty or exceeds
    /// `max_length`; returns `InvalidCharacters` if forbidden bytes are present.
    pub fn trim_and_validate(
        env: &Env,
        string: &String,
        max_length: u32,
    ) -> ValidationResult<String> {
        let bytes = string.to_bytes();
        let len = bytes.len();

        if len == 0 {
            return Err(ValidationError::InvalidStringLength);
        }

        // Find first non-whitespace index.
        let mut start = 0u32;
        loop {
            if start >= len {
                return Err(ValidationError::InvalidStringLength); // entirely whitespace
            }
            if !(bytes.get(start).unwrap() as char).is_ascii_whitespace() {
                break;
            }
            start += 1;
        }

        // Find last non-whitespace index.
        let mut end = len - 1;
        while end > start && (bytes.get(end).unwrap() as char).is_ascii_whitespace() {
            end -= 1;
        }

        // Build trimmed Bytes by copying the [start, end] range.
        let mut trimmed_bytes = Bytes::new(env);
        let mut i = start;
        while i <= end {
            trimmed_bytes.push_back(bytes.get(i).unwrap());
            i += 1;
        }

        let trimmed = String::from_bytes(env, &trimmed_bytes);
        Self::validate(&trimmed, max_length)?;
        Ok(trimmed)
    }
}

/// Bytes validation utilities
pub struct BytesValidator;

impl BytesValidator {
    /// Validates bytes for cross-chain addresses
    pub fn validate_cross_chain_address(bytes: &Bytes) -> ValidationResult<()> {
        // Most blockchain addresses are 20-32 bytes
        if bytes.len() < 20 || bytes.len() > 32 {
            return Err(ValidationError::InvalidBytesLength);
        }
        // Reject all-zero addresses (null address)
        let all_zero = bytes.iter().all(|b| b == 0);
        if all_zero {
            return Err(ValidationError::InvalidAddressFormat);
        }
        Ok(())
    }

    /// Validates bytes for general use
    pub fn validate_length(bytes: &Bytes, min_len: u32, max_len: u32) -> ValidationResult<()> {
        if bytes.len() < min_len || bytes.len() > max_len {
            return Err(ValidationError::InvalidBytesLength);
        }
        Ok(())
    }

    /// Validates packet payload (non-empty, within size limit)
    pub fn validate_payload(bytes: &Bytes) -> ValidationResult<()> {
        if bytes.is_empty() {
            return Err(ValidationError::InvalidCrossChainData);
        }
        if bytes.len() > config::MAX_PAYLOAD_SIZE {
            return Err(ValidationError::InvalidBytesLength);
        }
        Ok(())
    }
}

/// Cross-chain data validation utilities
pub struct CrossChainValidator;

impl CrossChainValidator {
    /// Validates destination chain data
    pub fn validate_destination_data(
        _env: &Env,
        chain_id: u32,
        destination_address: &Bytes,
    ) -> ValidationResult<()> {
        NumberValidator::validate_chain_id(chain_id)?;
        BytesValidator::validate_cross_chain_address(destination_address)?;
        Ok(())
    }

    /// Validates cross-chain message structure
    pub fn validate_cross_chain_message(
        env: &Env,
        source_chain: u32,
        destination_chain: u32,
        amount: i128,
        recipient: &Address,
    ) -> ValidationResult<()> {
        NumberValidator::validate_chain_id(source_chain)?;
        NumberValidator::validate_chain_id(destination_chain)?;
        NumberValidator::validate_amount(amount)?;
        AddressValidator::validate(env, recipient)?;
        Ok(())
    }
}

/// Escrow-specific validation utilities
pub struct EscrowValidator;

impl EscrowValidator {
    /// Validates escrow creation parameters
    pub fn validate_create_escrow(
        env: &Env,
        depositor: &Address,
        beneficiary: &Address,
        token: &Address,
        amount: i128,
        signers: &Vec<EscrowSigner>,
        threshold: u32,
        release_time: Option<u64>,
        refund_time: Option<u64>,
        arbitrator: &Address,
    ) -> Result<(), EscrowError> {
        // Validate addresses
        AddressValidator::validate(env, depositor)
            .map_err(|_| EscrowError::AmountMustBePositive)?;
        AddressValidator::validate(env, beneficiary)
            .map_err(|_| EscrowError::AmountMustBePositive)?;
        AddressValidator::validate(env, token).map_err(|_| EscrowError::AmountMustBePositive)?;
        AddressValidator::validate(env, arbitrator)
            .map_err(|_| EscrowError::AmountMustBePositive)?;

        // Validate amount
        NumberValidator::validate_amount(amount).map_err(|_| EscrowError::AmountMustBePositive)?;

        // Validate signers
        NumberValidator::validate_signer_count(signers.len() as usize)
            .map_err(|_| EscrowError::AtLeastOneSignerRequired)?;

        let mut total_weight: u32 = 0;
        for signer in signers.iter() {
            if signer.weight == 0 {
                return Err(EscrowError::InvalidSignerThreshold);
            }
            total_weight += signer.weight;
        }

        if threshold < 1 || total_weight == 0 || threshold > total_weight {
            return Err(EscrowError::InvalidSignerThreshold);
        }

        // Validate time constraints
        if let (Some(release), Some(refund)) = (release_time, refund_time) {
            if refund <= release {
                return Err(EscrowError::RefundTimeMustBeAfterReleaseTime);
            }
        }

        // Check for duplicate signers
        Self::check_duplicate_signers(signers)?;

        Ok(())
    }

    /// Checks for duplicate signers in the list
    pub fn check_duplicate_signers(signers: &Vec<EscrowSigner>) -> Result<(), EscrowError> {
        let len = signers.len();
        for i in 0..len {
            for j in (i + 1)..len {
                if signers.get(i).unwrap().address == signers.get(j).unwrap().address {
                    return Err(EscrowError::DuplicateSigner);
                }
            }
        }
        Ok(())
    }

    /// Validates EscrowParameters struct (refactored from individual parameters)
    pub fn validate_escrow_parameters(
        env: &Env,
        params: &crate::types::EscrowParameters,
    ) -> Result<(), EscrowError> {
        // Validate addresses
        AddressValidator::validate(env, &params.depositor)
            .map_err(|_| EscrowError::InvalidBeneficiary)?;
        AddressValidator::validate(env, &params.beneficiary)
            .map_err(|_| EscrowError::InvalidBeneficiary)?;
        AddressValidator::validate(env, &params.token).map_err(|_| EscrowError::InvalidToken)?;
        AddressValidator::validate(env, &params.arbitrator)
            .map_err(|_| EscrowError::InvalidArbitrator)?;

        // Validate amount
        NumberValidator::validate_amount(params.amount)
            .map_err(|_| EscrowError::AmountMustBePositive)?;

        // Validate signers
        NumberValidator::validate_signer_count(params.signers.len() as usize)
            .map_err(|_| EscrowError::AtLeastOneSignerRequired)?;

        // Validate threshold against total signer weight
        let mut total_weight: u32 = 0;
        for signer in params.signers.iter() {
            if signer.weight == 0 {
                return Err(EscrowError::InvalidSignerThreshold);
            }
            total_weight += signer.weight;
        }

        if params.threshold < 1 || total_weight == 0 || params.threshold > total_weight {
            return Err(EscrowError::InvalidSignerThreshold);
        }

        // Validate time constraints
        if let (Some(release), Some(refund)) = (params.release_time, params.refund_time) {
            if refund <= release {
                return Err(EscrowError::RefundTimeMustBeAfterReleaseTime);
            }
        }

        // Check for duplicate signers
        Self::check_duplicate_signers(&params.signers)?;

        // Additional validation: depositor must be different from beneficiary
        if params.depositor == params.beneficiary {
            return Err(EscrowError::DepositorCannotBeBeneficiary);
        }

        Ok(())
    }

    /// Validates escrow release conditions
    pub fn validate_release_conditions(
        escrow: &crate::types::Escrow,
        caller: &Address,
        current_time: u64,
    ) -> Result<(), EscrowError> {
        if escrow.status != crate::types::EscrowStatus::Pending {
            return Err(EscrowError::EscrowNotPending);
        }

        if !Self::is_authorized_caller(escrow, caller) {
            return Err(EscrowError::CallerNotAuthorized);
        }

        if escrow.approval_count < escrow.threshold {
            return Err(EscrowError::InsufficientApprovals);
        }

        if let Some(release_time) = escrow.release_time {
            if current_time < release_time {
                return Err(EscrowError::ReleaseTimeNotReached);
            }
        }

        Ok(())
    }

    /// Checks if caller is authorized to release escrow
    pub fn is_authorized_caller(escrow: &crate::types::Escrow, caller: &Address) -> bool {
        if caller.clone() == escrow.depositor || caller.clone() == escrow.beneficiary {
            return true;
        }

        // Check if caller is a signer
        for signer in escrow.signers.iter() {
            if signer.address == caller.clone() {
                return true;
            }
        }

        false
    }
}

/// Parameter sanitization utilities
pub struct InputSanitizer;

impl InputSanitizer {
    /// Clamps an i128 amount to the valid bridge range, returning an error if out of bounds.
    pub fn sanitize_amount(amount: i128) -> ValidationResult<i128> {
        if amount < config::MIN_BRIDGE_AMOUNT || amount > config::MAX_BRIDGE_AMOUNT {
            return Err(ValidationError::InvalidAmountRange);
        }
        Ok(amount)
    }

    /// Validates and returns a chain ID only if it is within the numeric range.
    pub fn sanitize_chain_id(chain_id: u32) -> ValidationResult<u32> {
        NumberValidator::validate_chain_id(chain_id)?;
        Ok(chain_id)
    }

    /// Validates destination bytes are non-empty, non-zero, and within address length bounds.
    pub fn sanitize_destination_address(bytes: &Bytes) -> ValidationResult<()> {
        BytesValidator::validate_cross_chain_address(bytes)
    }

    /// Trim whitespace from `string`, then validate length and character set.
    ///
    /// Use this instead of calling `StringValidator::validate` directly when the
    /// input originates from an untrusted user (description fields, reason strings,
    /// reward-type labels, etc.) so that leading/trailing whitespace is always
    /// stripped before the length cap is applied.
    pub fn sanitize_string(
        env: &Env,
        string: &String,
        max_length: u32,
    ) -> ValidationResult<String> {
        StringValidator::trim_and_validate(env, string, max_length)
    }
}

/// Bridge-specific validation utilities
pub struct BridgeValidator;

impl BridgeValidator {
    /// Validates bridge out parameters.
    /// Pass `supported_chains` to also verify the chain is registered; pass `None` to skip.
    pub fn validate_bridge_out(
        env: &Env,
        from: &Address,
        amount: i128,
        destination_chain: u32,
        destination_address: &Bytes,
    ) -> Result<(), crate::errors::BridgeError> {
        // Validate sender address
        AddressValidator::validate(env, from)
            .map_err(|_| crate::errors::BridgeError::InvalidInput)?;

        // Validate amount within bridge-specific bounds
        InputSanitizer::sanitize_amount(amount)
            .map_err(|_| crate::errors::BridgeError::AmountMustBePositive)?;

        // Validate chain ID numeric range
        InputSanitizer::sanitize_chain_id(destination_chain)
            .map_err(|_| crate::errors::BridgeError::DestinationChainNotSupported)?;

        // Validate chain is registered as supported
        let supported: soroban_sdk::Map<u32, bool> = env
            .storage()
            .instance()
            .get(&crate::storage::SUPPORTED_CHAINS)
            .unwrap_or_else(|| soroban_sdk::Map::new(env));
        if !supported.get(destination_chain).unwrap_or(false) {
            return Err(crate::errors::BridgeError::DestinationChainNotSupported);
        }

        // Validate destination address format (length + non-zero)
        InputSanitizer::sanitize_destination_address(destination_address)
            .map_err(|_| crate::errors::BridgeError::InvalidInput)?;

        Ok(())
    }

    /// Validates bridge completion parameters
    pub fn validate_bridge_completion(
        env: &Env,
        message: &crate::types::CrossChainMessage,
        validator_signatures: &Vec<Address>,
        min_validators: u32,
    ) -> Result<(), crate::errors::BridgeError> {
        // Enforce maximum validator count to prevent DoS via unbounded loop
        #[allow(clippy::cast_possible_truncation)]
        crate::dos_protection::check_batch_size(
            validator_signatures.len() as u32,
            crate::dos_protection::MAX_VALIDATORS_PER_COMPLETION,
        )?;

        // Validate validator signatures count
        if validator_signatures.len() < min_validators {
            return Err(crate::errors::BridgeError::InsufficientValidatorSignatures);
        }

        // Validate cross-chain message
        CrossChainValidator::validate_cross_chain_message(
            env,
            message.source_chain,
            message.destination_chain,
            message.amount,
            &message.recipient,
        )
        .map_err(|_| crate::errors::BridgeError::InvalidInput)?;

        Ok(())
    }
}

/// Rewards-specific validation utilities
pub struct RewardsValidator;

impl RewardsValidator {
    /// Validates reward issuance parameters
    pub fn validate_reward_issuance(
        env: &Env,
        recipient: &Address,
        amount: i128,
        reward_type: &String,
    ) -> Result<(), crate::errors::RewardsError> {
        AddressValidator::validate(env, recipient)
            .map_err(|_| crate::errors::RewardsError::AmountMustBePositive)?;

        NumberValidator::validate_amount(amount)
            .map_err(|_| crate::errors::RewardsError::AmountMustBePositive)?;

        InputSanitizer::sanitize_string(env, reward_type, config::MAX_STRING_LENGTH)
            .map_err(|_| crate::errors::RewardsError::AmountMustBePositive)?;

        Ok(())
    }

    /// Validates reward pool funding
    pub fn validate_pool_funding(
        env: &Env,
        funder: &Address,
        amount: i128,
    ) -> Result<(), crate::errors::RewardsError> {
        AddressValidator::validate(env, funder)
            .map_err(|_| crate::errors::RewardsError::AmountMustBePositive)?;

        NumberValidator::validate_amount(amount)
            .map_err(|_| crate::errors::RewardsError::AmountMustBePositive)?;

        Ok(())
    }
}
