# TeachLink Error Handling - Step-by-Step Testing Guide

## Overview

This guide provides a comprehensive step-by-step process to verify that the consistent error handling implementation (#364) has been successfully completed across the TeachLink smart contract.

---

## Part 1: Compile and Build Verification

### Step 1.1: Clean Build

```bash
cd contracts/teachlink
./scripts/clean.sh --deep
```

**Expected Result:** Build artifacts are removed successfully.

### Step 1.2: Build Release WASM

```bash
cargo build --release --target wasm32-unknown-unknown -p teachlink-contract
```

**Expected Result:**

- Build completes WITHOUT errors
- No warnings about `unwrap()`, `panic!`, or `expect()` in production code
- WASM file generated at: `target/wasm32-unknown-unknown/release/teachlink_contract.wasm`

**Verification Checklist:**

- [ ] No compilation errors
- [ ] No new warnings introduced
- [ ] WASM file exists and is valid

---

## Part 2: Static Analysis Verification

### Step 2.1: Check for Remaining Unsafe Patterns

```bash
# Check for unwrap() in production code
grep -r "\.unwrap()" contracts/teachlink/src/ --include="*.rs" | grep -v "#\[cfg(test)\]" | grep -v "//.*unwrap" | grep -v "test/"
```

**Expected Result:**

- Only entries showing `.unwrap_or()` with defaults (acceptable)
- No `.unwrap()` without fallback
- No entries from production code paths

**Verification:** If any `.unwrap()` found in production code, FAIL.

### Step 2.2: Check for Remaining Panics

```bash
# Check for panic! and assert! in production code
grep -rE "panic!|assert!|expect\(" contracts/teachlink/src/ --include="*.rs" | grep -v "#\[cfg(test)\]" | grep -v "//.*" | grep -v "test/"
```

**Expected Result:**

- No matches in production code
- `assert!()` only in test code (#[cfg(test)])
- `panic!()` only in test code or authorization
- `expect()` only in test code

**Verification:** If panic/assert found in production paths, FAIL.

### Step 2.3: Verify Result Types

```bash
# Check that error-prone functions return Result
grep -rE "pub fn|pub async fn" contracts/teachlink/src/[^t]*.rs | \
  grep -E "(analytics|access_control|reputation|bridge)" --include="*.rs"
```

**Expected Result:** Functions that can fail should return `Result<T, E>`:

- `update_participation: Result<(), ReputationError>`
- `update_course_progress: Result<(), ReputationError>`
- `rate_contribution: Result<(), ReputationError>`
- `get_top_chains_by_volume: Result<Vec<_>, AnalyticsError>`
- `get_top_chains_by_volume_bounded: Result<Vec<_>, AnalyticsError>`
- `check_role: Result<(), AccessControlError>`
- `get_token: Result<Address, BridgeError>`
- `get_admin: Result<Address, BridgeError>`

---

## Part 3: Error Enum Verification

### Step 3.1: Verify All Error Enums Exist

```bash
grep -E "#\[contracterror\]|pub enum.*Error" contracts/teachlink/src/errors.rs
```

**Expected Result:** The following error enums should exist:

- [ ] `BridgeError` (codes 100-147)
- [ ] `EscrowError` (codes 200-227)
- [ ] `RewardsError` (codes 300-309)
- [ ] `MobilePlatformError` (codes 400-407)
- [ ] `AccessControlError` (codes 500-505)
- [ ] `AnalyticsError` (codes 510-514)
- [ ] `ReputationError` (codes 520-525)
- [ ] `TokenizationError` (codes 530-536)
- [ ] `AdvancedReputationError` (codes 540-544)

### Step 3.2: Verify Result Type Aliases

```bash
grep "pub type.*Result<T>" contracts/teachlink/src/errors.rs
```

**Expected Result:** All Result type aliases defined:

- [ ] `BridgeResult<T>`
- [ ] `EscrowResult<T>`
- [ ] `RewardsResult<T>`
- [ ] `AccessControlResult<T>`
- [ ] `AnalyticsResult<T>`
- [ ] `ReputationResult<T>`
- [ ] `TokenizationResult<T>`
- [ ] `AdvancedReputationResult<T>`

### Step 3.3: Verify Error Code Ranges

```bash
awk '/^#\[contracterror\]/,/^}/' contracts/teachlink/src/errors.rs | \
  grep -E "= [0-9]+" | sort -t'=' -k2 -n
```

**Expected Result:** Error codes are:

- [ ] Unique (no duplicates)
- [ ] In correct ranges (no overlaps)
- [ ] Each error has meaningful variant name

---

## Part 4: Module-Specific Verification

### Step 4.1: Analytics Module

```bash
# Check analytics.rs for error handling
grep -n "get_top_chains_by_volume" contracts/teachlink/src/analytics.rs
```

**Expected Result:**

- Line ~340: `pub fn get_top_chains_by_volume_bounded(...) -> AnalyticsResult<Vec<...>>`
- Line ~385: `pub fn get_top_chains_by_volume(...) -> AnalyticsResult<Vec<...>>`
- Both functions return `Result` types
- No `.unwrap()` in sorting code

**Specific Test:**

```bash
# Verify no unwrap in sorting loops
sed -n '340,380p' contracts/teachlink/src/analytics.rs | grep "unwrap"
```

**Expected:** No unwrap found

### Step 4.2: Access Control Module

```bash
grep -n "check_role" contracts/teachlink/src/access_control.rs
```

**Expected Result:**

- Line ~36: `pub fn check_role(...) -> AccessControlResult<()>`
- Returns `Result` instead of `void`
- Error returned: `Err(AccessControlError::MissingRole)`
- No `panic!()` call

**Verification:**

```bash
sed -n '36,42p' contracts/teachlink/src/access_control.rs | grep -c "panic"
```

**Expected:** 0 panics found

### Step 4.3: Reputation Module

```bash
# Check all three functions return Result
grep -E "pub fn (update_participation|update_course_progress|rate_contribution)" \
  contracts/teachlink/src/reputation.rs
```

**Expected Result:** All three functions signature should show:

- `-> ReputationResult<()>`

**Verify no assert!:**

```bash
grep -c "assert!" contracts/teachlink/src/reputation.rs
```

**Expected:** Should be 0 or only in test code

### Step 4.4: Bridge Module

```bash
# Check getter functions
grep -A2 "pub fn get_token\|pub fn get_admin" contracts/teachlink/src/bridge.rs
```

**Expected Result:**

- Both return `Result<Address, BridgeError>`
- No `.unwrap()` calls
- Proper error conversion: `.map_err(|_| BridgeError::StorageError)`

---

## Part 5: Test Compilation and Execution

### Step 5.1: Run Unit Tests

```bash
cargo test --lib -p teachlink-contract
```

**Expected Result:**

- [ ] All tests pass
- [ ] No test panics
- [ ] Error handling tests compile

### Step 5.2: Run Integration Tests

```bash
cargo test --test error_handling_integration -p teachlink-contract
```

**Expected Result:**

- [ ] Test file compiles without errors
- [ ] All integration tests pass
- [ ] Compilation successful (dummy test passes)

### Step 5.3: Check Test Output for Error Cases

```bash
cargo test --lib -- --nocapture 2>&1 | grep -i "error\|result"
```

**Expected Result:**

- Error handling is properly tested
- No unexpected panics in test output
- Result types are properly used in tests

---

## Part 6: Code Review Verification

### Step 6.1: Analytics Refactor Review

**File:** `contracts/teachlink/src/analytics.rs`

Checklist:

- [ ] Imports include: `use crate::errors::{AnalyticsError, AnalyticsResult, ...}`
- [ ] `get_top_chains_by_volume_bounded` returns `AnalyticsResult<Vec<(u32, i128)>>`
- [ ] `get_top_chains_by_volume` returns `AnalyticsResult<Vec<(u32, i128)>>`
- [ ] Sorting code uses `.ok_or(AnalyticsError::InvalidIndex)?` instead of `.unwrap()`
- [ ] All Vec access properly handles errors

### Step 6.2: Access Control Refactor Review

**File:** `contracts/teachlink/src/access_control.rs`

Checklist:

- [ ] Imports include: `use crate::errors::{AccessControlError, AccessControlResult, ...}`
- [ ] `check_role` function returns `AccessControlResult<()>`
- [ ] No `panic!()` macro in check_role
- [ ] `grant_role` and `revoke_role` convert errors: `.map_err(|_| BridgeError::Unauthorized)?`
- [ ] All authorization checks return Results

### Step 6.3: Reputation Refactor Review

**File:** `contracts/teachlink/src/reputation.rs`

Checklist:

- [ ] Imports include: `use crate::errors::{ReputationError, ReputationResult}`
- [ ] `update_participation` returns `ReputationResult<()>`
- [ ] `update_course_progress` returns `ReputationResult<()>`
- [ ] `rate_contribution` returns `ReputationResult<()>`
- [ ] Invalid rating check: `if rating > 5 { return Err(ReputationError::InvalidRating) }`
- [ ] No `assert!()` macro in production code

### Step 6.4: Bridge Refactor Review

**File:** `contracts/teachlink/src/bridge.rs`

Checklist:

- [ ] `get_token` returns `Result<Address, BridgeError>`
- [ ] `get_admin` returns `Result<Address, BridgeError>`
- [ ] No `.unwrap()` on config getters
- [ ] Fee recipient error handling: `.map_err(|_| BridgeError::StorageError)?`
- [ ] All repository operations properly handle errors

### Step 6.5: Errors Module Review

**File:** `contracts/teachlink/src/errors.rs`

Checklist:

- [ ] `AccessControlError` enum defined with codes 500-505
- [ ] `AnalyticsError` enum defined with codes 510-514
- [ ] `ReputationError` enum defined with codes 520-525
- [ ] `TokenizationError` enum defined with codes 530-536
- [ ] `AdvancedReputationError` enum defined with codes 540-544
- [ ] All Result type aliases defined
- [ ] Error codes don't overlap
- [ ] All errors are contracterror decorated

---

## Part 7: Documentation Verification

### Step 7.1: Error Handling Guide

```bash
ls -la contracts/teachlink/ERROR_HANDLING_GUIDE.md
```

**Expected Result:** File exists with:

- [ ] Overview section
- [ ] Principles section
- [ ] Error hierarchy defined
- [ ] Implementation patterns documented
- [ ] Checklist for implementation
- [ ] Testing guidelines

### Step 7.2: Error Handling Tests

```bash
ls -la contracts/teachlink/tests/error_handling_integration.rs
```

**Expected Result:** File exists with:

- [ ] Test module defined
- [ ] Test cases for each refactored module
- [ ] Manual testing checklist
- [ ] Compilation test

---

## Part 8: Performance Verification (Optional)

### Step 8.1: Build Size Comparison

```bash
# Check WASM size (should not significantly increase)
ls -lh target/wasm32-unknown-unknown/release/teachlink_contract.wasm
```

**Expected Result:** WASM size is reasonable and not significantly increased

### Step 8.2: Gas Estimation

```bash
# If available, run gas benchmark
./scripts/check_performance.sh
```

**Expected Result:** No significant gas cost increase from error handling

---

## Part 9: Final Validation Checklist

| Item                                   | Status | Evidence                       |
| -------------------------------------- | ------ | ------------------------------ |
| Build succeeds without errors          | [ ]    | `cargo build --release` output |
| No unwrap() in production code         | [ ]    | grep results                   |
| No panic!/assert! in production code   | [ ]    | grep results                   |
| All error enums defined                | [ ]    | errors.rs review               |
| All Result types defined               | [ ]    | errors.rs review               |
| Analytics functions return Result      | [ ]    | function signatures            |
| Access Control functions return Result | [ ]    | function signatures            |
| Reputation functions return Result     | [ ]    | function signatures            |
| Bridge functions return Result         | [ ]    | function signatures            |
| Tests compile and pass                 | [ ]    | `cargo test` output            |
| Error codes are unique                 | [ ]    | manual review                  |
| Documentation updated                  | [ ]    | files exist                    |
| No silent failures                     | [ ]    | code review                    |

---

## Troubleshooting

### Issue: Build fails with "unwrap cannot find associated function"

**Solution:** Ensure all Result types are properly imported:

```rust
use crate::errors::{ErrorType, ErrorTypeResult};
```

### Issue: Tests fail with "expected Result, found T"

**Solution:** Update test code to handle Result types:

```rust
// Before
let result = function();

// After
let result = function()?;  // or
match function() {
    Ok(v) => { /* use v */ },
    Err(e) => { /* handle e */ },
}
```

### Issue: Compiler warning "unused Result"

**Solution:** Either use the Result or explicitly ignore it:

```rust
// Discouraged
function()?;  // unused result

// Encouraged
let _ = function()?;  // explicitly ignored
function()?;  // used via ? operator in function returning Result
```

---

## Success Criteria

The assignment is successfully completed when:

1. ✓ **All builds pass** - `cargo build --release` completes without errors
2. ✓ **No panics** - Production code contains no panic!/assert! macros
3. ✓ **No silent failures** - All fallible operations return Result types
4. ✓ **Specific errors** - Each error type has its own variant with meaningful code
5. ✓ **Proper propagation** - ? operator used for error propagation
6. ✓ **Tests pass** - All unit and integration tests pass
7. ✓ **Documentation complete** - Error handling guide and tests provided

---

## Next Steps After Verification

1. Run full test suite: `./scripts/test.sh`
2. Check linting: `./scripts/lint.sh`
3. Deploy to testnet: `./scripts/deploy-testnet.sh`
4. Verify on-chain behavior matches expected error handling

---

## Questions or Issues?

Refer to:

- [ERROR_HANDLING_GUIDE.md](ERROR_HANDLING_GUIDE.md) - Implementation patterns
- [tests/error_handling_integration.rs](tests/error_handling_integration.rs) - Test cases
- [src/errors.rs](src/errors.rs) - Error definitions
