# Enhanced Insurance System - API Reference

## Core Functions

### Initialization
```rust
fn initialize(
    env: Env,
    admin: Address,
    oracle: Address,
    token: Address,
) -> Result<(), InsuranceError>
```
Initialize the insurance contract with admin, oracle, and token addresses.

### Risk Management

#### Create/Update Risk Profile
```rust
fn create_risk_profile(
    env: Env,
    user: Address,
    factors: RiskFactors,
) -> Result<u64, InsuranceError>
```
Creates or updates a user's risk profile based on provided factors.

**Parameters:**
- `user`: User address
- `factors`: RiskFactors struct containing completion_rate, reputation_score, etc.

**Returns:** Profile ID

#### Get Risk Profile
```rust
fn get_risk_profile(env: Env, user: Address) -> Option<RiskProfile>
```
Retrieves a user's current risk profile.

#### Get Risk Multiplier
```rust
fn get_risk_multiplier(env: Env, risk_score: u32) -> Result<u32, InsuranceError>
```
Calculates premium multiplier based on risk score.

**Returns:** Multiplier in basis points (e.g., 15000 = 1.5x)

### Policy Management

#### Purchase Insurance Policy
```rust
fn purchase_policy(
    env: Env,
    user: Address,
    course_id: u64,
    coverage_amount: i128,
) -> Result<u64, InsuranceError>
```
Purchase insurance with dynamic premium based on user's risk profile.

**Parameters:**
- `user`: Policy holder address
- `course_id`: Course being insured
- `coverage_amount`: Coverage amount in tokens

**Returns:** Policy ID

#### Get Policy Information
```rust
fn get_policy(env: Env, policy_id: u64) -> Option<InsurancePolicy>
```
Retrieve policy details by ID.

#### Get User's Active Policies
```rust
fn get_active_policies(env: Env, user: Address) -> Vec<u64>
```
List all active policy IDs for a user.

### Claims Processing

#### File Insurance Claim
```rust
fn file_claim(
    env: Env,
    user: Address,
    policy_id: u64,
    evidence_hash: [u8; 32],
    reason: Bytes,
) -> Result<u64, InsuranceError>
```
Submit an insurance claim with evidence.

**Parameters:**
- `policy_id`: Associated policy ID
- `evidence_hash`: 32-byte cryptographic evidence hash
- `reason`: Claim reason description

**Returns:** Claim ID

#### Get Claim Information
```rust
fn get_claim(env: Env, claim_id: u64) -> Option<AdvancedClaim>
```
Retrieve claim details and status.

#### Get Pending Claims
```rust
fn get_pending_claims(env: Env) -> Vec<u64>
```
List claims awaiting oracle verification.

### Parametric Insurance

#### Create Parametric Trigger
```rust
fn create_parametric_trigger(
    env: Env,
    admin: Address,
    course_id: u64,
    metric: LearningMetric,
    threshold: i128,
    payout_amount: i128,
) -> Result<u64, InsuranceError>
```
Create automatic payout trigger based on learning outcomes.

**Parameters:**
- `metric`: LearningMetric type (CompletionPercentage, AssessmentScore, etc.)
- `threshold`: Threshold value for trigger activation
- `payout_amount`: Automatic payout amount

**Returns:** Trigger ID

#### Execute Parametric Trigger
```rust
fn execute_trigger(
    env: Env,
    trigger_id: u64,
    user: Address,
    actual_value: i128,
) -> Result<(), InsuranceError>
```
Execute trigger if threshold conditions are met.

### Pool Management

#### Create Insurance Pool
```rust
fn create_pool(
    env: Env,
    admin: Address,
    name: Bytes,
    target_utilization: u32,
    risk_reserve_ratio: u32,
) -> Result<u64, InsuranceError>
```
Create optimized insurance pool.

**Parameters:**
- `name`: Pool name/description
- `target_utilization`: Target utilization rate (basis points)
- `risk_reserve_ratio`: Minimum reserve ratio (basis points)

