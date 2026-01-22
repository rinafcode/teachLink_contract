use crate::events::{
    EscrowApprovedEvent, EscrowCreatedEvent, EscrowDisputedEvent, EscrowRefundedEvent,
    EscrowReleasedEvent, EscrowResolvedEvent,
};
use crate::storage::{ESCROW_COUNT, ESCROWS};
use crate::types::{DisputeOutcome, Escrow, EscrowApprovalKey, EscrowStatus};
use soroban_sdk::{symbol_short, vec, Address, Bytes, Env, IntoVal, Map, Vec};

pub struct EscrowManager;

impl EscrowManager {
    pub fn create_escrow(
        env: &Env,
        depositor: Address,
        beneficiary: Address,
        token: Address,
        amount: i128,
        signers: Vec<Address>,
        threshold: u32,
        release_time: Option<u64>,
        refund_time: Option<u64>,
        arbitrator: Address,
    ) -> u64 {
        depositor.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        if signers.len() == 0 {
            panic!("At least one signer required");
        }

        if threshold == 0 || threshold > signers.len() as u32 {
            panic!("Invalid signer threshold");
        }

        let now = env.ledger().timestamp();
        if let Some(refund_time) = refund_time {
            if refund_time < now {
                panic!("Refund time must be in the future");
            }
        }

        if let (Some(release), Some(refund)) = (release_time, refund_time) {
            if refund < release {
                panic!("Refund time must be after release time");
            }
        }

        Self::ensure_unique_signers(env, &signers);

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

        let mut escrow_count: u64 = env.storage().instance().get(&ESCROW_COUNT).unwrap_or(0u64);
        escrow_count += 1;
        env.storage().instance().set(&ESCROW_COUNT, &escrow_count);

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

        EscrowCreatedEvent { escrow }.publish(env);

        escrow_count
    }

    pub fn approve_release(env: &Env, escrow_id: u64, signer: Address) -> u32 {
        signer.require_auth();

        let mut escrow = Self::load_escrow(env, escrow_id);
        Self::ensure_pending(&escrow);

        if !Self::is_signer(&escrow.signers, &signer) {
            panic!("Signer not authorized");
        }

        let approval_key = EscrowApprovalKey {
            escrow_id,
            signer: signer.clone(),
        };
        if env.storage().persistent().has(&approval_key) {
            panic!("Signer already approved");
        }

        env.storage().persistent().set(&approval_key, &true);
        escrow.approval_count += 1;

        Self::save_escrow(env, escrow_id, escrow.clone());

        EscrowApprovedEvent {
            escrow_id,
            signer,
            approval_count: escrow.approval_count,
        }
        .publish(env);

        escrow.approval_count
    }

    pub fn release(env: &Env, escrow_id: u64, caller: Address) {
        caller.require_auth();

        let mut escrow = Self::load_escrow(env, escrow_id);
        Self::ensure_pending(&escrow);

        if !Self::is_release_caller(&escrow, &caller) {
            panic!("Caller not authorized");
        }

        if escrow.approval_count < escrow.threshold {
            panic!("Insufficient approvals");
        }

        if let Some(release_time) = escrow.release_time {
            let now = env.ledger().timestamp();
            if now < release_time {
                panic!("Release time not reached");
            }
        }

        Self::transfer_from_contract(env, &escrow.token, &escrow.beneficiary, escrow.amount);
        escrow.status = EscrowStatus::Released;
        Self::save_escrow(env, escrow_id, escrow.clone());

        EscrowReleasedEvent {
            escrow_id,
            beneficiary: escrow.beneficiary,
            amount: escrow.amount,
        }
        .publish(env);
    }

