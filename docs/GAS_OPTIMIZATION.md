# Gas Optimization Guide

This document outlines the gas optimization strategies used in the TeachLink Soroban smart contracts and provides guidance for maintaining high performance.

## Scope and Source of Truth

- Gas estimates below are derived from `gas_thresholds.json` and treated as planning budgets.
- Actual runtime usage should be measured with benchmark tests and compared against `gas_baseline.json`.
- CI regression policy currently fails on increases above 10% over baseline and warns above 5%.

## ⛽ Understanding Gas in Soroban

Soroban uses a resource-based metering system. Gas consumption is influenced by:
- **CPU Instructions**: The complexity of the logic.
- **Ledger Reads/Writes**: The size and frequency of state access.
- **Memory Usage**: The transient memory footprint during execution.

## 🛠️ Optimization Strategies

### 1. Efficient Data Structures
- Use `Map` and `Vec` judiciously. Prefer fixed-size structures when possible.
- Avoid nesting large collections inside state objects.
- Use `Symbol` instead of `String` for keys and identifiers where appropriate.

### 2. State Access Optimization
- **Batching**: Group related state updates to minimize ledger writes.
- **Laziness**: Only read data from the ledger when it is strictly necessary for the current execution path.
- **Storage Types**: Use `Temporary`, `Instance`, and `Persistent` storage appropriately to manage rent costs and lifecycle.

### 3. Loop Minimization
- Avoid O(n) operations over large collections in contract entrypoints.
- Use pagination or indexing patterns for retrieving large datasets.

## 📦 Per-Operation Cost Estimates

The following instruction and memory budgets are maintained as operational guardrails.

### Contract Operations

| Operation | Max Instructions | Max Memory (bytes) | Notes |
|---|---:|---:|---|
| `initialize` | 500,000 | 50,000 | Contract initialization |
| `add_validator` | 200,000 | 10,000 | Add validator to bridge |
| `add_supported_chain` | 200,000 | 10,000 | Register destination chain |
| `set_bridge_fee` | 150,000 | 5,000 | Update bridge fee |
| `read_query` | 100,000 | 5,000 | Read-only getters |

### Bridge and Cache Operations

| Operation | Max Instructions | Max Memory (bytes) | Notes |
|---|---:|---:|---|
| `bridge_out` | 800,000 | 50,000 | Lock tokens for bridge |
| `complete_bridge` | 1,000,000 | 80,000 | Finalize with signatures |
| `cancel_bridge` | 500,000 | 30,000 | Cancel pending bridge |
| `cache_hit` | 200,000 | 20,000 | Read fresh cached summary |
| `cache_miss_compute` | 1,500,000 | 100,000 | Recompute summary |
| `cache_invalidation` | 150,000 | 5,000 | Invalidate cache entry |

### Consensus and Tokenization

| Operation | Max Instructions | Max Memory (bytes) | Notes |
|---|---:|---:|---|
| `register_validator` | 500,000 | 30,000 | BFT validator registration |
| `create_proposal` | 400,000 | 30,000 | Consensus proposal creation |
| `vote_on_proposal` | 300,000 | 20,000 | Proposal voting |
| `mint_content_token` | 600,000 | 50,000 | NFT mint |
| `transfer_content_token` | 500,000 | 30,000 | NFT transfer |
| `update_metadata` | 400,000 | 30,000 | Metadata update |

### Rewards, Messaging, and Supporting Modules

| Operation | Max Instructions | Max Memory (bytes) | Notes |
|---|---:|---:|---|
| `initialize_rewards` | 400,000 | 20,000 | Rewards setup |
| `fund_reward_pool` | 500,000 | 20,000 | Add pool funds |
| `issue_reward` | 400,000 | 20,000 | Issue reward |
| `claim_rewards` | 600,000 | 30,000 | Claim pending rewards |
| `send_packet` | 700,000 | 50,000 | Cross-chain packet send |
| `deliver_packet` | 500,000 | 30,000 | Mark packet delivered |
| `send_notification` | 350,000 | 20,000 | Notification emit |
| `initialize_mobile_profile` | 400,000 | 30,000 | Mobile profile init |
| `create_audit_record` | 400,000 | 20,000 | Audit record write |

### Deployment Size Costs

| Metric | Limit |
|---|---:|
| WASM warning size | 256,000 bytes |
| WASM hard limit | 307,200 bytes |

## 📊 Gas Benchmarking

We maintain a baseline of gas consumption for all major entrypoints in `gas_baseline.json`.

### Running Gas Reports
To generate a current gas report, run:
```bash
./scripts/gas-report.sh
```

### Thresholds
The `gas_thresholds.json` file defines the maximum allowable gas for each operation. Continuous Integration (CI) will fail if any operation exceeds its defined threshold.

### Interpretation Guide

- A benchmark passing threshold does not guarantee optimality; compare against the previous baseline and module complexity.
- Prioritize optimization work for high-frequency operations first (for example, read queries and bridge completion).
- For regressions, inspect both instruction growth and memory growth. Memory spikes often indicate avoidable temporary allocations.

## ✅ Best Practices and Optimization Tips

- Keep storage access local and minimal: read once, mutate in-memory, write once.
- Use compact types for persisted state and avoid unbounded vectors in hot paths.
- Split expensive workflows into staged transactions where correctness allows it.
- Prefer deterministic branching and avoid deep nested conditionals in high-traffic entrypoints.
- Emit events for observability data that does not need on-chain queryability.
- Measure after every meaningful refactor with `scripts/run_gas_benchmarks.py` and update baselines only after review.

## 📝 Best Practices for Developers

- **Pre-calculate**: Perform heavy computations off-chain if the result can be verified easily on-chain.
- **Event Logging**: Use events for data that doesn't need to be accessed by other contracts, as events are cheaper than persistent storage.
- **Minimize Contract Size**: Smaller WASM binaries reduce deployment costs and initialization overhead.
