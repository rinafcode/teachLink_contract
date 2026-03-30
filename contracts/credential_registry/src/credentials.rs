//! Core credential operations: issue, revoke, query, and status checks.

use soroban_sdk::{symbol_short, Address, Bytes, BytesN, Env};

use crate::events::{Crediss, Credrev};

/// A credential record stored on-chain:
/// `(issuer_did, subject_did, metadata_ptr, expires_at, status)`
type CredRecord = (Bytes, Bytes, Bytes, i128, i32);

pub struct CredentialManager;

impl CredentialManager {
    /// Issue a new credential.
    ///
    /// Panics if a credential with the same hash already exists.
    pub fn issue(
        env: &Env,
        credential_hash: BytesN<32>,
        issuer: Address,
        issuer_did: Bytes,
        subject_did: Bytes,
        metadata_ptr: Bytes,
        expires_at: i128,
    ) {
        issuer.require_auth();

        let key = (symbol_short!("cred"), credential_hash.clone());
        assert!(
            !env.storage().persistent().has(&key),
            "credential already exists"
        );

        let record: CredRecord = (
            issuer_did.clone(),
            subject_did.clone(),
            metadata_ptr.clone(),
            expires_at,
            0i32,
        );
        env.storage().persistent().set(&key, &record);

        Crediss {
            credential_hash,
            issuer_did,
            subject_did,
            metadata_ptr,
            expires_at,
        }
        .publish(env);
    }

    /// Revoke an existing credential.
    ///
    /// Panics if the credential does not exist.
    pub fn revoke(env: &Env, credential_hash: BytesN<32>, issuer: Address) {
        issuer.require_auth();

        let key = (symbol_short!("cred"), credential_hash.clone());
        let opt: Option<CredRecord> = env.storage().persistent().get(&key);

        match opt {
            Some((issuer_did, subject_did, metadata_ptr, expires_at, _)) => {
                let record: CredRecord =
                    (issuer_did.clone(), subject_did.clone(), metadata_ptr, expires_at, 1i32);
                env.storage().persistent().set(&key, &record);

                Credrev {
                    credential_hash,
                    issuer_did,
                    subject_did,
                }
                .publish(env);
            }
            None => panic!("credential not found"),
        }
    }

    /// Retrieve a raw credential record, or `None` if not found.
    pub fn get(env: &Env, credential_hash: BytesN<32>) -> Option<CredRecord> {
        let key = (symbol_short!("cred"), credential_hash);
        env.storage().persistent().get(&key)
    }

    /// Return `true` if the credential exists, is not revoked, and has not expired.
    pub fn is_active(env: &Env, credential_hash: BytesN<32>, now_ts: i128) -> bool {
        match Self::get(env, credential_hash) {
            Some((_, _, _, expires_at, status)) => {
                if status == 1 {
                    return false;
                }
                if expires_at > 0 && now_ts > expires_at {
                    return false;
                }
                true
            }
            None => false,
        }
    }
}
