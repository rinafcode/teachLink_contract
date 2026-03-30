//! TeachLink Credential Registry Contract
//!
//! On-chain registry for verifiable credentials (VCs).
//! Functionality is split across focused modules:
//!
//! - [`types`]       — credential status enum
//! - [`events`]      — on-chain event definitions
//! - [`errors`]      — error definitions
//! - [`credentials`] — issue, revoke, query, and status logic

#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env};

mod credentials;
mod errors;
mod events;
mod types;

pub use events::{Crediss, Credrev};
pub use types::CredentialStatus;

/// Credential registry contract — delegates to focused sub-modules.
#[contract]
pub struct CredentialRegistryContract;

#[contractimpl]
impl CredentialRegistryContract {
    /// Issue a credential by storing its hash and metadata pointer.
    ///
    /// `credential_hash` should be a deterministic hash (e.g. SHA-256) of the full VC JSON.
    pub fn issue_credential(
        env: &Env,
        credential_hash: BytesN<32>,
        issuer: Address,
        issuer_did: Bytes,
        subject_did: Bytes,
        metadata_ptr: Bytes,
        expires_at: i128,
    ) {
        credentials::CredentialManager::issue(
            env,
            credential_hash,
            issuer,
            issuer_did,
            subject_did,
            metadata_ptr,
            expires_at,
        );
    }

    /// Revoke a credential. Caller must be the original issuer.
    pub fn revoke_credential(env: &Env, credential_hash: BytesN<32>, issuer: Address) {
        credentials::CredentialManager::revoke(env, credential_hash, issuer);
    }

    /// Get a credential record: `(issuer_did, subject_did, metadata_ptr, expires_at, status)`.
    pub fn get_credential(
        env: &Env,
        credential_hash: BytesN<32>,
    ) -> Option<(Bytes, Bytes, Bytes, i128, i32)> {
        credentials::CredentialManager::get(env, credential_hash)
    }

    /// Check if a credential is active (not revoked and not expired).
    pub fn is_active(env: &Env, credential_hash: BytesN<32>, now_ts: i128) -> bool {
        credentials::CredentialManager::is_active(env, credential_hash, now_ts)
    }
}
