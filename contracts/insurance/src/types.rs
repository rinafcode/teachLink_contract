use soroban_sdk::{contracttype, Address, String, Vec, Bytes};

/// Risk assessment factors for insurance pricing
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RiskFactors {
    /// User's historical completion rate (0-100)
    pub completion_rate: u32,
    /// User's reputation score (0-100)
    pub reputation_score: u32,
    /// Course difficulty level (1-10)
    pub course_difficulty: u32,
    /// Course duration in hours
    pub course_duration: u32,
    /// User's experience level (beginner=1, intermediate=2, advanced=3)
    pub experience_level: u32,
    /// Historical claim frequency for similar courses
    pub claim_frequency: u32,
    /// Time since last course completion
    pub time_since_last_completion: u64,
}

/// Risk profile with calculated risk score
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RiskProfile {
    /// Unique profile ID
    pub profile_id: u64,
    /// Associated user address
    pub user: Address,
    /// Risk factors used for calculation
    pub factors: RiskFactors,
    /// Calculated risk score (0-100, higher = riskier)
    pub risk_score: u32,
    /// Timestamp when profile was created/updated
    pub timestamp: u64,
}

/// Insurance policy with dynamic pricing
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InsurancePolicy {
    /// Unique policy ID
    pub policy_id: u64,
    /// Policy holder address
    pub holder: Address,
    /// Course ID being insured
    pub course_id: u64,
    /// Risk profile used for pricing
    pub risk_profile_id: u64,
    /// Base premium amount
    pub base_premium: i128,
    /// Risk adjustment multiplier (basis points, e.g., 12000 = 1.2x)
    pub risk_multiplier: u32,
    /// Final calculated premium
    pub final_premium: i128,
    /// Coverage amount
    pub coverage_amount: i128,
    /// Policy start timestamp
    pub start_time: u64,
    /// Policy expiration timestamp
    pub expiration_time: u64,
    /// Current policy status
    pub status: PolicyStatus,
}

/// Policy status enumeration
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PolicyStatus {
    /// Policy is active and providing coverage
    Active,
    /// Policy has been claimed against
    Claimed,
    /// Policy has expired
    Expired,
    /// Policy was cancelled
    Cancelled,
}

/// Parametric insurance trigger conditions
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParametricTrigger {
    /// Unique trigger ID
    pub trigger_id: u64,
    /// Course ID this trigger applies to
    pub course_id: u64,
    /// Learning outcome metric to monitor
    pub metric: LearningMetric,
    /// Threshold value that triggers payout
    pub threshold: i128,
    /// Payout amount when triggered
    pub payout_amount: i128,
    /// Whether this trigger is active
    pub is_active: bool,
}

/// Learning metrics for parametric insurance
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LearningMetric {
    /// Course completion percentage
    CompletionPercentage,
    /// Time to complete course
    CompletionTime,
    /// Assessment score
    AssessmentScore,
    /// Engagement level (0-100)
    EngagementLevel,
    /// Number of attempts
    AttemptCount,
}

/// Claims with AI verification
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdvancedClaim {
    /// Unique claim ID
    pub claim_id: u64,
    /// Associated policy ID
    pub policy_id: u64,
    /// Claim filing timestamp
    pub filed_at: u64,
    /// Claim status
    pub status: ClaimStatus,
    /// AI verification confidence score (0-100)
    pub ai_confidence: u32,
    /// Supporting evidence (as Bytes)
    pub evidence: Bytes,
    /// Oracle verification result (if used)
    pub oracle_verified: bool,
    /// Payout amount (if approved)
    pub payout_amount: i128,
    /// Reason for claim
    pub reason: String,
}

/// Claim status enumeration
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClaimStatus {
    /// Claim filed, awaiting processing
    Filed,
    /// AI verification in progress
    AiProcessing,
    /// AI verified, awaiting oracle confirmation
    AiVerified,
    /// Oracle confirmed, ready for payout
    OracleConfirmed,
    /// Claim approved and paid
    Approved,
    /// Claim rejected
    Rejected,
    /// Claim disputed
    Disputed,
}

/// Insurance pool with optimization features
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptimizedPool {
    /// Pool ID
    pub pool_id: u64,
    /// Pool name/description
    pub name: String,
    /// Total assets under management
    pub total_assets: i128,
    /// Current utilization rate (basis points)
    pub utilization_rate: u32,
    /// Target utilization rate
    pub target_utilization: u32,
    /// Risk reserve ratio (basis points)
    pub risk_reserve_ratio: u32,
    /// Reinsurance partner addresses
    pub reinsurance_partners: Vec<Address>,
    /// Pool status
    pub status: PoolStatus,
}

/// Pool status enumeration
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PoolStatus {
    /// Pool is active and accepting new policies
    Active,
    /// Pool is paused for maintenance
    Paused,
    /// Pool is in liquidation
    Liquidating,
    /// Pool is closed
    Closed,
}

/// Insurance token representing pool shares
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InsuranceToken {
    /// Token ID
    pub token_id: u64,
    /// Pool this token represents
    pub pool_id: u64,
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Total supply
    pub total_supply: i128,
    /// Current holder address
    pub holder: Address,
    /// Amount of shares held
    pub balance: i128,
}

/// Governance proposal for insurance parameters
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InsuranceProposal {
    /// Proposal ID
    pub proposal_id: u64,
    /// Proposal title
    pub title: String,
    /// Proposal description
    pub description: String,
    /// Type of parameter change
    pub proposal_type: ProposalType,
    /// New parameter value
    pub new_value: i128,
    /// Voting start timestamp
    pub voting_start: u64,
    /// Voting end timestamp
    pub voting_end: u64,
    /// Current vote count (for)
    pub votes_for: u64,
    /// Current vote count (against)
    pub votes_against: u64,
    /// Status of proposal
    pub status: ProposalStatus,
}

/// Proposal type enumeration
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProposalType {
    /// Change base premium rate
    PremiumRate,
    /// Change risk multiplier ranges
    RiskMultiplier,
    /// Change pool utilization targets
    UtilizationTarget,
    /// Add/remove reinsurance partner
    ReinsurancePartner,
    /// Change governance parameters
    Governance,
}

/// Proposal status enumeration
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProposalStatus {
    /// Proposal is active for voting
    Active,
    /// Proposal passed and implemented
    Passed,
    /// Proposal rejected
    Rejected,
    /// Proposal execution failed
    Failed,
}

/// Compliance report for regulatory purposes
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComplianceReport {
    /// Report ID
    pub report_id: u64,
    /// Report period start
    pub period_start: u64,
    /// Report period end
    pub period_end: u64,
    /// Total policies issued
    pub total_policies: u64,
    /// Total claims filed
    pub total_claims: u64,
    /// Total claims paid
    pub claims_paid: u64,
    /// Total premiums collected
    pub premiums_collected: i128,
    /// Total payouts made
    pub total_payouts: i128,
    /// Loss ratio (basis points)
    pub loss_ratio: u32,
    /// Reserve ratio (basis points)
    pub reserve_ratio: u32,
    /// Generated timestamp
    pub generated_at: u64,
}