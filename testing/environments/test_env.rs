/// Test environment management
use soroban_sdk::{Env, Address, testutils::Ledger};

pub struct TestEnvironment {
    pub env: Env,
    pub admin: Address,
    pub users: Vec<Address>,
    pub contracts: Vec<Address>,
}

impl TestEnvironment {
    pub fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        Self {
            env,
            admin: Address::generate(&Env::default()),
            users: Vec::new(),
            contracts: Vec::new(),
        }
    }

    pub fn create_users(&mut self, count: usize) {
        for _ in 0..count {
            self.users.push(Address::generate(&self.env));
        }
    }

    pub fn advance_time(&self, seconds: u64) {
        self.env.ledger().with_mut(|li| {
            li.timestamp += seconds;
        });
    }

    pub fn reset(&mut self) {
        self.env = Env::default();
        self.env.mock_all_auths();
        self.users.clear();
        self.contracts.clear();
    }
}
