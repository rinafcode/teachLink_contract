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
}

/// Escrow module errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EscrowError {
    AmountMustBePositive = 200,
    AtLeastOneSignerRequired = 201,
    InvalidSignerThreshold = 202,
    RefundTimeMustBeInFuture = 203,
    RefundTimeMustBeAfterReleaseTime = 204,
    DuplicateSigner = 205,
    SignerNotAuthorized = 206,
    SignerAlreadyApproved = 207,
    CallerNotAuthorized = 208,
    InsufficientApprovals = 209,
    ReleaseTimeNotReached = 210,
    OnlyDepositorCanRefund = 211,
    RefundNotEnabled = 212,
    RefundTimeNotReached = 213,
    OnlyDepositorCanCancel = 214,
    CannotCancelAfterApprovals = 215,
    OnlyDepositorOrBeneficiaryCanDispute = 216,
    EscrowNotInDispute = 217,
    OnlyArbitratorCanResolve = 218,
    EscrowNotPending = 219,
    EscrowNotFound = 220,
    ArbitratorNotAuthorized = 221,
    // Repository/Storage Errors
    StorageError = 222,
    InvalidBeneficiary = 226,
    InvalidToken = 223,
    InvalidArbitrator = 224,
    DepositorCannotBeBeneficiary = 225,
    ReentrancyDetected = 227,
}

/// Rewards module errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RewardsError {
    AlreadyInitialized = 300,
    AmountMustBePositive = 301,
    InsufficientRewardPoolBalance = 302,
    NoRewardsAvailable = 303,
    NoPendingRewards = 304,
    RateCannotBeNegative = 305,
    ReentrancyDetected = 306,
    ArithmeticOverflow = 307,
    AmountExceedsMaxLimit = 308,
}

/// Mobile platform module errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MobilePlatformError {
    DeviceNotSupported = 400,
    InsufficientStorage = 401,
    NetworkUnavailable = 402,
    AuthenticationFailed = 403,
    SyncFailed = 404,
    PaymentFailed = 405,
    SecurityViolation = 406,
    FeatureNotAvailable = 407,
}

/// Common errors that can be used across modules
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CommonError {
    Unauthorized = 400,
    InvalidInput = 401,
    InsufficientBalance = 402,
    TransferFailed = 403,
    StorageError = 404,
}

/// Result type alias for bridge operations
#[allow(dead_code)]
pub type BridgeResult<T> = core::result::Result<T, BridgeError>;

/// Result type alias for escrow operations
#[allow(dead_code)]
pub type EscrowResult<T> = core::result::Result<T, EscrowError>;

/// Result type alias for rewards operations
#[allow(dead_code)]
pub type RewardsResult<T> = core::result::Result<T, RewardsError>;

/// Result type alias for common operations
#[allow(dead_code)]
pub type CommonResult<T> = core::result::Result<T, CommonError>;
