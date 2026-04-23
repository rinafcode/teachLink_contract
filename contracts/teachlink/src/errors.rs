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
    StakeTooHigh = 115,
    InsufficientBalance = 116,
    ValidatorNotActive = 117,
    ByzantineThresholdNotMet = 118,
    // Slashing Errors
    ValidatorAlreadySlashed = 119,
    InvalidSlashingEvidence = 120,
    CannotSlashSelf = 121,
    // Multi-Chain Errors
    ChainNotActive = 122,
    AssetNotSupported = 123,
    InvalidChainConfiguration = 124,
    // Liquidity Errors
    InsufficientLiquidity = 125,
    SlippageExceeded = 126,
    InvalidLPAmount = 127,
    // Emergency Errors
    BridgePaused = 128,
    ChainPaused = 129,
    UnauthorizedPause = 130,
    CircuitBreakerTriggered = 131,
    // Message Passing Errors
    PacketNotFound = 132,
    PacketTimeout = 133,
    InvalidPayload = 134,
    // Atomic Swap Errors
    SwapNotFound = 135,
    InvalidHashlock = 136,
    TimelockExpired = 137,
    SwapAlreadyCompleted = 138,
    // General Errors
    Unauthorized = 139,
    InvalidInput = 140,
    RetryLimitExceeded = 141,
    RetryBackoffActive = 142,
    BridgeTransactionFailed = 143,
    // Repository/Storage Errors
    StorageError = 144,
    NotInitialized = 145,
    IncompatibleInterfaceVersion = 146,
    InvalidInterfaceVersionRange = 147,
    ReentrancyDetected = 148,
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
