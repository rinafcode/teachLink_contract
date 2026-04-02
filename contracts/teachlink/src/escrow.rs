use crate::arbitration::ArbitrationManager;
use crate::errors::EscrowError;
use crate::escrow_analytics::EscrowAnalyticsManager;
use crate::events::{
    EscrowApprovedEvent, EscrowCancelledEvent, EscrowCreatedEvent, EscrowDisputedEvent,
    EscrowRefundedEvent, EscrowReleasedEvent, EscrowResolvedEvent,
};
use crate::insurance::InsuranceManager;
use crate::repository::escrow_repository::EscrowAggregateRepository;
use crate::storage::{ESCROWS, ESCROW_COUNT};
use crate::types::{DisputeOutcome, Escrow, EscrowApprovalKey, EscrowSigner, EscrowStatus};
use crate::validation::EscrowValidator;
use soroban_sdk::{symbol_short, vec, Address, Bytes, Env, IntoVal, Map, Vec};

pub struct EscrowManager;

impl EscrowManager {
    /// Creates an escrow with validated parameters
    /// 
    /// # Arguments
    /// * `params` - EscrowParameters containing all escrow creation details
    /// 
    /// # Returns
    /// * `u64` - The ID of the created escrow
    /// 
    /// # Errors
    /// Returns `EscrowError` if validation fails
    pub fn create_escrow(
        env: &Env,
        params: EscrowParameters,
    ) -> Result<u64, EscrowError> {
        params.depositor.require_auth();

        EscrowValidator::validate_escrow_parameters(env, &params)?;

        env.invoke_contract::<()>(
            &params.token,
            &symbol_short!("transfer"),
            vec![
                env,
                params.depositor.clone().into_val(env),
                env.current_contract_address().into_val(env),
                params.amount.into_val(env),
            ],
        );

        // Process insurance premium
        let repo = EscrowAggregateRepository::new(env);
        if env.storage().instance().has(&crate::storage::INSURANCE_POOL) {
            let premium = InsuranceManager::calculate_premium(env, amount);
        if env
            .storage()
            .instance()
            .has(&crate::storage::INSURANCE_POOL)
        {
            let premium = InsuranceManager::calculate_premium(env, params.amount);
            if premium > 0 {
                InsuranceManager::pay_premium_internal(env, params.depositor.clone(), premium)?;
            }
        }

        // Get next escrow ID
        let escrow_id = repo.escrows.get_next_id().map_err(|_| EscrowError::StorageError)?;

        let now = env.ledger().timestamp();
        let escrow = Escrow {
            id: escrow_id,
            depositor,
            beneficiary,
            token: token.clone(),
            amount,
            signers,
            threshold,
            id: escrow_count,
            depositor: params.depositor.clone(),
            beneficiary: params.beneficiary.clone(),
            token: params.token.clone(),
            amount: params.amount,
            signers: params.signers.clone(),
            threshold: params.threshold,
            approval_count: 0,
            release_time: params.release_time,
            refund_time: params.refund_time,
            arbitrator: params.arbitrator.clone(),
            status: EscrowStatus::Pending,
            created_at: now,
            dispute_reason: None,
        };

        repo.escrows.save_escrow(&escrow).map_err(|_| EscrowError::StorageError)?;

        EscrowAnalyticsManager::update_creation(env, params.amount);

        EscrowCreatedEvent { escrow }.publish(env);

        Ok(escrow_id)
    }

    pub fn approve_release(
        env: &Env,
        escrow_id: u64,
        signer_addr: Address,
    ) -> Result<u32, EscrowError> {
        signer_addr.require_auth();

        let repo = EscrowAggregateRepository::new(env);
        
        let mut escrow = repo.escrows.get_escrow(escrow_id).ok_or(EscrowError::EscrowNotFound)?;
        Self::ensure_pending(&escrow)?;

        let signer_info = Self::get_signer_info(&escrow.signers, &signer_addr)
            .ok_or(EscrowError::SignerNotAuthorized)?;

        let approval_key = EscrowApprovalKey {
            escrow_id,
            signer: signer_addr.clone(),
        };

        if repo.approvals.has_approved(&approval_key) {
            return Err(EscrowError::SignerAlreadyApproved);
        }

        repo.approvals.approve(&approval_key).map_err(|_| EscrowError::StorageError)?;
        escrow.approval_count += signer_info.weight;

        repo.escrows.save_escrow(&escrow).map_err(|_| EscrowError::StorageError)?;

        EscrowApprovedEvent {
            escrow_id,
            signer: signer_addr,
            approval_count: escrow.approval_count,
        }
        .publish(env);

        Ok(escrow.approval_count)
    }

    pub fn release(env: &Env, escrow_id: u64, caller: Address) -> Result<(), EscrowError> {
        caller.require_auth();

        let repo = EscrowAggregateRepository::new(env);
        let mut escrow = repo.escrows.get_escrow(escrow_id).ok_or(EscrowError::EscrowNotFound)?;

        EscrowValidator::validate_release_conditions(&escrow, &caller, env.ledger().timestamp())?;

        Self::transfer_from_contract(env, &escrow.token, &escrow.beneficiary, escrow.amount);

        escrow.status = EscrowStatus::Released;
        repo.escrows.save_escrow(&escrow).map_err(|_| EscrowError::StorageError)?;

        EscrowReleasedEvent {
            escrow_id,
            beneficiary: escrow.beneficiary,
            amount: escrow.amount,
        }
        .publish(env);

        Ok(())
    }

    pub fn refund(env: &Env, escrow_id: u64, depositor: Address) -> Result<(), EscrowError> {
        depositor.require_auth();

        let repo = EscrowAggregateRepository::new(env);
        let mut escrow = repo.escrows.get_escrow(escrow_id).ok_or(EscrowError::EscrowNotFound)?;
        Self::ensure_pending(&escrow)?;

        if depositor != escrow.depositor {
            return Err(EscrowError::OnlyDepositorCanRefund);
        }

        let refund_time = escrow.refund_time.ok_or(EscrowError::RefundNotEnabled)?;

        let now = env.ledger().timestamp();

        if now < refund_time {
            return Err(EscrowError::RefundTimeNotReached);
        }

        Self::transfer_from_contract(env, &escrow.token, &escrow.depositor, escrow.amount);

        escrow.status = EscrowStatus::Refunded;
        repo.escrows.save_escrow(&escrow).map_err(|_| EscrowError::StorageError)?;

        EscrowRefundedEvent {
            escrow_id,
            depositor: escrow.depositor,
            amount: escrow.amount,
        }
        .publish(env);

        Ok(())
    }

    pub fn cancel(env: &Env, escrow_id: u64, depositor: Address) -> Result<(), EscrowError> {
        depositor.require_auth();

        let repo = EscrowAggregateRepository::new(env);
        let mut escrow = repo.escrows.get_escrow(escrow_id).ok_or(EscrowError::EscrowNotFound)?;
        Self::ensure_pending(&escrow)?;

        if depositor != escrow.depositor {
            return Err(EscrowError::OnlyDepositorCanCancel);
        }

        if escrow.approval_count > 0 {
            return Err(EscrowError::CannotCancelAfterApprovals);
        }

        Self::transfer_from_contract(env, &escrow.token, &escrow.depositor, escrow.amount);

        escrow.status = EscrowStatus::Cancelled;
        repo.escrows.save_escrow(&escrow).map_err(|_| EscrowError::StorageError)?;

        // Emit event
        EscrowCancelledEvent {
            escrow_id,
            depositor: escrow.depositor.clone(),
            amount: escrow.amount,
            cancelled_at: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    pub fn dispute(
        env: &Env,
        escrow_id: u64,
        disputer: Address,
        reason: Bytes,
    ) -> Result<(), EscrowError> {
        disputer.require_auth();

        let repo = EscrowAggregateRepository::new(env);
        let mut escrow = repo.escrows.get_escrow(escrow_id).ok_or(EscrowError::EscrowNotFound)?;
        Self::ensure_pending(&escrow)?;

        if disputer != escrow.depositor && disputer != escrow.beneficiary {
            return Err(EscrowError::OnlyDepositorOrBeneficiaryCanDispute);
        }

        // If arbitrator is default (zero address), pick a professional one
        if Self::arbitrator_is_empty(env, &escrow.arbitrator) {
            escrow.arbitrator = ArbitrationManager::pick_arbitrator(env)?;
        }

        escrow.status = EscrowStatus::Disputed;
        escrow.dispute_reason = Some(reason.clone());
        repo.escrows.save_escrow(&escrow).map_err(|_| EscrowError::StorageError)?;

        EscrowAnalyticsManager::update_dispute(env);

        EscrowDisputedEvent {
            escrow_id,
            disputer,
            reason,
        }
        .publish(env);

        Ok(())
    }

    pub fn resolve(
        env: &Env,
        escrow_id: u64,
        arbitrator: Address,
        outcome: DisputeOutcome,
    ) -> Result<(), EscrowError> {
        arbitrator.require_auth();

        let repo = EscrowAggregateRepository::new(env);
        let mut escrow = repo.escrows.get_escrow(escrow_id).ok_or(EscrowError::EscrowNotFound)?;

        if escrow.status != EscrowStatus::Disputed {
            return Err(EscrowError::EscrowNotInDispute);
        }

        if arbitrator != escrow.arbitrator {
            return Err(EscrowError::OnlyArbitratorCanResolve);
        }

        let new_status = match outcome {
            DisputeOutcome::ReleaseToBeneficiary => {
                Self::transfer_from_contract(
                    env,
                    &escrow.token,
                    &escrow.beneficiary,
                    escrow.amount,
                );
                EscrowStatus::Released
            }
            DisputeOutcome::RefundToDepositor => {
                Self::transfer_from_contract(env, &escrow.token, &escrow.depositor, escrow.amount);
                EscrowStatus::Refunded
            }
        };

        escrow.status = new_status.clone();

        ArbitrationManager::update_reputation(env, arbitrator, true)?;

        let now = env.ledger().timestamp();
        let created_at = escrow.created_at;
        repo.escrows.save_escrow(&escrow).map_err(|_| EscrowError::StorageError)?;
        EscrowAnalyticsManager::update_resolution(env, now - created_at);

        EscrowResolvedEvent {
            escrow_id,
            outcome,
            status: new_status,
        }
        .publish(env);

        Ok(())
    }

    pub fn auto_check_dispute(env: &Env, escrow_id: u64) -> Result<(), EscrowError> {
        let repo = EscrowAggregateRepository::new(env);
        let mut escrow = repo.escrows.get_escrow(escrow_id).ok_or(EscrowError::EscrowNotFound)?;
        
        if ArbitrationManager::check_stalled_escrow(env, &escrow) {
            escrow.status = EscrowStatus::Disputed;
            escrow.dispute_reason = Some(Bytes::from_slice(env, b"Automated stall detection"));
            escrow.arbitrator = ArbitrationManager::pick_arbitrator(env)?;
            repo.escrows.save_escrow(&escrow).map_err(|_| EscrowError::StorageError)?;

            EscrowDisputedEvent {
                escrow_id,
                disputer: env.current_contract_address(),
                reason: Bytes::from_slice(env, b"Automated stall detection"),
            }
            .publish(env);
        }
        Ok(())
    }

    // ---------- Views ----------

    pub fn get_escrow(env: &Env, escrow_id: u64) -> Option<Escrow> {
        let repo = EscrowAggregateRepository::new(env);
        repo.escrows.get_escrow(escrow_id)
    }

    pub fn get_escrow_count(env: &Env) -> u64 {
        let repo = EscrowAggregateRepository::new(env);
        repo.escrows.get_count().unwrap_or(0)
    }

    pub fn has_approved(env: &Env, escrow_id: u64, signer: Address) -> bool {
        let repo = EscrowAggregateRepository::new(env);
        let key = EscrowApprovalKey { escrow_id, signer };
        repo.approvals.has_approved(&key)
    }

    // ---------- Internal Helpers ----------

    fn get_signer_info(signers: &Vec<EscrowSigner>, signer_addr: &Address) -> Option<EscrowSigner> {
        for s in signers.iter() {
            if s.address == *signer_addr {
                return Some(s);
            }
        }
        None
    }

    fn is_signer(signers: &Vec<EscrowSigner>, signer_addr: &Address) -> bool {
        Self::get_signer_info(signers, signer_addr).is_some()
    }

    fn ensure_pending(escrow: &Escrow) -> Result<(), EscrowError> {
        if escrow.status != EscrowStatus::Pending {
            return Err(EscrowError::EscrowNotPending);
        }
        Ok(())
    }

    fn arbitrator_is_empty(env: &Env, arbitrator: &Address) -> bool {
        // Use current contract address as a signal for "no arbitrator assigned"
        *arbitrator == env.current_contract_address()
    }

    fn transfer_from_contract(env: &Env, token: &Address, to: &Address, amount: i128) {
        env.invoke_contract::<()>(
            token,
            &symbol_short!("transfer"),
            vec![
                env,
                env.current_contract_address().into_val(env),
                to.clone().into_val(env),
                amount.into_val(env),
            ],
        );
    }
}
