//! On-chain events emitted by the credential registry.

use soroban_sdk::{contractevent, Bytes, BytesN};

/// Emitted when a credential is issued.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Crediss {
    pub credential_hash: BytesN<32>,
    pub issuer_did: Bytes,
    pub subject_did: Bytes,
    pub metadata_ptr: Bytes,
    pub expires_at: i128,
}

/// Emitted when a credential is revoked.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Credrev {
    pub credential_hash: BytesN<32>,
    pub issuer_did: Bytes,
    pub subject_did: Bytes,
}
