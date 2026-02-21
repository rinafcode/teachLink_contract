#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, BytesN, Env};

#[contract]
pub struct CredentialRegistryContract;

#[derive(Clone)]
pub enum CredentialStatus {
    Active,
    Revoked,
    Expired,
}

#[contractimpl]
impl CredentialRegistryContract {
    // Issue a credential by storing its hash and metadata pointer.
    // `credential_hash` should be a deterministic hash (e.g., SHA-256) of the full VC JSON.
    pub fn issue_credential(
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
        let record: (Bytes, Bytes, Bytes, i128, i32) = (
            issuer_did.clone(),
            subject_did.clone(),
            metadata_ptr.clone(),
            expires_at,
            0i32,
        );
        env.storage().persistent().set(&key, &record);
        env.events().publish(
            (symbol_short!("crediss"),),
            (
                credential_hash,
                issuer_did,
                subject_did,
                metadata_ptr,
                expires_at,
            ),
        );
    }

    // Revoke a credential. Caller must be issuer (signed address)
    pub fn revoke_credential(env: &Env, credential_hash: BytesN<32>, issuer: Address) {
        issuer.require_auth();
        let key = (symbol_short!("cred"), credential_hash.clone());
        let opt: Option<(Bytes, Bytes, Bytes, i128, i32)> = env.storage().persistent().get(&key);
        match opt {
            Some((issuer_did, subject_did, metadata_ptr, expires_at, _status)) => {
                let record: (Bytes, Bytes, Bytes, i128, i32) = (
                    issuer_did.clone(),
                    subject_did.clone(),
                    metadata_ptr.clone(),
                    expires_at,
                    1i32,
                );
                env.storage().persistent().set(&key, &record);
                env.events().publish(
                    (symbol_short!("credrev"),),
                    (credential_hash, issuer_did, subject_did),
                );
            }
            None => panic!("credential not found"),
        }
    }

    // Get credential record: returns (issuer_did, subject_did, metadata_ptr, expires_at, status)
    pub fn get_credential(
        env: &Env,
        credential_hash: BytesN<32>,
    ) -> Option<(Bytes, Bytes, Bytes, i128, i32)> {
        let key = (symbol_short!("cred"), credential_hash.clone());
        env.storage().persistent().get(&key)
    }

    // Check if credential is active (not revoked and not expired)
    pub fn is_active(env: &Env, credential_hash: BytesN<32>, now_ts: i128) -> bool {
        match Self::get_credential(env, credential_hash.clone()) {
            Some((_issuer, _subject, _meta, expires_at, status)) => {
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

fn main() {}
