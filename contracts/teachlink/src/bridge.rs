use crate::errors::BridgeError;
use crate::events::{
    BridgeCancelledEvent, BridgeCompletedEvent, BridgeFailedEvent, BridgeFeeUpdatedEvent,
    BridgeInitiatedEvent, BridgeRetryEvent, ChainSupportedEvent, ChainUnsupportedEvent,
    DepositEvent, FeeRecipientUpdatedEvent, MinValidatorsUpdatedEvent, ReleaseEvent,
    ValidatorAddedEvent, ValidatorRemovedEvent,
};
use crate::storage::{
    ADMIN, BRIDGE_FAILURES, BRIDGE_FEE, BRIDGE_LAST_RETRY, BRIDGE_RETRY_COUNTS, BRIDGE_TXS,
    FEE_RECIPIENT, MIN_VALIDATORS, NONCE, SUPPORTED_CHAINS, TOKEN, VALIDATORS,
};
use crate::types::{BridgeTransaction, CrossChainMessage};
use crate::validation::BridgeValidator;
use soroban_sdk::{symbol_short, vec, Address, Bytes, Env, IntoVal, Map, Vec};

const BRIDGE_TIMEOUT_SECONDS: u64 = 604_800;
const MAX_BRIDGE_RETRY_ATTEMPTS: u32 = 5;
const BRIDGE_RETRY_DELAY_BASE_SECONDS: u64 = 300;

pub struct Bridge;

impl Bridge {
    /// Initialize the bridge contract
    /// - token: Address of the TeachLink token contract
    /// - admin: Address of the bridge administrator
    /// - min_validators: Minimum number of validators required for cross-chain verification
    pub fn initialize(
        env: &Env,
        token: Address,
        admin: Address,
        min_validators: u32,
        fee_recipient: Address,
    ) -> Result<(), BridgeError> {
        // Check if already initialized
        if env.storage().instance().has(&TOKEN) {
            return Err(BridgeError::AlreadyInitialized);
        }

        if min_validators == 0 {
            return Err(BridgeError::MinimumValidatorsMustBeAtLeastOne);
        }

        env.storage().instance().set(&TOKEN, &token);
        env.storage().instance().set(&ADMIN, &admin);
        env.storage()
            .instance()
            .set(&MIN_VALIDATORS, &min_validators);
        env.storage().instance().set(&NONCE, &0u64);
        env.storage().instance().set(&FEE_RECIPIENT, &fee_recipient);
        env.storage().instance().set(&BRIDGE_FEE, &0i128); // Default no fee

        // Initialize empty validators map
        let validators: Map<Address, bool> = Map::new(env);
        env.storage().instance().set(&VALIDATORS, &validators);

        // Initialize empty supported chains map
        let chains: Map<u32, bool> = Map::new(env);
        env.storage().instance().set(&SUPPORTED_CHAINS, &chains);

        Ok(())
    }

