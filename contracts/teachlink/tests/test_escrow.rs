use soroban_sdk::{
    contract, contractclient, contractimpl, symbol_short, Address, Bytes, Env, Map, Vec,
};

use teachlink_contract::{DisputeOutcome, EscrowStatus, TeachLinkBridge, TeachLinkBridgeClient};

#[contract]
pub struct TestToken;

#[contractimpl]
impl TestToken {
    pub fn initialize(env: Env, admin: Address) {
        env.storage().instance().set(&symbol_short!("admin"), &admin);
        let balances: Map<Address, i128> = Map::new(&env);
        env.storage().instance().set(&symbol_short!("balances"), &balances);
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        if amount <= 0 {
            panic!("Amount must be positive");
        }
        let admin: Address = env.storage().instance().get(&symbol_short!("admin")).unwrap();
        admin.require_auth();

        let mut balances = Self::load_balances(&env);
        let new_balance = balances.get(to.clone()).unwrap_or(0) + amount;
        balances.set(to, new_balance);
        env.storage().instance().set(&symbol_short!("balances"), &balances);
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        if amount <= 0 {
            panic!("Amount must be positive");
        }
        from.require_auth();

        let mut balances = Self::load_balances(&env);
        let from_balance = balances.get(from.clone()).unwrap_or(0);
        if from_balance < amount {
            panic!("Insufficient balance");
        }
        balances.set(from.clone(), from_balance - amount);
        let to_balance = balances.get(to.clone()).unwrap_or(0);
        balances.set(to, to_balance + amount);
        env.storage().instance().set(&symbol_short!("balances"), &balances);
    }

    pub fn balance(env: Env, owner: Address) -> i128 {
        let balances = Self::load_balances(&env);
        balances.get(owner).unwrap_or(0)
    }

    fn load_balances(env: &Env) -> Map<Address, i128> {
        env.storage()
            .instance()
            .get(&symbol_short!("balances"))
            .unwrap_or_else(|| Map::new(env))
    }
}

#[contractclient(name = "TestTokenClient")]
pub trait TestTokenClient {
    fn initialize(&self, admin: &Address);
    fn mint(&self, to: &Address, amount: &i128);
    fn transfer(&self, from: &Address, to: &Address, amount: &i128);
    fn balance(&self, owner: &Address) -> i128;
}

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
    token_client.mint(&depositor, &1_000);

    let mut signers = Vec::new(&env);
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    let escrow_id = escrow_client.create_escrow(
        &depositor,
        &beneficiary,
        &token_contract_id,
        &500,
        &signers,
        &2,
        &None,
        &None,
        &arbitrator,
    );

    assert_eq!(token_client.balance(&depositor), 500);
    assert_eq!(token_client.balance(&escrow_contract_id), 500);

    escrow_client.approve_escrow_release(&escrow_id, &signer1);
    escrow_client.approve_escrow_release(&escrow_id, &signer2);
    escrow_client.release_escrow(&escrow_id, &signer1);

    assert_eq!(token_client.balance(&beneficiary), 500);
    assert_eq!(token_client.balance(&escrow_contract_id), 0);

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
    token_client.mint(&depositor, &800);

    let mut signers = Vec::new(&env);
    signers.push_back(signer.clone());

    let escrow_id = escrow_client.create_escrow(
        &depositor,
        &beneficiary,
        &token_contract_id,
        &600,
        &signers,
        &1,
        &None,
        &None,
        &arbitrator,
    );

    let reason = Bytes::from_slice(&env, b"delay");
    escrow_client.dispute_escrow(&escrow_id, &beneficiary, &reason);
    escrow_client.resolve_escrow(&escrow_id, &arbitrator, &DisputeOutcome::RefundToDepositor);

    assert_eq!(token_client.balance(&depositor), 800);
    let escrow = escrow_client.get_escrow(&escrow_id).unwrap();
    assert_eq!(escrow.status, EscrowStatus::Refunded);
}
