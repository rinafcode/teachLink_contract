use soroban_sdk::contracterror;

/// Bridge module errors
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
    InvalidAddress = 228,
    InvalidAmount = 229,
    InvalidSignerCount = 230,
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
    StorageError = 309,
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

/// Access Control module errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AccessControlError {
    Unauthorized = 500,
    InvalidRole = 501,
    MissingRole = 502,
    RoleAlreadyGranted = 503,
    RoleNotGranted = 504,
    StorageError = 505,
}

/// Analytics module errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AnalyticsError {
    InvalidIndex = 510,
    InsufficientData = 511,
    ChainNotFound = 512,
    MetricsNotAvailable = 513,
    StorageError = 514,
}

/// Reputation module errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReputationError {
    InvalidPoints = 520,
    InvalidRating = 521,
    InvalidThreshold = 522,
    UserNotFound = 523,
    StorageError = 524,
    Unauthorized = 525,
}

/// Tokenization module errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TokenizationError {
    TokenNotFound = 530,
    InvalidTokenId = 531,
    UnauthorizedMint = 532,
    UnauthorizedBurn = 533,
    InvalidMetadata = 534,
    StorageError = 535,
    AmountMustBePositive = 536,
}

/// Advanced Reputation module errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AdvancedReputationError {
    UnauthorizedAccess = 540,
    InvalidInput = 541,
    CalculationOverflow = 542,
    StorageError = 543,
    NotInitialized = 544,
}

/// Result type alias for access control operations
#[allow(dead_code)]
pub type AccessControlResult<T> = core::result::Result<T, AccessControlError>;

/// Result type alias for analytics operations
#[allow(dead_code)]
pub type AnalyticsResult<T> = core::result::Result<T, AnalyticsError>;

/// Result type alias for reputation operations
#[allow(dead_code)]
pub type ReputationResult<T> = core::result::Result<T, ReputationError>;

/// Result type alias for tokenization operations
#[allow(dead_code)]
pub type TokenizationResult<T> = core::result::Result<T, TokenizationError>;

/// Result type alias for advanced reputation operations
#[allow(dead_code)]
pub type AdvancedReputationResult<T> = core::result::Result<T, AdvancedReputationError>;
