//! Dependency injection interfaces for inter-module communication.
//!
//! Each trait defines the surface that a calling module requires from a
//! collaborating module.  The concrete implementations live in their
//! respective modules; test code can substitute lightweight mocks.
//!
//! # Coupling map (before this PR)
//! ```text
//! escrow       → arbitration, insurance, escrow_analytics
//! tokenization → provenance
//! performance  → analytics
//! reporting    → analytics, audit, escrow_analytics
//! backup       → audit
//! ```
//!
//! After this PR every arrow is mediated by a trait defined here, so each
//! module can be compiled and tested with a mock collaborator.

use soroban_sdk::{Address, Bytes, Env, Vec};

use crate::errors::{BridgeError, EscrowError};
use crate::types::{
    BridgeMetrics, Escrow, EscrowMetrics, OperationType, ProvenanceRecord, TransferType,
};

// ---------------------------------------------------------------------------
// Arbitration port  (used by escrow)
// ---------------------------------------------------------------------------

/// Abstraction over the arbitration module used by `EscrowManager`.
pub trait ArbitrationPort {
    /// Select an active arbitrator from the registry.
    fn pick_arbitrator(env: &Env) -> Result<Address, EscrowError>;

    /// Return `true` when an escrow has been pending without approvals past
    /// the stall timeout.
    fn check_stalled(env: &Env, escrow: &Escrow) -> bool;

    /// Update the arbitrator's on-chain reputation score.
    fn record_resolution(
        env: &Env,
        arbitrator: Address,
        success: bool,
    ) -> Result<(), EscrowError>;
}

// ---------------------------------------------------------------------------
// Insurance port  (used by escrow)
// ---------------------------------------------------------------------------

/// Abstraction over the insurance module used by `EscrowManager`.
pub trait InsurancePort {
    /// Compute the insurance premium for a given escrow amount.
    fn calculate_premium(env: &Env, amount: i128) -> i128;

    /// Transfer the premium from `payer` to the insurance pool.
    fn pay_premium(env: &Env, payer: Address, amount: i128) -> Result<(), EscrowError>;
}

// ---------------------------------------------------------------------------
// Escrow observer  (used by escrow)
// ---------------------------------------------------------------------------

/// Callback interface for escrow lifecycle events consumed by analytics.
pub trait EscrowObserver {
    /// Called when a new escrow is created.
    fn on_created(env: &Env, amount: i128);

    /// Called when an escrow enters the Disputed state.
    fn on_disputed(env: &Env);

    /// Called when an escrow is resolved; `duration` is seconds since creation.
    fn on_resolved(env: &Env, duration: u64);
}

// ---------------------------------------------------------------------------
// Analytics port  (used by performance and reporting)
// ---------------------------------------------------------------------------

/// Abstraction over the analytics module used by `PerformanceManager` and
/// `ReportingManager`.
pub trait AnalyticsPort {
    /// Return the stored bridge metrics.
    fn bridge_metrics(env: &Env) -> BridgeMetrics;

    /// Compute a 0-10000 basis-point bridge health score.
    fn health_score(env: &Env) -> u32;

    /// Return up to `max` (chain_id, volume) pairs ordered by volume desc.
    fn top_chains_by_volume(env: &Env, max: u32) -> Vec<(u32, i128)>;
}

// ---------------------------------------------------------------------------
// Escrow metrics port  (used by reporting)
// ---------------------------------------------------------------------------

/// Abstraction over the escrow-analytics module used by `ReportingManager`.
pub trait EscrowMetricsPort {
    /// Return the current aggregated escrow metrics.
    fn get_metrics(env: &Env) -> EscrowMetrics;
}

// ---------------------------------------------------------------------------
// Audit port  (used by backup and reporting)
// ---------------------------------------------------------------------------

/// Abstraction over the audit module used by `BackupManager` and
/// `ReportingManager`.
pub trait AuditPort {
    /// Append an audit record and return its id.
    fn create_record(
        env: &Env,
        op: OperationType,
        operator: Address,
        details: Bytes,
        tx_hash: Bytes,
    ) -> Result<u64, BridgeError>;

    /// Return the total number of audit records stored.
    fn get_count(env: &Env) -> u64;
}

// ---------------------------------------------------------------------------
// Provenance port  (used by tokenization)
// ---------------------------------------------------------------------------

/// Abstraction over the provenance module used by `ContentTokenization`.
pub trait ProvenancePort {
    /// Append a transfer record to the provenance chain for `token_id`.
    fn record_transfer(
        env: &Env,
        token_id: u64,
        from: Option<Address>,
        to: Address,
        transfer_type: TransferType,
        notes: Option<Bytes>,
    );

    /// Return all provenance records for `token_id`.
    fn get_history(env: &Env, token_id: u64) -> Vec<ProvenanceRecord>;
}
