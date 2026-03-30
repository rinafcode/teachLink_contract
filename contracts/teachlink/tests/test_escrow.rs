#![allow(clippy::similar_names)]

use soroban_sdk::{
    contract, contractimpl, symbol_short, testutils::Address as _, Address, Bytes, Env, Map, Vec,
};

use teachlink_contract::{
    ArbitratorProfile, DisputeOutcome, EscrowError, EscrowParameters, EscrowRole, EscrowSigner,
    EscrowStatus, TeachLinkBridge, TeachLinkBridgeClient,
};

// ========== Mock Token ==========

#[contract]
pub struct TestToken;

#[contractimpl]
impl TestToken {
    pub fn initialize(env: Env, admin: Address) {
        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &admin);
        let balances: Map<Address, i128> = Map::new(&env);
        env.storage()
            .instance()
            .set(&symbol_short!("balances"), &balances);
    }

    pub fn balance(env: Env, address: Address) -> i128 {
        let balances: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&symbol_short!("balances"))
            .unwrap_or_else(|| Map::new(&env));
        balances.get(address).unwrap_or(0)
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .unwrap();
        admin.require_auth();
        let mut balances: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&symbol_short!("balances"))
            .unwrap_or_else(|| Map::new(&env));
        let current = balances.get(to.clone()).unwrap_or(0);
        balances.set(to, current + amount);
        env.storage()
            .instance()
            .set(&symbol_short!("balances"), &balances);
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        let mut balances: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&symbol_short!("balances"))
            .unwrap_or_else(|| Map::new(&env));
        let from_bal = balances.get(from.clone()).unwrap_or(0);
        let to_bal = balances.get(to.clone()).unwrap_or(0);
        assert!(from_bal >= amount, "Insufficient balance");
        balances.set(from, from_bal - amount);
        balances.set(to, to_bal + amount);
        env.storage()
            .instance()
            .set(&symbol_short!("balances"), &balances);
    }
}

// ========== Helpers ==========

struct EscrowSetup {
    env: Env,
    client: TeachLinkBridgeClient<'static>,
    token_client: TestTokenClient<'static>,
    token_id: Address,
    contract_id: Address,
    depositor: Address,
    beneficiary: Address,
    arbitrator: Address,
}

fn setup_escrow() -> EscrowSetup {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let token_id = env.register(TestToken, ());
    let token_client = TestTokenClient::new(&env, &token_id);

    let token_admin = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let arbitrator = Address::generate(&env);

    token_client.initialize(&token_admin);
    token_client.mint(&depositor, &5000);
    client.initialize(&token_id, &admin, &1, &fee_recipient);
    client.initialize_insurance_pool(&token_id, &0);

    EscrowSetup {
        env,
        client,
        token_client,
        token_id,
        contract_id,
        depositor,
        beneficiary,
        arbitrator,
    }
}

fn make_signers(env: &Env, signer: &Address) -> Vec<EscrowSigner> {
    let mut signers = Vec::new(env);
    signers.push_back(EscrowSigner {
        address: signer.clone(),
        role: EscrowRole::Primary,
        weight: 1,
    });
    signers
}

fn create_basic_escrow(s: &EscrowSetup) -> u64 {
    let signer = Address::generate(&s.env);
    let signers = make_signers(&s.env, &signer);
    let params = EscrowParameters {
        depositor: s.depositor.clone(),
        beneficiary: s.beneficiary.clone(),
        token: s.token_id.clone(),
        amount: 500,
        signers,
        threshold: 1,
        release_time: None,
        refund_time: None,
        arbitrator: s.arbitrator.clone(),
    };
    s.client.create_escrow(&params)
}

// ========== Happy Path Tests ==========

#[test]
fn test_escrow_release_flow() {
    let s = setup_escrow();
    let signer1 = Address::generate(&s.env);
    let signer2 = Address::generate(&s.env);

    let mut signers = Vec::new(&s.env);
    signers.push_back(EscrowSigner {
        address: signer1.clone(),
        role: EscrowRole::Primary,
        weight: 10,
    });
    signers.push_back(EscrowSigner {
        address: signer2.clone(),
        role: EscrowRole::Secondary,
        weight: 5,
    });

    let params = EscrowParameters {
        depositor: s.depositor.clone(),
        beneficiary: s.beneficiary.clone(),
        token: s.token_id.clone(),
        amount: 500,
        signers,
        threshold: 15,
        release_time: None,
        refund_time: None,
        arbitrator: s.arbitrator.clone(),
    };
    let id = s.client.create_escrow(&params);

    assert_eq!(s.token_client.balance(&s.depositor), 4500);
    assert_eq!(s.token_client.balance(&s.contract_id), 500);

    s.client.approve_escrow_release(&id, &signer1);
    s.client.approve_escrow_release(&id, &signer2);
    s.client.release_escrow(&id, &signer1);

    assert_eq!(s.token_client.balance(&s.beneficiary), 500);
    assert_eq!(
        s.client.get_escrow(&id).unwrap().status,
        EscrowStatus::Released
    );
}

