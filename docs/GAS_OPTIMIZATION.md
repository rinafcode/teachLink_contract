# Gas Optimization Guide

This document outlines the gas optimization strategies used in the TeachLink Soroban smart contracts and provides guidance for maintaining high performance.

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

## 📊 Gas Benchmarking

We maintain a baseline of gas consumption for all major entrypoints in `gas_baseline.json`.

### Running Gas Reports
To generate a current gas report, run:
```bash
./scripts/gas-report.sh
```

### Thresholds
The `gas_thresholds.json` file defines the maximum allowable gas for each operation. Continuous Integration (CI) will fail if any operation exceeds its defined threshold.

## 📝 Best Practices for Developers

- **Pre-calculate**: Perform heavy computations off-chain if the result can be verified easily on-chain.
- **Event Logging**: Use events for data that doesn't need to be accessed by other contracts, as events are cheaper than persistent storage.
- **Minimize Contract Size**: Smaller WASM binaries reduce deployment costs and initialization overhead.
