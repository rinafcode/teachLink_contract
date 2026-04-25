# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for TeachLink. An ADR captures an important architectural choice made during the project's evolution, along with the context, alternatives considered, and consequences.

## What is an ADR?

An ADR is a short document that records a significant technical decision. It is **not** a design document or a specification — it is a snapshot of _why_ something is the way it is, written at the time the decision was made.

ADRs are immutable once accepted. If a decision is reversed or superseded, the original ADR is marked accordingly and a new one is created.

## Index

| ID       | Title                                             | Status   | Date       |
|----------|---------------------------------------------------|----------|------------|
| ADR-0001 | Use Soroban (Rust) for Smart Contract Development | Accepted | 2024-01-15 |
| ADR-0002 | Tokenized Learning Rewards Architecture           | Accepted | 2024-02-10 |
| ADR-0003 | Cross-Chain Bridge Design                         | Accepted | 2024-03-05 |
| ADR-0004 | Indexer Technology Selection (TypeScript/NestJS)  | Accepted | 2024-03-20 |
| ADR-0005 | Observability Stack (Prometheus, Grafana)         | Accepted | 2024-04-08 |

> **New decision?** Copy [`template.md`](./template.md), assign the next sequential ID, fill it out, and open a PR. See the [Review Process](#review-process) section below.

---

## Directory Structure

```
docs/adr/
├── README.md          ← this file (index + process)
├── template.md        ← copy this when creating a new ADR
├── 0001-soroban-rust-smart-contracts.md
├── 0002-tokenized-learning-rewards.md
├── 0003-cross-chain-bridge-design.md
├── 0004-indexer-technology-selection.md
└── 0005-observability-stack.md
```

File names follow the pattern: `NNNN-kebab-case-title.md`

---

## Lifecycle

```
Proposed → Accepted → (optionally) Deprecated
                   → (optionally) Superseded by ADR-XXXX
```

| Status      | Meaning                                                               |
|-------------|-----------------------------------------------------------------------|
| Proposed    | Under discussion; not yet adopted                                     |
| Accepted    | Decision is in effect                                                 |
| Deprecated  | Decision is no longer relevant but the record is kept for history     |
| Superseded  | A newer ADR replaces this one; link to the replacement in the header  |

---

## Review Process

### Creating an ADR

1. Copy `template.md` to a new file: `NNNN-short-title.md`
2. Assign the next sequential ID (check the index table above)
3. Set status to `Proposed`
4. Fill out all required sections (Context, Decision, Alternatives, Consequences)
5. Open a PR and request review from at least one maintainer
6. Add the entry to the index table in this README in the same PR

### Review Criteria

Reviewers should verify that the ADR:

- Clearly separates the **problem** (Context) from the **solution** (Decision)
- Lists **at least two** alternatives with concrete rejection rationale
- Documents **both positive and negative** consequences
- Does **not** contain implementation instructions (those belong in code comments or separate docs)
- References the GitHub issue or PR that triggered the decision

### Merging

- An ADR may be merged once it has at least **one approving review** from a maintainer
- The status must be changed from `Proposed` to `Accepted` before or at merge time
- The index table in this README must be updated in the same PR

### Superseding an Existing ADR

1. Create the new ADR as `Proposed`
2. In the old ADR, change status to `Superseded by ADR-XXXX`
3. In the new ADR header, add `Supersedes: ADR-XXXX`
4. Update the index table

---

## Relationship to Other Documentation

| Document                         | Purpose                                               |
|----------------------------------|-------------------------------------------------------|
| `docs/ARCHITECTURE.md`           | High-level system overview and component diagrams     |
| `docs/NAMING_CONVENTIONS.md`     | Cross-module naming standards and enforcement         |
| `docs/adr/`                      | _Why_ the architecture looks the way it does          |
| `OBSERVABILITY.md`               | Monitoring and alerting runbook                       |
| `indexer/MONITORING.md`          | Indexer-specific monitoring details                   |

ADRs capture the **decision history** behind the architecture. For the current state of the system, refer to `docs/ARCHITECTURE.md`.