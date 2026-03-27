use soroban_sdk::contracterror;

/// Enhanced insurance module errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum InsuranceError {
    // Existing errors (499-504)
    NotInitialized = 499,
    AlreadyInitialized = 500,
    UserNotInsured = 501,
    ClaimNotFound = 502,
    ClaimAlreadyProcessed = 503,
    ClaimNotVerified = 504,

    // New errors for enhanced features (505-550)
    InvalidRiskFactors = 505,
    RiskProfileNotFound = 506,
    PolicyNotFound = 507,
    PolicyExpired = 508,
    PolicyAlreadyClaimed = 509,
    InsufficientPremium = 510,
    InvalidParametricTrigger = 511,
    TriggerNotFound = 512,
    TriggerNotActive = 513,
    InvalidLearningMetric = 514,
    ClaimAlreadyFiled = 515,
    ClaimNotInReviewableState = 516,
    AiVerificationFailed = 517,
    OracleVerificationRequired = 518,
    PoolNotFound = 519,
    PoolNotActive = 520,
    PoolUtilizationTooHigh = 521,
    ReinsurancePartnerNotFound = 522,
    InvalidTokenParameters = 523,
    TokenNotFound = 524,
    InsufficientTokenBalance = 525,
    ProposalNotFound = 526,
    VotingPeriodEnded = 527,
    AlreadyVoted = 528,
    InvalidProposalType = 529,
    ProposalNotActive = 530,
    ComplianceReportNotFound = 531,
    InvalidTimeRange = 532,
    AnalyticsNotAvailable = 533,
    RiskModelNotTrained = 534,
    ExternalOracleError = 535,
    CrossChainOperationFailed = 536,
    InvalidCrossChainParameters = 537,
    GovernanceQuorumNotMet = 538,
    UnauthorizedGovernanceAction = 539,
    RiskScoreOutOfRange = 540,
    PremiumCalculationError = 541,
    PayoutExceedsCoverage = 542,
    EvidenceHashInvalid = 543,
    DisputeResolutionFailed = 544,
    PoolLiquidityInsufficient = 545,
    TokenTransferFailed = 546,
    InvalidPoolConfiguration = 547,
    ReinsuranceLimitExceeded = 548,
    ParametricConditionNotMet = 549,
    ReportGenerationFailed = 550,
}

/// Result type alias for insurance operations
#[allow(dead_code)]
pub type InsuranceResult<T> = core::result::Result<T, InsuranceError>;
