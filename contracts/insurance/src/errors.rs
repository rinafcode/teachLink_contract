use soroban_sdk::contracterror;

/// Insurance module errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum InsuranceError {
    AlreadyInitialized = 500,
    UserNotInsured = 501,
    ClaimNotFound = 502,
    ClaimAlreadyProcessed = 503,
    ClaimNotVerified = 504,
}

/// Result type alias for insurance operations
#[allow(dead_code)]
pub type InsuranceResult<T> = core::result::Result<T, InsuranceError>;
