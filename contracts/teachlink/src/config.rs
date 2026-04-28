//! Centralized configuration for the TeachLink contract.
//!
//! All tunable constants live here. Modules import from this file instead of
//! defining their own `pub const` values, ensuring a single source of truth.
//!
//! # Sections
//! - [Analytics](#analytics)
//! - [Atomic Swaps](#atomic-swaps)
//! - [Audit & Compliance](#audit--compliance)
//! - [BFT Consensus](#bft-consensus)
//! - [Emergency & Circuit Breaker](#emergency--circuit-breaker)
//! - [Ledger Time](#ledger-time)
//! - [Liquidity & Fees](#liquidity--fees)
//! - [Message Passing](#message-passing)
//! - [Multichain](#multichain)
//! - [Network Recovery](#network-recovery)
//! - [Notifications](#notifications)
//! - [Performance Cache](#performance-cache)
//! - [Rate Limiting](#rate-limiting)
//! - [Slashing](#slashing)
//! - [Sustainability](#sustainability)
//! - [Upgrade](#upgrade)

// ===== Analytics =====

/// How often bridge metrics may be updated (seconds). Default: 1 hour.
pub const METRICS_UPDATE_INTERVAL: u64 = 3_600;

// ===== Atomic Swaps =====

/// Minimum timelock duration for an atomic swap (seconds). Default: 1 hour.
pub const SWAP_MIN_TIMELOCK: u64 = 3_600;

/// Maximum timelock duration for an atomic swap (seconds). Default: 7 days.
pub const SWAP_MAX_TIMELOCK: u64 = 604_800;

/// Required byte length of a hashlock preimage hash.
pub const SWAP_HASH_LENGTH: u32 = 32;

// ===== Audit & Compliance =====

/// Maximum number of audit records retained before circular-buffer wrap.
pub const AUDIT_MAX_RECORDS: u64 = 100_000;

/// Default compliance report period (seconds). Default: 7 days.
pub const AUDIT_COMPLIANCE_PERIOD: u64 = 604_800;

// ===== BFT Consensus =====

/// Minimum validator stake (stroops, 6-decimal). Default: 100 tokens.
pub const BFT_MIN_VALIDATOR_STAKE: i128 = 100_000_000;

/// Proposal expiry window (seconds). Default: 24 hours.
pub const BFT_PROPOSAL_TIMEOUT: u64 = 86_400;

/// Number of consensus rounds per validator rotation epoch.
pub const BFT_ROTATION_EPOCH_ROUNDS: u64 = 100;

/// Minimum reputation score for a validator to remain active (0-100).
pub const BFT_MIN_ACTIVE_REPUTATION: u32 = 40;

// ===== Emergency & Circuit Breaker =====

/// Number of seats on the security council.
pub const EMERGENCY_SECURITY_COUNCIL_SIZE: u32 = 5;

/// Daily volume tracking window (seconds). Default: 24 hours.
pub const EMERGENCY_DAILY_VOLUME_RESET: u64 = 86_400;

// ===== Ledger Time =====

/// Estimated seconds per Stellar ledger (used for lag calculations).
pub const LEDGER_EST_SECS: u64 = 5;

// ===== Liquidity & Fees =====

/// Base bridge fee in basis points. Default: 0.10%.
pub const LIQUIDITY_BASE_FEE_BPS: i128 = 10;

/// Maximum bridge fee in basis points. Default: 5%.
pub const LIQUIDITY_MAX_FEE_BPS: i128 = 500;

/// Minimum bridge fee in basis points. Default: 0.01%.
pub const LIQUIDITY_MIN_FEE_BPS: i128 = 1;

/// Pool utilization threshold (basis points) above which dynamic fees apply.
pub const LIQUIDITY_UTILIZATION_THRESHOLD: u32 = 8_000;

// ===== Message Passing =====

/// Default cross-chain packet timeout (seconds). Default: 24 hours.
pub const MSG_DEFAULT_PACKET_TIMEOUT: u64 = 86_400;

