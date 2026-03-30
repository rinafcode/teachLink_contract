//! Credential registry data types.

/// Status of a credential.
#[derive(Clone, PartialEq)]
pub enum CredentialStatus {
    Active,
    Revoked,
    Expired,
}

impl CredentialStatus {
    /// Numeric representation stored on-chain: 0 = Active, 1 = Revoked.
    pub fn to_i32(status: &CredentialStatus) -> i32 {
        match status {
            CredentialStatus::Active => 0,
            CredentialStatus::Revoked => 1,
            CredentialStatus::Expired => 2,
        }
    }
}
