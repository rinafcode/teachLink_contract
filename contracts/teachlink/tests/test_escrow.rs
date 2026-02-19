#![allow(clippy::similar_names)]

use soroban_sdk::{
    contract, contractimpl, symbol_short, testutils::Address as _, Address, Bytes, Env, Map, Vec,
};

use teachlink_contract::{
    ArbitratorProfile, DisputeOutcome, EscrowParameters, EscrowRole, EscrowSigner, EscrowStatus,
    TeachLinkBridge, TeachLinkBridgeClient,
};

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

        let current_balance = balances.get(to.clone()).unwrap_or(0);
        balances.set(to.clone(), current_balance + amount);
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

        let from_balance = balances.get(from.clone()).unwrap_or(0);
        let to_balance = balances.get(to.clone()).unwrap_or(0);

        assert!(from_balance >= amount, "Insufficient balance");

        balances.set(from, from_balance - amount);
        balances.set(to, to_balance + amount);

        env.storage()
            .instance()
            .set(&symbol_short!("balances"), &balances);
    }
}

//
// ==========================
// Escrow Tests
// ==========================
//

#[test]
fn test_escrow_release_flow() {
    let env = Env::default();

    let escrow_contract_id = env.register(TeachLinkBridge, ());
    let escrow_client = TeachLinkBridgeClient::new(&env, &escrow_contract_id);

    let token_contract_id = env.register(TestToken, ());
    let token_client = TestTokenClient::new(&env, &token_contract_id);

    let token_admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let arbitrator = Address::generate(&env);

    env.mock_all_auths();

    token_client.initialize(&token_admin);
    token_client.mint(&depositor, &2_000); // More for premium

    escrow_client.initialize_insurance_pool(&token_contract_id, &100); // 1% premium

    let mut signers = Vec::new(&env);
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
        depositor: depositor.clone(),
        beneficiary: beneficiary.clone(),
        token: token_contract_id.clone(),
        amount: 500,
        signers: signers.clone(),
        threshold: 15, // Need both
        release_time: None,
        refund_time: None,
        arbitrator: arbitrator.clone(),
    };
    let escrow_id = escrow_client.create_escrow(&params);

    assert_eq!(token_client.balance(&depositor), 1495);
    assert_eq!(token_client.balance(&escrow_contract_id), 505); // 500 escrow + 5 premium

    escrow_client.approve_escrow_release(&escrow_id, &signer1);
    escrow_client.approve_escrow_release(&escrow_id, &signer2);
    escrow_client.release_escrow(&escrow_id, &signer1);

    assert_eq!(token_client.balance(&beneficiary), 500);
    assert_eq!(token_client.balance(&depositor), 1495);
    assert_eq!(token_client.balance(&depositor), 1495);

    let escrow = escrow_client.get_escrow(&escrow_id).unwrap();
    assert_eq!(escrow.status, EscrowStatus::Released);
}

#[test]
fn test_escrow_dispute_refund() {
    let env = Env::default();

    let escrow_contract_id = env.register(TeachLinkBridge, ());
    let escrow_client = TeachLinkBridgeClient::new(&env, &escrow_contract_id);

    let token_contract_id = env.register(TestToken, ());
    let token_client = TestTokenClient::new(&env, &token_contract_id);

    let token_admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let signer = Address::generate(&env);
    let arbitrator = Address::generate(&env);

    env.mock_all_auths();

    token_client.initialize(&token_admin);
    token_client.mint(&depositor, &1000);

    escrow_client.initialize_insurance_pool(&token_contract_id, &0); // No premium for this test

    let mut signers = Vec::new(&env);
    signers.push_back(EscrowSigner {
        address: signer.clone(),
        role: EscrowRole::Primary,
        weight: 1,
    });

    let params = EscrowParameters {
        depositor: depositor.clone(),
        beneficiary: beneficiary.clone(),
        token: token_contract_id.clone(),
        amount: 600,
        signers: signers.clone(),
        threshold: 1,
        release_time: None,
        refund_time: None,
        arbitrator: arbitrator.clone(),
    };
    let escrow_id = escrow_client.create_escrow(&params);

    let reason = Bytes::from_slice(&env, b"delay");

    escrow_client.dispute_escrow(&escrow_id, &beneficiary, &reason);

    escrow_client.resolve_escrow(&escrow_id, &arbitrator, &DisputeOutcome::RefundToDepositor);

    assert_eq!(token_client.balance(&depositor), 1000);

    let escrow = escrow_client.get_escrow(&escrow_id).unwrap();
    assert_eq!(escrow.status, EscrowStatus::Refunded);
}

#[test]
fn test_professional_arbitration_picking() {
    let env = Env::default();
    env.mock_all_auths();

    let escrow_contract_id = env.register(TeachLinkBridge, ());
    let escrow_client = TeachLinkBridgeClient::new(&env, &escrow_contract_id);

    let depositor = Address::generate(&env);
    let token_contract_id = env.register(TestToken, ());
    let token_client = TestTokenClient::new(&env, &token_contract_id);
    token_client.initialize(&Address::generate(&env));
    token_client.mint(&depositor, &1000);

    escrow_client.initialize_insurance_pool(&token_contract_id, &0); // No premium for this test
    let beneficiary = Address::generate(&env);
    let arb_addr = Address::generate(&env);

    // Register professional arbitrator
    let profile = ArbitratorProfile {
        address: arb_addr.clone(),
        name: soroban_sdk::String::from_str(&env, "Expert Judge"),
        specialization: Vec::new(&env),
        reputation_score: 500,
        total_resolved: 0,
        dispute_types_handled: Vec::new(&env),
        is_active: true,
    };
    escrow_client.register_arbitrator(&profile);

    let mut signers = Vec::new(&env);
    signers.push_back(EscrowSigner {
        address: Address::generate(&env),
        role: EscrowRole::Primary,
        weight: 1,
    });

    let params = EscrowParameters {
        depositor: depositor.clone(),
        beneficiary: beneficiary.clone(),
        token: token_contract_id.clone(),
        amount: 100,
        signers,
        threshold: 1,
        release_time: None,
        refund_time: None,
        arbitrator: escrow_contract_id.clone(), // Use contract address as "no arbitrator set" signal
    };
    
    // In Soroban, Address doesn't have a reliable "zero" constant easily accessible like this
    // So I'll just use a random one and check if my logic handles "empty" or I'll fix the logic to pick if zero.
    // Actually, I'll just use the depositor's address as "none" for this test or similar.
    // Wait, I'll just test the `dispute` logic picks the registered arb.
    
    let escrow_id = escrow_client.create_escrow(&params);
    let reason = Bytes::from_slice(&env, b"help");
    
    escrow_client.dispute_escrow(&escrow_id, &depositor, &reason);
    
    let escrow = escrow_client.get_escrow(&escrow_id).unwrap();
    assert_eq!(escrow.arbitrator, arb_addr);
}
