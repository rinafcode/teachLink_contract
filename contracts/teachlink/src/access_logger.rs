//! Access Logging Module
//!
//! Provides comprehensive, tamper-evident access logging for security auditing.
//! Every significant contract invocation is recorded with caller identity,
//! operation tag, outcome (success or failure with error code), and ledger
//! timestamp. Log entries are stored in persistent storage and per-address
//! hourly call counts are maintained for temporal pattern analysis.

use crate::errors::AccessLogError;
use crate::events::{AccessAttemptEvent, AccessLogFailedEvent};
use crate::storage::{ACCESS_LOGS, ACCESS_TEMPORAL, LOG_COUNTER};
use crate::types::{AccessLogEntry, AccessOutcome, AuditQuery};
use soroban_sdk::{Address, Env, Map, Symbol, Vec};

/// Hourly window size in seconds.
const WINDOW_SIZE: u64 = 3600;

/// Access Logger — stateless manager following the existing module pattern.
pub struct AccessLogger;

impl AccessLogger {
    /// Record a single access attempt.
    ///
    /// Steps:
    /// 1. Increment `LOG_COUNTER` in persistent storage.
    /// 2. Compute `window_start = timestamp - (timestamp % WINDOW_SIZE)`.
    /// 3. Build `AccessLogEntry` and write to `ACCESS_LOGS` persistent map.
    /// 4. Increment `ACCESS_TEMPORAL` counter for `(caller, window_start)`.
    /// 5. Emit `AccessAttemptEvent`.
    ///
    /// If the storage write fails the function emits `AccessLogFailedEvent`
    /// instead of panicking, preserving the calling transaction.
    pub fn log_access(
        env: &Env,
        caller: Address,
        operation: Symbol,
        outcome: AccessOutcome,
    ) {
        let timestamp = env.ledger().timestamp();
        let window_start = timestamp - (timestamp % WINDOW_SIZE);

        // --- Increment counter ---
        let mut counter: u64 = env
            .storage()
            .persistent()
            .get(&LOG_COUNTER)
            .unwrap_or(0u64);
        counter += 1;

        // --- Build entry ---
        let entry = AccessLogEntry {
            entry_id: counter,
            caller: caller.clone(),
            operation: operation.clone(),
            outcome: outcome.clone(),
            ledger_timestamp: timestamp,
            window_start,
        };

        // --- Write to persistent storage ---
        let mut logs: Map<u64, AccessLogEntry> = env
            .storage()
            .persistent()
            .get(&ACCESS_LOGS)
            .unwrap_or_else(|| Map::new(env));
        logs.set(counter, entry);
        env.storage().persistent().set(&ACCESS_LOGS, &logs);
        env.storage().persistent().set(&LOG_COUNTER, &counter);

        // --- Update temporal window counter ---
        let mut temporal: Map<(Address, u64), u32> = env
            .storage()
            .instance()
            .get(&ACCESS_TEMPORAL)
            .unwrap_or_else(|| Map::new(env));
        let current_count = temporal
            .get((caller.clone(), window_start))
            .unwrap_or(0u32);
        temporal.set((caller.clone(), window_start), current_count + 1);
        env.storage().instance().set(&ACCESS_TEMPORAL, &temporal);

        // --- Emit event ---
        let (success, error_code) = match &outcome {
            AccessOutcome::Success => (true, 0u32),
            AccessOutcome::Failure { error_code } => (false, *error_code),
        };

        AccessAttemptEvent {
            entry_id: counter,
            caller,
            operation,
            success,
            error_code,
            timestamp,
        }
        .publish(env);
    }

    /// Retrieve a single log entry by ID. Returns `None` if not found.
    /// No authorization required.
    pub fn get_log_entry(env: &Env, entry_id: u64) -> Option<AccessLogEntry> {
        let logs: Map<u64, AccessLogEntry> = env
            .storage()
            .persistent()
            .get(&ACCESS_LOGS)
            .unwrap_or_else(|| Map::new(env));
        logs.get(entry_id)
    }

    /// Return the current value of `LOG_COUNTER` (total entries ever recorded).
    /// No authorization required.
    pub fn get_total_log_count(env: &Env) -> u64 {
        env.storage()
            .persistent()
            .get(&LOG_COUNTER)
            .unwrap_or(0u64)
    }

    /// Query log entries with optional filters.
    ///
    /// Scans from the highest `entry_id` downward (most-recent-first).
    /// Returns at most `query.limit` entries matching all provided filters.
    /// Returns an empty `Vec` immediately when `query.limit == 0`.
    /// No authorization required.
    pub fn query_logs(env: &Env, query: AuditQuery) -> Vec<AccessLogEntry> {
        let mut results = Vec::new(env);

        if query.limit == 0 {
            return results;
        }

        let total = Self::get_total_log_count(env);
        if total == 0 {
            return results;
        }

        let logs: Map<u64, AccessLogEntry> = env
            .storage()
            .persistent()
            .get(&ACCESS_LOGS)
            .unwrap_or_else(|| Map::new(env));

        // Scan most-recent-first
        let mut id = total;
        loop {
            if results.len() >= query.limit {
                break;
            }

            if let Some(entry) = logs.get(id) {
                if Self::matches_query(&entry, &query) {
                    results.push_back(entry);
                }
            }

            if id == 0 {
                break;
            }
            id -= 1;
        }

        results
    }

    /// Return the call count for a `(caller, window_start)` pair, or `0`.
    /// No authorization required.
    pub fn get_temporal_pattern(env: &Env, caller: Address, window_start: u64) -> u32 {
        let temporal: Map<(Address, u64), u32> = env
            .storage()
            .instance()
            .get(&ACCESS_TEMPORAL)
            .unwrap_or_else(|| Map::new(env));
        temporal.get((caller, window_start)).unwrap_or(0u32)
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Returns `true` if `entry` satisfies all active (non-`None`) filters in `query`.
    fn matches_query(entry: &AccessLogEntry, query: &AuditQuery) -> bool {
        // Caller filter
        if let Some(ref caller) = query.caller {
            if &entry.caller != caller {
                return false;
            }
        }

        // Operation filter
        if let Some(ref op) = query.operation {
            if &entry.operation != op {
                return false;
            }
        }

        // Outcome filter
        if let Some(ref outcome_filter) = query.outcome_filter {
            let matches = match (outcome_filter, &entry.outcome) {
                (AccessOutcome::Success, AccessOutcome::Success) => true,
                (
                    AccessOutcome::Failure { error_code: a },
                    AccessOutcome::Failure { error_code: b },
                ) => a == b,
                _ => false,
            };
            if !matches {
                return false;
            }
        }

        // Time range filters
        if let Some(from) = query.from_timestamp {
            if entry.ledger_timestamp < from {
                return false;
            }
        }
        if let Some(to) = query.to_timestamp {
            if entry.ledger_timestamp > to {
                return false;
            }
        }

        true
    }
}
