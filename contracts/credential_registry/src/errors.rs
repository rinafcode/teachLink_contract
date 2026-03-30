//! Error definitions for the credential registry contract.

/// Errors that can occur during credential operations.
pub enum CredentialError {
    /// A credential with this hash already exists.
    AlreadyExists,
    /// No credential found for the given hash.
    NotFound,
}

impl CredentialError {
    pub fn message(&self) -> &'static str {
        match self {
            CredentialError::AlreadyExists => "credential already exists",
            CredentialError::NotFound => "credential not found",
        }
    }
}
