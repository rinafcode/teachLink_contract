//! Bridge Domain Repositories
//!
//! This module provides repository implementations for bridge-related data access.
//! These repositories encapsulate all storage operations for the bridge domain,
//! hiding implementation details from business logic.

use crate::repository::generic::{
    GenericCounterRepository, GenericMapRepository, SingleValueRepository,
};
use crate::repository::traits::InstanceStorage;
use crate::repository::StorageError;
use crate::storage::{
    ADMIN, BRIDGE_FAILURES, BRIDGE_FEE, BRIDGE_LAST_RETRY, BRIDGE_RETRY_COUNTS, BRIDGE_TXS,
    FEE_RECIPIENT, MIN_VALIDATORS, NONCE, SUPPORTED_CHAINS, TOKEN, VALIDATORS,
};
use crate::types::BridgeTransaction;
use soroban_sdk::{Address, Bytes, Env, IntoVal, Map, Val};

/// Repository for bridge configuration
pub struct BridgeConfigRepository<'a> {
    storage: InstanceStorage<'a>,
}

impl<'a> BridgeConfigRepository<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self {
            storage: InstanceStorage::new(env),
        }
    }

    pub fn get_token(&self) -> Result<Address, StorageError> {
        self.storage.get(&TOKEN).ok_or(StorageError::NotFound)
    }

    pub fn set_token(&self, token: &Address) -> Result<(), StorageError> {
        self.storage.set(&TOKEN, token);
        Ok(())
    }

    pub fn get_admin(&self) -> Result<Address, StorageError> {
        self.storage.get(&ADMIN).ok_or(StorageError::NotFound)
    }

    pub fn set_admin(&self, admin: &Address) -> Result<(), StorageError> {
        self.storage.set(&ADMIN, admin);
        Ok(())
    }

    pub fn get_min_validators(&self) -> Result<u32, StorageError> {
        Ok(self.storage.get(&MIN_VALIDATORS).unwrap_or(1))
    }

    pub fn set_min_validators(&self, min_validators: u32) -> Result<(), StorageError> {
        self.storage.set(&MIN_VALIDATORS, &min_validators);
        Ok(())
    }

    pub fn get_fee_recipient(&self) -> Result<Address, StorageError> {
        self.storage
            .get(&FEE_RECIPIENT)
            .ok_or(StorageError::NotFound)
    }

    pub fn set_fee_recipient(&self, recipient: &Address) -> Result<(), StorageError> {
        self.storage.set(&FEE_RECIPIENT, recipient);
        Ok(())
    }

    pub fn get_bridge_fee(&self) -> Result<i128, StorageError> {
        Ok(self.storage.get(&BRIDGE_FEE).unwrap_or(0))
    }

    pub fn set_bridge_fee(&self, fee: i128) -> Result<(), StorageError> {
        self.storage.set(&BRIDGE_FEE, &fee);
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.storage.has(&TOKEN)
    }
}

/// Repository for validator management
pub struct ValidatorRepository<'a> {
    storage: InstanceStorage<'a>,
}

impl<'a> ValidatorRepository<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self {
            storage: InstanceStorage::new(env),
        }
    }

    pub fn get_validators(&self) -> Map<Address, bool> {
        self.storage
            .get(&VALIDATORS)
            .unwrap_or_else(|| Map::new(self.storage.env()))
    }

    pub fn add_validator(&self, validator: &Address) -> Result<(), StorageError> {
        let mut validators = self.get_validators();
        validators.set(validator.clone(), true);
        self.storage.set(&VALIDATORS, &validators);
        Ok(())
    }

    pub fn remove_validator(&self, validator: &Address) -> Result<(), StorageError> {
        let mut validators = self.get_validators();
        validators.set(validator.clone(), false);
        self.storage.set(&VALIDATORS, &validators);
        Ok(())
    }

    pub fn is_validator(&self, address: &Address) -> bool {
        let validators = self.get_validators();
        validators.get(address.clone()).unwrap_or(false)
    }

    pub fn get_validator_count(&self) -> u32 {
        let validators = self.get_validators();
        validators.iter().filter(|(_, v)| *v).count() as u32
    }
}

/// Repository for supported chains
pub struct ChainRepository<'a> {
    storage: InstanceStorage<'a>,
}

impl<'a> ChainRepository<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self {
            storage: InstanceStorage::new(env),
        }
    }

    pub fn get_supported_chains(&self) -> Map<u32, bool> {
        self.storage
            .get(&SUPPORTED_CHAINS)
            .unwrap_or_else(|| Map::new(self.storage.env()))
    }

    pub fn add_chain(&self, chain_id: u32) -> Result<(), StorageError> {
        let mut chains = self.get_supported_chains();
        chains.set(chain_id, true);
        self.storage.set(&SUPPORTED_CHAINS, &chains);
        Ok(())
    }

    pub fn remove_chain(&self, chain_id: u32) -> Result<(), StorageError> {
        let mut chains = self.get_supported_chains();
        chains.set(chain_id, false);
        self.storage.set(&SUPPORTED_CHAINS, &chains);
        Ok(())
    }

    pub fn is_chain_supported(&self, chain_id: u32) -> bool {
        let chains = self.get_supported_chains();
        chains.get(chain_id).unwrap_or(false)
    }
}

/// Repository for bridge transactions
pub struct BridgeTransactionRepository<'a> {
    storage: InstanceStorage<'a>,
    nonce_repo: GenericCounterRepository<'a, soroban_sdk::Symbol>,
}

