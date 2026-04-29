# Design Document: Access Logging & Security Audit

## Overview

This design introduces an `AccessLogger` module within `contracts/teachlink/src/` that provides a unified, tamper-evident audit trail for all significant contract invocations. The module records caller identity, operation tag, outcome (success or failure with error code), and ledger timestamp for every access attempt. Log entries are stored in Soroban persistent storage to survive TTL expiry, and per-address hourly call counts are maintained in instance storage for temporal pattern analysis.

The design follows the existing module pattern in the TeachLink codebase: a dedicated `access_logger.rs` file containing a `pub struct AccessLogger` with associated functions, new types added to `types.rs`, new storage keys added to `storage.rs`, and new events added to `events.rs`. The module is then wired into `lib.rs` as public contract entry points.

### Key Design Decisions

- **Persistent storage for log entries**: Unlike instance storage (which can expire), persistent storage ensures the audit trail survives indefinitely. This matches the requirement for tamper-evident, long-lived records.
- **Instance storage for temporal counters**: Hourly window counters are operational data used for anomaly detection. Instance storage is appropriate since they are accessed frequently and can be reconstructed from log entries if needed.
- **Append-only design**: No delete or modify functions are exposed. The `LOG_COUNTER` only ever increments, and entries are written once and never overwritten.
- **`Symbol` as `OperationTag`**: Soroban `Symbol` values are limited to 9 characters, making them compact and XDR-serializable. This constraint is enforced at the type level.
- **No admin gate on reads**: Query functions are read-only and non-sensitive, so they are open to any caller. This matches the existing pattern in the codebase (e.g., `get_bridge_metrics`, `get_audit_record`).

---

## Architecture

The `AccessLogger` is a stateless manager struct (matching the pattern of `SlashingManager`, `LiquidityManager`, etc.) that operates on Soroban storage through the `Env` reference. It does not hold any state itself.

```mermaid
graph TD
    subgraph Contract Entry Points (lib.rs)
        A[log_access]
        B[get_log_entry]
        C[query_logs]
        D[get_total_log_count]
        E[get_temporal_pattern]
    end

    subgraph AccessLogger Module (access_logger.rs)
        F[AccessLogger::log_access]
        G[AccessLogger::get_log_entry]
        H[AccessLogger::query_logs]
        I[AccessLogger::get_total_log_count]
        J[AccessLogger::get_temporal_pattern]
    end

    subgraph Storage
        K[(Persistent: ACCESS_LOGS\nentry_id -> AccessLogEntry)]
        L[(Persistent: LOG_COUNTER\nu64)]
        M[(Instance: ACCESS_TEMPORAL\n(caller, window_start) -> u32)]
    end

    subgraph Events
        N[AccessAttemptEvent]
        O[AccessLogFailedEvent]
    end

    A --> F
    B --> G
    C --> H
    D --> I
    E --> J

    F --> K
    F --> L
    F --> M
    F --> N
    F -.->|on write failure| O

    G --> K
    H --> K
    I --> L
    J --> M
```

### Storage Layout

| Key               | Storage Type | Value Type                 | Description                               |
| ----------------- | ------------ | -------------------------- | ----------------------------------------- |
| `LOG_COUNTER`     | Persistent   | `u64`                      | Monotonically increasing entry ID counter |
| `ACCESS_LOGS`     | Persistent   | `Map<u64, AccessLogEntry>` | All log entries keyed by entry_id         |
| `ACCESS_TEMPORAL` | Instance     | `Map<(Address, u64), u32>` | Per-address hourly call counts            |

> **Note on `ACCESS_TEMPORAL` key encoding**: Soroban `Map` keys must implement `IntoVal`. The composite key `(caller: Address, window_start: u64)` will be stored as a two-element tuple, which Soroban serializes as an XDR `ScVec`. This is the same pattern used for `EscrowApprovalKey` in the existing codebase.

---

## Components and Interfaces

### New Types (`types.rs`)

```rust
/// Identifies the contract function that was accessed. Must be ≤ 9 chars.
pub type OperationTag = soroban_sdk::Symbol;

/// The outcome of a single access attempt.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccessOutcome {
    Success,
    Failure { error_code: u32 },
}

/// A single immutable record of one access attempt.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessLogEntry {
    pub entry_id: u64,
    pub caller: Address,
    pub operation: Symbol,
    pub outcome: AccessOutcome,
    pub ledger_timestamp: u64,
    pub window_start: u64,
}

/// Filter parameters for audit queries.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditQuery {
    pub caller: Option<Address>,
    pub operation: Option<Symbol>,
    pub outcome_filter: Option<AccessOutcome>,
    pub from_timestamp: Option<u64>,
    pub to_timestamp: Option<u64>,
    pub limit: u32,
}
```

