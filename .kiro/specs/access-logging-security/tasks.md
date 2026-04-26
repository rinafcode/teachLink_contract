# Implementation Plan: Access Logging & Security Audit

## Overview

Implement the `AccessLogger` module in `contracts/teachlink/src/` following the existing manager-struct pattern. The work proceeds in layers: types → storage keys → events → errors → core module → lib.rs wiring → property-based tests.

## Tasks

- [ ] 1. Add new types to `types.rs`
  - Add `AccessOutcome` enum (`Success`, `Failure { error_code: u32 }`) with `#[contracttype]`
  - Add `AccessLogEntry` struct with fields: `entry_id: u64`, `caller: Address`, `operation: Symbol`, `outcome: AccessOutcome`, `ledger_timestamp: u64`, `window_start: u64`
  - Add `AuditQuery` struct with fields: `caller: Option<Address>`, `operation: Option<Symbol>`, `outcome_filter: Option<AccessOutcome>`, `from_timestamp: Option<u64>`, `to_timestamp: Option<u64>`, `limit: u32`
  - All three types must derive `Clone, Debug, Eq, PartialEq` and be annotated `#[contracttype]`
  - _Requirements: 2.1, 4.1_

- [ ] 2. Add storage keys to `storage.rs`
  - Add `LOG_COUNTER: Symbol = symbol_short!("log_cnt")` (persistent, `u64`)
  - Add `ACCESS_LOGS: Symbol = symbol_short!("acc_logs")` (persistent, `Map<u64, AccessLogEntry>`)
  - Add `ACCESS_TEMPORAL: Symbol = symbol_short!("acc_tmp")` (instance, `Map<(Address, u64), u32>`)
  - _Requirements: 1.2, 1.3, 3.1_

- [ ] 3. Add events to `events.rs`
  - Add `AccessAttemptEvent` with fields: `entry_id: u64`, `caller: Address`, `operation: Symbol`, `success: bool`, `error_code: u32`, `timestamp: u64`
  - Add `AccessLogFailedEvent` with fields: `caller: Address`, `operation: Symbol`, `timestamp: u64`
  - Both must be annotated `#[contractevent]` and derive `Clone, Debug`
  - _Requirements: 5.1, 5.2_

- [ ] 4. Add `AccessLogError` to `errors.rs`
  - Add `AccessLogError` enum with `#[contracterror]`: `StorageWriteFailed = 500`, `InvalidOperationTag = 501`, `InvalidLimit = 502`
  - Add `pub type AccessLogResult<T> = core::result::Result<T, AccessLogError>;`
  - _Requirements: 1.4_

- [ ] 5. Implement `AccessLogger` module in `access_logger.rs`
  - [ ] 5.1 Create `contracts/teachlink/src/access_logger.rs` with `pub struct AccessLogger;`
    - Implement `log_access(env: &Env, caller: Address, operation: Symbol, outcome: AccessOutcome)`
      - Increment `LOG_COUNTER` in persistent storage (start at 1)
      - Compute `window_start = timestamp - (timestamp % 3600)`
      - Build and write `AccessLogEntry` to `ACCESS_LOGS` persistent map
      - Increment `ACCESS_TEMPORAL` counter for `(caller, window_start)` in instance storage
      - Emit `AccessAttemptEvent`; on write failure emit `AccessLogFailedEvent` instead
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.2, 2.3, 2.4, 3.1, 3.2, 3.4, 5.3, 5.4_

  - [ ]\* 5.2 Write property test: Storage Round-Trip (Property 1)
    - **Property 1: Storage Round-Trip**
    - For any caller, operation, outcome: after `log_access`, `get_log_entry(entry_id)` returns `Some(entry)` with all fields matching
    - **Validates: Requirements 1.1, 1.3, 4.5, 4.8**

  - [ ] 5.3 Implement `get_log_entry(env: &Env, entry_id: u64) -> Option<AccessLogEntry>`
    - Read from `ACCESS_LOGS` persistent map; return `None` if not found
    - No authorization required
    - _Requirements: 4.5, 6.2_

  - [ ] 5.4 Implement `get_total_log_count(env: &Env) -> u64`
    - Return current value of `LOG_COUNTER` from persistent storage (0 if unset)
    - No authorization required
    - _Requirements: 4.6, 6.3_

  - [ ]\* 5.5 Write property test: Monotonic Counter Invariant (Property 2)
    - **Property 2: Monotonic Counter Invariant**
    - For N calls to `log_access`, entry_ids are strictly increasing and `get_total_log_count() == N`
    - **Validates: Requirements 1.2, 4.6**

  - [ ] 5.6 Implement `query_logs(env: &Env, query: AuditQuery) -> Vec<AccessLogEntry>`
    - Return empty `Vec` immediately when `query.limit == 0`
    - Scan from highest `entry_id` downward (most-recent-first)
    - Apply all non-`None` filters conjunctively (caller, operation, outcome_filter, from_timestamp, to_timestamp)
    - Stop after collecting `query.limit` matching entries
    - No authorization required
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.7, 6.1_

  - [ ] 5.7 Implement `get_temporal_pattern(env: &Env, caller: Address, window_start: u64) -> u32`
    - Read from `ACCESS_TEMPORAL` instance map; return `0` if not found
    - No authorization required
    - _Requirements: 3.3, 6.1_

