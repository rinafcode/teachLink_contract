# Issue #329: Add Fuzzing Tests for Edge Cases

## Overview

The TeachLink contract has no fuzz testing in place. Fuzzing automatically generates unexpected or malformed inputs to surface bugs, panics, and validation gaps that hand-written tests typically miss.

## Problem

Manual test cases cover known scenarios. Fuzzing covers the unknown:
- Inputs at integer boundaries (0, u64::MAX, i128::MIN)
- Malformed or empty byte strings
- Randomly ordered sequences of contract calls
- Unexpected combinations of valid-looking arguments

Without fuzzing, subtle vulnerabilities and edge-case panics may reach production.

## Acceptance Criteria

- [ ] **Discover unexpected inputs** — run a fuzzer against all public entry points with arbitrary argument generation
- [ ] **Test boundary conditions** — explicitly cover min/max values for every numeric parameter
- [ ] **Find validation gaps** — confirm that invalid inputs are rejected with the correct error, not a panic
- [ ] **Automate fuzzing** — integrate fuzzing into CI so it runs on every PR (time-boxed, e.g. 60 seconds)

## Implementation Notes

Use `cargo-fuzz` (libFuzzer) or `honggfuzz-rs` for Rust fuzzing. A minimal fuzz target looks like:

```rust
// fuzz/fuzz_targets/fuzz_register.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        let env = soroban_sdk::Env::default();
        // call contract entry point with fuzzed input
        let _ = std::panic::catch_unwind(|| {
            TeachLinkContract::register(env.clone(), input.into());
        });
    }
});
```

Any panic that is not an expected contract error is a bug.

## Labels

`testing` · `security` · `fuzzing` · `Stellar Wave`
