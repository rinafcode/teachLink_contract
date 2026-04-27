
//! # Mock Token Service
//! Issue #381 — Mock SEP-41 token for sandbox testing
//!
//! Lets tests mint, transfer, and check balances without
//! deploying a real token contract to any network.

use soroban_sdk::{
    contract, contractimpl, contracttype,
    token::TokenInterface,
    Address, Env, String,
};

/// Internal storage keys for the mock token
#[contracttype]
enum DataKey {
    Balance(Address),
    Allowance(Address, Address), // (owner, spender)
    Admin,
    Decimals,
    Name,
    Symbol,
}

/// A minimal SEP-41 compatible mock token.
/// Deploy this in sandbox tests instead of a real token.
#[contract]
pub struct MockToken;

#[contractimpl]
impl MockToken {
    /// Initialize the mock token. Call once after deploying.
    pub fn initialize(
        env: Env,
        admin: Address,
        decimals: u32,
        name: String,
        symbol: String,
    ) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Decimals, &decimals);
        env.storage().instance().set(&DataKey::Name, &name);
        env.storage().instance().set(&DataKey::Symbol, &symbol);
    }

    /// Mint tokens to any address. Only callable by admin.
    /// In sandbox tests, admin is typically the `carol` account.
    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        assert!(amount > 0, "mint amount must be positive");

        let current: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &(current + amount));
    }

    /// Burn tokens from an address. Only callable by admin.
    pub fn burn_from_admin(env: Env, from: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let current: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);

        assert!(current >= amount, "insufficient balance to burn");

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &(current - amount));
    }

    /// Get balance of any address (no auth required — public).
    pub fn balance_of(env: Env, address: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(address))
            .unwrap_or(0)
    }
}

/// Standard token interface (subset used in tests)
#[contractimpl]
impl TokenInterface for MockToken {
    fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        env.storage()
            .temporary()
            .get(&DataKey::Allowance(from, spender))
            .unwrap_or(0)
    }

    fn approve(env: Env, from: Address, spender: Address, amount: i128, _expiration_ledger: u32) {
        from.require_auth();
        env.storage()
            .temporary()
            .set(&DataKey::Allowance(from, spender), &amount);
    }

    fn balance(env: Env, id: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(id))
            .unwrap_or(0)
    }

    fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        let from_balance: i128 = Self::balance(env.clone(), from.clone());
        assert!(from_balance >= amount, "insufficient balance");

        let to_balance: i128 = Self::balance(env.clone(), to.clone());

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &(from_balance - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &(to_balance + amount));
    }

    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        let allowance = Self::allowance(env.clone(), from.clone(), spender.clone());
        assert!(allowance >= amount, "insufficient allowance");

        // Reduce allowance
        env.storage()
            .temporary()
            .set(&DataKey::Allowance(from.clone(), spender), &(allowance - amount));

        // Execute transfer
        Self::transfer(env, from, to, amount);
    }

    fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();
        let current = Self::balance(env.clone(), from.clone());
        assert!(current >= amount, "insufficient balance to burn");
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &(current - amount));
    }

    fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();
        let allowance = Self::allowance(env.clone(), from.clone(), spender.clone());
        assert!(allowance >= amount, "insufficient allowance");
        env.storage()
            .temporary()
            .set(&DataKey::Allowance(from.clone(), spender), &(allowance - amount));
        Self::burn(env, from, amount);
    }

    fn decimals(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::Decimals).unwrap_or(7)
    }

    fn name(env: Env) -> String {
        env.storage().instance().get(&DataKey::Name).unwrap()
    }

    fn symbol(env: Env) -> String {
        env.storage().instance().get(&DataKey::Symbol).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    fn setup() -> (Env, MockTokenClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, MockToken);
        let client = MockTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(
            &admin,
            &7u32,
            &String::from_str(&env, "Mock TeachLink Token"),
            &String::from_str(&env, "mTLT"),
        );

        (env, client, admin)
    }

    #[test]
    fn mint_increases_balance() {
        let (_env, client, admin) = setup();
        let recipient = Address::generate(&_env);

        client.mint(&admin, &recipient, &1_000_0000000i128);
        assert_eq!(client.balance(&recipient), 1_000_0000000i128);
    }

    #[test]
    fn transfer_moves_funds() {
        let (_env, client, admin) = setup();
        let alice = Address::generate(&_env);
        let bob = Address::generate(&_env);

        client.mint(&admin, &alice, &500_0000000i128);
        client.transfer(&alice, &bob, &100_0000000i128);

        assert_eq!(client.balance(&alice), 400_0000000i128);
        assert_eq!(client.balance(&bob),   100_0000000i128);
    }

    #[test]
    #[should_panic(expected = "insufficient balance")]
    fn transfer_more_than_balance_panics() {
        let (_env, client, admin) = setup();
        let alice = Address::generate(&_env);
        let bob = Address::generate(&_env);

        client.mint(&admin, &alice, &10_0000000i128);
        client.transfer(&alice, &bob, &999_0000000i128); // should panic
    }

    #[test]
    fn approve_and_transfer_from_works() {
        let (_env, client, admin) = setup();
        let alice = Address::generate(&_env);
        let spender = Address::generate(&_env);
        let bob = Address::generate(&_env);

        client.mint(&admin, &alice, &200_0000000i128);
        client.approve(&alice, &spender, &50_0000000i128, &999u32);
        client.transfer_from(&spender, &alice, &bob, &50_0000000i128);

        assert_eq!(client.balance(&alice), 150_0000000i128);
        assert_eq!(client.balance(&bob),    50_0000000i128);
        assert_eq!(client.allowance(&alice, &spender), 0);
    }

    #[test]
    fn burn_reduces_balance() {
        let (_env, client, admin) = setup();
        let alice = Address::generate(&_env);

        client.mint(&admin, &alice, &100_0000000i128);
        client.burn(&alice, &30_0000000i128);
        assert_eq!(client.balance(&alice), 70_0000000i128);
    }
}
