# Performance & Regression Testing

This document describes the performance testing suite and regression detection mechanisms for TeachLink.

## 🧪 Testing Overview

TeachLink employs a multi-layered approach to performance testing:

1. **Gas Profiling**: Measuring the gas cost of every contract entrypoint.
2. **Benchmarking**: Using `cargo bench` to measure CPU execution time for critical algorithms.
3. **Load Testing**: Simulating high volumes of transactions to identify bottlenecks.

## 📈 Regression Detection

We use a "Gold Master" approach for gas costs. The `gas_baseline.json` file stores the "known good" gas costs for the current main branch.

### How it works:
1. Every Pull Request triggers a gas profile run.
2. The results are compared against `gas_baseline.json`.
3. If gas usage increases beyond a 5% margin (configurable), the CI build fails with a "Performance Regression" warning.

## 🚀 Running Tests Locally

### Gas Profiling
```bash
./scripts/test-gas.sh
```

### Micro-benchmarks
```bash
cargo bench
```

## 📊 Interpreting Results

Results are outputted in both human-readable (terminal) and machine-readable (JSON) formats.

| Metric | Tool | Goal |
|--------|------|------|
| Gas Cost | Soroban CLI | Stay under threshold |
| Execution Time | Criterion.rs | Minimize ms/op |
| Memory | Valgrind (WASM) | Avoid leaks/bloat |

## 🛠️ Adding New Performance Tests

When adding a new contract feature:
1. Create a corresponding benchmark in the `benches/` directory.
2. Add the initial gas baseline to `gas_baseline.json`.
3. Ensure the feature doesn't negatively impact existing performance metrics.