**Returns:** Pool ID

#### Add Reinsurance Partner
```rust
fn add_reinsurance_partner(
    env: Env,
    admin: Address,
    pool_id: u64,
    partner: Address,
    allocation_percentage: u32,
) -> Result<(), InsuranceError>
```
Add reinsurance partner to pool with allocation percentage.

#### Optimize Pool Utilization
```rust
fn optimize_pool_utilization(
    env: Env,
    pool_id: u64,
) -> Result<PoolPerformance, InsuranceError>
```
Calculate and optimize pool performance metrics.

#### Get Pool Information
```rust
fn get_pool(env: Env, pool_id: u64) -> Option<OptimizedPool>
```
Retrieve pool configuration and status.

#### Get Active Pools
```rust
fn get_active_pools(env: Env) -> Vec<u64>
```
List all active pool IDs.

### Governance

#### Create Governance Proposal
```rust
fn create_proposal(
    env: Env,
    proposer: Address,
    title: Bytes,
    description: Bytes,
    proposal_type: ProposalType,
    new_value: i128,
) -> Result<u64, InsuranceError>
```
Create community governance proposal.

**Parameters:**
- `proposal_type`: Type of parameter change
- `new_value`: New parameter value

**Returns:** Proposal ID

#### Vote on Proposal
```rust
fn vote(
    env: Env,
    voter: Address,
    proposal_id: u64,
    support: bool,
) -> Result<(), InsuranceError>
```
Vote for or against a proposal.

#### Execute Proposal
```rust
fn execute_proposal(
    env: Env,
    admin: Address,
    proposal_id: u64,
) -> Result<(), InsuranceError>
```
Execute approved proposal after voting period.

#### Get Proposal Information
```rust
fn get_proposal(env: Env, proposal_id: u64) -> Option<InsuranceProposal>
```
Retrieve proposal details and vote counts.

#### Get Governance Parameters
```rust
fn get_governance_parameters(env: Env) -> GovernanceParameters
```
Retrieve current governance configuration.

### Tokenization

#### Create Insurance Token
```rust
fn create_insurance_token(
    env: Env,
    admin: Address,
    pool_id: u64,
    name: Bytes,
    symbol: Bytes,
    total_supply: i128,
) -> Result<u64, InsuranceError>
```
Create tokenized representation of insurance pool shares.

**Returns:** Token ID

#### Transfer Tokens
```rust
fn transfer_tokens(
    env: Env,
    from: Address,
    to: Address,
    token_id: u64,
    amount: i128,
) -> Result<(), InsuranceError>
```
Transfer insurance tokens between addresses.

#### Get Token Information
```rust
fn get_insurance_token(env: Env, token_id: u64) -> Option<InsuranceToken>
```
Retrieve token details.

#### Get Token Balance
```rust
fn get_token_balance(env: Env, holder: Address, token_id: u64) -> i128
```
Get token balance for specific holder.

### Analytics & Compliance

#### Record Daily Metrics
```rust
fn record_daily_metrics(env: Env) -> Result<(), InsuranceError>
```
Record daily insurance metrics for analytics.

#### Generate Actuarial Report
```rust
fn generate_actuarial_report(env: Env, days: u32) -> Result<RiskDistribution, InsuranceError>
```
Generate risk distribution analysis report.

#### Generate Compliance Report
```rust
fn generate_compliance_report(
    env: Env,
    admin: Address,
    period_days: u32,
) -> Result<u64, InsuranceError>
```
Generate regulatory compliance report.

**Returns:** Report ID

#### Get Compliance Report
```rust
fn get_compliance_report(env: Env, report_id: u64) -> Option<ComplianceReport>
```
Retrieve compliance report by ID.

### Cross-Chain

#### Register Chain Bridge
```rust
fn register_chain_bridge(
    env: Env,
    admin: Address,
    chain_id: Address,
    bridge_address: Address,
) -> Result<(), InsuranceError>
```
Register cross-chain bridge for multi-chain operations.

