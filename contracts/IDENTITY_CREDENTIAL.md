**Identity & Credential Contracts (Soroban)**

Overview
- `identity-registry`: on-chain DID registry. Stores DID -> controller, auth methods, recovery address.
- `credential-registry`: on-chain credential index. Stores credential hash -> (issuer DID, subject DID, metadata pointer, expires_at, status).

Key on-chain guarantees
- Deterministic verification can check presence and status of a credential on-chain.
- Full VC JSON and ZK proofs remain off-chain; only hashes/roots and status bits stored on-chain.

Next steps / integration notes
- Wire `credential-registry` to call `identity-registry` for authoritative issuer controller checks.
- Add Merkle/bitmap-based revocation root support for efficient revocation proofs.
- Implement cross-contract calls and auth to allow DID controllers (not raw addresses) to issue/revoke.
- Add off-chain ZK proof verifier support: store verification circuits' commitment roots on-chain and provide helper APIs for verifiers.
- Marketplace, federation, selective-disclosure circuits, and biometric-binding are implemented off-chain; contracts store anchors/roots.

Files added
- `contracts/identity_registry` — Cargo + src/lib.rs
- `contracts/credential_registry` — Cargo + src/lib.rs

Testing & build
- Use the workspace's soroban toolchain and existing patterns (see other `contracts/*` crates) to build and test.
