use crate::errors::BridgeError;
use crate::events::{
    BridgeCancelledEvent, BridgeCompletedEvent, BridgeFailedEvent, BridgeFeeUpdatedEvent,
    BridgeInitiatedEvent, BridgeRetryEvent, ChainSupportedEvent, ChainUnsupportedEvent,
    DepositEvent, FeeRecipientUpdatedEvent, MinValidatorsUpdatedEvent, ReleaseEvent,
    ValidatorAddedEvent, ValidatorRemovedEvent,
};
use crate::reentrancy;
use crate::repository::bridge_repository::BridgeRepository;
use crate::storage::{
    ADMIN, BRIDGE_FAILURES, BRIDGE_FEE, BRIDGE_GUARD, BRIDGE_LAST_RETRY, BRIDGE_RETRY_COUNTS,
    BRIDGE_TXS, FEE_RECIPIENT, MIN_VALIDATORS, NONCE, SUPPORTED_CHAINS, TOKEN, VALIDATORS,
};
use crate::types::{BridgeTransaction, CrossChainMessage};
use crate::validation::BridgeValidator;
use soroban_sdk::{symbol_short, vec, Address, Bytes, Env, IntoVal, Map, Vec};

/// Bridge transaction timeout (7 days).  After this period a pending
/// transaction can be cancelled and the locked tokens refunded.
const BRIDGE_TIMEOUT_SECONDS: u64 = 604_800;

/// Maximum number of retry attempts before a bridge transaction is permanently
/// marked as failed.  Prevents infinite retry loops consuming gas.
const MAX_BRIDGE_RETRY_ATTEMPTS: u32 = 5;

