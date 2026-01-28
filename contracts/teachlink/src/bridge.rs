use crate::events::{BridgeCompletedEvent, BridgeInitiatedEvent, DepositEvent, ReleaseEvent};
use crate::storage::{
    ADMIN, BRIDGE_FEE, BRIDGE_TXS, FEE_RECIPIENT, MIN_VALIDATORS, NONCE, SUPPORTED_CHAINS, TOKEN,
    VALIDATORS,
};
use crate::types::{BridgeTransaction, CrossChainMessage, TeachLinkError};
use soroban_sdk::{symbol_short, vec, Address, Env, IntoVal, Map, Vec};

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
    ) -> Result<(), TeachLinkError> {
        // Check if already initialized
        if env.storage().instance().has(&TOKEN) {
            return Err(TeachLinkError::AlreadyInitialized);
        }

        // Validate min_validators
        if min_validators == 0 {
            return Err(TeachLinkError::MinValidatorsRequired);
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
    ) {
        from.require_auth();

        // Validate inputs
        if amount <= 0 {
            panic!("ERR_INVALID_AMOUNT: Amount must be positive");
        }

        // Check if destination chain is supported
        let supported_chains: Map<u32, bool> = env
            .storage()
            .instance()
            .get(&SUPPORTED_CHAINS)
            .unwrap_or_else(|| Map::new(env));
        if !supported_chains.get(destination_chain).unwrap_or(false) {
            panic!("ERR_UNSUPPORTED_CHAIN: Destination chain not supported");
        }

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
    }

    /// Complete a bridge transaction (mint/release tokens on Stellar)
    /// This is called by validators after verifying the transaction on the source chain
    /// - message: Cross-chain message containing transaction details
    /// - validator_signatures: List of validator addresses that have verified this transaction
    pub fn complete_bridge(
        env: &Env,
        message: CrossChainMessage,
        validator_signatures: Vec<Address>,
    ) {
        // Validate that we have enough validator signatures
        let min_validators: u32 = env.storage().instance().get(&MIN_VALIDATORS).unwrap();
        if (validator_signatures.len() as u32) < min_validators {
            panic!("ERR_INSUFFICIENT_VALIDATORS: Not enough validator signatures");
        }

        // Verify all signatures are from valid validators
        let validators: Map<Address, bool> = env.storage().instance().get(&VALIDATORS).unwrap();
        for validator in validator_signatures.iter() {
            if !validators.get(validator.clone()).unwrap_or(false) {
                panic!("ERR_INVALID_VALIDATOR: Invalid validator signature");
            }
        }

        // Check for duplicate nonce to prevent replay attacks
        let mut processed_nonces: Map<u64, bool> = env
            .storage()
            .persistent()
            .get(&NONCE)
            .unwrap_or_else(|| Map::new(env));
        if processed_nonces.get(message.nonce).unwrap_or(false) {
            panic!("ERR_DUPLICATE_NONCE: Nonce already processed");
        }
        processed_nonces.set(message.nonce, true);
        env.storage().persistent().set(&NONCE, &processed_nonces);

        // Get token address
        let token: Address = env.storage().instance().get(&TOKEN).unwrap();

        // Verify token matches
        if message.token != token {
            panic!("ERR_TOKEN_MISMATCH: Token mismatch");
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
    }

    /// Cancel a bridge transaction and refund locked tokens
    /// Only callable after a timeout period
    /// - nonce: The nonce of the bridge transaction to cancel
    pub fn cancel_bridge(env: &Env, nonce: u64) -> Result<(), TeachLinkError> {
        // Get bridge transaction
        let bridge_txs: Map<u64, BridgeTransaction> = env
            .storage()
            .instance()
            .get(&BRIDGE_TXS)
            .unwrap_or_else(|| Map::new(env));
        let bridge_tx = bridge_txs
            .get(nonce)
            .ok_or(TeachLinkError::BridgeTransactionNotFound)?;

        // Check timeout (7 days = 604800 seconds)
        const TIMEOUT: u64 = 604800;
        let elapsed = env.ledger().timestamp() - bridge_tx.timestamp;
        if elapsed < TIMEOUT {
            return Err(TeachLinkError::TimeoutNotReached);
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

        Ok(())
    }

    // ========== Admin Functions ==========

    /// Add a validator (admin only)
    pub fn add_validator(env: &Env, validator: Address) -> Result<(), TeachLinkError> {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        let mut validators: Map<Address, bool> = env.storage().instance().get(&VALIDATORS).unwrap();
        validators.set(validator, true);
        env.storage().instance().set(&VALIDATORS, &validators);

        Ok(())
    }

    /// Remove a validator (admin only)
    pub fn remove_validator(env: &Env, validator: Address) -> Result<(), TeachLinkError> {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        let mut validators: Map<Address, bool> = env.storage().instance().get(&VALIDATORS).unwrap();
        validators.set(validator, false);
        env.storage().instance().set(&VALIDATORS, &validators);

        Ok(())
    }

    /// Add a supported destination chain (admin only)
    pub fn add_supported_chain(env: &Env, chain_id: u32) -> Result<(), TeachLinkError> {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        let mut chains: Map<u32, bool> = env.storage().instance().get(&SUPPORTED_CHAINS).unwrap();
        chains.set(chain_id, true);
        env.storage().instance().set(&SUPPORTED_CHAINS, &chains);

        Ok(())
    }

    /// Remove a supported destination chain (admin only)
    pub fn remove_supported_chain(env: &Env, chain_id: u32) -> Result<(), TeachLinkError> {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        let mut chains: Map<u32, bool> = env.storage().instance().get(&SUPPORTED_CHAINS).unwrap();
        chains.set(chain_id, false);
        env.storage().instance().set(&SUPPORTED_CHAINS, &chains);

        Ok(())
    }

    /// Set bridge fee (admin only)
    pub fn set_bridge_fee(env: &Env, fee: i128) -> Result<(), TeachLinkError> {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        if fee < 0 {
            return Err(TeachLinkError::InvalidAmount);
        }

        env.storage().instance().set(&BRIDGE_FEE, &fee);

        Ok(())
    }

    /// Set fee recipient (admin only)
    pub fn set_fee_recipient(env: &Env, fee_recipient: Address) -> Result<(), TeachLinkError> {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        env.storage().instance().set(&FEE_RECIPIENT, &fee_recipient);

        Ok(())
    }

    /// Set minimum validators (admin only)
    pub fn set_min_validators(env: &Env, min_validators: u32) -> Result<(), TeachLinkError> {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        if min_validators == 0 {
            return Err(TeachLinkError::MinValidatorsRequired);
        }

        env.storage()
            .instance()
            .set(&MIN_VALIDATORS, &min_validators);

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
