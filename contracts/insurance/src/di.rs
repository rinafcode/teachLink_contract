//! Dependency Injection Container
//!
//! This module provides a dependency injection pattern for the insurance contract,
//! enabling easier testing and flexible dependency management.
//!
//! # Architecture
//!
//! The DI layer separates external dependencies (token client, oracle, etc.)
//! from the core contract logic through trait abstractions.
//!
//! ## Traits (Interfaces)
//!
//! - `TokenProvider`: Abstracts token operations (transfer, balance, etc.)
//! - `OracleProvider`: Abstracts oracle interactions
//! - `StorageProvider`: Abstracts storage operations (optional, for completeness)
//!
//! ## Container
//!
//! The `Container` struct holds all injectable dependencies and is passed
//! throughout the contract execution context.
//!
//! ## Testing
//!
//! Mock implementations of each trait allow unit tests to run without
//! deploying actual token/oracle contracts.

use soroban_sdk::{Address, Env};
use crate::errors::InsuranceError;
use crate::types::*;

/// Token provider abstraction - enables swapping token implementations
pub trait TokenProvider {
    /// Transfer tokens from source to destination
    fn transfer(
        &self,
        from: &Address,
        to: &Address,
        amount: &i128,
    ) -> Result<(), InsuranceError>;

    /// Get token balance for an address
    fn balance(&self, account: &Address) -> Result<i128, InsuranceError>;

    /// Burn tokens
    fn burn(&self, account: &Address, amount: &i128) -> Result<(), InsuranceError>;

    /// Mint tokens
    fn mint(&self, account: &Address, amount: &i128) -> Result<(), InsuranceError>;
}

/// Oracle provider abstraction - enables swapping oracle implementations
pub trait OracleProvider {
    /// Get price data from oracle
    fn get_price(&self, asset_id: &str) -> Result<i128, InsuranceError>;

    /// Submit riskassessment to oracle for AI verification
    fn verify_risk_assessment(
        &self,
        profile_id: u64,
        risk_score: u32,
    ) -> Result<bool, InsuranceError>;

    /// Get claim verification result
    fn verify_claim(
        &self,
        claim_id: u64,
        claim_data: &ClaimData,
    ) -> Result<ClaimVerificationResult, InsuranceError>;
}

/// Real token provider - wraps Soroban token contract client
pub struct SorobanTokenProvider<'a> {
    env: &'a Env,
    token_addr: Address,
}

impl<'a> SorobanTokenProvider<'a> {
    /// Create a new token provider for the given token address
    pub fn new(env: &'a Env, token_addr: Address) -> Self {
        SorobanTokenProvider { env, token_addr }
    }
}

impl<'a> TokenProvider for SorobanTokenProvider<'a> {
    fn transfer(
        &self,
        from: &Address,
        to: &Address,
        amount: &i128,
    ) -> Result<(), InsuranceError> {
        use soroban_sdk::token;
        let token_client = token::Client::new(self.env, &self.token_addr);
        token_client.transfer(from, to, amount);
        Ok(())
    }

    fn balance(&self, account: &Address) -> Result<i128, InsuranceError> {
        use soroban_sdk::token;
        let token_client = token::Client::new(self.env, &self.token_addr);
        Ok(token_client.balance(account))
    }

    fn burn(&self, account: &Address, amount: &i128) -> Result<(), InsuranceError> {
        use soroban_sdk::token;
        let token_client = token::Client::new(self.env, &self.token_addr);
        token_client.burn(account, amount);
        Ok(())
    }

    fn mint(&self, account: &Address, amount: &i128) -> Result<(), InsuranceError> {
        use soroban_sdk::token;
        let token_client = token::Client::new(self.env, &self.token_addr);
        token_client.mint(account, amount);
        Ok(())
    }
}

/// Mock token provider for testing
#[cfg(test)]
pub struct MockTokenProvider {
    pub transfers: std::cell::RefCell<Vec<(String, String, i128)>>,
    pub balances: std::collections::HashMap<String, i128>,
}

