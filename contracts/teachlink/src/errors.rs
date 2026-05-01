use soroban_sdk::contracterror;

/// Bridge module errors.
///
/// Error codes are in the range 100–147.  Each code is stable across contract
/// upgrades — never reuse or renumber a code, only append new ones.
///
/// # Code Ranges
/// | Range   | Domain                          |
/// |---------|---------------------------------|
/// | 100–110 | Core bridge operations          |
/// | 111–117 | BFT consensus                   |
/// | 118–120 | Validator slashing              |
/// | 121–123 | Multi-chain configuration       |
/// | 124–126 | Liquidity pool                  |
/// | 127–130 | Emergency / circuit breaker     |
/// | 131–133 | Cross-chain message passing     |
/// | 134–137 | Atomic swaps (HTLC)             |
/// | 138–142 | General / retry                 |
/// | 143–147 | Storage / versioning / reentrancy|
///
/// # TODO
/// - Add `BridgeError::RateLimitExceeded` (148) for per-user rate limiting
///   once the rate-limiting module is fully integrated.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BridgeError {
    AlreadyInitialized = 100,
    AmountMustBePositive = 101,
    DestinationChainNotSupported = 102,
    InsufficientValidatorSignatures = 103,
    InvalidValidatorSignature = 104,
    NonceAlreadyProcessed = 105,
    TokenMismatch = 106,
    BridgeTransactionNotFound = 107,
    TimeoutNotReached = 108,
    FeeCannotBeNegative = 109,
    MinimumValidatorsMustBeAtLeastOne = 110,
    // BFT Consensus Errors
    ProposalNotFound = 111,
    ProposalAlreadyVoted = 112,
    ProposalExpired = 113,
    InsufficientStake = 114,
    InsufficientBalance = 115,
    ValidatorNotActive = 116,
    ByzantineThresholdNotMet = 117,
    // Slashing Errors
    ValidatorAlreadySlashed = 118,
    InvalidSlashingEvidence = 119,
    CannotSlashSelf = 120,
    // Multi-Chain Errors
    ChainNotActive = 121,
    AssetNotSupported = 122,
    InvalidChainConfiguration = 123,
    // Liquidity Errors
    InsufficientLiquidity = 124,
    SlippageExceeded = 125,
    InvalidLPAmount = 126,
    // Emergency Errors
    BridgePaused = 127,
    ChainPaused = 128,
    UnauthorizedPause = 129,
    CircuitBreakerTriggered = 130,
    // Message Passing Errors
    PacketNotFound = 131,
    PacketTimeout = 132,
    InvalidPayload = 133,
    // Atomic Swap Errors
    SwapNotFound = 134,
    InvalidHashlock = 135,
    TimelockExpired = 136,
    SwapAlreadyCompleted = 137,
    // General Errors
    Unauthorized = 138,
    InvalidInput = 139,
    RetryLimitExceeded = 140,
    RetryBackoffActive = 141,
    BridgeTransactionFailed = 142,
    // Repository/Storage Errors
    StorageError = 143,
    NotInitialized = 144,
    IncompatibleInterfaceVersion = 145,
    InvalidInterfaceVersionRange = 146,
    ReentrancyDetected = 147,
    BatchSizeLimitExceeded = 148,
    InvalidTimestamp = 149,
}

pub type AccessControlResult<T> = Result<T, AccessControlError>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AccessControlError {
    MissingRole = 200,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AccessLogError {
    StorageError = 210,
}

pub type AnalyticsResult<T> = Result<T, AnalyticsError>;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AnalyticsError {
    InvalidIndex = 220,
    StorageError = 221,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EscrowError {
    AmountMustBePositive = 300,
    ArbitratorNotAuthorized = 301,
    AtLeastOneSignerRequired = 302,
    CallerNotAuthorized = 303,
    DepositorCannotBeBeneficiary = 304,
    DuplicateSigner = 305,
    EscrowNotPending = 306,
    InsufficientApprovals = 307,
    InvalidAddress = 308,
    InvalidAmount = 309,
    InvalidArbitrator = 310,
    InvalidBeneficiary = 311,
    InvalidSignerCount = 312,
    InvalidSignerThreshold = 313,
    InvalidTimestamp = 314,
    InvalidToken = 315,
    ReentrancyDetected = 316,
    RefundTimeMustBeAfterReleaseTime = 317,
    ReleaseTimeNotReached = 318,
    SignerAlreadyApproved = 319,
    SignerNotAuthorized = 320,
    StorageError = 321,
}

pub type EscrowAnalyticsResult<T> = Result<T, EscrowAnalyticsError>;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EscrowAnalyticsError {
    StorageError = 330,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GovernanceError {
    AlreadyVoted = 340,
    GovernanceProposalNotActive = 341,
    GovernanceProposalNotFound = 342,
    ProposalsNotInitialized = 343,
    VotingPeriodEnded = 344,
    VotingStillInProgress = 345,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MobilePlatformError {
    DeviceNotSupported = 350,
    InsufficientStorage = 351,
    PaymentFailed = 352,
}

pub type ProvenanceResult<T> = Result<T, ProvenanceError>;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProvenanceError {
    StorageError = 360,
}

pub type RateLimitingResult<T> = Result<T, RateLimitingError>;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RateLimitingError {
    StorageError = 370,
    RateLimitExceeded = 371,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RewardsError {
    AlreadyInitialized = 380,
    AmountExceedsMaxLimit = 381,
    AmountMustBePositive = 382,
    ArithmeticOverflow = 383,
    InsufficientRewardPoolBalance = 384,
    NoPendingRewards = 385,
    NoRewardsAvailable = 386,
    RateCannotBeNegative = 387,
    ReentrancyDetected = 388,
    StorageError = 389,
}

pub type TokenizationResult<T> = Result<T, TokenizationError>;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TokenizationError {
    InvalidMetadata = 390,
    StorageError = 391,
    TokenNotFound = 392,
    UnauthorizedMint = 393,
}