### New Storage Keys (`storage.rs`)

```rust
// Access Logging Storage
pub const LOG_COUNTER: Symbol = symbol_short!("log_cnt");
pub const ACCESS_LOGS: Symbol = symbol_short!("acc_logs");
pub const ACCESS_TEMPORAL: Symbol = symbol_short!("acc_tmp");
```

### New Events (`events.rs`)

```rust
/// Emitted for every successfully recorded access log entry.
#[contractevent]
#[derive(Clone, Debug)]
pub struct AccessAttemptEvent {
    pub entry_id: u64,
    pub caller: Address,
    pub operation: Symbol,
    pub success: bool,
    pub error_code: u32,
    pub timestamp: u64,
}

/// Emitted when the log write itself fails (fallback observability).
#[contractevent]
#[derive(Clone, Debug)]
pub struct AccessLogFailedEvent {
    pub caller: Address,
    pub operation: Symbol,
    pub timestamp: u64,
}
```

### `AccessLogger` Module (`access_logger.rs`)

```rust
pub struct AccessLogger;

impl AccessLogger {
    /// Record a single access attempt. Called by contract entry points after
    /// the underlying operation completes (or fails).
    ///
    /// Steps:
    /// 1. Increment LOG_COUNTER in persistent storage.
    /// 2. Compute window_start = timestamp - (timestamp % 3600).
    /// 3. Build AccessLogEntry and write to ACCESS_LOGS persistent map.
    /// 4. Increment ACCESS_TEMPORAL counter for (caller, window_start).
    /// 5. Emit AccessAttemptEvent.
    /// On any panic/failure in steps 1-4, emit AccessLogFailedEvent instead.
    pub fn log_access(
        env: &Env,
        caller: Address,
        operation: Symbol,
        outcome: AccessOutcome,
    );

    /// Retrieve a single log entry by ID. Returns None if not found.
    /// No authorization required.
    pub fn get_log_entry(env: &Env, entry_id: u64) -> Option<AccessLogEntry>;

    /// Return the current value of LOG_COUNTER (total entries ever recorded).
    /// No authorization required.
    pub fn get_total_log_count(env: &Env) -> u64;

    /// Query log entries with optional filters. Scans from highest entry_id
    /// downward (most-recent-first). Returns at most `query.limit` entries.
    /// Returns empty Vec if limit == 0.
    /// No authorization required.
    pub fn query_logs(env: &Env, query: AuditQuery) -> Vec<AccessLogEntry>;

    /// Return the call count for a (caller, window_start) pair, or 0.
    /// No authorization required.
    pub fn get_temporal_pattern(
        env: &Env,
        caller: Address,
        window_start: u64,
    ) -> u32;
}
```

### Contract Entry Points (`lib.rs` additions)

```rust
/// Record an access attempt (called internally or by authorized callers).
pub fn log_access(env: Env, caller: Address, operation: Symbol, outcome: AccessOutcome);

/// Retrieve a log entry by ID.
pub fn get_log_entry(env: Env, entry_id: u64) -> Option<AccessLogEntry>;

/// Return total number of log entries ever recorded.
pub fn get_total_log_count(env: Env) -> u64;

/// Query log entries with filters.
pub fn query_logs(env: Env, query: AuditQuery) -> Vec<AccessLogEntry>;

/// Return per-address call count for a given hourly window.
pub fn get_temporal_pattern(env: Env, caller: Address, window_start: u64) -> u32;
```

---

## Data Models

### `AccessLogEntry` Field Details

| Field              | Type            | Description                                       |
| ------------------ | --------------- | ------------------------------------------------- |
| `entry_id`         | `u64`           | Unique monotonically increasing ID (1-based)      |
| `caller`           | `Address`       | The address that authorized the invocation        |
| `operation`        | `Symbol`        | OperationTag identifying the function (≤ 9 chars) |
| `outcome`          | `AccessOutcome` | `Success` or `Failure { error_code }`             |
| `ledger_timestamp` | `u64`           | `env.ledger().timestamp()` at time of logging     |
| `window_start`     | `u64`           | `ledger_timestamp - (ledger_timestamp % 3600)`    |

### `AuditQuery` Filter Semantics

All `Option` fields are applied conjunctively (AND). A `None` field is ignored (matches all). The `limit` field caps the result count; `limit = 0` returns an empty result immediately without scanning.

### Temporal Window Encoding

The `ACCESS_TEMPORAL` map uses a composite key `(Address, u64)` encoded as a Soroban tuple. The `window_start` is always aligned to a 3600-second boundary:

```
window_start = timestamp - (timestamp % 3600)
```