- [ ] 6. Checkpoint — ensure the module compiles cleanly
  - Run `cargo build` targeting the teachlink contract; fix any type or import errors before proceeding.
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 7. Wire `AccessLogger` into `lib.rs`
  - Add `mod access_logger;` to the module declarations in `lib.rs`
  - Export `AccessLogEntry`, `AccessOutcome`, `AuditQuery` in the `pub use types::{ ... }` block
  - Export `AccessLogError` in the `pub use errors::{ ... }` block
  - Add the following public contract entry points to `TeachLinkBridge`:
    - `pub fn log_access(env: Env, caller: Address, operation: Symbol, outcome: AccessOutcome)`
    - `pub fn get_log_entry(env: Env, entry_id: u64) -> Option<AccessLogEntry>`
    - `pub fn get_total_log_count(env: Env) -> u64`
    - `pub fn query_logs(env: Env, query: AuditQuery) -> Vec<AccessLogEntry>`
    - `pub fn get_temporal_pattern(env: Env, caller: Address, window_start: u64) -> u32`
  - _Requirements: 1.1, 4.1, 4.5, 4.6, 3.3_

- [ ] 8. Write property-based tests in `property_based_tests.rs`
  - [ ]\* 8.1 Write property test: Success Outcome Consistency (Property 3)
    - **Property 3: Success Outcome Consistency**
    - For any `log_access` with `AccessOutcome::Success`, stored entry has `outcome == Success` and emitted event has `success = true`, `error_code = 0`
    - **Validates: Requirements 2.2, 5.3**

  - [ ]\* 8.2 Write property test: Failure Outcome and Error Code Consistency (Property 4)
    - **Property 4: Failure Outcome and Error Code Consistency**
    - For any `log_access` with `AccessOutcome::Failure { error_code: e }`, stored entry preserves `e` and emitted event has `success = false`, `error_code = e`
    - **Validates: Requirements 1.5, 2.3, 5.4**

  - [ ]\* 8.3 Write property test: Event Emission Completeness (Property 5)
    - **Property 5: Event Emission Completeness**
    - For any successful `log_access`, exactly one `AccessAttemptEvent` is emitted with fields matching the stored entry
    - **Validates: Requirements 2.4, 5.1**

  - [ ]\* 8.4 Write property test: Outcome Filter Correctness (Property 6)
    - **Property 6: Outcome Filter Correctness**
    - For mixed-outcome entries, `query_logs` with a `Success` filter returns only `Success` entries; with a `Failure` filter returns only `Failure` entries
    - **Validates: Requirements 2.5**

  - [ ]\* 8.5 Write property test: Temporal Window Counting (Property 7)
    - **Property 7: Temporal Window Counting**
    - For N calls within the same 3600-second window, `get_temporal_pattern` returns N and each entry's `window_start` equals `timestamp - (timestamp % 3600)`
    - **Validates: Requirements 3.1, 3.2, 3.4, 3.5**

  - [ ]\* 8.6 Write property test: Query Limit and Ordering (Property 8)
    - **Property 8: Query Limit and Ordering**
    - For M entries and limit L, `query_logs` returns `min(M, L)` entries in descending `entry_id` order
    - **Validates: Requirements 4.2, 4.7**

  - [ ]\* 8.7 Write property test: Conjunctive Filter Correctness (Property 9)
    - **Property 9: Conjunctive Filter Correctness**
    - For any multi-filter `AuditQuery`, every returned entry satisfies all active filter conditions simultaneously
    - **Validates: Requirements 4.4**

- [ ] 9. Final checkpoint — ensure all tests pass
  - Run `cargo test` for the teachlink contract; all existing and new tests must pass.
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for a faster MVP
- All property tests belong in the existing `property_based_tests.rs` module under a new `access_logging` submodule, tagged `// Feature: access-logging-security, Property N: ...`
- `symbol_short!` keys must be ≤ 9 characters — verify each new key before committing
- The `ACCESS_TEMPORAL` composite key `(Address, u64)` follows the same tuple-encoding pattern as `EscrowApprovalKey`
- No delete or modify functions should ever be added — the audit trail is append-only (Requirement 6.4)
