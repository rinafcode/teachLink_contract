use crate::arbitration::ArbitrationManager;
use crate::errors::EscrowError;
use crate::escrow_analytics::EscrowAnalyticsManager;
use crate::events::{
    EscrowApprovedEvent, EscrowCreatedEvent, EscrowDisputedEvent, EscrowRefundedEvent,
    EscrowReleasedEvent, EscrowResolvedEvent,
};
// TODO: Implement insurance module
/*
use crate::insurance::InsuranceManager;
*/
use crate::storage::{ESCROWS, ESCROW_COUNT};
use crate::types::{DisputeOutcome, Escrow, EscrowApprovalKey, EscrowSigner, EscrowStatus};
use crate::validation::EscrowValidator;
use soroban_sdk::{symbol_short, vec, Address, Bytes, Env, IntoVal, Map, Vec};

pub struct EscrowManager;

impl EscrowManager {
    pub fn create_escrow(
        env: &Env,
        depositor: Address,
        beneficiary: Address,
        token: Address,
        amount: i128,
        signers: Vec<EscrowSigner>,
        threshold: u32,
        release_time: Option<u64>,
        refund_time: Option<u64>,
        arbitrator: Address,
    ) -> Result<u64, EscrowError> {
        depositor.require_auth();

        EscrowValidator::validate_create_escrow(
            env,
            &depositor,
            &beneficiary,
            &token,
            amount,
            &signers,
            threshold,
            release_time,
            refund_time,
            &arbitrator,
        )?;

        env.invoke_contract::<()>(
            &token,
            &symbol_short!("transfer"),
            vec![
                env,
                depositor.clone().into_val(env),
                env.current_contract_address().into_val(env),
                amount.into_val(env),
            ],
        );

        let mut escrow_count: u64 = env.storage().instance().get(&ESCROW_COUNT).unwrap_or(0);
        escrow_count += 1;
        env.storage().instance().set(&ESCROW_COUNT, &escrow_count);

        let now = env.ledger().timestamp();
        let escrow = Escrow {
            id: escrow_count,
            depositor,
            beneficiary,
            token: token.clone(),
            amount,
            signers,
            threshold,
            approval_count: 0,
            release_time,
            refund_time,
            arbitrator,
            status: EscrowStatus::Pending,
            created_at: now,
            dispute_reason: None,
        };

        let mut escrows = Self::load_escrows(env);
        escrows.set(escrow_count, escrow.clone());
        env.storage().instance().set(&ESCROWS, &escrows);

        EscrowAnalyticsManager::update_creation(env, amount);

        EscrowCreatedEvent { escrow }.publish(env);

        Ok(escrow_count)
    }

    pub fn approve_release(
        env: &Env,
        escrow_id: u64,
        signer_addr: Address,
    ) -> Result<u32, EscrowError> {
        signer_addr.require_auth();

        let mut escrow = Self::load_escrow(env, escrow_id)?;
        Self::ensure_pending(&escrow)?;

        let signer_info = Self::get_signer_info(&escrow.signers, &signer_addr)
            .ok_or(EscrowError::SignerNotAuthorized)?;

        let approval_key = EscrowApprovalKey {
            escrow_id,
            signer: signer_addr.clone(),
        };

        if env.storage().persistent().has(&approval_key) {
            return Err(EscrowError::SignerAlreadyApproved);
        }

        env.storage().persistent().set(&approval_key, &true);
        escrow.approval_count += signer_info.weight;

        Self::save_escrow(env, escrow_id, escrow.clone());

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

        let mut escrow = Self::load_escrow(env, escrow_id)?;

        EscrowValidator::validate_release_conditions(&escrow, &caller, env.ledger().timestamp())?;

        Self::transfer_from_contract(env, &escrow.token, &escrow.beneficiary, escrow.amount);

        escrow.status = EscrowStatus::Released;
        Self::save_escrow(env, escrow_id, escrow.clone());

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

        let mut escrow = Self::load_escrow(env, escrow_id)?;
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
        Self::save_escrow(env, escrow_id, escrow.clone());

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

        let mut escrow = Self::load_escrow(env, escrow_id)?;
        Self::ensure_pending(&escrow)?;

        if depositor != escrow.depositor {
            return Err(EscrowError::OnlyDepositorCanCancel);
        }

        if escrow.approval_count > 0 {
            return Err(EscrowError::CannotCancelAfterApprovals);
        }

        Self::transfer_from_contract(env, &escrow.token, &escrow.depositor, escrow.amount);

        escrow.status = EscrowStatus::Cancelled;
        Self::save_escrow(env, escrow_id, escrow.clone());

        Ok(())
    }

    pub fn dispute(
        env: &Env,
        escrow_id: u64,
        disputer: Address,
        reason: Bytes,
    ) -> Result<(), EscrowError> {
        disputer.require_auth();

        let mut escrow = Self::load_escrow(env, escrow_id)?;
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
        Self::save_escrow(env, escrow_id, escrow);

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

        let mut escrow = Self::load_escrow(env, escrow_id)?;

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
        Self::save_escrow(env, escrow_id, escrow);
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
        let mut escrow = Self::load_escrow(env, escrow_id)?;
        if ArbitrationManager::check_stalled_escrow(env, &escrow) {
            escrow.status = EscrowStatus::Disputed;
            escrow.dispute_reason = Some(Bytes::from_slice(env, b"Automated stall detection"));
            escrow.arbitrator = ArbitrationManager::pick_arbitrator(env)?;
            Self::save_escrow(env, escrow_id, escrow);

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
        Self::load_escrows(env).get(escrow_id)
    }

    pub fn get_escrow_count(env: &Env) -> u64 {
        env.storage().instance().get(&ESCROW_COUNT).unwrap_or(0)
    }

    pub fn has_approved(env: &Env, escrow_id: u64, signer: Address) -> bool {
        let key = EscrowApprovalKey { escrow_id, signer };
        env.storage().persistent().has(&key)
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

    fn load_escrows(env: &Env) -> Map<u64, Escrow> {
        env.storage()
            .instance()
            .get(&ESCROWS)
            .unwrap_or_else(|| Map::new(env))
    }

    fn load_escrow(env: &Env, escrow_id: u64) -> Result<Escrow, EscrowError> {
        let escrows = Self::load_escrows(env);
        escrows.get(escrow_id).ok_or(EscrowError::EscrowNotFound)
    }

    fn save_escrow(env: &Env, escrow_id: u64, escrow: Escrow) {
        let mut escrows = Self::load_escrows(env);
        escrows.set(escrow_id, escrow);
        env.storage().instance().set(&ESCROWS, &escrows);
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