For example, timestamp `1_700_000_500` → `window_start = 1_700_000_500 - (1_700_000_500 % 3600) = 1_699_999_200`.

---

## Correctness Properties

_A property is a characteristic or behavior that should hold true across all valid executions of a system — essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees._

### Property 1: Storage Round-Trip

_For any_ caller address, operation symbol, and outcome, after calling `log_access`, calling `get_log_entry` with the returned `entry_id` SHALL return `Some(entry)` where `entry.caller == caller`, `entry.operation == operation`, `entry.outcome == outcome`, and `entry.entry_id` matches the counter value at the time of the call.

**Validates: Requirements 1.1, 1.3, 4.5, 4.8**

### Property 2: Monotonic Counter Invariant

_For any_ sequence of N calls to `log_access`, the resulting `entry_id` values SHALL be strictly monotonically increasing, and `get_total_log_count()` SHALL equal the total number of entries ever recorded.

**Validates: Requirements 1.2, 4.6**

### Property 3: Success Outcome Consistency

_For any_ `log_access` call with `AccessOutcome::Success`, the stored `AccessLogEntry.outcome` SHALL be `Success`, and the emitted `AccessAttemptEvent` SHALL have `success = true` and `error_code = 0`.

**Validates: Requirements 2.2, 5.3**

### Property 4: Failure Outcome and Error Code Consistency

_For any_ `log_access` call with `AccessOutcome::Failure { error_code: e }`, the stored `AccessLogEntry.outcome` SHALL be `Failure { error_code: e }`, and the emitted `AccessAttemptEvent` SHALL have `success = false` and `error_code = e`.

**Validates: Requirements 1.5, 2.3, 5.4**

### Property 5: Event Emission Completeness

_For any_ successful `log_access` call, exactly one `AccessAttemptEvent` SHALL be emitted with `entry_id`, `caller`, `operation`, `success`, `error_code`, and `timestamp` fields matching the stored `AccessLogEntry`.

**Validates: Requirements 2.4, 5.1**

### Property 6: Outcome Filter Correctness

_For any_ set of log entries with mixed outcomes and any `AuditQuery` with a non-None `outcome_filter`, all entries returned by `query_logs` SHALL have an `outcome` variant that matches the filter, and no entry with a non-matching outcome SHALL appear in the result.

**Validates: Requirements 2.5**

### Property 7: Temporal Window Counting

_For any_ caller address and any N calls to `log_access` within the same 3600-second window (same `window_start`), `get_temporal_pattern(caller, window_start)` SHALL return exactly N, and each stored `AccessLogEntry.window_start` SHALL equal `ledger_timestamp - (ledger_timestamp % 3600)`.

**Validates: Requirements 3.1, 3.2, 3.4, 3.5**

### Property 8: Query Limit and Ordering

_For any_ set of M log entries and any `AuditQuery` with `limit = L > 0`, `query_logs` SHALL return at most `min(M, L)` entries, and the returned entries SHALL be ordered by descending `entry_id` (most-recent-first).

**Validates: Requirements 4.2, 4.7**

### Property 9: Conjunctive Filter Correctness

_For any_ `AuditQuery` with multiple non-None filter fields, every entry returned by `query_logs` SHALL satisfy all active filter conditions simultaneously (caller match AND operation match AND outcome match AND timestamp range).

**Validates: Requirements 4.4**

---

## Error Handling

### New Error Type

A dedicated `AccessLogError` enum is added to `errors.rs`:

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AccessLogError {
    StorageWriteFailed = 500,
    InvalidOperationTag = 501,  // operation symbol > 9 chars
    InvalidLimit = 502,
}
```

### Failure Scenarios

| Scenario                                                     | Handling                                                          |
| ------------------------------------------------------------ | ----------------------------------------------------------------- |
| Persistent storage write fails                               | Emit `AccessLogFailedEvent`; do not panic the calling transaction |
| `operation` Symbol exceeds 9 chars                           | Return `Err(AccessLogError::InvalidOperationTag)` before writing  |
| `limit = 0` in `query_logs`                                  | Return empty `Vec` immediately (not an error)                     |
| `entry_id` not found in `get_log_entry`                      | Return `None` (not an error)                                      |
| `(caller, window_start)` not found in `get_temporal_pattern` | Return `0` (not an error)                                         |

### Soroban Storage Failure Handling

Soroban storage operations (`env.storage().persistent().set(...)`) panic on failure rather than returning `Result`. To implement the `AccessLogFailedEvent` fallback (Requirement 1.4), the `log_access` function uses a try-catch pattern via a helper that wraps the write in a way that can be observed. In practice, Soroban storage failures are extremely rare (they indicate ledger-level issues), but the event provides off-chain observability.

---

## Testing Strategy

### Unit Tests

Unit tests cover specific examples and edge cases:

- `test_log_access_success`: Log a success entry, retrieve it, verify all fields.
- `test_log_access_failure`: Log a failure entry with a specific error code, verify `error_code` is preserved.
- `test_query_logs_limit_zero`: Verify `query_logs` with `limit=0` returns empty Vec.
- `test_query_logs_caller_filter`: Log entries for two callers, query by one, verify only that caller's entries are returned.
- `test_query_logs_time_range`: Log entries at different timestamps, query with `from_timestamp`/`to_timestamp`, verify range filtering.
- `test_get_temporal_pattern_no_entries`: Query a window with no entries, verify result is 0.
- `test_read_functions_no_auth`: Call `query_logs`, `get_log_entry`, `get_total_log_count` from a non-admin address, verify they succeed.
- `test_no_delete_api`: Verify no delete/modify functions exist on `AccessLogger`.

### Property-Based Tests

This feature uses [proptest](https://github.com/proptest-rs/proptest) (already a dependency in `property_based_tests.rs`) for property-based testing.

Each property test runs a minimum of 100 iterations.

**Tag format**: `// Feature: access-logging-security, Property {N}: {property_text}`

