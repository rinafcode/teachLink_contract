use crate::types::*;
use soroban_sdk::{contracttype, Address, Vec};


pub struct QualityVote {
    pub token_id: u64,
    pub voter: Address,
    pub score: u8,
}


pub enum DataKey {
    // Core configuration
    Admin,
    Oracle,
    Token,

    // RBAC & Governance
    HasRole(u32, Address),     // Role ID -> Address -> bool
    PendingAdminTransfer,      // Address (New Admin)
    AdminTransferTimestamp,    // u64 (When transfer can be executed)
    MultiSigSigners,           // Vec<Address>
    MultiSigThreshold,         // u32
    CriticalOperation(u32),    // Operation ID -> CriticalOperation
    OperationApprovals(u32),   // Operation ID -> Vec<Address>
    OperationCount,            // u64
    
    // Risk assessment
    RiskProfile(u64),
    RiskProfileByUser(Address),
    RiskProfileCount,
    RiskModelWeights,

    // Insurance policies
    Policy(u64),
    PolicyByUser(Address, u64), // user -> course_id -> policy_id
    PolicyCount,
    ActivePolicies(Address), // user -> Vec<policy_ids>

    // Claims
    Claim(u64),
    ClaimByPolicy(u64),
    ClaimCount,
    PendingClaims,

    // Parametric insurance
    ParametricTrigger(u64),
    TriggerByCourse(u64),
    TriggerCount,

    // Insurance pools
    Pool(u64),
    PoolCount,
    ActivePools,
    PoolUtilization(u64),

    // Reinsurance
    ReinsurancePartner(Address),
    ReinsurancePartners,
    ReinsuranceAllocation(u64, Address), // pool_id -> partner -> allocation

    // Insurance tokens
    InsuranceToken(u64),
    TokenByPool(u64),
    TokenHolder(Address, u64), // holder -> token_id -> balance
    TokenCount,

    // Governance
    Proposal(u64),
    ProposalCount,
    Vote(Address, u64), // voter -> proposal_id -> has_voted
    GovernanceParameters,

    // Analytics
    DailyMetrics(u64),   // timestamp (day) -> metrics
    MonthlyMetrics(u64), // timestamp (month) -> metrics
    RiskDistribution,
    PoolPerformance(u64), // pool_id -> performance metrics

    // Compliance
    ComplianceReport(u64),
    ReportCount,
    LastReportGeneration,

    // Cross-chain
    ChainBridge(Address), // chain_id -> bridge_address
    CrossChainClaim(u64),

    // Configuration parameters
    BasePremiumRate,
    RiskMultiplierRanges,
    UtilizationTargets,
    MinimumRiskReserve,
    GovernanceQuorum,
    VotingPeriod,
}



/// Risk multiplier ranges based on risk scores
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RiskMultiplierRanges {
    pub low_risk_min: u32,    // 0-30 risk score
    pub low_risk_max: u32,    // 10000 = 1.0x
    pub medium_risk_min: u32, // 31-60 risk score
    pub medium_risk_max: u32, // 15000 = 1.5x
    pub high_risk_min: u32,   // 61-100 risk score
    pub high_risk_max: u32,   // 30000 = 3.0x
}


/// Risk distribution statistics
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RiskDistribution {
    pub low_risk_count: u64,    // 0-30
    pub medium_risk_count: u64, // 31-60
    pub high_risk_count: u64,   // 61-100
    pub average_risk_score: u32,
    pub risk_std_dev: u32,
}

/// Pool performance metrics
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolPerformance {
    pub pool_id: u64,
    pub period_start: u64,
    pub period_end: u64,
    pub total_assets: i128,
    pub premiums_earned: i128,
    pub claims_paid: i128,
    pub net_profit: i128,
    pub utilization_rate: u32,
    pub loss_ratio: u32,
    pub roi_percentage: i32, // Basis points, can be negative
}