#### Create Cross-Chain Policy
```rust
fn create_cross_chain_policy(
    env: Env,
    user: Address,
    course_id: u64,
    coverage_amount: i128,
    target_chain: Address,
) -> Result<u64, InsuranceError>
```
Create insurance policy for cross-chain coverage.

## Data Structures

### RiskFactors
```rust
struct RiskFactors {
    completion_rate: u32,          // 0-100
    reputation_score: u32,         // 0-100
    course_difficulty: u32,        // 1-10
    course_duration: u32,          // hours
    experience_level: u32,         // 1-3
    claim_frequency: u32,          // count
    time_since_last_completion: u64, // seconds
}
```

### RiskProfile
```rust
struct RiskProfile {
    profile_id: u64,
    user: Address,
    factors: RiskFactors,
    risk_score: u32,               // 0-100
    timestamp: u64,
}
```

### InsurancePolicy
```rust
struct InsurancePolicy {
    policy_id: u64,
    holder: Address,
    course_id: u64,
    risk_profile_id: u64,
    base_premium: i128,
    risk_multiplier: u32,          // basis points
    final_premium: i128,
    coverage_amount: i128,
    start_time: u64,
    expiration_time: u64,
    status: PolicyStatus,
}
```

### AdvancedClaim
```rust
struct AdvancedClaim {
    claim_id: u64,
    policy_id: u64,
    filed_at: u64,
    status: ClaimStatus,
    ai_confidence: u32,            // 0-100
    evidence_hash: [u8; 32],
    oracle_verified: bool,
    payout_amount: i128,
    reason: String,
}
```

### ParametricTrigger
```rust
struct ParametricTrigger {
    trigger_id: u64,
    course_id: u64,
    metric: LearningMetric,
    threshold: i128,
    payout_amount: i128,
    is_active: bool,
}
```

### OptimizedPool
```rust
struct OptimizedPool {
    pool_id: u64,
    name: String,
    total_assets: i128,
    utilization_rate: u32,         // basis points
    target_utilization: u32,
    risk_reserve_ratio: u32,
    reinsurance_partners: Vec<Address>,
    status: PoolStatus,
}
```

### InsuranceProposal
```rust
struct InsuranceProposal {
    proposal_id: u64,
    title: String,
    description: String,
    proposal_type: ProposalType,
    new_value: i128,
    voting_start: u64,
    voting_end: u64,
    votes_for: u64,
    votes_against: u64,
    status: ProposalStatus,
}
```

### InsuranceToken
```rust
struct InsuranceToken {
    token_id: u64,
    pool_id: u64,
    name: String,
    symbol: String,
    total_supply: i128,
    holder: Address,
    balance: i128,
}
```

### ComplianceReport
```rust
struct ComplianceReport {
    report_id: u64,
    period_start: u64,
    period_end: u64,
    total_policies: u64,
    total_claims: u64,
    claims_paid: u64,
    premiums_collected: i128,
    total_payouts: i128,
    loss_ratio: u32,               // basis points
    reserve_ratio: u32,            // basis points
    generated_at: u64,
}
```

## Enumerations

### PolicyStatus
- `Active`: Policy providing coverage
- `Claimed`: Policy has been claimed against
- `Expired`: Policy has expired
- `Cancelled`: Policy was cancelled

### ClaimStatus
- `Filed`: Claim submitted
- `AiProcessing`: AI verification in progress
- `AiVerified`: AI verified, awaiting oracle
- `OracleConfirmed`: Oracle confirmed
- `Approved`: Claim paid
- `Rejected`: Claim denied
- `Disputed`: Claim in dispute

### LearningMetric
- `CompletionPercentage`: Course completion %
- `CompletionTime`: Time to complete course
- `AssessmentScore`: Assessment results
- `EngagementLevel`: User engagement score
- `AttemptCount`: Number of attempts

