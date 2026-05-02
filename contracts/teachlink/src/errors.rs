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
    InvalidTimestamp = 148,
    BatchSizeLimitExceeded = 149,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AccessControlError {
    Unauthorized = 200,
    RoleNotFound = 201,
    StorageError = 202,
    MissingRole = 203,
}

pub type AccessControlResult<T> = core::result::Result<T, AccessControlError>;

impl From<AccessControlError> for BridgeError {
    fn from(_: AccessControlError) -> Self {
        BridgeError::Unauthorized
    }
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AccessLogError {
    StorageError = 210,
    InvalidInput = 211,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AnalyticsError {
    StorageError = 220,
    InvalidInput = 221,
    InvalidIndex = 222,
}

pub type AnalyticsResult<T> = core::result::Result<T, AnalyticsError>;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EscrowError {
    AmountMustBePositive = 230,
    AtLeastOneSignerRequired = 231,
    InvalidSignerThreshold = 232,
    RefundTimeMustBeAfterReleaseTime = 233,
    DuplicateSigner = 234,
    EscrowNotPending = 235,
    CallerNotAuthorized = 236,
    InsufficientApprovals = 237,
    ReleaseTimeNotReached = 238,
    SignerNotAuthorized = 239,
    ArbitratorNotAuthorized = 240,
    StorageError = 241,
    ReentrancyDetected = 242,
    SignerAlreadyApproved = 243,
    InvalidBeneficiary = 244,
    InvalidToken = 245,
    InvalidArbitrator = 246,
    DepositorCannotBeBeneficiary = 247,
    InvalidSignerCount = 248,
    InvalidTimestamp = 249,
    InvalidAddress = 250,
    InvalidAmount = 251,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EscrowAnalyticsError {
    StorageError = 250,
    InvalidInput = 251,
}

pub type EscrowAnalyticsResult<T> = core::result::Result<T, EscrowAnalyticsError>;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GovernanceError {
    Unauthorized = 260,
    InvalidInput = 261,
    StorageError = 262,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MobilePlatformError {
    DeviceNotSupported = 270,
    InsufficientStorage = 271,
    PaymentFailed = 272,
    StorageError = 273,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProvenanceError {
    StorageError = 280,
    InvalidInput = 281,
}

pub type ProvenanceResult<T> = core::result::Result<T, ProvenanceError>;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RateLimitingError {
    RateLimitExceeded = 290,
    StorageError = 291,
    InvalidInput = 292,
}

pub type RateLimitingResult<T> = core::result::Result<T, RateLimitingError>;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RewardsError {
    AlreadyInitialized = 300,
    AmountMustBePositive = 301,
    InsufficientRewardPoolBalance = 302,
    NoRewardsAvailable = 303,
    NoPendingRewards = 304,
    RateCannotBeNegative = 305,
    StorageError = 306,
    ReentrancyDetected = 307,
    AmountExceedsMaxLimit = 308,
    ArithmeticOverflow = 309,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ScoreError {
    StorageError = 310,
    ArithmeticOverflow = 311,
    CourseAlreadyCompleted = 312,
}

pub type ScoreResult<T> = core::result::Result<T, ScoreError>;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TokenizationError {
    StorageError = 320,
    TokenNotFound = 321,
    UnauthorizedMint = 322,
    InvalidMetadata = 323,
}

pub type TokenizationResult<T> = core::result::Result<T, TokenizationError>;
