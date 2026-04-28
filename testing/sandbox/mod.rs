//! # TeachLink Sandbox Environment
//! Issue #381 — Comprehensive testing sandbox
//!
//! Provides a fully isolated local test environment with:
//! - Mock Stellar/Soroban environment via soroban_sdk::Env
//! - Pre-funded test accounts
//! - Mock token contract
//! - Quick-iteration helpers

pub mod fixtures;
pub mod mock_token;

use soroban_sdk::{Address, Env};

/// Central sandbox environment used in all local tests.
/// Wraps soroban_sdk::Env with helpers for quick setup.
///
/// # Example
/// ```rust
/// let sb = SandboxEnv::new();
/// let alice = sb.accounts.alice();
/// sb.fund_account(&alice, 1_000_0000000);
/// ```
pub struct SandboxEnv {
    /// The underlying Soroban mock environment
    pub env: Env,
    /// Pre-built named test accounts
    pub accounts: fixtures::TestAccounts,
}

impl SandboxEnv {
    /// Create a fresh sandbox — call this at the top of every test.
    /// Each call gives you a clean slate with no shared state.
    pub fn new() -> Self {
        let env = Env::default();
        // Allow all auth in sandbox — don't require real signatures
        env.mock_all_auths();

        let accounts = fixtures::TestAccounts::new(&env);

        Self { env, accounts }
    }

    /// Fund an account with a given amount of stroops (1 XLM = 10_000_000 stroops).
    pub fn fund_account(&self, _address: &Address, _amount_stroops: i128) {
        // In the mock env, balances are managed by the mock token.
        // This is a no-op hook — extend if you need balance assertions.
    }

    /// Fast-forward the sandbox ledger by `n` seconds.
    /// Useful for testing time-locked operations.
    pub fn advance_time(&self, seconds: u64) {
        self.env.ledger().with_mut(|l| {
            l.timestamp += seconds;
            l.sequence_number += 1;
        });
    }

    /// Fast-forward by a number of ledger sequences.
    pub fn advance_ledger(&self, ledgers: u32) {
        self.env.ledger().with_mut(|l| {
            l.sequence_number += ledgers;
            l.timestamp += u64::from(ledgers) * 5; // ~5s per ledger
        });
    }

    /// Print current sandbox ledger info (helpful for debugging).
    pub fn print_state(&self) {
        let ledger = self.env.ledger();
        println!(
            "[Sandbox] Ledger: seq={} timestamp={}",
            ledger.sequence(),
            ledger.timestamp(),
        );
    }
}

impl Default for SandboxEnv {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_creates_clean_env() {
        let sb = SandboxEnv::new();
        assert_eq!(sb.env.ledger().sequence(), 0);
    }

    #[test]
    fn sandbox_time_advance_works() {
        let sb = SandboxEnv::new();
        let before = sb.env.ledger().timestamp();
        sb.advance_time(60);
        let after = sb.env.ledger().timestamp();
        assert_eq!(after - before, 60);
    }

    #[test]
    fn sandbox_ledger_advance_works() {
        let sb = SandboxEnv::new();
        let before = sb.env.ledger().sequence();
        sb.advance_ledger(10);
        assert_eq!(sb.env.ledger().sequence() - before, 10);
    }

    #[test]
    fn sandbox_accounts_are_distinct() {
        let sb = SandboxEnv::new();
        let alice = sb.accounts.alice();
        let bob = sb.accounts.bob();
        assert_ne!(alice, bob);
    }

    #[test]
    fn two_sandboxes_are_isolated() {
        let sb1 = SandboxEnv::new();
        let sb2 = SandboxEnv::new();
        sb1.advance_time(999);
        // sb2 is unaffected
        assert_eq!(sb2.env.ledger().timestamp(), 0);
    }
}

