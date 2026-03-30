//! Dependency Injection Container
//!
//! This module provides a dependency injection pattern for the insurance contract,
//! enabling easier testing and flexible dependency management.

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

    /// Submit risk assessment to oracle for AI verification
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

#[cfg(test)]
mod mocks {
    use super::*;

    /// Mock token provider for testing
    pub struct MockTokenProvider;

    impl TokenProvider for MockTokenProvider {
        fn transfer(
            &self,
            _from: &Address,
            _to: &Address,
            _amount: &i128,
        ) -> Result<(), InsuranceError> {
            Ok(())
        }

        fn balance(&self, _account: &Address) -> Result<i128, InsuranceError> {
            Ok(1000000)
        }

        fn burn(&self, _account: &Address, _amount: &i128) -> Result<(), InsuranceError> {
            Ok(())
        }

        fn mint(&self, _account: &Address, _amount: &i128) -> Result<(), InsuranceError> {
            Ok(())
        }
    }

    /// Mock oracle provider for testing
    pub struct MockOracleProvider;

    impl OracleProvider for MockOracleProvider {
        fn get_price(&self, _asset_id: &str) -> Result<i128, InsuranceError> {
            Ok(1000)
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
}

#[cfg(test)]
pub use mocks::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_token_provider() {
        let _provider = MockTokenProvider;
        // Tests verify mock providers are available
        // Real integration tests would use these with contract code
    }

    #[test]
    fn test_mock_oracle_provider() {
        let _provider = MockOracleProvider;
        // Tests verify mock providers are available
    }
}
