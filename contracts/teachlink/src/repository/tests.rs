//! Storage Abstraction Layer Tests
//! 
//! This module provides comprehensive tests for the repository pattern implementation.

use crate::repository::bridge_repository::{
    BridgeConfigRepository, BridgeRepository, BridgeTransactionRepository,
    BridgeRetryRepository, ValidatorRepository, ChainRepository,
};
use crate::repository::escrow_repository::{EscrowRepository, EscrowApprovalRepository};
use crate::repository::generic::{GenericCounterRepository, GenericMapRepository, SingleValueRepository};
use crate::repository::traits::{InstanceStorage, PersistentStorage, StorageBackend};
use crate::repository::StorageError;
use crate::storage::{ADMIN, TOKEN, VALIDATORS, ESCROWS, ESCROW_COUNT};
use crate::types::{BridgeTransaction, Escrow, EscrowApprovalKey};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Bytes, Env, Map};

#[test]
fn test_instance_storage_backend() {
    let env = Env::default();
    let storage = InstanceStorage::new(&env);
    
    let key = symbol_short!("test_key");
    let value = 42u64;
    
    // Test has (should be false initially)
    assert!(!storage.has(&key));
    
    // Test set
    storage.set(&key, &value);
    
    // Test has (should be true now)
    assert!(storage.has(&key));
    
    // Test get
    let retrieved: Option<u64> = storage.get(&key);
    assert_eq!(retrieved, Some(value));
    
    // Test remove
    storage.remove(&key);
    assert!(!storage.has(&key));
}

#[test]
fn test_persistent_storage_backend() {
    let env = Env::default();
    let storage = PersistentStorage::new(&env);
    
    let key = symbol_short!("persistent_key");
    let value = 100u64;
    
    storage.set(&key, &value);
    assert!(storage.has(&key));
    
    let retrieved: Option<u64> = storage.get(&key);
    assert_eq!(retrieved, Some(value));
}

#[test]
fn test_single_value_repository() {
    let env = Env::default();
    let storage = InstanceStorage::new(&env);
    let repo: SingleValueRepository<Address, u64> = SingleValueRepository::new(&storage);
    
    let key = Address::generate(&env);
    let value = 123u64;
    
    // Test get on non-existent key
    let result = repo.get(&key).unwrap();
    assert!(result.is_none());
    
    // Test set
    repo.set(&key, &value).unwrap();
    
    // Test get on existing key
    let result = repo.get(&key).unwrap();
    assert_eq!(result, Some(value));
    
    // Test exists
    assert!(repo.exists(&key).unwrap());
    
    // Test remove
    repo.remove(&key).unwrap();
    assert!(!repo.exists(&key).unwrap());
}

#[test]
fn test_generic_counter_repository() {
    let env = Env::default();
    let storage = InstanceStorage::new(&env);
    let counter = GenericCounterRepository::new(&storage, symbol_short!("counter"));
    
    // Test initial value
    assert_eq!(counter.get().unwrap(), 0);
    
    // Test increment
    assert_eq!(counter.increment().unwrap(), 1);
    assert_eq!(counter.get().unwrap(), 1);
    
    // Test multiple increments
    assert_eq!(counter.increment().unwrap(), 2);
    assert_eq!(counter.increment().unwrap(), 3);
    
    // Test reset
    counter.reset().unwrap();
    assert_eq!(counter.get().unwrap(), 0);
}

#[test]
fn test_generic_map_repository() {
    let env = Env::default();
    let storage = InstanceStorage::new(&env);
    let map_repo: GenericMapRepository<u64, u64> = 
        GenericMapRepository::new(&storage, symbol_short!("map"));
    
    // Test get on empty map
    assert!(map_repo.get(&1).unwrap().is_none());
    
    // Test set
    map_repo.set(&1, &100).unwrap();
    map_repo.set(&2, &200).unwrap();
    
    // Test get
    assert_eq!(map_repo.get(&1).unwrap(), Some(100));
    assert_eq!(map_repo.get(&2).unwrap(), Some(200));
    
    // Test contains
    assert!(map_repo.contains(&1).unwrap());
    assert!(!map_repo.contains(&3).unwrap());
    
    // Test all
    let all = map_repo.all().unwrap();
    assert_eq!(all.len(), 2);
    
    // Test remove
    map_repo.remove(&1).unwrap();
    assert!(map_repo.get(&1).unwrap().is_none());
}

#[test]
fn test_bridge_config_repository() {
    let env = Env::default();
    let repo = BridgeConfigRepository::new(&env);
    
    // Initially not initialized
    assert!(!repo.is_initialized());
    
    // Set config
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    
    repo.set_token(&token).unwrap();
    repo.set_admin(&admin).unwrap();
    repo.set_min_validators(3).unwrap();
    repo.set_bridge_fee(100).unwrap();
    repo.set_fee_recipient(&admin).unwrap();
    
    // Verify config
    assert!(repo.is_initialized());
    assert_eq!(repo.get_token().unwrap(), token);
    assert_eq!(repo.get_admin().unwrap(), admin);
    assert_eq!(repo.get_min_validators().unwrap(), 3);
    assert_eq!(repo.get_bridge_fee().unwrap(), 100);
}

