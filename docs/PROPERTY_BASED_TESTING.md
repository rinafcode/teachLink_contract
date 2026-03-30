# Property-Based Testing Documentation

## Overview

This document describes the comprehensive property-based testing implementation for the teachLink contract. Property-based testing complements traditional unit tests by verifying that algorithms maintain their essential properties across a wide range of inputs, helping catch edge cases and ensure mathematical correctness.

## What is Property-Based Testing?

Property-based testing is a testing methodology where instead of testing specific examples, we test properties that should hold true for all valid inputs. This approach:

- **Discovers edge cases** that traditional testing might miss
- **Verifies mathematical invariants** in complex algorithms
- **Provides confidence** in algorithmic correctness
- **Catches regression bugs** early

## Implementation Architecture

### Test Frameworks Used

1. **Proptest** - Primary property-based testing framework
   - Generates random but structured test data
   - Shrinks failing cases to minimal examples
   - Configurable test case counts and strategies

2. **QuickCheck** - Fuzzing and random testing
   - Lightweight fuzzing for input validation
   - Arbitrary trait implementations for custom types
   - High-performance random testing

### Test Structure

```
contracts/teachlink/src/
├── property_based_tests.rs    # Core property-based test implementations
└── lib.rs                    # Module exports

contracts/teachlink/tests/
└── property_based_tests.rs    # Test runners and integration tests

scripts/
├── run_property_tests.sh      # Bash automation script
└── run_property_tests.ps1    # PowerShell automation script
```

## Algorithm Properties Tested

### 1. BFT Consensus Algorithm

#### Properties Verified

**Byzantine Threshold Safety**
```rust
// For n validators, threshold = floor(2n/3) + 1
let threshold = (2 * n_validators) / 3 + 1;
assert!(threshold <= n_validators);  // Never exceeds total
assert!(threshold > n_validators - faulty_tolerance);  // Protects against faulty nodes
```

**Consensus State Consistency**
- Total stake remains non-negative during validator operations
- Active validator count stays within expected bounds
- State transitions maintain invariants

**Proposal Voting Properties**
- If votes ≥ threshold, consensus is reached
- Threshold is always positive and reasonable
- Vote counting is monotonic

#### Test Cases
- Validator counts: 1-100
- Various operation sequences (add/remove validators)
- Different voting scenarios
- Edge cases (minimum validators, maximum thresholds)

### 2. Assessment System

#### Properties Verified

**Score Calculation Bounds**
```rust
let earned = correct_answers * points_per_question;
let total_possible = total_questions * points_per_question;
assert!(earned <= total_possible);  // Score never exceeds maximum
assert!(percentage <= 100);        // Percentage bounded by 100
```

**Adaptive Difficulty Monotonicity**
- Higher performance → higher or equal difficulty
- Lower performance → lower or equal difficulty
- Difficulty always within valid range (1-10)

**Plagiarism Detection Threshold**
- 100% match always detected as plagiarism
- 0% match never detected as plagiarism
- 90% threshold consistency

#### Test Cases
- Question counts: 1-100
- Point values: 1-10
- Performance ratios: 0-100%
- Match percentages: 0-100%

### 3. Analytics Calculations

#### Properties Verified

**Moving Average Convergence**
- EMA stays within min-max range of input values
- EMA converges toward recent values
- No overflow or underflow conditions

**Health Score Bounds**
- Health score always between 0-100
- Perfect inputs → high health score (≥90)
- Zero inputs → low health score (≤20)
- Weighted average consistency

**Volume Tracking Consistency**
- Total volume equals sum of all transactions
- Transaction count matches number of operations
- Average calculation correctness

#### Test Cases
- Time series data: 1-1000 points
- Various performance metrics
- Large volume numbers (stress testing)
- Edge cases (empty data, single points)

### 4. Atomic Swap Operations

#### Properties Verified

**Timelock Bounds**
- Valid timelocks: 1 hour to 7 days
- Invalid timelocks rejected
- Boundary condition handling

**Hash Verification Consistency**
- Same preimage → same hash (deterministic)
- Different preimages → different hashes (collision resistance)
- Hash length consistency (32 bytes)