    /// Bridge tokens out to another chain (lock/burn tokens on Stellar)
    /// - amount: Amount of tokens to bridge
    /// - destination_chain: Chain ID of the destination blockchain
    /// - destination_address: Address on the destination chain (encoded as bytes)
    pub fn bridge_out(
        env: &Env,
        from: Address,
        amount: i128,
        destination_chain: u32,
        destination_address: soroban_sdk::Bytes,
    ) -> Result<u64, BridgeError> {
        from.require_auth();

        // Validate all input parameters (includes supported-chain registry check)
        BridgeValidator::validate_bridge_out(
            env,
            &from,
            amount,
            destination_chain,
            &destination_address,
        )?;

        // Get token address
        let token: Address = env.storage().instance().get(&TOKEN).unwrap();

        // Transfer tokens from user to bridge (locking them)
        env.invoke_contract::<()>(
            &token,
            &symbol_short!("transfer"),
            vec![
                env,
                from.into_val(env),
                env.current_contract_address().into_val(env),
                amount.into_val(env),
            ],
        );

        // Apply bridge fee if configured
        let fee: i128 = env.storage().instance().get(&BRIDGE_FEE).unwrap_or(0i128);
        let fee_recipient: Address = env.storage().instance().get(&FEE_RECIPIENT).unwrap();
        let amount_after_fee = if fee > 0 && fee < amount {
            env.invoke_contract::<()>(
                &token,
                &symbol_short!("transfer"),
                vec![
                    env,
                    env.current_contract_address().into_val(env),
                    fee_recipient.into_val(env),
                    fee.into_val(env),
                ],
            );
            amount - fee
        } else {
            amount
        };

        // Generate nonce for this transaction
        let mut nonce: u64 = env.storage().instance().get(&NONCE).unwrap_or(0u64);
        nonce += 1;
        env.storage().instance().set(&NONCE, &nonce);

        // Create bridge transaction record
        let bridge_tx = BridgeTransaction {
            nonce,
            token: token.clone(),
            amount: amount_after_fee,
            recipient: from.clone(),
            destination_chain,
            destination_address: destination_address.clone(),
            timestamp: env.ledger().timestamp(),
        };

        // Store bridge transaction
        let mut bridge_txs: Map<u64, BridgeTransaction> = env
            .storage()
            .instance()
            .get(&BRIDGE_TXS)
            .unwrap_or_else(|| Map::new(env));
        bridge_txs.set(nonce, bridge_tx.clone());
        env.storage().instance().set(&BRIDGE_TXS, &bridge_txs);

        let mut retry_counts: Map<u64, u32> = env
            .storage()
            .instance()
            .get(&BRIDGE_RETRY_COUNTS)
            .unwrap_or_else(|| Map::new(env));
        retry_counts.set(nonce, 0);
        env.storage()
            .instance()
            .set(&BRIDGE_RETRY_COUNTS, &retry_counts);

        let mut last_retry: Map<u64, u64> = env
            .storage()
            .instance()
            .get(&BRIDGE_LAST_RETRY)
            .unwrap_or_else(|| Map::new(env));
        last_retry.set(nonce, env.ledger().timestamp());
        env.storage()
            .instance()
            .set(&BRIDGE_LAST_RETRY, &last_retry);

        // Emit events
        BridgeInitiatedEvent {
            nonce,
            transaction: bridge_tx.clone(),
        }
        .publish(env);

        DepositEvent {
            nonce,
            from,
            amount: amount_after_fee,
            destination_chain,
            destination_address,
        }
        .publish(env);

        Ok(nonce)
    }

    /// Complete a bridge transaction (mint/release tokens on Stellar)
    /// This is called by validators after verifying the transaction on the source chain
    /// - message: Cross-chain message containing transaction details
    /// - validator_signatures: List of validator addresses that have verified this transaction
    pub fn complete_bridge(
        env: &Env,
        message: CrossChainMessage,
        validator_signatures: Vec<Address>,
    ) -> Result<(), BridgeError> {
        // Validate all input parameters
        let min_validators: u32 = env.storage().instance().get(&MIN_VALIDATORS).unwrap();
        BridgeValidator::validate_bridge_completion(
            env,
            &message,
            &validator_signatures,
            min_validators,
        )?;

        // Verify all signatures are from valid validators
        let validators: Map<Address, bool> = env.storage().instance().get(&VALIDATORS).unwrap();
        for validator in validator_signatures.iter() {
            if !validators.get(validator.clone()).unwrap_or(false) {
                return Err(BridgeError::InvalidValidatorSignature);
            }
        }

        // Check for duplicate nonce to prevent replay attacks
        let mut processed_nonces: Map<u64, bool> = env
            .storage()
            .persistent()
            .get(&NONCE)
            .unwrap_or_else(|| Map::new(env));
        if processed_nonces.get(message.nonce).unwrap_or(false) {
            return Err(BridgeError::NonceAlreadyProcessed);
        }
        processed_nonces.set(message.nonce, true);
        env.storage().persistent().set(&NONCE, &processed_nonces);

        // Get token address
        let token: Address = env.storage().instance().get(&TOKEN).unwrap();

        // Verify token matches
        if message.token != token {
            return Err(BridgeError::TokenMismatch);
        }

        // Mint/release tokens to recipient
        env.invoke_contract::<()>(
            &token,
            &symbol_short!("mint"),
            vec![
                env,
                message.recipient.into_val(env),
                message.amount.into_val(env),
            ],
        );

        // Emit events
        BridgeCompletedEvent {
            nonce: message.nonce,
            message: message.clone(),
        }
        .publish(env);

        ReleaseEvent {
            nonce: message.nonce,
            recipient: message.recipient.clone(),
            amount: message.amount,
            source_chain: message.source_chain,
        }
        .publish(env);

        let mut bridge_txs: Map<u64, BridgeTransaction> = env
            .storage()
            .instance()
            .get(&BRIDGE_TXS)
            .unwrap_or_else(|| Map::new(env));
        if bridge_txs.contains_key(message.nonce) {
            bridge_txs.remove(message.nonce);
            env.storage().instance().set(&BRIDGE_TXS, &bridge_txs);
        }

        Self::clear_bridge_retry_metadata(env, message.nonce);

        Ok(())
    }

