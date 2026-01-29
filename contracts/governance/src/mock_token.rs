//! Mock Token Contract for Testing
//!
//! A simple ERC20-like token for testing governance functionality.

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenDataKey {
    Admin,
    Balances,
    Allowances,
    TotalSupply,
    Name,
    Symbol,
    Decimals,
}

#[contract]
pub struct MockToken;

#[cfg(not(target_family = "wasm"))]
#[contractimpl]
impl MockToken {
    /// Initialize the mock token
    pub fn initialize(env: Env, admin: Address, name: String, symbol: String, decimals: u32) {
        if env.storage().instance().has(&TokenDataKey::Admin) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&TokenDataKey::Admin, &admin);
        env.storage().instance().set(&TokenDataKey::Name, &name);
        env.storage().instance().set(&TokenDataKey::Symbol, &symbol);
        env.storage()
            .instance()
            .set(&TokenDataKey::Decimals, &decimals);
        env.storage()
            .instance()
            .set(&TokenDataKey::TotalSupply, &0i128);

        let balances: Map<Address, i128> = Map::new(&env);
        env.storage()
            .instance()
            .set(&TokenDataKey::Balances, &balances);
    }

    /// Mint tokens to an address (admin only)
    pub fn mint(env: Env, to: Address, amount: i128) {
        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let admin: Address = env
            .storage()
            .instance()
            .get(&TokenDataKey::Admin)
            .expect("Not initialized");
        admin.require_auth();

        let mut balances = Self::load_balances(&env);
        let new_balance = balances.get(to.clone()).unwrap_or(0) + amount;
        balances.set(to, new_balance);
        env.storage()
            .instance()
            .set(&TokenDataKey::Balances, &balances);

        let total_supply: i128 = env
            .storage()
            .instance()
            .get(&TokenDataKey::TotalSupply)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&TokenDataKey::TotalSupply, &(total_supply + amount));
    }

    /// Burn tokens from an address
    pub fn burn(env: Env, from: Address, amount: i128) {
        if amount <= 0 {
            panic!("Amount must be positive");
        }
        from.require_auth();

        let mut balances = Self::load_balances(&env);
        let from_balance = balances.get(from.clone()).unwrap_or(0);
        if from_balance < amount {
            panic!("Insufficient balance");
        }

        balances.set(from, from_balance - amount);
        env.storage()
            .instance()
            .set(&TokenDataKey::Balances, &balances);

        let total_supply: i128 = env
            .storage()
            .instance()
            .get(&TokenDataKey::TotalSupply)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&TokenDataKey::TotalSupply, &(total_supply - amount));
    }

    /// Transfer tokens from one address to another
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
        env.storage()
            .instance()
            .set(&TokenDataKey::Balances, &balances);
    }

    /// Get the balance of an address
    pub fn balance(env: Env, owner: Address) -> i128 {
        let balances = Self::load_balances(&env);
        balances.get(owner).unwrap_or(0)
    }

    /// Get total supply
    pub fn total_supply(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&TokenDataKey::TotalSupply)
            .unwrap_or(0)
    }

    /// Get token name
    pub fn name(env: Env) -> String {
        env.storage()
            .instance()
            .get(&TokenDataKey::Name)
            .expect("Not initialized")
    }

    /// Get token symbol
    pub fn symbol(env: Env) -> String {
        env.storage()
            .instance()
            .get(&TokenDataKey::Symbol)
            .expect("Not initialized")
    }

    /// Get token decimals
    pub fn decimals(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&TokenDataKey::Decimals)
            .unwrap_or(18)
    }

    /// Get admin address
    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&TokenDataKey::Admin)
            .expect("Not initialized")
    }

    // Internal helper to load balances
    fn load_balances(env: &Env) -> Map<Address, i128> {
        env.storage()
            .instance()
            .get(&TokenDataKey::Balances)
            .unwrap_or_else(|| Map::new(env))
    }
}