### ProposalType
- `PremiumRate`: Base premium rate changes
- `RiskMultiplier`: Risk multiplier ranges
- `UtilizationTarget`: Pool utilization targets
- `ReinsurancePartner`: Reinsurance partnerships
- `Governance`: Governance parameters

### PoolStatus
- `Active`: Pool accepting policies
- `Paused`: Pool maintenance
- `Liquidating`: Pool in liquidation
- `Closed`: Pool closed

## Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 499 | NotInitialized | Contract not initialized |
| 500 | AlreadyInitialized | Contract already initialized |
| 501 | UserNotInsured | User has no active insurance |
| 502 | ClaimNotFound | Claim ID not found |
| 503 | ClaimAlreadyProcessed | Claim already processed |
| 504 | ClaimNotVerified | Claim not verified for payout |
| 505 | InvalidRiskFactors | Risk factors out of range |
| 506 | RiskProfileNotFound | User risk profile not found |
| 507 | PolicyNotFound | Policy ID not found |
| 508 | PolicyExpired | Policy has expired |
| 509 | PolicyAlreadyClaimed | Policy already has claim |
| 510 | InsufficientPremium | Premium amount too low |
| 511 | InvalidParametricTrigger | Invalid trigger parameters |
| 512 | TriggerNotFound | Trigger ID not found |
| 513 | TriggerNotActive | Trigger is deactivated |
| 514 | InvalidLearningMetric | Invalid learning metric type |
| 515 | ClaimAlreadyFiled | Claim already exists for policy |
| 516 | ClaimNotInReviewableState | Claim not ready for review |
| 517 | AiVerificationFailed | AI verification failed |
| 518 | OracleVerificationRequired | Oracle verification needed |
| 519 | PoolNotFound | Pool ID not found |
| 520 | PoolNotActive | Pool is not active |
| 521 | PoolUtilizationTooHigh | Pool utilization exceeded |
| 522 | ReinsurancePartnerNotFound | Reinsurance partner not found |
| 523 | InvalidTokenParameters | Invalid token parameters |
| 524 | TokenNotFound | Token ID not found |
| 525 | InsufficientTokenBalance | Insufficient token balance |
| 526 | ProposalNotFound | Proposal ID not found |
| 527 | VotingPeriodEnded | Voting period has ended |
| 528 | AlreadyVoted | Voter already voted |
| 529 | InvalidProposalType | Invalid proposal type |
| 530 | ProposalNotActive | Proposal not in active state |
| 531 | ComplianceReportNotFound | Report ID not found |
| 532 | InvalidTimeRange | Invalid time range specified |
| 533 | AnalyticsNotAvailable | Analytics data not available |
| 534 | RiskModelNotTrained | Risk model not configured |
| 535 | ExternalOracleError | External oracle service error |
| 536 | CrossChainOperationFailed | Cross-chain operation failed |
| 537 | InvalidCrossChainParameters | Invalid cross-chain parameters |
| 538 | GovernanceQuorumNotMet | Voting quorum not reached |
| 539 | UnauthorizedGovernanceAction | Unauthorized governance action |
| 540 | RiskScoreOutOfRange | Risk score outside valid range |
| 541 | PremiumCalculationError | Error calculating premium |
| 542 | PayoutExceedsCoverage | Payout exceeds coverage amount |
| 543 | EvidenceHashInvalid | Invalid evidence hash format |
| 544 | DisputeResolutionFailed | Dispute resolution failed |
| 545 | PoolLiquidityInsufficient | Insufficient pool liquidity |
| 546 | TokenTransferFailed | Token transfer operation failed |
| 547 | InvalidPoolConfiguration | Invalid pool configuration |
| 548 | ReinsuranceLimitExceeded | Reinsurance allocation exceeded |
| 549 | ParametricConditionNotMet | Parametric trigger conditions not met |
| 550 | ReportGenerationFailed | Compliance report generation failed |