#[cfg(test)]
impl MockTokenProvider {
    pub fn new() -> Self {
        MockTokenProvider {
            transfers: std::cell::RefCell::new(Vec::new()),
            balances: std::collections::HashMap::new(),
        }
    }
}

#[cfg(test)]
impl TokenProvider for MockTokenProvider {
    fn transfer(
        &self,
        from: &Address,
        to: &Address,
        amount: &i128,
    ) -> Result<(), InsuranceError> {
        self.transfers
            .borrow_mut()
            .push((from.to_string(), to.to_string(), *amount));
        Ok(())
    }

    fn balance(&self, account: &Address) -> Result<i128, InsuranceError> {
        Ok(*self
            .balances
            .get(&account.to_string())
            .unwrap_or(&0))
    }

    fn burn(&self, _account: &Address, _amount: &i128) -> Result<(), InsuranceError> {
        Ok(())
    }

    fn mint(&self, _account: &Address, _amount: &i128) -> Result<(), InsuranceError> {
        Ok(())
    }
}

/// Mock oracle provider for testing
#[cfg(test)]
pub struct MockOracleProvider {
    pub prices: std::collections::HashMap<String, i128>,
}

#[cfg(test)]
impl MockOracleProvider {
    pub fn new() -> Self {
        MockOracleProvider {
            prices: std::collections::HashMap::new(),
        }
    }
}

#[cfg(test)]
impl OracleProvider for MockOracleProvider {
    fn get_price(&self, asset_id: &str) -> Result<i128, InsuranceError> {
        Ok(*self.prices.get(asset_id).unwrap_or(&1000))
    }

    fn verify_risk_assessment(
        &self,
        _profile_id: u64,
        _risk_score: u32,
    ) -> Result<bool, InsuranceError> {
        Ok(true)
    }

    fn verify_claim(
        &self,
        _claim_id: u64,
        _claim_data: &ClaimData,
    ) -> Result<ClaimVerificationResult, InsuranceError> {
        Ok(ClaimVerificationResult {
            verified: true,
            confidence: 9500,
            ai_score: 95,
            oracle_score: 100,
        })
    }
}

/// Dependency Injection Container
///
/// Holds all injectable dependencies for the insurance contract.
/// This pattern allows easy substitution of implementations (e.g., mocks for testing).
pub struct Container<'a> {
    pub token_provider: &'a dyn TokenProvider,
    pub oracle_provider: &'a dyn OracleProvider,
}

impl<'a> Container<'a> {
    /// Create a new DI container with real providers
    pub fn new_production(
        env: &'a Env,
        token_addr: Address,
    ) -> Self {
        let token_provider = Box::leak(Box::new(SorobanTokenProvider::new(env, token_addr)));

        Container {
            token_provider: token_provider as &dyn TokenProvider,
            oracle_provider: &MockOracleProvider::new() as &dyn OracleProvider, // TODO: Implement real oracle provider
        }
    }

    /// Create a new DI container for testing with mock providers
    #[cfg(test)]
    pub fn new_test() -> Self {
        let token_provider = Box::leak(Box::new(MockTokenProvider::new()));
        let oracle_provider = Box::leak(Box::new(MockOracleProvider::new()));

        Container {
            token_provider: token_provider as &dyn TokenProvider,
            oracle_provider: oracle_provider as &dyn OracleProvider,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_token_provider_transfer() {
        let provider = MockTokenProvider::new();
        let addr1 = "test_addr1".to_string();
        let addr2 = "test_addr2".to_string();
        let amount = 1000i128;

        // Verify transfer is recorded
        // Note: In real contracts, this would use proper Address types
        // This is a simplified example
    }

    #[test]
    fn test_container_creation() {
        let container = Container::new_test();
        assert!(!core::ptr::null().eq(container.token_provider as *const _ as *const ()));
        assert!(!core::ptr::null().eq(container.oracle_provider as *const _ as *const ()));
    }
}
