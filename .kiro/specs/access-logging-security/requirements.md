# Requirements Document

## Introduction

The TeachLink smart contract workspace currently lacks comprehensive access logging for security auditing. While individual modules emit domain-specific events (rewards, escrow, bridge, etc.), there is no unified mechanism to record all access attempts, distinguish successes from failures, capture temporal patterns, or support structured audit analysis across modules.

This feature introduces a dedicated `AccessLogger` module within `contracts/teachlink/src/` that intercepts and records every significant contract invocation — including the caller identity, the operation attempted, the outcome (success or failure), and the ledger timestamp. The log is stored on-chain in Soroban persistent storage and is queryable for audit analysis. Temporal pattern data (per-address call frequency windows) is maintained to support anomaly detection.

## Glossary

- **AccessLogger**: The new Soroban module responsible for recording, storing, and querying access log entries.
- **AccessLogEntry**: A single immutable record of one access attempt, containing caller, operation, outcome, timestamp, and optional error code.
- **AccessOutcome**: An enum with variants `Success` and `Failure(error_code: u32)` describing the result of an access attempt.
- **OperationTag**: A `soroban_sdk::Symbol` (≤ 9 chars) identifying the contract function that was accessed (e.g., `bridge_out`, `claim_rwd`, `slash_val`).
- **TemporalWindow**: A fixed 3600-second (1-hour) bucket used to aggregate per-address call counts for pattern analysis.
- **AuditQuery**: A struct carrying filter parameters (caller, operation, outcome, time range) used to retrieve matching log entries.
- **LOG_COUNTER**: Persistent storage key tracking the monotonically increasing log entry ID.
- **ACCESS_LOGS**: Persistent storage key mapping `u64` entry IDs to `AccessLogEntry` values.
- **ACCESS_TEMPORAL**: Instance storage key mapping `(Address, u64 window_start)` pairs to `u32` call counts.
- **Caller**: The `Address` that authorized and invoked a contract function.
- **Admin**: The address stored under the `ADMIN` storage key, authorized to query and manage audit data.

---

## Requirements

### Requirement 1: Record Every Access Attempt

**User Story:** As a security auditor, I want every significant contract invocation to be recorded with caller identity and outcome, so that I have a complete, tamper-evident trail of who accessed what and whether it succeeded.

#### Acceptance Criteria

1. WHEN any public contract function completes (successfully or with an error), THE AccessLogger SHALL persist an `AccessLogEntry` containing: a monotonically increasing `entry_id`, the `caller` Address, the `operation` OperationTag, the `outcome` (Success or Failure with error code), and the `ledger_timestamp`.
2. THE AccessLogger SHALL assign each `AccessLogEntry` a unique `entry_id` by incrementing `LOG_COUNTER` in persistent storage before writing the entry.
3. THE AccessLogger SHALL store each `AccessLogEntry` under the key `(ACCESS_LOGS, entry_id)` in persistent storage so entries survive ledger TTL expiry.
4. IF persistent storage write fails for any reason, THEN THE AccessLogger SHALL emit an `AccessLogFailedEvent` containing the `caller`, `operation`, and `ledger_timestamp` so the failure is observable off-chain.
5. THE AccessLogger SHALL record failure entries with the numeric error code extracted from the `contracterror` variant, preserving the original error type information.

---

### Requirement 2: Track Success and Failure Outcomes

**User Story:** As a security auditor, I want each log entry to clearly distinguish successful operations from failed ones with their error codes, so that I can identify attack patterns, repeated failures, and unauthorized access attempts.

#### Acceptance Criteria

1. THE `AccessOutcome` type SHALL have exactly two variants: `Success` and `Failure { error_code: u32 }`, and SHALL be a `#[contracttype]` enum so it is serializable to Soroban XDR.
2. WHEN a contract function returns `Ok(...)`, THE AccessLogger SHALL record `AccessOutcome::Success` for that entry.
3. WHEN a contract function returns `Err(e)`, THE AccessLogger SHALL record `AccessOutcome::Failure { error_code: e as u32 }` for that entry, preserving the numeric discriminant of the error.
4. THE AccessLogger SHALL emit an `AccessAttemptEvent` for every recorded entry, containing `entry_id`, `caller`, `operation`, `outcome`, and `timestamp`, so off-chain indexers can track outcomes without reading storage.
5. WHEN querying log entries by outcome, THE AccessLogger SHALL return only entries whose `outcome` variant matches the requested filter, without modifying any stored state.

---

### Requirement 3: Record Temporal Patterns

**User Story:** As a security analyst, I want per-address call frequency aggregated into hourly windows, so that I can detect anomalous access bursts and temporal attack patterns.

#### Acceptance Criteria