#[test]
fn test_escrow_dispute_and_resolve_refund() {
    let s = setup_escrow();
    let id = create_basic_escrow(&s);

    let reason = Bytes::from_slice(&s.env, b"delay");
    s.client.dispute_escrow(&id, &s.beneficiary, &reason);

    assert_eq!(
        s.client.get_escrow(&id).unwrap().status,
        EscrowStatus::Disputed
    );

    s.client
        .resolve_escrow(&id, &s.arbitrator, &DisputeOutcome::RefundToDepositor);

    assert_eq!(
        s.client.get_escrow(&id).unwrap().status,
        EscrowStatus::Refunded
    );
    assert_eq!(s.token_client.balance(&s.depositor), 5000); // full refund
}

#[test]
fn test_escrow_dispute_and_resolve_release() {
    let s = setup_escrow();
    let id = create_basic_escrow(&s);

    let reason = Bytes::from_slice(&s.env, b"quality");
    s.client.dispute_escrow(&id, &s.depositor, &reason);
    s.client
        .resolve_escrow(&id, &s.arbitrator, &DisputeOutcome::ReleaseToBeneficiary);

    assert_eq!(
        s.client.get_escrow(&id).unwrap().status,
        EscrowStatus::Released
    );
    assert_eq!(s.token_client.balance(&s.beneficiary), 500);
}

#[test]
fn test_escrow_cancel_before_approvals() {
    let s = setup_escrow();
    let id = create_basic_escrow(&s);

    s.client.cancel_escrow(&id, &s.depositor);

    assert_eq!(
        s.client.get_escrow(&id).unwrap().status,
        EscrowStatus::Cancelled
    );
    assert_eq!(s.token_client.balance(&s.depositor), 5000); // funds returned
}

// ========== Error Condition Tests ==========

#[test]
fn test_approve_by_non_signer_fails() {
    let s = setup_escrow();
    let id = create_basic_escrow(&s);
    let stranger = Address::generate(&s.env);

    let result = s.client.try_approve_escrow_release(&id, &stranger);
    assert_eq!(result, Err(Ok(EscrowError::SignerNotAuthorized)));
}

#[test]
fn test_double_approve_fails() {
    let s = setup_escrow();
    let signer = Address::generate(&s.env);
    let signers = make_signers(&s.env, &signer);

    let params = EscrowParameters {
        depositor: s.depositor.clone(),
        beneficiary: s.beneficiary.clone(),
        token: s.token_id.clone(),
        amount: 500,
        signers,
        threshold: 1,
        release_time: None,
        refund_time: None,
        arbitrator: s.arbitrator.clone(),
    };
    let id = s.client.create_escrow(&params);

    s.client.approve_escrow_release(&id, &signer);
    let result = s.client.try_approve_escrow_release(&id, &signer);
    assert_eq!(result, Err(Ok(EscrowError::SignerAlreadyApproved)));
}

#[test]
fn test_dispute_by_stranger_fails() {
    let s = setup_escrow();
    let id = create_basic_escrow(&s);
    let stranger = Address::generate(&s.env);
    let reason = Bytes::from_slice(&s.env, b"fraud");

    let result = s.client.try_dispute_escrow(&id, &stranger, &reason);
    assert_eq!(
        result,
        Err(Ok(EscrowError::OnlyDepositorOrBeneficiaryCanDispute))
    );
}

#[test]
fn test_resolve_by_wrong_arbitrator_fails() {
    let s = setup_escrow();
    let id = create_basic_escrow(&s);
    let reason = Bytes::from_slice(&s.env, b"issue");
    s.client.dispute_escrow(&id, &s.depositor, &reason);

    let wrong_arb = Address::generate(&s.env);
    let result = s
        .client
        .try_resolve_escrow(&id, &wrong_arb, &DisputeOutcome::RefundToDepositor);
    assert_eq!(result, Err(Ok(EscrowError::OnlyArbitratorCanResolve)));
}

#[test]
fn test_resolve_non_disputed_escrow_fails() {
    let s = setup_escrow();
    let id = create_basic_escrow(&s);

    let result =
        s.client
            .try_resolve_escrow(&id, &s.arbitrator, &DisputeOutcome::RefundToDepositor);
    assert_eq!(result, Err(Ok(EscrowError::EscrowNotInDispute)));
}

