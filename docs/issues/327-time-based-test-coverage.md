# Issue #327: Add Comprehensive Time-Based Test Coverage

## Overview

TeachLink contract functions that depend on ledger timestamps (deadlines, timeouts, reward windows) have no dedicated time-based test coverage. This leaves an entire class of bugs undetected.

## Problem

Time-sensitive logic is notoriously error-prone:
- Off-by-one errors at exact boundary timestamps
- Incorrect handling of leap years in duration calculations
- Timezone assumptions baked into timestamp arithmetic
- Timeout conditions that never trigger or trigger too early

Without tests that manipulate ledger time, these bugs only surface in production.

## Acceptance Criteria

- [ ] **Test time boundaries** — assert correct behavior at exactly `deadline - 1`, `deadline`, and `deadline + 1` ledger timestamps
- [ ] **Handle leap years** — verify duration calculations that span Feb 29 produce correct results
- [ ] **Test timezone handling** — confirm all timestamps are treated as UTC with no implicit local-time conversion
- [ ] **Cover timeout scenarios** — test that operations correctly expire, that expired operations are rejected, and that re-activation after expiry behaves as specified

## Implementation Notes

The Soroban test environment allows direct manipulation of ledger time:

```rust
#[test]
fn reward_window_expires_correctly() {
    let env = Env::default();
    // set ledger time to just before deadline
    env.ledger().set(LedgerInfo {
        timestamp: DEADLINE - 1,
        ..Default::default()
    });
    // assert action is still valid

    // advance past deadline
    env.ledger().set(LedgerInfo {
        timestamp: DEADLINE + 1,
        ..Default::default()
    });
    // assert action is now rejected
}
```

All time-dependent contract functions should have at least one boundary test and one expiry test.

## Labels

`testing` · `quality` · `time-handling` · `Stellar Wave`
