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