    pub fn mark_bridge_failed(env: &Env, nonce: u64, reason: Bytes) -> Result<(), BridgeError> {
        if reason.is_empty() {
            return Err(BridgeError::InvalidInput);
        }

        let bridge_txs: Map<u64, BridgeTransaction> = env
            .storage()
            .instance()
            .get(&BRIDGE_TXS)
            .unwrap_or_else(|| Map::new(env));
        if !bridge_txs.contains_key(nonce) {
            return Err(BridgeError::BridgeTransactionNotFound);
        }

        let mut failures: Map<u64, Bytes> = env
            .storage()
            .instance()
            .get(&BRIDGE_FAILURES)
            .unwrap_or_else(|| Map::new(env));
        failures.set(nonce, reason.clone());
        env.storage().instance().set(&BRIDGE_FAILURES, &failures);

        // Emit event
        BridgeFailedEvent {
            nonce,
            reason: reason.clone(),
            failed_at: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    pub fn retry_bridge(env: &Env, nonce: u64) -> Result<u32, BridgeError> {
        let bridge_txs: Map<u64, BridgeTransaction> = env
            .storage()
            .instance()
            .get(&BRIDGE_TXS)
            .unwrap_or_else(|| Map::new(env));
        let bridge_tx = bridge_txs
            .get(nonce)
            .ok_or(BridgeError::BridgeTransactionNotFound)?;

        let current_time = env.ledger().timestamp();
        if current_time.saturating_sub(bridge_tx.timestamp) >= BRIDGE_TIMEOUT_SECONDS {
            return Err(BridgeError::PacketTimeout);
        }

        let mut retry_counts: Map<u64, u32> = env
            .storage()
            .instance()
            .get(&BRIDGE_RETRY_COUNTS)
            .unwrap_or_else(|| Map::new(env));
        let retry_count = retry_counts.get(nonce).unwrap_or(0);
        if retry_count >= MAX_BRIDGE_RETRY_ATTEMPTS {
            return Err(BridgeError::RetryLimitExceeded);
        }

        let mut last_retry: Map<u64, u64> = env
            .storage()
            .instance()
            .get(&BRIDGE_LAST_RETRY)
            .unwrap_or_else(|| Map::new(env));
        let last_retry_at = last_retry.get(nonce).unwrap_or(bridge_tx.timestamp);

        let backoff_multiplier = 1u64 << retry_count;
        let retry_delay = BRIDGE_RETRY_DELAY_BASE_SECONDS.saturating_mul(backoff_multiplier);
        let next_allowed_retry = last_retry_at.saturating_add(retry_delay);

        if current_time < next_allowed_retry {
            return Err(BridgeError::RetryBackoffActive);
        }

        let updated_retry_count = retry_count + 1;
        retry_counts.set(nonce, updated_retry_count);
        env.storage()
            .instance()
            .set(&BRIDGE_RETRY_COUNTS, &retry_counts);
        last_retry.set(nonce, current_time);
        env.storage()
            .instance()
            .set(&BRIDGE_LAST_RETRY, &last_retry);

        let mut failures: Map<u64, Bytes> = env
            .storage()
            .instance()
            .get(&BRIDGE_FAILURES)
            .unwrap_or_else(|| Map::new(env));
        failures.remove(nonce);
        env.storage().instance().set(&BRIDGE_FAILURES, &failures);

        // Emit event
        BridgeRetryEvent {
            nonce,
            retry_count: updated_retry_count,
            retried_at: current_time,
        }
        .publish(env);

        Ok(updated_retry_count)
    }

    /// Cancel a bridge transaction and refund locked tokens
    /// Only callable after a timeout period
    /// - nonce: The nonce of the bridge transaction to cancel
    pub fn cancel_bridge(env: &Env, nonce: u64) -> Result<(), BridgeError> {
        // Get bridge transaction
        let bridge_txs: Map<u64, BridgeTransaction> = env
            .storage()
            .instance()
            .get(&BRIDGE_TXS)
            .unwrap_or_else(|| Map::new(env));
        let bridge_tx = bridge_txs
            .get(nonce)
            .ok_or(BridgeError::BridgeTransactionNotFound)?;

        let failures: Map<u64, Bytes> = env
            .storage()
            .instance()
            .get(&BRIDGE_FAILURES)
            .unwrap_or_else(|| Map::new(env));

        // Allow refunds for timed-out or explicitly failed transactions
        let elapsed = env.ledger().timestamp().saturating_sub(bridge_tx.timestamp);
        if elapsed < BRIDGE_TIMEOUT_SECONDS && !failures.contains_key(nonce) {
            return Err(BridgeError::TimeoutNotReached);
        }

        // Get token address
        let token: Address = env.storage().instance().get(&TOKEN).unwrap();

        // Refund tokens to original recipient
        env.invoke_contract::<()>(
            &token,
            &symbol_short!("transfer"),
            vec![
                env,
                env.current_contract_address().into_val(env),
                bridge_tx.recipient.into_val(env),
                bridge_tx.amount.into_val(env),
            ],
        );

        // Remove from bridge transactions
        let mut updated_txs = bridge_txs;
        updated_txs.remove(nonce);
        env.storage().instance().set(&BRIDGE_TXS, &updated_txs);

        Self::clear_bridge_retry_metadata(env, nonce);

        // Emit event
        BridgeCancelledEvent {
            nonce,
            refunded_to: bridge_tx.recipient.clone(),
            amount: bridge_tx.amount,
            cancelled_at: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    pub fn refund_bridge_transaction(env: &Env, nonce: u64) -> Result<(), BridgeError> {
        Self::cancel_bridge(env, nonce)
    }

    fn clear_bridge_retry_metadata(env: &Env, nonce: u64) {
        let mut retry_counts: Map<u64, u32> = env
            .storage()
            .instance()
            .get(&BRIDGE_RETRY_COUNTS)
            .unwrap_or_else(|| Map::new(env));
        retry_counts.remove(nonce);
        env.storage()
            .instance()
            .set(&BRIDGE_RETRY_COUNTS, &retry_counts);

        let mut last_retry: Map<u64, u64> = env
            .storage()
            .instance()
            .get(&BRIDGE_LAST_RETRY)
            .unwrap_or_else(|| Map::new(env));
        last_retry.remove(nonce);
        env.storage()
            .instance()
            .set(&BRIDGE_LAST_RETRY, &last_retry);

        let mut failures: Map<u64, Bytes> = env
            .storage()
            .instance()
            .get(&BRIDGE_FAILURES)
            .unwrap_or_else(|| Map::new(env));
        failures.remove(nonce);
        env.storage().instance().set(&BRIDGE_FAILURES, &failures);
    }

    // ========== Admin Functions ==========

    /// Add a validator (admin only)
    #[allow(clippy::unnecessary_wraps)]
    pub fn add_validator(env: &Env, validator: Address) -> Result<(), BridgeError> {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        let mut validators: Map<Address, bool> = env.storage().instance().get(&VALIDATORS).unwrap();
        validators.set(validator.clone(), true);
        env.storage().instance().set(&VALIDATORS, &validators);

        // Emit event
        ValidatorAddedEvent {
            validator: validator.clone(),
            added_by: admin.clone(),
            added_at: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    /// Remove a validator (admin only)
    #[allow(clippy::unnecessary_wraps)]
    pub fn remove_validator(env: &Env, validator: Address) -> Result<(), BridgeError> {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        let mut validators: Map<Address, bool> = env.storage().instance().get(&VALIDATORS).unwrap();
        validators.set(validator.clone(), false);
        env.storage().instance().set(&VALIDATORS, &validators);

        // Emit event
        ValidatorRemovedEvent {
            validator: validator.clone(),
            removed_by: admin.clone(),
            removed_at: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    /// Add a supported destination chain (admin only)
    #[allow(clippy::unnecessary_wraps)]
    pub fn add_supported_chain(env: &Env, chain_id: u32) -> Result<(), BridgeError> {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        let mut chains: Map<u32, bool> = env.storage().instance().get(&SUPPORTED_CHAINS).unwrap();
        chains.set(chain_id, true);
        env.storage().instance().set(&SUPPORTED_CHAINS, &chains);

        // Emit event
        ChainSupportedEvent {
            chain_id,
            added_by: admin.clone(),
            added_at: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    /// Remove a supported destination chain (admin only)
    #[allow(clippy::unnecessary_wraps)]
    pub fn remove_supported_chain(env: &Env, chain_id: u32) -> Result<(), BridgeError> {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        let mut chains: Map<u32, bool> = env.storage().instance().get(&SUPPORTED_CHAINS).unwrap();
        chains.set(chain_id, false);
        env.storage().instance().set(&SUPPORTED_CHAINS, &chains);

        // Emit event
        ChainUnsupportedEvent {
            chain_id,
            removed_by: admin.clone(),
            removed_at: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    /// Set bridge fee (admin only)
    pub fn set_bridge_fee(env: &Env, fee: i128) -> Result<(), BridgeError> {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        if fee < 0 {
            return Err(BridgeError::FeeCannotBeNegative);
        }

        let old_fee: i128 = env.storage().instance().get(&BRIDGE_FEE).unwrap_or(0i128);
        env.storage().instance().set(&BRIDGE_FEE, &fee);

        // Emit event
        BridgeFeeUpdatedEvent {
            old_fee,
            new_fee: fee,
            updated_by: admin.clone(),
            updated_at: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    /// Set fee recipient (admin only)
    #[allow(clippy::unnecessary_wraps)]
    pub fn set_fee_recipient(env: &Env, fee_recipient: Address) -> Result<(), BridgeError> {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        let old_recipient: Address = env.storage().instance().get(&FEE_RECIPIENT).unwrap();
        env.storage().instance().set(&FEE_RECIPIENT, &fee_recipient);

        // Emit event
        FeeRecipientUpdatedEvent {
            old_recipient,
            new_recipient: fee_recipient,
            updated_by: admin.clone(),
            updated_at: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    /// Set minimum validators (admin only)
    pub fn set_min_validators(env: &Env, min_validators: u32) -> Result<(), BridgeError> {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        if min_validators == 0 {
            return Err(BridgeError::MinimumValidatorsMustBeAtLeastOne);
        }

        let old_min: u32 = env.storage().instance().get(&MIN_VALIDATORS).unwrap();
        env.storage()
            .instance()
            .set(&MIN_VALIDATORS, &min_validators);

        // Emit event
        MinValidatorsUpdatedEvent {
            old_min,
            new_min: min_validators,
            updated_by: admin.clone(),
            updated_at: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    // ========== View Functions ==========

    /// Get the bridge transaction by nonce
    pub fn get_bridge_transaction(env: &Env, nonce: u64) -> Option<BridgeTransaction> {
        let bridge_txs: Map<u64, BridgeTransaction> = env
            .storage()
            .instance()
            .get(&BRIDGE_TXS)
            .unwrap_or_else(|| Map::new(env));
        bridge_txs.get(nonce)
    }

    /// Check if a chain is supported
    pub fn is_chain_supported(env: &Env, chain_id: u32) -> bool {
        let chains: Map<u32, bool> = env
            .storage()
            .instance()
            .get(&SUPPORTED_CHAINS)
            .unwrap_or_else(|| Map::new(env));
        chains.get(chain_id).unwrap_or(false)
    }

    /// Check if an address is a validator
    pub fn is_validator(env: &Env, address: Address) -> bool {
        let validators: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&VALIDATORS)
            .unwrap_or_else(|| Map::new(env));
        validators.get(address).unwrap_or(false)
    }

    /// Get the current nonce
    pub fn get_nonce(env: &Env) -> u64 {
        env.storage().instance().get(&NONCE).unwrap_or(0u64)
    }

    /// Get the bridge fee
    pub fn get_bridge_fee(env: &Env) -> i128 {
        env.storage().instance().get(&BRIDGE_FEE).unwrap_or(0i128)
    }

    /// Get the token address
    pub fn get_token(env: &Env) -> Address {
        env.storage().instance().get(&TOKEN).unwrap()
    }

    /// Get the admin address
    pub fn get_admin(env: &Env) -> Address {
        env.storage().instance().get(&ADMIN).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::{Bridge, BRIDGE_RETRY_DELAY_BASE_SECONDS};
    use crate::errors::BridgeError;
    use crate::storage::{BRIDGE_TXS, MIN_VALIDATORS, NONCE, TOKEN, VALIDATORS};
    use crate::types::{BridgeTransaction, CrossChainMessage};
    use crate::TeachLinkBridge;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::{vec, Address, Bytes, Env, Map, Vec};

    fn set_time(env: &Env, timestamp: u64) {
        env.ledger().with_mut(|ledger_info| {
            ledger_info.timestamp = timestamp;
        });
    }

    fn seed_bridge_tx(env: &Env, nonce: u64, timestamp: u64) {
        let token = Address::generate(env);
        let sender = Address::generate(env);
        env.storage().instance().set(&TOKEN, &token);

        let tx = BridgeTransaction {
            nonce,
            token,
            amount: 500,
            recipient: sender,
            destination_chain: 2,
            destination_address: Bytes::from_slice(env, b"dest"),
            timestamp,
        };

        let mut txs: Map<u64, BridgeTransaction> = Map::new(env);
        txs.set(nonce, tx);
        env.storage().instance().set(&BRIDGE_TXS, &txs);
    }

    #[test]
    fn mark_bridge_failed_requires_existing_tx() {
        let env = Env::default();
        let contract_id = env.register(TeachLinkBridge, ());
        let result = env.as_contract(&contract_id, || {
            Bridge::mark_bridge_failed(&env, 99, Bytes::from_slice(&env, b"failure"))
        });
        assert_eq!(result, Err(BridgeError::BridgeTransactionNotFound));
    }

    #[test]
    fn complete_bridge_rejects_replay_when_nonce_already_processed() {
        let env = Env::default();
        let contract_id = env.register(TeachLinkBridge, ());
        env.as_contract(&contract_id, || {
            let token = Address::generate(&env);
            let validator = Address::generate(&env);
            let recipient = Address::generate(&env);

            env.storage().instance().set(&TOKEN, &token);
            env.storage().instance().set(&MIN_VALIDATORS, &1u32);

            let mut validators: Map<Address, bool> = Map::new(&env);
            validators.set(validator.clone(), true);
            env.storage().instance().set(&VALIDATORS, &validators);

            let mut processed: Map<u64, bool> = Map::new(&env);
            processed.set(7u64, true);
            env.storage().persistent().set(&NONCE, &processed);

            let message = CrossChainMessage {
                source_chain: 1,
                source_tx_hash: Bytes::from_slice(&env, &[0xab; 32]),
                nonce: 7,
                token: token.clone(),
                amount: 100,
                recipient: recipient.clone(),
                destination_chain: 2,
            };

            let sigs: Vec<Address> = vec![&env, validator];
            let r = Bridge::complete_bridge(&env, message, sigs);
            assert_eq!(r, Err(BridgeError::NonceAlreadyProcessed));
        });
    }

    #[test]
    fn retry_bridge_enforces_backoff_and_limit() {
        let env = Env::default();
        let contract_id = env.register(TeachLinkBridge, ());
        env.as_contract(&contract_id, || {
            set_time(&env, 10_000);
            seed_bridge_tx(&env, 1, 10_000);

            let retry_too_early = Bridge::retry_bridge(&env, 1);
            assert_eq!(retry_too_early, Err(BridgeError::RetryBackoffActive));

            let mut now = 10_000u64;
            for retry_count in 0..5u32 {
                now += BRIDGE_RETRY_DELAY_BASE_SECONDS * (1u64 << retry_count);
                set_time(&env, now);
                let updated_retry_count = Bridge::retry_bridge(&env, 1).expect("retry should pass");
                assert_eq!(updated_retry_count, retry_count + 1);
            }

            set_time(&env, now + 100_000);
            let retry_over_limit = Bridge::retry_bridge(&env, 1);
            assert_eq!(retry_over_limit, Err(BridgeError::RetryLimitExceeded));
        });
    }
}