/// Maximum packet delivery retry attempts.
pub const MSG_MAX_RETRY_ATTEMPTS: u32 = 5;

/// Base delay between packet retries (seconds). Default: 5 minutes.
pub const MSG_RETRY_DELAY_BASE: u64 = 300;

// ===== Multichain =====

/// Maximum number of supported external chains.
pub const MULTICHAIN_MAX_CHAINS: u32 = 100;

/// Maximum number of registered multi-chain assets.
pub const MULTICHAIN_MAX_ASSETS: u32 = 1_000;

// ===== Network Recovery =====

/// Maximum automatic retry attempts for a failed operation.
pub const RECOVERY_MAX_RETRY_ATTEMPTS: u32 = 5;

/// Initial exponential-backoff delay (seconds). Default: 1 minute.
pub const RECOVERY_INITIAL_BACKOFF_SECS: u64 = 60;

/// Maximum backoff delay (seconds). Default: 1 hour.
pub const RECOVERY_MAX_BACKOFF_SECS: u64 = 3_600;

/// Backoff multiplier applied on each retry.
pub const RECOVERY_BACKOFF_MULTIPLIER: u64 = 2;

// ===== Notifications =====

/// Sentinel value indicating immediate (non-scheduled) delivery.
pub const NOTIF_IMMEDIATE_DELIVERY: u64 = 0;

/// Minimum scheduling delay for a notification (seconds). Default: 1 minute.
pub const NOTIF_MIN_DELAY_SECS: u64 = 60;

/// Maximum scheduling delay for a notification (seconds). Default: 30 days.
pub const NOTIF_MAX_DELAY_SECS: u64 = 86_400 * 30;

/// Maximum notifications processed per batch.
pub const NOTIF_BATCH_SIZE: u32 = 100;

/// Default TTL for notification event storage (seconds). Default: 7 days.
pub const NOTIF_DEFAULT_EVENT_TTL_SECS: u64 = 86_400 * 7;

// ===== Performance Cache =====

/// Bridge summary cache TTL (seconds). Default: 1 hour.
pub const PERF_CACHE_TTL_SECS: u64 = 3_600;

/// Maximum chains included in the cached top-by-volume list.
pub const PERF_MAX_TOP_CHAINS: u32 = 20;

// ===== Rate Limiting =====

/// Default maximum calls allowed per rate-limit window.
pub const RATE_LIMIT_DEFAULT_MAX_CALLS: u32 = 100;

/// Default rate-limit window size in ledgers.
pub const RATE_LIMIT_DEFAULT_WINDOW_LEDGERS: u32 = 600;

// ===== Slashing =====

/// Slash percentage for double-vote offence (basis points). Default: 50%.
pub const SLASH_DOUBLE_VOTE_BPS: u32 = 5_000;

/// Slash percentage for invalid-signature offence (basis points). Default: 10%.
pub const SLASH_INVALID_SIGNATURE_BPS: u32 = 1_000;

/// Slash percentage for inactivity offence (basis points). Default: 5%.
pub const SLASH_INACTIVITY_BPS: u32 = 500;

/// Slash percentage for byzantine behaviour (basis points). Default: 100%.
pub const SLASH_BYZANTINE_BPS: u32 = 10_000;

/// Slash percentage for malicious behaviour (basis points). Default: 100%.
pub const SLASH_MALICIOUS_BPS: u32 = 10_000;

// ===== Sustainability =====

/// Minimum content tokens minted to reach full content-creation score.
pub const SUSTAIN_CONTENT_SCORE_CAP: u64 = 1_000;

/// Minimum active users to reach full user-adoption score.
pub const SUSTAIN_USER_SCORE_CAP: u64 = 1_000;

// ===== Upgrade =====

/// Window within which a contract upgrade may be rolled back (seconds).
/// Default: 30 days.
pub const UPGRADE_ROLLBACK_WINDOW_SECS: u64 = 86_400 * 30;
