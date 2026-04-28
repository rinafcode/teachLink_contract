# ADR-0001: Use Soroban (Rust) for Smart Contract Development

**Status:** Accepted  
**Date:** 2024-01-15  
**Authors:** [@rinafcode]  
**Reviewers:** [@rinafcode]  
**Tags:** contract, infrastructure

---

## Context

TeachLink requires a smart contract platform to manage tokenized learning rewards, proof-of-participation records, and educator incentive payouts. The contract must be auditable, deterministic, and deployable on a network with low transaction fees suitable for micro-reward use cases.

Several smart contract platforms were available at the time of this decision. The choice of platform determines the programming language, toolchain, deployment target, and long-term maintenance burden.

Key constraints:

- Micro-transactions (reward payouts as small as fractions of a token) require negligible fees
- The team has existing experience with Rust systems programming
- The contract surface area is large (bridge, escrow, rewards, reputation, assessment modules), so type safety and compile-time correctness guarantees are important
- Long-term auditability matters; the contract code must be readable and verifiable by external reviewers

## Decision

We will write TeachLink's smart contracts in Rust targeting the Soroban platform on the Stellar network. All contract modules (bridge, escrow, tokenization, rewards, reputation, assessment, emergency, audit, analytics, reporting, backup) will be compiled to `wasm32-unknown-unknown` and deployed as a single Soroban contract.

## Alternatives Considered

| Alternative | Reason Rejected |
|-------------|-----------------|
| Solidity on EVM (Ethereum / Polygon) | Gas fees unsuitable for micro-reward transactions; EVM bytecode harder to audit than Rust source; team preference for Rust type system |
| Move on Aptos / Sui | Ecosystem too early at decision time; limited tooling and auditor familiarity; cross-chain bridge support was less mature |
| CosmWasm (Rust on Cosmos) | Viable technically, but Stellar's Horizon API and SEP standards provide better fiat on/off-ramp integrations needed for educator payouts |
| Ink! on Substrate | Strong Rust support, but Polkadot parachain deployment complexity was disproportionate to the project's scale |

## Consequences

### Positive

- Rust's type system eliminates entire classes of contract bugs at compile time
- `cargo test` and `cargo clippy` integrate naturally into CI, giving immediate feedback on contract correctness
- Soroban's WASM execution environment is deterministic and sandboxed
- Stellar's low fee model supports the micro-reward use cases central to TeachLink
- The `wasm32-unknown-unknown` target produces compact, verifiable artifacts

### Negative / Trade-offs

- Soroban is a relatively new platform; the ecosystem of third-party libraries and auditors is smaller than EVM
- Cross-chain bridge to EVM networks requires a custom bridge module, adding significant complexity
- Windows development requires additional setup (MSVC toolchain or Docker) due to WASM linker limitations on MinGW

### Neutral

- The `wasm32-unknown-unknown` compile target is separate from native test compilation; developers must be aware of the two-target workflow (`cargo build --target wasm32-unknown-unknown` vs `cargo test`)

## Implementation Notes

**Affected modules:**

- `contracts/teachlink/` (entire contract workspace)

**Build command:**

```bash
cargo build --release --target wasm32-unknown-unknown -p teachlink-contract
```

**Related issues / PRs:** Initial project setup

---

## Review Checklist

- [x] Context accurately describes the problem without solution bias
- [x] Decision is stated clearly and unambiguously
- [x] At least two alternatives are documented with rejection rationale
- [x] Consequences cover both positive and negative outcomes
- [x] Status field is set correctly
- [x] Tags accurately reflect the domain
- [x] Linked to the relevant GitHub issue or PR