**Swap Rate Calculation**
- Rate always non-negative
- Inverse proportionality to initiator amount
- Direct proportionality to counterparty amount

**State Transitions**
- Initiated → Completed/Refunded/Expired
- Final states don't transition further
- Valid state machine behavior

#### Test Cases
- Timelock values: 0-1,000,000 seconds
- Hash inputs: 32-byte arrays
- Amount ranges: 1-1,000,000 tokens
- State transition sequences

### 5. Input Validation Fuzzing

#### Properties Verified

**Address Validation**
- Valid length (3-64 characters) accepted
- Empty or too long addresses rejected
- Character set validation

**Amount Validation**
- Positive amounts accepted
- Zero or negative amounts rejected
- Large number handling

**Hash Length Validation**
- Exactly 32 bytes accepted
- Other lengths rejected
- Empty hash handling

**Question Difficulty Bounds**
- Valid range (1-10) accepted
- Out of range values rejected
- Boundary condition testing

## Test Configuration

### Proptest Configuration

```rust
let mut config = ProptestConfig::default();
config.cases = 100;              // Standard test cases
config.max_shrink_iters = 1000;   // Shrinking iterations
```

### QuickCheck Configuration

```rust
QuickCheck::new()
    .tests(1000)     // Number of random tests
    .quickcheck(prop_fn);
```

### Environment Variables

- `PROPTEST_CASES`: Override number of test cases
- `PROPTEST_SEED`: Random seed for reproducibility
- `QUICKCHECK_TESTS`: Number of fuzzing iterations

## Automation Scripts

### Bash Script (`run_property_tests.sh`)

```bash
./scripts/run_property_tests.sh
```

Features:
- Runs all property-based test suites
- Generates coverage reports
- Extended testing with random seeds
- Performance benchmarking
- Colored output and error handling

### PowerShell Script (`run_property_tests.ps1`)

```powershell
.\scripts\run_property_tests.ps1
```

Features:
- Cross-platform Windows support
- Verbose mode option
- Coverage generation toggle
- Configurable test parameters

## Running Tests

### Standard Property Tests

```bash
# Run all property-based tests
cargo test --test property_based_tests --features testutils --release

# Run specific test modules
cargo test test_bft_threshold_properties --features testutils --release
cargo test test_assessment_properties --features testutils --release
```

### Fuzzing Tests

```bash
# Run QuickCheck fuzzing
cargo test test_quickcheck_fuzzing --features testutils --release

# Extended fuzzing
QUICKCHECK_TESTS=10000 cargo test test_quickcheck_fuzzing --features testutils --release
```

### Stress Testing

```bash
# Large input testing
cargo test test_stress_large_inputs --features testutils --release

# Edge case testing
cargo test test_edge_cases --features testutils --release
```

### Coverage Generation

```bash
# Install coverage tool
cargo install cargo-llvm-cov

# Generate coverage
cargo llvm-cov --features testutils --release --lcov --output-path lcov.info
```

## Invariants Documented

### BFT Consensus Invariants

1. **Safety**: No two honest validators decide different values
2. **Liveness**: Eventually, all honest validators decide on some value
3. **Threshold**: `threshold = floor(2n/3) + 1` guarantees BFT properties
4. **Monotonicity**: Adding validators can only increase threshold
5. **Boundedness**: Threshold never exceeds total validators

### Assessment System Invariants

1. **Score Bounds**: `0 ≤ score ≤ total_possible`
2. **Percentage Bounds**: `0 ≤ percentage ≤ 100`
3. **Difficulty Range**: `1 ≤ difficulty ≤ 10`
4. **Adaptation Monotonicity**: Performance correlates with difficulty
5. **Plagiarism Threshold**: `match_count > 0.9 * total_questions` triggers detection

### Analytics Invariants

1. **EMA Range**: `min(values) ≤ EMA ≤ max(values)`
2. **Health Score Range**: `0 ≤ health_score ≤ 100`
3. **Volume Conservation**: Total volume equals sum of transactions
4. **Average Consistency**: `average = sum / count` for count > 0
5. **Rate Non-negativity**: All calculated rates ≥ 0

### Atomic Swap Invariants

