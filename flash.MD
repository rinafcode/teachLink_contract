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
