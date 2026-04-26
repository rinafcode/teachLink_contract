cat > testing/sandbox/fixtures.rs << 'EOF'
//! # Test Fixtures
//! Issue #381 — Pre-built test accounts and data helpers
//!
//! Named accounts make tests readable:
//!   alice = typical learner
//!   bob   = typical educator
//!   carol = platform admin / third party
//!   dave  = adversarial / edge-case actor

use soroban_sdk::{Address, Env};

/// Named test accounts for readable, expressive tests.
pub struct TestAccounts {
    alice: Address,
    bob: Address,
    carol: Address,
    dave: Address,
}

impl TestAccounts {
    /// Create all named accounts bound to the given environment.
    pub fn new(env: &Env) -> Self {
        Self {
            alice: Address::generate(env),
            bob:   Address::generate(env),
            carol: Address::generate(env),
            dave:  Address::generate(env),
        }
    }

    /// Alice — typical learner account
    pub fn alice(&self) -> Address { self.alice.clone() }

    /// Bob — typical educator account
    pub fn bob(&self) -> Address { self.bob.clone() }

    /// Carol — platform admin or neutral third party
    pub fn carol(&self) -> Address { self.carol.clone() }

    /// Dave — adversarial or edge-case actor
    pub fn dave(&self) -> Address { self.dave.clone() }
}

/// Standard token amounts used across tests (in stroops, 1 XLM = 10_000_000)
pub mod amounts {
    pub const ONE_XLM: i128         = 10_000_000;
    pub const TEN_XLM: i128         = 100_000_000;
    pub const HUNDRED_XLM: i128     = 1_000_000_000;
    pub const THOUSAND_XLM: i128    = 10_000_000_000;
    pub const PLATFORM_FEE_BPS: i128 = 250; // 2.5%
}

/// Standard time values used across tests (in seconds)
pub mod time {
    pub const ONE_MINUTE:  u64 = 60;
    pub const ONE_HOUR:    u64 = 3_600;
    pub const ONE_DAY:     u64 = 86_400;
    pub const ONE_WEEK:    u64 = 604_800;
    pub const ONE_MONTH:   u64 = 2_592_000;
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn all_accounts_generated() {
        let env = Env::default();
        let accounts = TestAccounts::new(&env);
        // All four accounts must be distinct addresses
        let all = [
            accounts.alice(),
            accounts.bob(),
            accounts.carol(),
            accounts.dave(),
        ];
        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                assert_ne!(all[i], all[j], "accounts[{i}] == accounts[{j}] — must be unique");
            }
        }
    }

    #[test]
    fn amount_constants_are_correct() {
        use amounts::*;
        assert_eq!(ONE_XLM * 10, TEN_XLM);
        assert_eq!(TEN_XLM * 10, HUNDRED_XLM);
        assert_eq!(HUNDRED_XLM * 10, THOUSAND_XLM);
    }
}
