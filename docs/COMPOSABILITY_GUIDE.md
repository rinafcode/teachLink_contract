# Composability Guide

TeachLink is designed to be a foundational layer for decentralized education. This guide explains how other contracts and dApps can integrate with and build upon the TeachLink ecosystem.

## 🧱 Modular Architecture

TeachLink exposes a set of standard interfaces that allow for seamless integration:

- **Identity & Reputation**: Query learner profiles and verify credentials.
- **Rewards Engine**: Integrate custom reward tokens or payout logic.
- **Escrow Services**: Use TeachLink's secure escrow for course payments.

## 🔌 Integration Patterns

### 1. Cross-Contract Calls
Other Soroban contracts can call TeachLink functions by importing the `TeachLinkContract` interface.

```rust
// Example: A custom course contract calling TeachLink to reward a student
let teachlink_client = TeachLinkClient::new(&env, &teachlink_id);
teachlink_client.reward_learner(&student, &course_id, &amount);
```

### 2. Event Hooks
dApps can listen for TeachLink events (e.g., `CourseCompleted`, `CredentialIssued`) to trigger external actions like sending emails or updating off-chain databases.

### 3. Permissionless Extensions
Developers can build "wrappers" around TeachLink to add custom logic, such as:
- **DAO Governance**: Voting on course quality.
- **Yield Farming**: Staking rewards for educators.

## 📜 Interface Stability

We adhere to strict versioning. Breaking changes to public interfaces are announced in advance and documented in `docs/versions/`.

## 🛠️ Developer Resources

- **API Reference**: Detailed documentation of all entrypoints in [API_REFERENCE.md](API_REFERENCE.md).
- **Testnet Address**: `CC...` (Update upon deployment).
- **SDKs**: (Coming Soon) TypeScript and Python wrappers for TeachLink.
