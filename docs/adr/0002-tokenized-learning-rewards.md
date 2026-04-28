# ADR-0002: Tokenized Learning Rewards Architecture

**Status:** Accepted  
**Date:** 2024-02-10  
**Authors:** [@rinafcode]  
**Reviewers:** [@rinafcode]  
**Tags:** contract, platform

---

## Context

TeachLink's core value proposition is rewarding learners and educators with on-chain tokens tied to verifiable learning activity. The contract must mint, distribute, and track reward tokens in a way that is transparent, resistant to manipulation, and useful as a credential signal (proof-of-participation).

Design questions at the time of this decision:

- Should rewards use a custom token standard or the Stellar Asset Contract (SAC)?
- Should participation proofs be stored on-chain or referenced via content-addressed hashes?
- How should educator incentives be structured relative to learner rewards to prevent gaming?

## Decision

We will implement the rewards module as a contract-native distribution system that issues reward tokens through the `rewards` module and records proof-of-participation as on-chain events (Soroban contract events) rather than storing full participation data in contract storage. Educator incentives will be handled through a separate escrow module that releases funds upon verified learner milestone completion, not upon content upload.

## Alternatives Considered

| Alternative | Reason Rejected |
|-------------|-----------------|
| Store full participation records in contract storage | Contract storage on Soroban is metered and expensive for large payloads; event-based proofs are sufficient for auditability and cheaper to emit |
| Educator rewards on content upload | Creates perverse incentive to flood the platform with low-quality content; milestone-based release ties educator payout to actual learner success |
| Use only Stellar Classic assets (no SAC / contract layer) | Lacks programmable distribution logic; cannot encode conditional releases or reputation-weighted rewards |

## Consequences

### Positive

- Proof-of-participation events are permanent, publicly verifiable, and indexable by the TypeScript indexer layer
- Escrow-based educator incentives align educator success with learner outcomes
- Keeping participation data off-chain storage (in events) keeps contract storage footprint small and predictable
- The separation of `rewards`, `escrow`, and `tokenization` modules allows each to be upgraded or audited independently

### Negative / Trade-offs

- Soroban events are not queryable from within the contract itself; external consumers (indexer) must subscribe and store event history for queries
- Escrow release logic requires reliable milestone verification; an oracle or off-chain verifier must signal completion, introducing a trust assumption

### Neutral

- The event-based participation record means the indexer is a required component for any UI that displays learning history; the contract alone is not sufficient for a full product experience

## Implementation Notes

**Affected modules:**

- `contracts/teachlink/rewards`
- `contracts/teachlink/escrow`
- `contracts/teachlink/tokenization`
- `contracts/teachlink/reputation`
- `indexer/` (event subscription and storage)

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