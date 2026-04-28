# Issue #334: Add Comprehensive Benchmark Test Suite

## Overview

The TeachLink contract currently lacks benchmark tests, making it impossible to measure or track performance characteristics across contract operations. This issue tracks the work needed to establish a full benchmark suite.

## Problem

Without benchmarks, there is no baseline for:
- Transaction throughput under load
- Storage read/write costs
- Cryptographic operation overhead
- Bulk operation performance

Performance regressions can go undetected, and optimization efforts have no measurable target.

## Acceptance Criteria

- [ ] **Benchmark transactions** — measure invocation cost and latency for core contract entry points
- [ ] **Test storage operations** — profile ledger reads and writes (get, put, remove) under varying data sizes
- [ ] **Profile crypto operations** — measure hashing, signature verification, and key derivation costs
- [ ] **Measure bulk operations** — benchmark loops, batch mints, and multi-recipient reward distributions

## Implementation Notes

Soroban contracts can be benchmarked using the `soroban-sdk` test environment with CPU and memory instruction counting:

```rust
#[test]
fn bench_reward_distribution() {
    let env = Env::default();
    env.budget().reset_default();
    // ... invoke contract function ...
    println!("CPU instructions: {}", env.budget().cpu_instruction_count());
    println!("Memory bytes: {}", env.budget().memory_bytes_count());
}
```

Results should be captured in CI and compared against a stored baseline to catch regressions.

## Labels

`testing` · `performance` · `benchmarking` · `Stellar Wave`
