# Issue #328: Add Comprehensive Contract Upgrade Testing

## Overview

The TeachLink contract has no test coverage for upgrade scenarios. Soroban supports in-place contract upgrades, but upgrading without tests risks data loss, broken APIs, and unrecoverable state.

## Problem

Contract upgrades are high-risk operations:
- Existing ledger state must survive the upgrade intact
- Callers depending on the old ABI must not silently break
- A failed upgrade with no rollback path can permanently lock funds or data

Currently there are no tests that exercise any of these scenarios.

## Acceptance Criteria

- [ ] **Test state migration** — verify that all ledger entries written by v1 are readable and correct after upgrading to v2
- [ ] **Verify API compatibility** — confirm that entry points present in v1 still behave identically in v2 (no silent breaking changes)
- [ ] **Check data preservation** — assert that balances, registrations, and reward records are unchanged post-upgrade
- [ ] **Test rollback scenarios** — simulate a failed upgrade and verify the contract remains functional on the previous version

## Implementation Notes

Soroban's test environment supports deploying a contract, writing state, then re-uploading a new WASM and calling `update_current_contract_wasm`:

```rust
#[test]
fn upgrade_preserves_state() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkContractV1);
    // write state via v1
    // upload v2 wasm and upgrade
    env.deployer().update_current_contract_wasm(V2_WASM);
    // assert state is intact via v2 entry points
}
```

Both the old and new WASM blobs should be compiled as test fixtures and checked into `tests/fixtures/`.

## Labels

`testing` · `upgrades` · `quality` · `Stellar Wave`
