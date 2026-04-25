# ADR-0003: Cross-Chain Bridge Design

**Status:** Accepted  
**Date:** 2024-03-05  
**Authors:** [@rinafcode]  
**Reviewers:** [@rinafcode]  
**Tags:** contract, cross-chain, security

---

## Context

TeachLink operates on Stellar/Soroban, but learners and educators may hold assets or identities on other networks (EVM chains, in particular). To avoid locking users into a single chain, TeachLink needs a mechanism to bridge assets and credential signals between Stellar and external blockchains.

A bridge introduces significant security surface area: bridge contracts are among the most frequently exploited components in the broader blockchain ecosystem. The design must balance interoperability with security and auditability.

Key constraints:

- The bridge must not require a trusted single operator (single point of failure / centralization risk)
- Slashing conditions must exist to punish malicious or faulty validators
- The design must be extensible to additional chains beyond the initial target

## Decision

We will implement the bridge as a BFT (Byzantine Fault Tolerant) multi-validator consensus module within the TeachLink contract. The `bridge` module handles asset lock/unlock on the Stellar side, the `bft_consensus` module manages validator set and quorum logic, and the `slashing` module enforces penalties for misbehavior. Liquidity management across chains is handled by the `liquidity` module.

No single validator can unilaterally authorize a cross-chain transfer; a quorum signature is required.

## Alternatives Considered

| Alternative | Reason Rejected |
|-------------|-----------------|
| Use an existing third-party bridge (e.g., Wormhole, Axelar) | Introduces external dependency and trust assumptions on a third-party protocol; limits control over slashing and dispute resolution logic |
| Trusted relayer (single operator) | Single point of failure; compromised relayer key results in total bridge loss; unacceptable for a platform managing educator and learner funds |
| Optimistic bridge with fraud proofs | Introduces a challenge window (latency) that degrades UX for micro-transactions; more complex to implement correctly in Soroban's execution model |

## Consequences

### Positive

- BFT consensus eliminates single-point-of-failure risk at the bridge layer
- Slashing creates economic incentives for validators to behave correctly
- The modular design (`bridge`, `bft_consensus`, `slashing`, `multichain`, `liquidity`) allows each component to be tested and audited in isolation
- Extensible to additional chains by adding new chain adapters to the `multichain` module

### Negative / Trade-offs

- Building and maintaining a custom BFT consensus module is significantly more complex than delegating to a third-party bridge
- Validator set management (onboarding, rotation, slashing) requires ongoing operational overhead
- BFT consensus requires a minimum validator set size to be meaningful; a small or poorly distributed validator set reduces the security guarantees

### Neutral

- Cross-chain transfers have higher latency than native Stellar transactions due to the consensus round requirement; this is expected and documented for users

## Implementation Notes

**Affected modules:**

- `contracts/teachlink/bridge`
- `contracts/teachlink/bft_consensus`
- `contracts/teachlink/slashing`
- `contracts/teachlink/multichain`
- `contracts/teachlink/liquidity`

**Related issues / PRs:** #

---

## Review Checklist

- [x] Context accurately describes the problem without solution bias
- [x] Decision is stated clearly and unambiguously
- [x] At least two alternatives are documented with rejection rationale
- [x] Consequences cover both positive and negative outcomes
- [x] Status field is set correctly
- [x] Tags accurately reflect the domain
- [x] Linked to the relevant GitHub issue or PR