impl<'a> BridgeTransactionRepository<'a> {
    pub fn new(env: &'a Env) -> Self {
        let storage = InstanceStorage::new(env);
        Self {
            storage: storage.clone(),
            nonce_repo: GenericCounterRepository::new(storage, NONCE),
        }
    }

    pub fn get_transactions(&self) -> Map<u64, BridgeTransaction> {
        self.storage
            .get(&BRIDGE_TXS)
            .unwrap_or_else(|| Map::new(self.storage.env()))
    }

    pub fn get_transaction(&self, nonce: u64) -> Option<BridgeTransaction> {
        let txs = self.get_transactions();
        txs.get(nonce)
    }

    pub fn save_transaction(&self, tx: &BridgeTransaction) -> Result<(), StorageError> {
        let mut txs = self.get_transactions();
        txs.set(tx.nonce, tx.clone());
        self.storage.set(&BRIDGE_TXS, &txs);
        Ok(())
    }

    pub fn remove_transaction(&self, nonce: u64) -> Result<(), StorageError> {
        let mut txs = self.get_transactions();
        if txs.contains_key(nonce) {
            txs.remove(nonce);
            self.storage.set(&BRIDGE_TXS, &txs);
        }
        Ok(())
    }

    pub fn get_next_nonce(&self) -> Result<u64, StorageError> {
        self.nonce_repo.increment()
    }

    pub fn get_current_nonce(&self) -> Result<u64, StorageError> {
        self.nonce_repo.get()
    }

    pub fn has_transaction(&self, nonce: u64) -> bool {
        let txs = self.get_transactions();
        txs.contains_key(nonce)
    }
}

/// Repository for bridge retry metadata
pub struct BridgeRetryRepository<'a> {
    storage: InstanceStorage<'a>,
}

impl<'a> BridgeRetryRepository<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self {
            storage: InstanceStorage::new(env),
        }
    }

    pub fn get_retry_counts(&self) -> Map<u64, u32> {
        self.storage
            .get(&BRIDGE_RETRY_COUNTS)
            .unwrap_or_else(|| Map::new(self.storage.env()))
    }

    pub fn get_retry_count(&self, nonce: u64) -> u32 {
        let counts = self.get_retry_counts();
        counts.get(nonce).unwrap_or(0)
    }

    pub fn set_retry_count(&self, nonce: u64, count: u32) -> Result<(), StorageError> {
        let mut counts = self.get_retry_counts();
        counts.set(nonce, count);
        self.storage.set(&BRIDGE_RETRY_COUNTS, &counts);
        Ok(())
    }

    pub fn increment_retry_count(&self, nonce: u64) -> Result<u32, StorageError> {
        let count = self.get_retry_count(nonce);
        let new_count = count + 1;
        self.set_retry_count(nonce, new_count)?;
        Ok(new_count)
    }

    pub fn get_last_retry(&self) -> Map<u64, u64> {
        self.storage
            .get(&BRIDGE_LAST_RETRY)
            .unwrap_or_else(|| Map::new(self.storage.env()))
    }

    pub fn get_last_retry_time(&self, nonce: u64) -> u64 {
        let last_retry = self.get_last_retry();
        last_retry.get(nonce).unwrap_or(0)
    }

    pub fn set_last_retry_time(&self, nonce: u64, timestamp: u64) -> Result<(), StorageError> {
        let mut last_retry = self.get_last_retry();
        last_retry.set(nonce, timestamp);
        self.storage.set(&BRIDGE_LAST_RETRY, &last_retry);
        Ok(())
    }

    pub fn get_failures(&self) -> Map<u64, Bytes> {
        self.storage
            .get(&BRIDGE_FAILURES)
            .unwrap_or_else(|| Map::new(self.storage.env()))
    }

    pub fn get_failure(&self, nonce: u64) -> Option<Bytes> {
        let failures = self.get_failures();
        failures.get(nonce)
    }

    pub fn set_failure(&self, nonce: u64, reason: &Bytes) -> Result<(), StorageError> {
        let mut failures = self.get_failures();
        failures.set(nonce, reason.clone());
        self.storage.set(&BRIDGE_FAILURES, &failures);
        Ok(())
    }

    pub fn clear_failure(&self, nonce: u64) -> Result<(), StorageError> {
        let mut failures = self.get_failures();
        if failures.contains_key(nonce) {
            failures.remove(nonce);
            self.storage.set(&BRIDGE_FAILURES, &failures);
        }
        Ok(())
    }

    pub fn clear_retry_metadata(&self, nonce: u64) -> Result<(), StorageError> {
        // Clear retry count
        let mut counts = self.get_retry_counts();
        if counts.contains_key(nonce) {
            counts.remove(nonce);
            self.storage.set(&BRIDGE_RETRY_COUNTS, &counts);
        }

        // Clear last retry time
        let mut last_retry = self.get_last_retry();
        if last_retry.contains_key(nonce) {
            last_retry.remove(nonce);
            self.storage.set(&BRIDGE_LAST_RETRY, &last_retry);
        }

        // Clear failure
        self.clear_failure(nonce)?;

        Ok(())
    }
}

/// Aggregate repository for all bridge operations
pub struct BridgeRepository<'a> {
    pub config: BridgeConfigRepository<'a>,
    pub validators: ValidatorRepository<'a>,
    pub chains: ChainRepository<'a>,
    pub transactions: BridgeTransactionRepository<'a>,
    pub retry: BridgeRetryRepository<'a>,
}

impl<'a> BridgeRepository<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self {
            config: BridgeConfigRepository::new(env),
            validators: ValidatorRepository::new(env),
            chains: ChainRepository::new(env),
            transactions: BridgeTransactionRepository::new(env),
            retry: BridgeRetryRepository::new(env),
        }
    }
}
