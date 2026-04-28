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
}