1. **Timelock Range**: `1 hour ≤ timelock ≤ 7 days`
2. **Hash Determinism**: Same input always produces same hash
3. **Hash Length**: All hashes are exactly 32 bytes
4. **Rate Non-negativity**: Swap rates are never negative
5. **State Machine**: Valid transitions only between defined states

## Edge Cases Covered

### Numerical Edge Cases

- Zero values (amounts, counts, stakes)
- Maximum values (u64::MAX, i128::MAX)
- Boundary conditions (just at limits)
- Overflow/underflow prevention

### Algorithm Edge Cases

- Minimum validator sets (1, 2, 3)
- Empty collections (no questions, no transactions)
- Single-element operations
- Maximum capacity scenarios

### Input Validation Edge Cases

- Empty strings and arrays
- Maximum length inputs
- Invalid character sequences
- Malformed data structures

## Performance Considerations

### Test Execution Time

- Standard property tests: ~30 seconds
- Extended testing: ~2-5 minutes
- Fuzzing tests: ~1-2 minutes
- Stress tests: ~30 seconds

### Memory Usage

- Proptest generates up to 100 test cases per property
- QuickCheck uses configurable iteration counts
- Large input tests bounded to prevent memory issues

### Optimization Strategies

- Use `--release` mode for faster execution
- Parallel test execution with cargo-nextest
- Configurable test case counts for CI/CD

## Integration with CI/CD

### GitHub Actions Example

```yaml
- name: Run Property-Based Tests
  run: |
    ./scripts/run_property_tests.sh
    cargo llvm-cov --features testutils --release --lcov --output-path lcov.info
```

### Coverage Requirements

- Minimum coverage: 80% for tested modules
- Critical paths: 95% coverage
- Property tests: 100% of invariants covered

## Best Practices

### Writing Property Tests

1. **Identify Essential Properties**: Focus on mathematical invariants
2. **Use Appropriate Strategies**: Match data generation to domain
3. **Test Edge Cases**: Include boundary conditions
4. **Provide Good Shrinking**: Enable minimal failure examples
5. **Document Invariants**: Clear property descriptions

### Test Data Generation

1. **Realistic Ranges**: Use domain-appropriate value ranges
2. **Avoid Impossible Cases**: Filter out invalid combinations
3. **Include Edge Values**: Boundary conditions in strategies
4. **Vary Input Sizes**: Test different scales

### Maintenance

1. **Regular Updates**: Keep tests in sync with code changes
2. **Coverage Monitoring**: Track property test coverage
3. **Performance Monitoring**: Watch test execution times
4. **Failure Analysis**: Investigate property test failures promptly

## Troubleshooting

### Common Issues

1. **Non-deterministic Failures**: Use fixed seeds for reproduction
2. **Performance Issues**: Reduce test case counts or optimize strategies
3. **Memory Issues**: Limit input sizes or use streaming
4. **False Positives**: Review property definitions

### Debugging Tips

1. **Use Shrinking**: Let proptest find minimal failing cases
2. **Fixed Seeds**: Reproduce failures consistently
3. **Verbose Output**: Enable detailed logging
4. **Isolate Properties**: Test individual properties separately

## Future Enhancements

### Planned Improvements

1. **Model-Based Testing**: State machine testing for complex workflows
2. **Contract-Level Testing**: Full smart contract property testing
3. **Cross-Chain Properties**: Multi-chain interaction invariants
4. **Performance Properties**: Latency and throughput guarantees
5. **Security Properties**: Attack resistance verification

### Additional Algorithms

1. **Liquidity Pool Properties**: AMM mathematical invariants
2. **Reward Distribution**: Fairness and conservation properties
3. **Reputation System**: Score calculation consistency
4. **Message Passing**: Delivery guarantee properties

## Conclusion

Property-based testing provides a powerful complement to traditional testing approaches for the teachLink contract. By focusing on mathematical invariants and algorithmic properties, we can achieve higher confidence in the correctness and robustness of complex financial and consensus algorithms.

Regular execution of property-based tests helps catch edge cases, prevents regressions, and ensures that the contract maintains its essential properties across all valid inputs and conditions.

For questions or contributions to the property-based testing suite, please refer to the [Contributing Guidelines](../CONTRIBUTING.md) and open an issue or pull request.