#[test]
fn test_validator_repository() {
    let env = Env::default();
    let repo = ValidatorRepository::new(&env);
    
    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    
    // Initially no validators
    assert!(!repo.is_validator(&validator1));
    assert_eq!(repo.get_validator_count(), 0);
    
    // Add validator
    repo.add_validator(&validator1).unwrap();
    assert!(repo.is_validator(&validator1));
    assert!(!repo.is_validator(&validator2));
    assert_eq!(repo.get_validator_count(), 1);
    
    // Add another validator
    repo.add_validator(&validator2).unwrap();
    assert_eq!(repo.get_validator_count(), 2);
    
    // Remove validator
    repo.remove_validator(&validator1).unwrap();
    assert!(!repo.is_validator(&validator1));
    assert_eq!(repo.get_validator_count(), 1);
}

#[test]
fn test_chain_repository() {
    let env = Env::default();
    let repo = ChainRepository::new(&env);
    
    // Initially no chains
    assert!(!repo.is_chain_supported(1));
    
    // Add chain
    repo.add_chain(1).unwrap();
    repo.add_chain(2).unwrap();
    
    assert!(repo.is_chain_supported(1));
    assert!(repo.is_chain_supported(2));
    assert!(!repo.is_chain_supported(3));
    
    // Remove chain
    repo.remove_chain(1).unwrap();
    assert!(!repo.is_chain_supported(1));
    assert!(repo.is_chain_supported(2));
}

#[test]
fn test_bridge_transaction_repository() {
    let env = Env::default();
    let repo = BridgeTransactionRepository::new(&env);
    
    let token = Address::generate(&env);
    let recipient = Address::generate(&env);
    
    let tx = BridgeTransaction {
        nonce: 1,
        token: token.clone(),
        amount: 1000,
        recipient: recipient.clone(),
        destination_chain: 1,
        destination_address: Bytes::from_slice(&env, b"dest"),
        timestamp: env.ledger().timestamp(),
    };
    
    // Initially no transactions
    assert!(repo.get_transaction(1).is_none());
    
    // Save transaction
    repo.save_transaction(&tx).unwrap();
    
    // Get transaction
    let retrieved = repo.get_transaction(1).unwrap();
    assert_eq!(retrieved.amount, 1000);
    
    // Test nonce counter
    let initial_nonce = repo.get_current_nonce().unwrap();
    let new_nonce = repo.get_next_nonce().unwrap();
    assert_eq!(new_nonce, initial_nonce + 1);
    
    // Remove transaction
    repo.remove_transaction(1).unwrap();
    assert!(repo.get_transaction(1).is_none());
}

#[test]
fn test_escrow_repository() {
    let env = Env::default();
    let repo = EscrowRepository::new(&env);
    
    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let token = Address::generate(&env);
    
    let escrow = Escrow {
        id: 1,
        depositor: depositor.clone(),
        beneficiary: beneficiary.clone(),
        token: token.clone(),
        amount: 5000,
        signers: soroban_sdk::Vec::new(&env),
        threshold: 1,
        approval_count: 0,
        release_time: None,
        refund_time: None,
        arbitrator: depositor.clone(),
        status: crate::types::EscrowStatus::Pending,
        created_at: env.ledger().timestamp(),
        dispute_reason: None,
    };
    
    // Save escrow
    repo.save_escrow(&escrow).unwrap();
    
    // Get escrow
    let retrieved = repo.get_escrow(1).unwrap();
    assert_eq!(retrieved.amount, 5000);
    
    // Test counter
    let count = repo.get_count().unwrap();
    assert!(count >= 0);
    
    // Update escrow
    repo.update_status(1, crate::types::EscrowStatus::Released).unwrap();
    let updated = repo.get_escrow(1).unwrap();
    assert_eq!(updated.status, crate::types::EscrowStatus::Released);
}

#[test]
fn test_escrow_approval_repository() {
    let env = Env::default();
    let escrow_repo = EscrowRepository::new(&env);
    let approval_repo = EscrowApprovalRepository::new(&env);
    
    let signer = Address::generate(&env);
    let escrow_id = 1u64;
    
    let key = EscrowApprovalKey {
        escrow_id,
        signer: signer.clone(),
    };
    
    // Initially not approved
    assert!(!approval_repo.has_approved(&key));
    
    // Approve
    approval_repo.approve(&key).unwrap();
    
    // Now approved
    assert!(approval_repo.has_approved(&key));
}

#[test]
fn test_bridge_repository_aggregate() {
    let env = Env::default();
    let repo = BridgeRepository::new(&env);
    
    // Should be able to access all sub-repositories
    let _config = &repo.config;
    let _validators = &repo.validators;
    let _chains = &repo.chains;
    let _transactions = &repo.transactions;
    let _retry = &repo.retry;
    
    // Initialize config
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    
    repo.config.set_token(&token).unwrap();
    repo.config.set_admin(&admin).unwrap();
    
    assert!(repo.config.is_initialized());
}

#[test]
fn test_storage_error_handling() {
    let env = Env::default();
    let storage = InstanceStorage::new(&env);
    let repo: SingleValueRepository<u64, u64> = SingleValueRepository::new(&storage);
    
    // Getting non-existent key should return Ok(None), not an error
    let result = repo.get(&999);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_concurrent_storage_operations() {
    let env = Env::default();
    let storage = InstanceStorage::new(&env);
    
    // Test multiple operations on same storage
    let key1 = symbol_short!("key1");
    let key2 = symbol_short!("key2");
    
    storage.set(&key1, &100u64);
    storage.set(&key2, &200u64);
    
    assert!(storage.has(&key1));
    assert!(storage.has(&key2));
    
    let val1: Option<u64> = storage.get(&key1);
    let val2: Option<u64> = storage.get(&key2);
    
    assert_eq!(val1, Some(100));
    assert_eq!(val2, Some(200));
    
    storage.remove(&key1);
    
    assert!(!storage.has(&key1));
    assert!(storage.has(&key2));
}
