//! Concurrency Tests for Escrow Repository
//!
//! This module contains tests specifically designed to validate thread-safety
//! and race condition prevention in the escrow approval system.

use crate::repository::escrow_repository::EscrowAggregateRepository;
use crate::repository::StorageError;
use crate::types::{Escrow, EscrowSigner, EscrowStatus, EscrowRole};
use soroban_sdk::{Address, Env, Vec};

#[test]
fn test_atomic_escrow_approval_prevents_race_conditions() {
        let env = Env::default();
        let repo = EscrowAggregateRepository::new(&env);

        let depositor = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let signer1 = Address::generate(&env);
        let signer2 = Address::generate(&env);
        let token = Address::generate(&env);

        // Create escrow with multiple signers
        let mut signers = Vec::new(&env);
        signers.push_back(EscrowSigner {
            address: signer1.clone(),
            role: EscrowRole::Primary,
            weight: 1,
        });
        signers.push_back(EscrowSigner {
            address: signer2.clone(),
            role: EscrowRole::Secondary,
            weight: 1,
        });

        let escrow = Escrow {
            id: 1,
            depositor: depositor.clone(),
            beneficiary: beneficiary.clone(),
            token: token.clone(),
            amount: 1000,
            signers: signers.clone(),
            threshold: 2,
            approval_count: 0,
            release_time: None,
            refund_time: None,
            arbitrator: depositor.clone(),
            status: EscrowStatus::Pending,
            created_at: env.ledger().timestamp(),
            dispute_reason: None,
        };

        repo.escrows.save_escrow(&escrow).expect("Should save escrow");

        // Test sequential approvals work correctly
        let count1 = repo.approve_escrow(1, &signer1).expect("First approval should succeed");
        assert_eq!(count1, 1);

        let count2 = repo.approve_escrow(1, &signer2).expect("Second approval should succeed");
        assert_eq!(count2, 2);

        // Verify final state
        let updated_escrow = repo.escrows.get_escrow(1).unwrap();
        assert_eq!(updated_escrow.approval_count, 2);
    }

    #[test]
    fn test_duplicate_approval_prevention() {
        let env = Env::default();
        let repo = EscrowAggregateRepository::new(&env);

        let depositor = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let signer = Address::generate(&env);
        let token = Address::generate(&env);

        let mut signers = Vec::new(&env);
        signers.push_back(EscrowSigner {
            address: signer.clone(),
            role: EscrowRole::Primary,
            weight: 1,
        });

        let escrow = Escrow {
            id: 1,
            depositor: depositor.clone(),
            beneficiary: beneficiary.clone(),
            token: token.clone(),
            amount: 1000,
            signers: signers.clone(),
            threshold: 1,
            approval_count: 0,
            release_time: None,
            refund_time: None,
            arbitrator: depositor.clone(),
            status: EscrowStatus::Pending,
            created_at: env.ledger().timestamp(),
            dispute_reason: None,
        };

        repo.escrows.save_escrow(&escrow).expect("Should save escrow");

        // First approval should succeed
        let count1 = repo.approve_escrow(1, &signer).expect("First approval should succeed");
        assert_eq!(count1, 1);

        // Subsequent approvals should fail
        for _ in 0..5 {
            let result = repo.approve_escrow(1, &signer);
            assert!(matches!(result, Err(StorageError::AlreadyExists)),
                   "Duplicate approval should fail with AlreadyExists");
        }

        // Count should still be 1
        let final_escrow = repo.escrows.get_escrow(1).unwrap();
        assert_eq!(final_escrow.approval_count, 1);
    }

    #[test]
    fn test_unauthorized_signer_rejection() {
        let env = Env::default();
        let repo = EscrowAggregateRepository::new(&env);

        let depositor = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let authorized_signer = Address::generate(&env);
        let unauthorized_signer = Address::generate(&env);
        let token = Address::generate(&env);

        let mut signers = Vec::new(&env);
        signers.push_back(EscrowSigner {
            address: authorized_signer.clone(),
            role: EscrowRole::Primary,
            weight: 1,
        });

        let escrow = Escrow {
            id: 1,
            depositor: depositor.clone(),
            beneficiary: beneficiary.clone(),
            token: token.clone(),
            amount: 1000,
            signers: signers.clone(),
            threshold: 1,
            approval_count: 0,
            release_time: None,
            refund_time: None,
            arbitrator: depositor.clone(),
            status: EscrowStatus::Pending,
            created_at: env.ledger().timestamp(),
            dispute_reason: None,
        };

        repo.escrows.save_escrow(&escrow).expect("Should save escrow");

        // Authorized approval should work
        let result = repo.approve_escrow(1, &authorized_signer);
        assert!(result.is_ok(), "Authorized signer should be able to approve");

        // Unauthorized approval should fail
        let result = repo.approve_escrow(1, &unauthorized_signer);
        assert!(matches!(result, Err(StorageError::Unauthorized)),
               "Unauthorized signer should be rejected");
    }

    #[test]
    fn test_nonexistent_escrow_handling() {
        let env = Env::default();
        let repo = EscrowAggregateRepository::new(&env);

        let signer = Address::generate(&env);

        // Attempting to approve non-existent escrow should fail
        let result = repo.approve_escrow(999, &signer);
        assert!(matches!(result, Err(StorageError::NotFound)),
               "Approving non-existent escrow should fail");
    }

    #[test]
    fn test_approval_state_consistency() {
        let env = Env::default();
        let repo = EscrowAggregateRepository::new(&env);

        let depositor = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let signer1 = Address::generate(&env);
        let signer2 = Address::generate(&env);
        let signer3 = Address::generate(&env);
        let token = Address::generate(&env);

        let mut signers = Vec::new(&env);
        signers.push_back(EscrowSigner {
            address: signer1.clone(),
            role: EscrowRole::Primary,
            weight: 1,
        });
        signers.push_back(EscrowSigner {
            address: signer2.clone(),
            role: EscrowRole::Secondary,
            weight: 1,
        });
        signers.push_back(EscrowSigner {
            address: signer3.clone(),
            role: EscrowRole::Secondary,
            weight: 1,
        });

        let escrow = Escrow {
            id: 1,
            depositor: depositor.clone(),
            beneficiary: beneficiary.clone(),
            token: token.clone(),
            amount: 1000,
            signers: signers.clone(),
            threshold: 2,
            approval_count: 0,
            release_time: None,
            refund_time: None,
            arbitrator: depositor.clone(),
            status: EscrowStatus::Pending,
            created_at: env.ledger().timestamp(),
            dispute_reason: None,
        };

        repo.escrows.save_escrow(&escrow).expect("Should save escrow");

        // Approve with signer1
        repo.approve_escrow(1, &signer1).expect("Should approve");

        // Check state consistency
        assert!(repo.has_approved(1, &signer1));
        assert!(!repo.has_approved(1, &signer2));
        assert!(!repo.has_approved(1, &signer3));

        let escrow_state = repo.escrows.get_escrow(1).unwrap();
        assert_eq!(escrow_state.approval_count, 1);

        let approvals = repo.get_escrow_approvals(1).unwrap();
        assert_eq!(approvals.len(), 1);
        assert!(approvals.contains(&signer1));

        // Approve with signer2
        repo.approve_escrow(1, &signer2).expect("Should approve");

        // Check state consistency again
        assert!(repo.has_approved(1, &signer1));
        assert!(repo.has_approved(1, &signer2));
        assert!(!repo.has_approved(1, &signer3));

        let escrow_state = repo.escrows.get_escrow(1).unwrap();
        assert_eq!(escrow_state.approval_count, 2);

        let approvals = repo.get_escrow_approvals(1).unwrap();
        assert_eq!(approvals.len(), 2);
        assert!(approvals.contains(&signer1));
        assert!(approvals.contains(&signer2));
    }
}