#### Property Test 1: Storage Round-Trip

```
// Feature: access-logging-security, Property 1: storage round-trip
proptest! {
    fn test_storage_round_trip(
        caller in arb_address(),
        operation in arb_operation_tag(),
        outcome in arb_outcome(),
    ) {
        // log_access, then get_log_entry, verify all fields match
    }
}
```

#### Property Test 2: Monotonic Counter Invariant

```
// Feature: access-logging-security, Property 2: monotonic counter invariant
proptest! {
    fn test_monotonic_counter(n in 1u32..=50) {
        // log n entries, collect entry_ids, verify strictly increasing
        // verify get_total_log_count() == n
    }
}
```

#### Property Test 3: Success Outcome Consistency

```
// Feature: access-logging-security, Property 3: success outcome consistency
proptest! {
    fn test_success_outcome_consistency(
        caller in arb_address(),
        operation in arb_operation_tag(),
    ) {
        // log with Success, verify stored outcome == Success
        // verify emitted event has success=true, error_code=0
    }
}
```

#### Property Test 4: Failure Outcome and Error Code Consistency

```
// Feature: access-logging-security, Property 4: failure outcome and error code consistency
proptest! {
    fn test_failure_outcome_consistency(
        caller in arb_address(),
        operation in arb_operation_tag(),
        error_code in 0u32..=u32::MAX,
    ) {
        // log with Failure { error_code }, verify stored error_code matches
        // verify emitted event has success=false and matching error_code
    }
}
```

#### Property Test 5: Event Emission Completeness

```
// Feature: access-logging-security, Property 5: event emission completeness
proptest! {
    fn test_event_emission(
        caller in arb_address(),
        operation in arb_operation_tag(),
        outcome in arb_outcome(),
    ) {
        // log_access, capture emitted events, verify exactly one AccessAttemptEvent
        // with fields matching the stored entry
    }
}
```

#### Property Test 6: Outcome Filter Correctness

```
// Feature: access-logging-security, Property 6: outcome filter correctness
proptest! {
    fn test_outcome_filter(entries in vec(arb_log_entry_params(), 1..=20)) {
        // log all entries, query with Success filter, verify all results are Success
        // query with Failure filter, verify all results are Failure
    }
}
```

#### Property Test 7: Temporal Window Counting

```
// Feature: access-logging-security, Property 7: temporal window counting
proptest! {
    fn test_temporal_window_counting(
        caller in arb_address(),
        base_timestamp in 3600u64..=u64::MAX/2,
        n in 1u32..=20,
    ) {
        // log n entries with timestamps in the same window
        // verify get_temporal_pattern returns n
        // verify each entry.window_start == base_timestamp - (base_timestamp % 3600)
    }
}
```

#### Property Test 8: Query Limit and Ordering

```
// Feature: access-logging-security, Property 8: query limit and ordering
proptest! {
    fn test_query_limit_and_ordering(
        m in 1u32..=30,
        l in 1u32..=30,
    ) {
        // log m entries, query with limit l
        // verify result.len() == min(m, l)
        // verify entry_ids are in descending order
    }
}
```

#### Property Test 9: Conjunctive Filter Correctness

```
// Feature: access-logging-security, Property 9: conjunctive filter correctness
proptest! {
    fn test_conjunctive_filters(entries in vec(arb_log_entry_params(), 1..=20)) {
        // log entries with varying caller/operation/outcome/timestamp
        // apply multi-filter query, verify every result satisfies all active filters
    }
}
```
