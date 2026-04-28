# ADR-0004: Indexer Technology Selection (TypeScript / NestJS)

**Status:** Accepted  
**Date:** 2024-03-20  
**Authors:** [@rinafcode]  
**Reviewers:** [@rinafcode]  
**Tags:** indexer, infrastructure

---

## Context

The TeachLink smart contract emits Soroban events for all significant state changes (reward distributions, escrow releases, participation proofs, bridge transfers). These events are not directly queryable from within the contract or via simple RPC calls in a structured way. A separate indexer process is needed to subscribe to Soroban events, transform them, store them in a queryable database, and expose them to client applications via an API.

The indexer is an optional but practically required component for any production UI. It also hosts the recommendation and analytics features that are not feasible to run on-chain.

Key constraints:

- Must integrate with Stellar Horizon and Soroban RPC event subscription endpoints
- Must support long-running background processes (event listeners, retry logic)
- The team has more TypeScript experience than Go or Python for service development

## Decision

We will implement the indexer as a TypeScript/NestJS service. NestJS provides a structured module and dependency injection system suitable for a long-running service with multiple subsystems (event ingestion, storage, API, monitoring). The service subscribes to Soroban events via the Stellar SDK and stores processed data in a PostgreSQL database.

## Alternatives Considered

| Alternative | Reason Rejected |
|-------------|-----------------|
| Go service | Strong concurrency model, but team TypeScript familiarity is higher; NestJS module structure maps well to the multi-feature indexer design |
| Python (FastAPI / Celery) | Viable, but async event processing patterns in Python are less ergonomic than TypeScript; type safety weaker without strict mypy enforcement |
| The Graph protocol | Designed primarily for EVM chains; Soroban subgraph support was not mature at decision time |
| Rust (same language as contract) | Would maximize code sharing for type definitions, but NestJS ecosystem provides more out-of-the-box tooling for REST APIs, background jobs, and observability integration |

## Consequences

### Positive

- NestJS's module system naturally maps to TeachLink's multi-feature architecture (one module per contract feature domain)
- TypeScript's type system allows sharing schema types between indexer and frontend clients
- Large ecosystem of Stellar JavaScript/TypeScript SDKs (`@stellar/stellar-sdk`)
- NestJS integrates well with Prometheus metrics exporters used in the observability stack (ADR-0005)

### Negative / Trade-offs

- TypeScript/Node.js has higher memory overhead than Go or Rust for long-running processes
- The indexer introduces a separate runtime dependency; operators must manage both the contract deployment and the indexer service
- Schema drift between on-chain event payloads and indexer DTOs must be managed carefully (see `NAMING_CONVENTIONS.md`)

### Neutral

- The indexer is architecturally optional; the contract functions correctly without it, but the full product experience (learning history, analytics, recommendations) requires it

## Implementation Notes

**Affected modules:**

- `indexer/` (entire indexer workspace)

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