/// Base delay between retry attempts (5 minutes).  Combined with the attempt
/// counter this implements an exponential back-off:
///   delay = BASE * 2^(attempt - 1)
/// so retries are spaced at 5 min, 10 min, 20 min, 40 min, 80 min.
///
/// # TODO
/// - Expose `MAX_BRIDGE_RETRY_ATTEMPTS` and `BRIDGE_RETRY_DELAY_BASE_SECONDS`
///   as admin-configurable parameters so they can be tuned without a contract
///   upgrade.
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
        let repo = BridgeRepository::new(env);

        // Check if already initialized
        if repo.config.is_initialized() {
            return Err(BridgeError::AlreadyInitialized);
        }

        if min_validators == 0 {
            return Err(BridgeError::MinimumValidatorsMustBeAtLeastOne);
        }

        repo.config
            .set_token(&token)
            .map_err(|_| BridgeError::StorageError)?;
        repo.config
            .set_admin(&admin)
            .map_err(|_| BridgeError::StorageError)?;
        repo.config
            .set_min_validators(min_validators)
            .map_err(|_| BridgeError::StorageError)?;
        repo.config
            .set_bridge_fee(0)
            .map_err(|_| BridgeError::StorageError)?;
        repo.config
            .set_fee_recipient(&fee_recipient)
            .map_err(|_| BridgeError::StorageError)?;

        // Initialize nonce to 0
        repo.transactions
            .get_current_nonce()
            .map_err(|_| BridgeError::StorageError)
            .ok();

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

        reentrancy::with_guard(env, &BRIDGE_GUARD, BridgeError::ReentrancyDetected, || {
            // Validate all input parameters (includes supported-chain registry check)
            BridgeValidator::validate_bridge_out(
                env,
                &from,
                amount,
                destination_chain,
                &destination_address,
            )?;

            let repo = BridgeRepository::new(env);

            // Check if destination chain is supported
            if !repo.chains.is_chain_supported(destination_chain) {
                return Err(BridgeError::DestinationChainNotSupported);
            }

            // Get token address
            let token = repo
                .config
                .get_token()
                .map_err(|_| BridgeError::NotInitialized)?;

            // Apply bridge fee if configured
            let fee = repo.config.get_bridge_fee().unwrap_or(0);
            let fee_recipient = repo.config.get_fee_recipient().unwrap();
            let amount_after_fee = if fee > 0 && fee < amount {
                amount - fee
            } else {
                amount
            };

            // Generate nonce for this transaction and create bridge transaction record
            let nonce = repo
                .transactions
                .get_next_nonce()
                .map_err(|_| BridgeError::StorageError)?;

            let bridge_tx = BridgeTransaction {
                nonce,
                token: token.clone(),
                amount: amount_after_fee,
                recipient: from.clone(),
                destination_chain,
                destination_address: destination_address.clone(),
                timestamp: env.ledger().timestamp(),
            };

            // Store bridge transaction and retry metadata before external calls
            repo.transactions
                .save_transaction(&bridge_tx)
                .map_err(|_| BridgeError::StorageError)?;
            repo.retry
                .set_retry_count(nonce, 0)
                .map_err(|_| BridgeError::StorageError)?;
            repo.retry
                .set_last_retry_time(nonce, env.ledger().timestamp())
                .map_err(|_| BridgeError::StorageError)?;

            // Transfer tokens from user to bridge (locking them)
            env.invoke_contract::<()>(
                &token,
                &symbol_short!("transfer"),
                vec![
                    env,
                    from.clone().into_val(env),
                    env.current_contract_address().into_val(env),
                    amount.into_val(env),
                ],
            );

            if fee > 0 && fee < amount {
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
            }

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
        })
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
        reentrancy::with_guard(env, &BRIDGE_GUARD, BridgeError::ReentrancyDetected, || {
            let repo = BridgeRepository::new(env);

            // Validate all input parameters
            let min_validators = repo.config.get_min_validators().unwrap_or(1);
            BridgeValidator::validate_bridge_completion(
                env,
                &message,
                &validator_signatures,
                min_validators,
            )?;

            // Verify all signatures are from valid validators
            for validator in validator_signatures.iter() {
                if !repo.validators.is_validator(&validator) {
                    return Err(BridgeError::InvalidValidatorSignature);
                }
            }

            // Check for duplicate nonce to prevent replay attacks (using persistent storage)
            let processed_nonces_key = NONCE;
            let mut processed_nonces: Map<u64, bool> = env
                .storage()
                .persistent()
                .get(&processed_nonces_key)
                .unwrap_or_else(|| Map::new(env));
            if processed_nonces.get(message.nonce).unwrap_or(false) {
                return Err(BridgeError::NonceAlreadyProcessed);
            }
            processed_nonces.set(message.nonce, true);
            env.storage()
                .persistent()
                .set(&processed_nonces_key, &processed_nonces);

            let token = repo
                .config
                .get_token()
                .map_err(|_| BridgeError::NotInitialized)?;
            if message.token != token {
                return Err(BridgeError::TokenMismatch);
            }

            // Remove transaction metadata before external mint interaction
            repo.transactions
                .remove_transaction(message.nonce)
                .map_err(|_| BridgeError::StorageError)?;
            repo.retry
                .clear_retry_metadata(message.nonce)
                .map_err(|_| BridgeError::StorageError)?;

            env.invoke_contract::<()>(
                &token,
                &symbol_short!("mint"),
                vec![
                    env,
                    message.recipient.into_val(env),
                    message.amount.into_val(env),
                ],
            );

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

            Ok(())
        })
    }

    pub fn mark_bridge_failed(env: &Env, nonce: u64, reason: Bytes) -> Result<(), BridgeError> {
        if reason.is_empty() {
            return Err(BridgeError::InvalidInput);
        }

        let repo = BridgeRepository::new(env);

        if !repo.transactions.has_transaction(nonce) {
            return Err(BridgeError::BridgeTransactionNotFound);
        }

        repo.retry
            .set_failure(nonce, &reason)
            .map_err(|_| BridgeError::StorageError)?;
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
        let repo = BridgeRepository::new(env);

        let bridge_tx = repo
            .transactions
            .get_transaction(nonce)
            .ok_or(BridgeError::BridgeTransactionNotFound)?;

        let current_time = env.ledger().timestamp();
        if current_time.saturating_sub(bridge_tx.timestamp) >= BRIDGE_TIMEOUT_SECONDS {
            return Err(BridgeError::PacketTimeout);
        }

        let retry_count = repo.retry.get_retry_count(nonce);
        if retry_count >= MAX_BRIDGE_RETRY_ATTEMPTS {
            return Err(BridgeError::RetryLimitExceeded);
        }

        let last_retry_at = repo.retry.get_last_retry_time(nonce);
        let last_retry_at = if last_retry_at == 0 {
            bridge_tx.timestamp
        } else {
            last_retry_at
        };

        let backoff_multiplier = 1u64 << retry_count;
        let retry_delay = BRIDGE_RETRY_DELAY_BASE_SECONDS.saturating_mul(backoff_multiplier);
        let next_allowed_retry = last_retry_at.saturating_add(retry_delay);

        if current_time < next_allowed_retry {
            return Err(BridgeError::RetryBackoffActive);
        }

        let updated_retry_count = retry_count + 1;
        repo.retry
            .set_retry_count(nonce, updated_retry_count)
            .map_err(|_| BridgeError::StorageError)?;
        repo.retry
            .set_last_retry_time(nonce, current_time)
            .map_err(|_| BridgeError::StorageError)?;

        // Clear failure record
        repo.retry
            .clear_failure(nonce)
            .map_err(|_| BridgeError::StorageError)?;

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
        reentrancy::with_guard(env, &BRIDGE_GUARD, BridgeError::ReentrancyDetected, || {
            let repo = BridgeRepository::new(env);

            let bridge_tx = repo
                .transactions
                .get_transaction(nonce)
                .ok_or(BridgeError::BridgeTransactionNotFound)?;

            let elapsed = env.ledger().timestamp().saturating_sub(bridge_tx.timestamp);
            let has_failed = repo.retry.get_failure(nonce).is_some();

            if elapsed < BRIDGE_TIMEOUT_SECONDS && !has_failed {
                return Err(BridgeError::TimeoutNotReached);
            }

            let token = repo
                .config
                .get_token()
                .map_err(|_| BridgeError::NotInitialized)?;

            // Remove transaction metadata before external transfer interaction
            repo.transactions
                .remove_transaction(nonce)
                .map_err(|_| BridgeError::StorageError)?;
            repo.retry
                .clear_retry_metadata(nonce)
                .map_err(|_| BridgeError::StorageError)?;

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

            BridgeCancelledEvent {
                nonce,
                refunded_to: bridge_tx.recipient.clone(),
                amount: bridge_tx.amount,
                cancelled_at: env.ledger().timestamp(),
            }
            .publish(env);

            Ok(())
        })
    }

    pub fn refund_bridge_transaction(env: &Env, nonce: u64) -> Result<(), BridgeError> {
        Self::cancel_bridge(env, nonce)
    }

    // ========== Admin Functions ==========

    /// Add a validator (admin only)
    #[allow(clippy::unnecessary_wraps)]
    pub fn add_validator(env: &Env, validator: Address) -> Result<(), BridgeError> {
        // Multi-layered authorization: Identity + Role check
        // Wait, add_validator usually takes an admin caller.
        // Let's assume the caller is passed or retrieved via require_auth() on the provided address.
        // But add_validator signature usually implies adding a *new* validator.
        // Let's check the caller's auth.
        // Usually, the contract entry point (in lib.rs) handles the caller.
        // If this is an internal implementation, it should take the caller.
        let _caller = validator.clone();

        // Actually, let's look at the original code:
        // let admin = repo.config.get_admin().map_err(|_| BridgeError::NotInitialized)?;
        // admin.require_auth();

        // I will assume for now that we want the caller to be an authorized ValidatorManager.
        // Since the caller isn't passed here, I'll use the one from repo.config or assume it's checked in lib.rs.
        // Better: let's change the signature to take the admin/caller if possible,
        // but if I can't change it easily, I'll use the one stored in repo.config as a fallback for the "current admin".

        let repo = BridgeRepository::new(env);
        let admin = repo
            .config
            .get_admin()
            .map_err(|_| BridgeError::NotInitialized)?;
        admin.require_auth();

        crate::access_control::AccessControlManager::check_role(
            env,
            &admin,
            crate::types::AccessRole::ValidatorManager,
        );

        repo.validators
            .add_validator(&validator)
            .map_err(|_| BridgeError::StorageError)?;
        let mut validators: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&VALIDATORS)
            .unwrap_or_else(|| Map::new(env));
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
        let repo = BridgeRepository::new(env);
        let admin = repo
            .config
            .get_admin()
            .map_err(|_| BridgeError::NotInitialized)?;
        admin.require_auth();

        crate::access_control::AccessControlManager::check_role(
            env,
            &admin,
            crate::types::AccessRole::ValidatorManager,
        );

        repo.validators
            .remove_validator(&validator)
            .map_err(|_| BridgeError::StorageError)?;
        let mut validators: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&VALIDATORS)
            .unwrap_or_else(|| Map::new(env));
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
        let repo = BridgeRepository::new(env);
        let admin = repo
            .config
            .get_admin()
            .map_err(|_| BridgeError::NotInitialized)?;
        admin.require_auth();

        crate::access_control::AccessControlManager::check_role(
            env,
            &admin,
            crate::types::AccessRole::BridgeOperator,
        );

        repo.chains
            .add_chain(chain_id)
            .map_err(|_| BridgeError::StorageError)?;

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
        let repo = BridgeRepository::new(env);
        let admin = repo
            .config
            .get_admin()
            .map_err(|_| BridgeError::NotInitialized)?;
        admin.require_auth();

        crate::access_control::AccessControlManager::check_role(
            env,
            &admin,
            crate::types::AccessRole::BridgeOperator,
        );

        repo.chains
            .remove_chain(chain_id)
            .map_err(|_| BridgeError::StorageError)?;

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
        let repo = BridgeRepository::new(env);
        let admin = repo
            .config
            .get_admin()
            .map_err(|_| BridgeError::NotInitialized)?;
        admin.require_auth();

        crate::access_control::AccessControlManager::check_role(
            env,
            &admin,
            crate::types::AccessRole::Admin,
        );

        if fee < 0 {
            return Err(BridgeError::FeeCannotBeNegative);
        }

        repo.config
            .set_bridge_fee(fee)
            .map_err(|_| BridgeError::StorageError)?;
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
        let repo = BridgeRepository::new(env);
        let admin = repo
            .config
            .get_admin()
            .map_err(|_| BridgeError::NotInitialized)?;
        admin.require_auth();

        crate::access_control::AccessControlManager::check_role(
            env,
            &admin,
            crate::types::AccessRole::Admin,
        );

        repo.config
            .set_fee_recipient(&fee_recipient)
            .map_err(|_| BridgeError::StorageError)?;
        let old_recipient: Address = env
            .storage()
            .instance()
            .get(&FEE_RECIPIENT)
            .ok_or(BridgeError::NotInitialized)?;
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
        let repo = BridgeRepository::new(env);
        let admin = repo
            .config
            .get_admin()
            .map_err(|_| BridgeError::NotInitialized)?;
        admin.require_auth();

        crate::access_control::AccessControlManager::check_role(
            env,
            &admin,
            crate::types::AccessRole::Admin,
        );

        if min_validators == 0 {
            return Err(BridgeError::MinimumValidatorsMustBeAtLeastOne);
        }

        repo.config
            .set_min_validators(min_validators)
            .map_err(|_| BridgeError::StorageError)?;
        let old_min: u32 = env
            .storage()
            .instance()
            .get(&MIN_VALIDATORS)
            .ok_or(BridgeError::NotInitialized)?;
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
        let repo = BridgeRepository::new(env);
        repo.transactions.get_transaction(nonce)
    }

    /// Check if a chain is supported
    pub fn is_chain_supported(env: &Env, chain_id: u32) -> bool {
        let repo = BridgeRepository::new(env);
        repo.chains.is_chain_supported(chain_id)
    }

    /// Check if an address is a validator
    pub fn is_validator(env: &Env, address: Address) -> bool {
        let repo = BridgeRepository::new(env);
        repo.validators.is_validator(&address)
    }

    /// Get the current nonce
    pub fn get_nonce(env: &Env) -> u64 {
        let repo = BridgeRepository::new(env);
        repo.transactions.get_current_nonce().unwrap_or(0)
    }

    /// Get the bridge fee
    pub fn get_bridge_fee(env: &Env) -> i128 {
        let repo = BridgeRepository::new(env);
        repo.config.get_bridge_fee().unwrap_or(0)
    }

    /// Get the token address
    pub fn get_token(env: &Env) -> Address {
        let repo = BridgeRepository::new(env);
        repo.config.get_token().unwrap()
    }

    /// Get the admin address
    pub fn get_admin(env: &Env) -> Address {
        let repo = BridgeRepository::new(env);
        repo.config.get_admin().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::{Bridge, BRIDGE_RETRY_DELAY_BASE_SECONDS};
    use crate::errors::BridgeError;
    use crate::storage::{BRIDGE_GUARD, BRIDGE_TXS, MIN_VALIDATORS, NONCE, TOKEN, VALIDATORS};
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
    fn cancel_bridge_rejects_when_reentrancy_guard_active() {
        let env = Env::default();
        let contract_id = env.register(TeachLinkBridge, ());
        env.as_contract(&contract_id, || {
            env.storage().instance().set(&BRIDGE_GUARD, &true);
            let result = Bridge::cancel_bridge(&env, 1);
            assert_eq!(result, Err(BridgeError::ReentrancyDetected));
        });
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
