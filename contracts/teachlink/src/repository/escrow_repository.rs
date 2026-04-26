//! Escrow Domain Repositories
//!
//! This module provides repository implementations for escrow-related data access.
//! These repositories encapsulate all storage operations for the escrow domain.

use crate::repository::generic::{GenericCounterRepository, GenericMapRepository};
use crate::repository::traits::InstanceStorage;
use crate::repository::StorageError;
use crate::storage::{ESCROWS, ESCROW_COUNT};
use crate::types::{Escrow, EscrowApprovalKey};
use soroban_sdk::{Address, Env, Map};

/// Repository for escrow management
pub struct EscrowRepository<'a> {
    storage: InstanceStorage<'a>,
    counter: GenericCounterRepository<'a, soroban_sdk::Symbol>,
}

impl<'a> EscrowRepository<'a> {
    pub fn new(env: &'a Env) -> Self {
        let storage = InstanceStorage::new(env);
        Self {
            storage: storage.clone(),
            counter: GenericCounterRepository::new(storage, ESCROW_COUNT),
        }
    }

    /// Get all escrows
    pub fn get_escrows(&self) -> Map<u64, Escrow> {
        self.storage
            .get(&ESCROWS)
            .unwrap_or_else(|| Map::new(self.storage.env()))
    }

    /// Get a specific escrow by ID
    pub fn get_escrow(&self, escrow_id: u64) -> Option<Escrow> {
        let escrows = self.get_escrows();
        escrows.get(escrow_id)
    }

    /// Save an escrow
    pub fn save_escrow(&self, escrow: &Escrow) -> Result<(), StorageError> {
        let mut escrows = self.get_escrows();
        escrows.set(escrow.id, escrow.clone());
        self.storage.set(&ESCROWS, &escrows);
        Ok(())
    }

    /// Create a new escrow and return its ID
    pub fn create_escrow(&self, escrow: &Escrow) -> Result<u64, StorageError> {
        self.save_escrow(escrow)?;
        Ok(escrow.id)
    }

    /// Get next escrow ID (increments counter)
    pub fn get_next_id(&self) -> Result<u64, StorageError> {
        self.counter.increment()
    }

    /// Get current escrow count
    pub fn get_count(&self) -> Result<u64, StorageError> {
        self.counter.get()
    }

    /// Check if escrow exists
    pub fn exists(&self, escrow_id: u64) -> bool {
        let escrows = self.get_escrows();
        escrows.contains_key(escrow_id)
    }

    /// Update escrow status
    pub fn update_status(
        &self,
        escrow_id: u64,
        status: crate::types::EscrowStatus,
    ) -> Result<(), StorageError> {
        let mut escrow = self.get_escrow(escrow_id).ok_or(StorageError::NotFound)?;
        escrow.status = status;
        self.save_escrow(&escrow)
    }

    /// Increment approval count for an escrow
    pub fn increment_approval_count(&self, escrow_id: u64) -> Result<u32, StorageError> {
        let mut escrow = self.get_escrow(escrow_id).ok_or(StorageError::NotFound)?;
        escrow.approval_count += 1;
        let new_count = escrow.approval_count;
        self.save_escrow(&escrow)?;
        Ok(new_count)
    }
}

/// Repository for escrow approvals (persistent storage)
pub struct EscrowApprovalRepository<'a> {
    storage: crate::repository::traits::PersistentStorage<'a>,
}

impl<'a> EscrowApprovalRepository<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self {
            storage: crate::repository::traits::PersistentStorage::new(env),
        }
    }

    /// Record an approval
    pub fn approve(&self, key: &EscrowApprovalKey) -> Result<(), StorageError> {
        self.storage.set(key, &true);
        Ok(())
    }

    /// Check if signer has approved
    pub fn has_approved(&self, key: &EscrowApprovalKey) -> bool {
        self.storage.has(key)
    }

    /// Get all approvals for an escrow
    pub fn get_escrow_approvals(
        &self,
        escrow_id: u64,
        signers: &soroban_sdk::Vec<crate::types::EscrowSigner>,
    ) -> soroban_sdk::Vec<Address> {
        let mut approved = soroban_sdk::Vec::new(self.storage.env());
        for signer in signers.iter() {
            let key = EscrowApprovalKey {
                escrow_id,
                signer: signer.address.clone(),
            };
            if self.has_approved(&key) {
                approved.push_back(signer.address.clone());
            }
        }
        approved
    }
}

/// Aggregate repository for all escrow operations
pub struct EscrowAggregateRepository<'a> {
    pub escrows: EscrowRepository<'a>,
    pub approvals: EscrowApprovalRepository<'a>,
}

impl<'a> EscrowAggregateRepository<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self {
            escrows: EscrowRepository::new(env),
            approvals: EscrowApprovalRepository::new(env),
        }
    }
}
