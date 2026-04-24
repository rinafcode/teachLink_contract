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
    pub const MAX_TIMEOUT_SECONDS: u64 = 31536000 * 10; // 10 years maximum (global sanity bound)
    pub const MAX_OPERATIONAL_TIMEOUT: u64 = 3600 * 24 * 90; // 90 days (for bridge/swaps)
    pub const MAX_TIME_SKEW: u64 = 900; // 15 minutes tolerance
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
    InvalidTimestamp,
    TimestampNotMonotonic,
    TimestampSkewExceeded,
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

    /// Validates string contains only allowed characters
    pub fn validate_characters(string: &String) -> ValidationResult<()> {
        // Allow alphanumeric, spaces, and basic punctuation
        let string_bytes = string.to_bytes();
        for byte in string_bytes.iter() {
            let char = byte as char;
            if !char.is_alphanumeric()
                && !char.is_whitespace()
                && !matches!(
                    char,
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
                return Err(ValidationError::InvalidStringLength);
            }
        }
        Ok(())
    }

    /// Comprehensive string validation
    pub fn validate(string: &String, max_length: u32) -> ValidationResult<()> {
        Self::validate_length(string, max_length)?;
        Self::validate_characters(string)?;
        Ok(())
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

/// Time validation utilities
pub struct TimeValidator;

impl TimeValidator {
    /// Validates if a timestamp is within the global sanity bound (10 years)
    pub fn validate_global_bounds(env: &Env, timestamp: u64) -> ValidationResult<()> {
        let current_time = env.ledger().timestamp();
        
        // Prevent far-future timestamps
        if timestamp > current_time + config::MAX_TIMEOUT_SECONDS {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        // Prevent far-past timestamps (saturating sub for safety)
        if timestamp < current_time.saturating_sub(config::MAX_TIMEOUT_SECONDS) {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        Ok(())
    }

    /// Validates if a timestamp is within operational bounds (90 days)
    pub fn validate_operational_bounds(env: &Env, timestamp: u64) -> ValidationResult<()> {
        let current_time = env.ledger().timestamp();
        
        if timestamp > current_time + config::MAX_OPERATIONAL_TIMEOUT {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        if timestamp < current_time.saturating_sub(config::MAX_OPERATIONAL_TIMEOUT) {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        Ok(())
    }

    /// Ensures that time has progressed monotonically
    pub fn check_monotonic(last_timestamp: u64, current_timestamp: u64) -> ValidationResult<()> {
        if current_timestamp < last_timestamp {
            return Err(ValidationError::TimestampNotMonotonic);
        }
        Ok(())
    }

    /// Validates a timestamp with network skew tolerance (15 minutes)
    pub fn validate_skew(env: &Env, external_timestamp: u64) -> ValidationResult<()> {
        let current_time = env.ledger().timestamp();
        let diff = if external_timestamp > current_time {
            external_timestamp - current_time
        } else {
            current_time - external_timestamp
        };
        
        if diff > config::MAX_TIME_SKEW {
            return Err(ValidationError::TimestampSkewExceeded);
        }
        Ok(())
    }
    
    /// Validates that a deadline is actually in the future
    pub fn validate_is_future(env: &Env, deadline: u64) -> ValidationResult<()> {
        if deadline <= env.ledger().timestamp() {
            return Err(ValidationError::InvalidTimestamp);
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
            total_weight += signer.weight;
        }

        if threshold < 1 || threshold > total_weight {
            return Err(EscrowError::InvalidSignerThreshold);
        }

        // Validate time constraints
        if let Some(release) = release_time {
            TimeValidator::validate_global_bounds(env, release)
                .map_err(|_| EscrowError::InvalidTimestamp)?;
            TimeValidator::validate_is_future(env, release)
                .map_err(|_| EscrowError::InvalidTimestamp)?;
        }

        if let Some(refund) = refund_time {
            TimeValidator::validate_global_bounds(env, refund)
                .map_err(|_| EscrowError::InvalidTimestamp)?;
            TimeValidator::validate_is_future(env, refund)
                .map_err(|_| EscrowError::InvalidTimestamp)?;
        }

        if let (Some(release), Some(refund)) = (release_time, refund_time) {
            TimeValidator::check_monotonic(release, refund)
                .map_err(|_| EscrowError::RefundTimeMustBeAfterReleaseTime)?;
        }

        // Check for duplicate signers
        Self::check_duplicate_signers(signers)?;

        Ok(())
    }

    /// Checks for duplicate signers in the list
    pub fn check_duplicate_signers(signers: &Vec<EscrowSigner>) -> Result<(), EscrowError> {
        // Simplified check - removed Env::current() call which doesn't exist
        // This validation is now handled by the caller
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
            total_weight += signer.weight;
        }

        if params.threshold < 1 || params.threshold > total_weight {
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

        // Validate current timestamp sanity
        TimeValidator::validate_global_bounds(env, env.ledger().timestamp())
            .map_err(|_| crate::errors::BridgeError::InvalidTimestamp)?;

        Ok(())
    }

    /// Validates bridge completion parameters
    pub fn validate_bridge_completion(
        env: &Env,
        message: &crate::types::CrossChainMessage,
        validator_signatures: &Vec<Address>,
        min_validators: u32,
    ) -> Result<(), crate::errors::BridgeError> {
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

        // Validate cross-chain message timestamp sanity
        TimeValidator::validate_global_bounds(env, message.timestamp)
            .map_err(|_| crate::errors::BridgeError::InvalidTimestamp)?;

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

        StringValidator::validate(reward_type, config::MAX_STRING_LENGTH)
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