    pub fn refund(env: &Env, escrow_id: u64, depositor: Address) {
        depositor.require_auth();

        let mut escrow = Self::load_escrow(env, escrow_id);
        Self::ensure_pending(&escrow);

        if depositor != escrow.depositor {
            panic!("Only depositor can refund");
        }

        let refund_time = escrow.refund_time.unwrap_or_else(|| panic!("Refund not enabled"));
        let now = env.ledger().timestamp();
        if now < refund_time {
            panic!("Refund time not reached");
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
    }

    pub fn cancel(env: &Env, escrow_id: u64, depositor: Address) {
        depositor.require_auth();

        let mut escrow = Self::load_escrow(env, escrow_id);
        Self::ensure_pending(&escrow);

        if depositor != escrow.depositor {
            panic!("Only depositor can cancel");
        }

        if escrow.approval_count > 0 {
            panic!("Cannot cancel after approvals");
        }

        Self::transfer_from_contract(env, &escrow.token, &escrow.depositor, escrow.amount);
        escrow.status = EscrowStatus::Cancelled;
        Self::save_escrow(env, escrow_id, escrow.clone());
    }

    pub fn dispute(env: &Env, escrow_id: u64, disputer: Address, reason: Bytes) {
        disputer.require_auth();

        let mut escrow = Self::load_escrow(env, escrow_id);
        Self::ensure_pending(&escrow);

        if disputer != escrow.depositor && disputer != escrow.beneficiary {
            panic!("Only depositor or beneficiary can dispute");
        }

        escrow.status = EscrowStatus::Disputed;
        escrow.dispute_reason = Some(reason.clone());
        Self::save_escrow(env, escrow_id, escrow);

        EscrowDisputedEvent {
            escrow_id,
            disputer,
            reason,
        }
        .publish(env);
    }

    pub fn resolve(env: &Env, escrow_id: u64, arbitrator: Address, outcome: DisputeOutcome) {
        arbitrator.require_auth();

        let mut escrow = Self::load_escrow(env, escrow_id);
        if escrow.status != EscrowStatus::Disputed {
            panic!("Escrow not in dispute");
        }

        if arbitrator != escrow.arbitrator {
            panic!("Only arbitrator can resolve");
        }

        let new_status = match outcome {
            DisputeOutcome::ReleaseToBeneficiary => {
                Self::transfer_from_contract(env, &escrow.token, &escrow.beneficiary, escrow.amount);
                EscrowStatus::Released
            }
            DisputeOutcome::RefundToDepositor => {
                Self::transfer_from_contract(env, &escrow.token, &escrow.depositor, escrow.amount);
                EscrowStatus::Refunded
            }
        };

        escrow.status = new_status.clone();
        Self::save_escrow(env, escrow_id, escrow);

        EscrowResolvedEvent {
            escrow_id,
            outcome,
            status: new_status,
        }
        .publish(env);
    }

    pub fn get_escrow(env: &Env, escrow_id: u64) -> Option<Escrow> {
        let escrows = Self::load_escrows(env);
        escrows.get(escrow_id)
    }

    pub fn get_escrow_count(env: &Env) -> u64 {
        env.storage().instance().get(&ESCROW_COUNT).unwrap_or(0u64)
    }

    pub fn has_approved(env: &Env, escrow_id: u64, signer: Address) -> bool {
        let approval_key = EscrowApprovalKey { escrow_id, signer };
        env.storage().persistent().has(&approval_key)
    }

    fn ensure_unique_signers(env: &Env, signers: &Vec<Address>) {
        let mut seen: Map<Address, bool> = Map::new(env);
        for signer in signers.iter() {
            if seen.get(signer.clone()).unwrap_or(false) {
                panic!("Duplicate signer");
            }
            seen.set(signer.clone(), true);
        }
    }

    fn is_signer(signers: &Vec<Address>, signer: &Address) -> bool {
        for candidate in signers.iter() {
            if candidate == *signer {
                return true;
            }
        }
        false
    }

    fn is_release_caller(escrow: &Escrow, caller: &Address) -> bool {
        if *caller == escrow.depositor || *caller == escrow.beneficiary {
            return true;
        }
        Self::is_signer(&escrow.signers, caller)
    }

    fn ensure_pending(escrow: &Escrow) {
        if escrow.status != EscrowStatus::Pending {
            panic!("Escrow not pending");
        }
    }

    fn load_escrows(env: &Env) -> Map<u64, Escrow> {
        env.storage()
            .instance()
            .get(&ESCROWS)
            .unwrap_or_else(|| Map::new(env))
    }

    fn load_escrow(env: &Env, escrow_id: u64) -> Escrow {
        let escrows = Self::load_escrows(env);
        escrows
            .get(escrow_id)
            .unwrap_or_else(|| panic!("Escrow not found"))
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
