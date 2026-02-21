use crate::types::*;
use soroban_sdk::{contracttype, Address, Vec};

/// Storage keys for the enhanced insurance contract
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    // Core configuration
    Admin,
    Oracle,
    Token,
    
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
    DailyMetrics(u64), // timestamp (day) -> metrics
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

/// Risk model weights for AI-powered assessment
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RiskModelWeights {
    pub completion_rate_weight: u32,      // 25%
    pub reputation_score_weight: u32,     // 20%
    pub course_difficulty_weight: u32,    // 15%
    pub course_duration_weight: u32,      // 10%
    pub experience_level_weight: u32,     // 15%
    pub claim_frequency_weight: u32,      // 10%
    pub time_factor_weight: u32,          // 5%
}

impl Default for RiskModelWeights {
    fn default() -> Self {
        Self {
            completion_rate_weight: 2500,
            reputation_score_weight: 2000,
            course_difficulty_weight: 1500,
            course_duration_weight: 1000,
            experience_level_weight: 1500,
            claim_frequency_weight: 1000,
            time_factor_weight: 500,
        }
    }
}

/// Governance parameters
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GovernanceParameters {
    pub quorum_percentage: u32,        // Required voting quorum (basis points)
    pub voting_period_days: u32,       // Voting period in days
    pub execution_delay_hours: u32,    // Delay before executing passed proposals
    pub proposal_threshold: u64,       // Minimum tokens to create proposal
    pub veto_power_enabled: bool,      // Whether admin can veto proposals
}

impl Default for GovernanceParameters {
    fn default() -> Self {
        Self {
            quorum_percentage: 5000,       // 50%
            voting_period_days: 7,
            execution_delay_hours: 24,
            proposal_threshold: 1000,
            veto_power_enabled: true,
        }
    }
}

/// Risk multiplier ranges based on risk scores
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RiskMultiplierRanges {
    pub low_risk_min: u32,        // 0-30 risk score
    pub low_risk_max: u32,        // 10000 = 1.0x
    pub medium_risk_min: u32,     // 31-60 risk score
    pub medium_risk_max: u32,     // 15000 = 1.5x
    pub high_risk_min: u32,       // 61-100 risk score
    pub high_risk_max: u32,       // 30000 = 3.0x
}

impl Default for RiskMultiplierRanges {
    fn default() -> Self {
        Self {
            low_risk_min: 0,
            low_risk_max: 10000,
            medium_risk_min: 31,
            medium_risk_max: 15000,
            high_risk_min: 61,
            high_risk_max: 30000,
        }
    }
}

/// Pool utilization targets
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UtilizationTargets {
    pub target_rate: u32,         // 8000 = 80%
    pub max_rate: u32,           // 9500 = 95%
    pub min_reserve_ratio: u32,  // 1500 = 15%
}

impl Default for UtilizationTargets {
    fn default() -> Self {
        Self {
            target_rate: 8000,
            max_rate: 9500,
            min_reserve_ratio: 1500,
        }
    }
}

/// Daily insurance metrics
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DailyMetrics {
    pub date: u64,                   // Unix timestamp (day)
    pub policies_issued: u64,
    pub premiums_collected: i128,
    pub claims_filed: u64,
    pub claims_paid: u64,
    pub total_payouts: i128,
    pub active_policies: u64,
    pub pool_utilization: u32,
    pub average_risk_score: u32,
}

/// Risk distribution statistics
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RiskDistribution {
    pub low_risk_count: u64,      // 0-30
    pub medium_risk_count: u64,   // 31-60
    pub high_risk_count: u64,     // 61-100
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
    pub roi_percentage: i32,      // Basis points, can be negative
}