#[test]
fn test_cancel_by_non_depositor_fails() {
    let s = setup_escrow();
    let id = create_basic_escrow(&s);

    let result = s.client.try_cancel_escrow(&id, &s.beneficiary);
    assert_eq!(result, Err(Ok(EscrowError::OnlyDepositorCanCancel)));
}

#[test]
fn test_cancel_after_approval_fails() {
    let s = setup_escrow();
    let signer = Address::generate(&s.env);
    let signers = make_signers(&s.env, &signer);

    let params = EscrowParameters {
        depositor: s.depositor.clone(),
        beneficiary: s.beneficiary.clone(),
        token: s.token_id.clone(),
        amount: 500,
        signers,
        threshold: 1,
        release_time: None,
        refund_time: None,
        arbitrator: s.arbitrator.clone(),
    };
    let id = s.client.create_escrow(&params);

    s.client.approve_escrow_release(&id, &signer);
    let result = s.client.try_cancel_escrow(&id, &s.depositor);
    assert_eq!(result, Err(Ok(EscrowError::CannotCancelAfterApprovals)));
}

#[test]
fn test_get_nonexistent_escrow() {
    let s = setup_escrow();
    assert!(s.client.get_escrow(&999).is_none());
}

// ========== View Function Tests ==========

#[test]
fn test_escrow_count_increments() {
    let s = setup_escrow();
    assert_eq!(s.client.get_escrow_count(), 0);

    create_basic_escrow(&s);
    assert_eq!(s.client.get_escrow_count(), 1);

    create_basic_escrow(&s);
    assert_eq!(s.client.get_escrow_count(), 2);
}

#[test]
fn test_has_escrow_approval() {
    let s = setup_escrow();
    let signer = Address::generate(&s.env);
    let signers = make_signers(&s.env, &signer);

    let params = EscrowParameters {
        depositor: s.depositor.clone(),
        beneficiary: s.beneficiary.clone(),
        token: s.token_id.clone(),
        amount: 500,
        signers,
        threshold: 1,
        release_time: None,
        refund_time: None,
        arbitrator: s.arbitrator.clone(),
    };
    let id = s.client.create_escrow(&params);

    assert!(!s.client.has_escrow_approval(&id, &signer));
    s.client.approve_escrow_release(&id, &signer);
    assert!(s.client.has_escrow_approval(&id, &signer));
}

// ========== Professional Arbitration ==========

#[test]
fn test_auto_arbitrator_assignment() {
    let s = setup_escrow();
    let arb_addr = Address::generate(&s.env);

    let profile = ArbitratorProfile {
        address: arb_addr.clone(),
        name: soroban_sdk::String::from_str(&s.env, "Judge"),
        specialization: Vec::new(&s.env),
        reputation_score: 500,
        total_resolved: 0,
        dispute_types_handled: Vec::new(&s.env),
        is_active: true,
    };
    s.client.register_arbitrator(&profile);

    let signer = Address::generate(&s.env);
    let signers = make_signers(&s.env, &signer);

    let params = EscrowParameters {
        depositor: s.depositor.clone(),
        beneficiary: s.beneficiary.clone(),
        token: s.token_id.clone(),
        amount: 100,
        signers,
        threshold: 1,
        release_time: None,
        refund_time: None,
        arbitrator: s.contract_id.clone(), // signal: no arbitrator
    };
    let id = s.client.create_escrow(&params);

    let reason = Bytes::from_slice(&s.env, b"help");
    s.client.dispute_escrow(&id, &s.depositor, &reason);

    assert_eq!(s.client.get_escrow(&id).unwrap().arbitrator, arb_addr);
}

// ========== Insurance Premium ==========

#[test]
fn test_escrow_with_insurance_premium() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let token_id = env.register(TestToken, ());
    let token_client = TestTokenClient::new(&env, &token_id);

    let token_admin = Address::generate(&env);
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let arbitrator = Address::generate(&env);

    token_client.initialize(&token_admin);
    token_client.mint(&depositor, &2000);

    client.initialize(&token_id, &admin, &1, &Address::generate(&env));
    client.initialize_insurance_pool(&token_id, &100); // 1% premium

    let signer = Address::generate(&env);
    let signers = make_signers(&env, &signer);

    let params = EscrowParameters {
        depositor: depositor.clone(),
        beneficiary,
        token: token_id.clone(),
        amount: 1000,
        signers,
        threshold: 1,
        release_time: None,
        refund_time: None,
        arbitrator,
    };
    client.create_escrow(&params);

    // 1000 escrow + 10 premium (1%) = 1010 deducted
    assert_eq!(token_client.balance(&depositor), 990);
    assert_eq!(token_client.balance(&contract_id), 1010);
}