1. WHEN an access attempt is logged, THE AccessLogger SHALL compute the `window_start` as `ledger_timestamp - (ledger_timestamp % 3600)` and increment the call count stored at `(ACCESS_TEMPORAL, caller, window_start)` in instance storage.
2. THE AccessLogger SHALL initialize the call count to `1` for a `(caller, window_start)` pair that has no existing entry, and SHALL increment by `1` for subsequent calls within the same window.
3. WHEN `get_temporal_pattern(caller, window_start)` is called, THE AccessLogger SHALL return the `u32` call count for that `(caller, window_start)` pair, or `0` if no calls were recorded in that window.
4. THE `window_start` value stored in `AccessLogEntry` SHALL equal `ledger_timestamp - (ledger_timestamp % 3600)`, ensuring entries are correctly bucketed for analysis.
5. FOR ALL valid `(caller, window_start)` pairs, the sum of call counts across all entries in that window SHALL equal the value returned by `get_temporal_pattern(caller, window_start)` (round-trip consistency property).

---

### Requirement 4: Enable Audit Analysis

**User Story:** As a compliance officer, I want to query access logs by caller, operation, outcome, and time range, so that I can generate audit reports and investigate security incidents.

#### Acceptance Criteria

1. THE AccessLogger SHALL expose a `query_logs(env, query: AuditQuery) -> Vec<AccessLogEntry>` function that accepts an `AuditQuery` struct with optional fields: `caller: Option<Address>`, `operation: Option<OperationTag>`, `outcome_filter: Option<AccessOutcome>`, `from_timestamp: Option<u64>`, `to_timestamp: Option<u64>`, and `limit: u32`.
2. WHEN `query_logs` is called with a non-zero `limit`, THE AccessLogger SHALL return at most `limit` entries matching all provided filters, scanning from the highest `entry_id` downward (most-recent-first).
3. WHEN `query_logs` is called with `limit = 0`, THE AccessLogger SHALL return an empty `Vec<AccessLogEntry>` without scanning storage.
4. WHEN multiple filters are provided in `AuditQuery`, THE AccessLogger SHALL apply all filters conjunctively (AND logic), returning only entries that satisfy every non-`None` filter field.
5. THE AccessLogger SHALL expose a `get_log_entry(env, entry_id: u64) -> Option<AccessLogEntry>` function that returns the entry for the given ID, or `None` if it does not exist.
6. THE AccessLogger SHALL expose a `get_total_log_count(env) -> u64` function that returns the current value of `LOG_COUNTER`, representing the total number of entries ever recorded.
7. WHEN `query_logs` is called with only a `caller` filter and a `limit` of N, THE AccessLogger SHALL return the N most recent entries for that caller in descending `entry_id` order.
8. FOR ALL `entry_id` values in `[1, get_total_log_count()]`, `get_log_entry(entry_id)` SHALL return `Some(entry)` where `entry.entry_id == entry_id` (storage round-trip property).

---

### Requirement 5: Access Log Event Schema

**User Story:** As an off-chain indexer operator, I want well-defined Soroban contract events emitted for every access log entry, so that I can index and alert on security-relevant activity without polling contract storage.

#### Acceptance Criteria

1. THE AccessLogger SHALL define and emit an `AccessAttemptEvent` `#[contractevent]` struct with fields: `entry_id: u64`, `caller: Address`, `operation: soroban_sdk::Symbol`, `success: bool`, `error_code: u32` (0 when success), and `timestamp: u64`.
2. THE AccessLogger SHALL define and emit an `AccessLogFailedEvent` `#[contractevent]` struct with fields: `caller: Address`, `operation: soroban_sdk::Symbol`, and `timestamp: u64`, used only when the log write itself fails.
3. WHEN `AccessAttemptEvent` is emitted for a successful operation, THE AccessLogger SHALL set `success = true` and `error_code = 0`.
4. WHEN `AccessAttemptEvent` is emitted for a failed operation, THE AccessLogger SHALL set `success = false` and `error_code` to the numeric error discriminant.
5. THE `operation` field in `AccessAttemptEvent` SHALL be a `soroban_sdk::Symbol` of at most 9 characters, matching the `OperationTag` used in the corresponding `AccessLogEntry`.

---

### Requirement 6: Admin-Gated Log Management

**User Story:** As a contract administrator, I want to control who can invoke log management functions, so that audit data cannot be tampered with by unauthorized parties.

#### Acceptance Criteria

1. WHEN `query_logs` is called by any address, THE AccessLogger SHALL permit the call without requiring admin authorization, since audit queries are read-only and non-sensitive.
2. WHEN `get_log_entry` is called by any address, THE AccessLogger SHALL permit the call without requiring admin authorization.
3. WHEN `get_total_log_count` is called by any address, THE AccessLogger SHALL permit the call without requiring admin authorization.
4. THE AccessLogger SHALL NOT expose any function that deletes or modifies existing `AccessLogEntry` records, ensuring the audit trail is append-only and immutable once written.
5. WHERE the contract has an initialized `ADMIN` key in instance storage, THE AccessLogger SHALL use that same admin address for any future privileged log management operations (e.g., archiving), maintaining consistency with the existing admin model.
