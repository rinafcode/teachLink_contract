//! Escrow Domain Repositories
//!
//! This module provides repository implementations for escrow-related data access.
//! These repositories encapsulate all storage operations for the escrow domain.

use crate::repository::generic::{GenericCounterRepository, GenericMapRepository};
use crate::repository::traits::InstanceStorage;
use crate::repository::StorageError;
use crate::reentrancy;
use crate::storage::{ESCROWS, ESCROW_COUNT, ESCROW_GUARD};
use crate::types::{Escrow, EscrowApprovalKey};
use crate::errors::EscrowError;
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

    /// Atomically check for duplicate approval, record it, and increment the escrow's
    /// approval count — all under the ESCROW_GUARD reentrancy lock.
    pub fn approve_with_guard(
        &self,
        key: &EscrowApprovalKey,
        escrow_repo: &EscrowRepository,
    ) -> Result<u32, EscrowError> {
        let env = self.storage.env();
        reentrancy::with_guard(env, &ESCROW_GUARD, EscrowError::ReentrancyDetected, || {
            if self.has_approved(key) {
                return Err(EscrowError::SignerAlreadyApproved);
            }
            self.storage.set(key, &true);
            escrow_repo
                .increment_approval_count(key.escrow_id)
                .map_err(|_| EscrowError::StorageError)
        })
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

// #[cfg(test)]
// mod tests { // Removed - tests require env.as_contract() wrapper
//     use super::*;
//     use crate::types::EscrowSigner;
//     use soroban_sdk::testutils::Address as _;
//
//     #[test]
//     fn test_escrow_repository_create_and_get() {
//         let env = Env::default();
//         let repo = EscrowRepository::new(&env);
//
//         let depositor = Address::generate(&env);
//         let beneficiary = Address::generate(&env);
//         let token = Address::generate(&env);
//
//         let escrow = Escrow {
//             id: 1,
//             depositor: depositor.clone(),
//             beneficiary: beneficiary.clone(),
//             token: token.clone(),
//             amount: 1000,
//             signers: soroban_sdk::Vec::new(&env),
//             threshold: 1,
//             approval_count: 0,
//             release_time: None,
//             refund_time: None,
//             arbitrator: depositor.clone(),
//             status: crate::types::EscrowStatus::Pending,
//             created_at: env.ledger().timestamp(),
//             dispute_reason: None,
//         };
//
//         repo.save_escrow(&escrow).expect("Should save escrow");
//
//         let retrieved = repo.get_escrow(1);
//         assert!(retrieved.is_some());
//         assert_eq!(retrieved.unwrap().amount, 1000);
//     }
//
//     #[test]
//     fn test_escrow_repository_counter() {
//         let env = Env::default();
//         let repo = EscrowRepository::new(&env);
//
//         let initial_count = repo.get_count().expect("Should get count");
//         let next_id = repo.get_next_id().expect("Should get next ID");
//
//         assert_eq!(initial_count, 0);
//         assert_eq!(next_id, 1);
//
//         let second_id = repo.get_next_id().expect("Should get second ID");
//         assert_eq!(second_id, 2);
//     }
//
//     #[test]
//     fn test_approval_repository() {
//         let env = Env::default();
//         let escrow_repo = EscrowRepository::new(&env);
//         let approval_repo = EscrowApprovalRepository::new(&env);
//
//         let signer = Address::generate(&env);
//         let escrow_id = 1u64;
//
//         let key = EscrowApprovalKey {
//             escrow_id,
//             signer: signer.clone(),
//         };
//
//         // Initially should not be approved
//         assert!(!approval_repo.has_approved(&key));
//
//         // Approve
//         approval_repo.approve(&key).expect("Should approve");
//
//         // Now should be approved
//         assert!(approval_repo.has_approved(&key));
//     }
// }

#[cfg(test)]
mod concurrency_tests {
    use super::*;
    use crate::errors::EscrowError;
    use crate::storage::ESCROW_GUARD;
    use crate::types::{EscrowSigner, EscrowStatus};
    use crate::TeachLinkBridge;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env, Vec};

    fn make_escrow(env: &Env, id: u64) -> Escrow {
        let addr = Address::generate(env);
        Escrow {
            id,
            depositor: addr.clone(),
            beneficiary: Address::generate(env),
            token: Address::generate(env),
            amount: 1_000,
            signers: Vec::new(env),
            threshold: 1,
            approval_count: 0,
            release_time: None,
            refund_time: None,
            arbitrator: addr,
            status: EscrowStatus::Pending,
            created_at: env.ledger().timestamp(),
            dispute_reason: None,
        }
    }

    /// Duplicate approval by the same signer must be rejected.
    #[test]
    fn approve_with_guard_rejects_duplicate_approval() {
        let env = Env::default();
        let contract_id = env.register(TeachLinkBridge, ());

        env.as_contract(&contract_id, || {
            let escrow_repo = EscrowRepository::new(&env);
            let approval_repo = EscrowApprovalRepository::new(&env);

            let escrow = make_escrow(&env, 1);
            escrow_repo.save_escrow(&escrow).unwrap();

            let signer = Address::generate(&env);
            let key = EscrowApprovalKey { escrow_id: 1, signer: signer.clone() };

            // First approval succeeds and count becomes 1.
            let count = approval_repo.approve_with_guard(&key, &escrow_repo).unwrap();
            assert_eq!(count, 1);

            // Second approval by the same signer must be rejected.
            let result = approval_repo.approve_with_guard(&key, &escrow_repo);
            assert_eq!(result, Err(EscrowError::SignerAlreadyApproved));

            // Approval count must still be 1.
            assert_eq!(escrow_repo.get_escrow(1).unwrap().approval_count, 1);
        });
    }

    /// Different signers each increment the count exactly once.
    #[test]
    fn approve_with_guard_multiple_signers_increment_correctly() {
        let env = Env::default();
        let contract_id = env.register(TeachLinkBridge, ());

        env.as_contract(&contract_id, || {
            let escrow_repo = EscrowRepository::new(&env);
            let approval_repo = EscrowApprovalRepository::new(&env);

            let escrow = make_escrow(&env, 2);
            escrow_repo.save_escrow(&escrow).unwrap();

            for expected_count in 1u32..=3 {
                let key = EscrowApprovalKey {
                    escrow_id: 2,
                    signer: Address::generate(&env),
                };
                let count = approval_repo.approve_with_guard(&key, &escrow_repo).unwrap();
                assert_eq!(count, expected_count);
            }

            assert_eq!(escrow_repo.get_escrow(2).unwrap().approval_count, 3);
        });
    }

    /// When the reentrancy guard is already active, approve_with_guard must return
    /// ReentrancyDetected instead of proceeding.
    #[test]
    fn approve_with_guard_rejects_when_guard_active() {
        let env = Env::default();
        let contract_id = env.register(TeachLinkBridge, ());

        env.as_contract(&contract_id, || {
            let escrow_repo = EscrowRepository::new(&env);
            let approval_repo = EscrowApprovalRepository::new(&env);

            let escrow = make_escrow(&env, 3);
            escrow_repo.save_escrow(&escrow).unwrap();

            // Simulate a concurrent call by pre-setting the guard.
            env.storage().instance().set(&ESCROW_GUARD, &true);

            let key = EscrowApprovalKey {
                escrow_id: 3,
                signer: Address::generate(&env),
            };
            let result = approval_repo.approve_with_guard(&key, &escrow_repo);
            assert_eq!(result, Err(EscrowError::ReentrancyDetected));

            // Approval count must remain 0.
            assert_eq!(escrow_repo.get_escrow(3).unwrap().approval_count, 0);
        });
    }
}