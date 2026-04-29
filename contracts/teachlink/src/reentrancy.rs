//! Reentrancy Guard Module
//!
//! Provides a storage-backed mutex to prevent reentrant calls within the same
//! contract invocation. On Soroban, contract execution is single-threaded, but
//! a contract can call itself indirectly through cross-contract invocations.
//! This guard detects and blocks such recursive entry points.
//!
//! # Algorithm
//!
//! The guard uses a boolean flag stored in instance storage keyed by a caller-
//! supplied `Symbol`. Before executing the protected closure:
//!   1. Read the flag; if `true`, a reentrant call is in progress → return error.
//!   2. Set the flag to `true` (acquire the lock).
//!   3. Execute the closure and capture its result.
//!   4. Set the flag back to `false` (release the lock), regardless of outcome.
//!   5. Return the captured result.
//!
//! Because Soroban rolls back all storage writes on transaction failure, the flag
//! is automatically cleared if the outer transaction aborts — no manual cleanup
//! is needed.
//!
//! # Usage
//!
//! Each protected entry point should use a unique `Symbol` key so that
//! independent operations do not block each other unnecessarily.
//!
//! ```ignore
//! reentrancy::with_guard(env, &BRIDGE_GUARD, BridgeError::ReentrancyDetected, || {
//!     // critical section
//! })
//! ```

use soroban_sdk::{Env, Symbol};

/// Executes `f` inside a reentrancy-protected critical section.
///
/// # Parameters
/// - `env`              – Soroban environment reference.
/// - `key`              – Unique storage key used as the mutex flag.
/// - `reentrancy_error` – Error value returned when a reentrant call is detected.
/// - `f`                – Closure containing the logic to protect.
///
/// # Returns
/// The `Result` produced by `f`, or `Err(reentrancy_error)` if the guard is
/// already active.
///
/// # TODO
/// - Consider upgrading to a counter-based guard to support intentional
///   recursive patterns (e.g., nested escrow releases) in future versions.
pub fn with_guard<T, E, F>(env: &Env, key: &Symbol, reentrancy_error: E, f: F) -> Result<T, E>
where
    E: Copy,
    F: FnOnce() -> Result<T, E>,
{
    // Step 1: Check if the guard is already active (reentrant call detected).
    let active = env
        .storage()
        .instance()
        .get::<_, bool>(key)
        .unwrap_or(false);
    if active {
        return Err(reentrancy_error);
    }

    // Step 2: Acquire the lock by setting the flag to true.
    env.storage().instance().set(key, &true);

    // Step 3: Execute the protected closure.
    let result = f();

    // Step 4: Release the lock unconditionally.
    // NOTE: If `f` panics, Soroban aborts the transaction and rolls back all
    // storage, so the flag is implicitly cleared. This explicit reset handles
    // the non-panic `Err` path.
    env.storage().instance().set(key, &false);

    result
}
