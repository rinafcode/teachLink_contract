use soroban_sdk::{contracttype, Address, Bytes};

/// Error types for governance contract operations
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GovernanceError {
    /// Contract already initialized
    AlreadyInitialized = 1,
    /// Contract not yet initialized
    NotInitialized = 2,
    /// Proposal not found
    ProposalNotFound = 3,
    /// Proposal is not in the expected status
    InvalidProposalStatus = 4,
    /// Voting period is not active
    VotingPeriodNotActive = 5,
    /// Address has already voted on this proposal
    AlreadyVoted = 6,
    /// Address has no voting power (zero token balance)
    NoVotingPower = 7,
    /// Insufficient token balance to create proposal
    InsufficientBalance = 8,
    /// Voting period has not ended yet
    VotingPeriodNotEnded = 9,
    /// Execution delay period has not passed
    ExecutionDelayNotMet = 10,
    /// Only proposer or admin can perform this action
    UnauthorizedCaller = 11,
    /// Proposer can only cancel during voting period
    ProposerCannotCancelAfterVoting = 12,
    /// Cannot cancel executed proposal
    CannotCancelExecutedProposal = 13,
    /// Invalid governance parameters
    InvalidGovernanceConfig = 14,
    /// Title cannot be empty
    EmptyTitle = 15,
    /// Description cannot be empty
    EmptyDescription = 16,
    /// Delegation circular reference detected
    CircularDelegation = 17,
    /// Maximum delegation depth exceeded
    DelegationDepthExceeded = 18,
    /// Cannot delegate to self
    SelfDelegation = 19,
    /// Delegation not found
    DelegationNotFound = 20,
    /// Insufficient staked tokens
    InsufficientStake = 21,
    /// Staking lock period not met
    StakeLockNotMet = 22,
    /// Dispute not found
    DisputeNotFound = 23,
    /// Appeal deadline passed
    AppealDeadlinePassed = 24,
    /// Insufficient quadratic voting credits
    InsufficientQVCredits = 25,
    /// Simulation not found
    SimulationNotFound = 26,
}

/// Types of proposals that can be created in the governance system
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalType {
    /// Change platform fee parameters
    FeeChange,
    /// Update governance or platform parameters
    ParameterUpdate,
    /// Toggle platform features on/off
    FeatureToggle,
    /// Custom proposal with arbitrary execution data
    Custom,
    /// Governance parameter change proposal
    GovernanceChange,
    /// Treasury spending proposal
    TreasurySpend,
    /// Emergency action proposal
    Emergency,
}

/// Status of a proposal throughout its lifecycle
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    /// Proposal created, waiting for voting to start
    Pending,
    /// Voting is active
    Active,
    /// Voting ended with quorum met and majority for
    Passed,
    /// Voting ended with quorum not met or majority against
    Failed,
    /// Proposal has been executed
    Executed,
    /// Proposal was cancelled
    Cancelled,
    /// Proposal is under dispute
    Disputed,
    /// Proposal passed appeal resolution
    Appealed,
}

/// Vote direction options
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VoteDirection {
    For,
    Against,
    Abstain,
}

/// Individual vote record
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vote {
    /// Voter address
    pub voter: Address,
    /// Proposal being voted on
    pub proposal_id: u64,
    /// Voting power at time of vote
    pub power: i128,
    /// Vote direction
    pub direction: VoteDirection,
    /// Timestamp of vote
    pub timestamp: u64,
    /// Whether this vote includes delegated power
    pub includes_delegated: bool,
    /// Amount of delegated power included
    pub delegated_power: i128,
}

/// Key for storing individual votes
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteKey {
    pub proposal_id: u64,
    pub voter: Address,
}

/// Complete proposal information
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    /// Unique proposal identifier
    pub id: u64,
    /// Address that created the proposal
    pub proposer: Address,
    /// Short title for the proposal
    pub title: Bytes,
    /// Detailed description of the proposal
    pub description: Bytes,
    /// Type of proposal
    pub proposal_type: ProposalType,
    /// Current status
    pub status: ProposalStatus,
    /// Creation timestamp
    pub created_at: u64,
    /// When voting begins
    pub voting_start: u64,
    /// When voting ends
    pub voting_end: u64,
    /// Total votes for
    pub for_votes: i128,
    /// Total votes against
    pub against_votes: i128,
    /// Total abstention votes
    pub abstain_votes: i128,
    /// Optional execution data for the proposal
    pub execution_data: Option<Bytes>,
    /// Whether quadratic voting is enabled for this proposal
    pub quadratic_voting: bool,
    /// Total unique voters count
    pub voter_count: u32,
}

/// Governance configuration parameters
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernanceConfig {
    /// Governance token address
    pub token: Address,
    /// Admin address
    pub admin: Address,
    /// Minimum tokens required to create a proposal
    pub proposal_threshold: i128,
    /// Minimum total votes required for quorum
    pub quorum: i128,
    /// Duration of voting period in seconds
    pub voting_period: u64,
    /// Delay before execution after passing (in seconds)
    pub execution_delay: u64,
    /// Maximum delegation chain depth  
    pub max_delegation_depth: u32,
    /// Whether quadratic voting is enabled globally
    pub quadratic_voting_enabled: bool,
    /// Staking multiplier (basis points, 10000 = 1x)
    pub staking_multiplier: u32,
}

// ========== Delegation Types ==========

/// Delegation record tracking vote delegation from one address to another
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Delegation {
    /// Address delegating their voting power
    pub delegator: Address,
    /// Address receiving the delegated power
    pub delegate: Address,
    /// Timestamp when delegation was created
    pub created_at: u64,
    /// Whether the delegation is currently active
    pub active: bool,
    /// Optional expiry timestamp for time-bounded delegation
    pub expires_at: u64,
}

/// Key for delegation storage
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegationKey {
    pub delegator: Address,
}

/// Aggregated delegated power for a delegate
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegatedPower {
    /// Address that has received delegations
    pub delegate: Address,
    /// Total delegated voting power
    pub total_power: i128,
    /// Number of delegators
    pub delegator_count: u32,
}

// ========== Quadratic Voting Types ==========

/// Key for quadratic voting credits
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QVCreditKey {
    pub voter: Address,
    pub proposal_id: u64,
}

/// Quadratic voting credit allocation
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QVCredits {
    /// Total credits allocated
    pub total_credits: i128,
    /// Credits spent on this proposal
    pub spent_credits: i128,
    /// Number of votes purchased (quadratic cost)
    pub votes_purchased: i128,
}

// ========== Staking Types ==========

/// Staking configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakingConfig {
    /// Minimum stake amount
    pub min_stake: i128,
    /// Lock period in seconds
    pub lock_period: u64,
    /// Power multiplier in basis points (10000 = 1x, 15000 = 1.5x)
    pub power_multiplier: u32,
    /// Whether staking is enabled
    pub enabled: bool,
}

/// Individual staking record
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakeInfo {
    /// Staker address
    pub staker: Address,
    /// Amount staked
    pub amount: i128,
    /// When staking started
    pub staked_at: u64,
    /// Lock-up expiry timestamp
    pub lock_until: u64,
    /// Accumulated voting power bonus from staking
    pub power_bonus: i128,
}

/// Key for staking storage
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakeKey {
    pub staker: Address,
}

// ========== Analytics Types ==========

/// Per-address governance participation metrics
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipationRecord {
    /// Address of the participant
    pub participant: Address,
    /// Total proposals voted on
    pub proposals_voted: u32,
    /// Total proposals created
    pub proposals_created: u32,
    /// Total voting power used across all votes
    pub total_power_used: i128,
    /// Number of times served as delegate
    pub delegation_count: u32,
    /// Last activity timestamp
    pub last_active: u64,
    /// Participation score (0-10000 basis points)
    pub participation_score: u32,
}

/// Global governance analytics snapshot
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernanceAnalytics {
    /// Total proposals created
    pub total_proposals: u64,
    /// Total votes cast
    pub total_votes_cast: u64,
    /// Total unique voters
    pub unique_voters: u32,
    /// Average turnout percentage (basis points)
    pub avg_turnout_bps: u32,
    /// Total delegations active
    pub active_delegations: u32,
    /// Total staked tokens
    pub total_staked: i128,
    /// Proposals passed count
    pub proposals_passed: u64,
    /// Proposals failed count
    pub proposals_failed: u64,
    /// Last analytics update timestamp
    pub last_updated: u64,
}

// ========== Dispute Resolution Types ==========

/// Status of a governance dispute
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeStatus {
    /// Dispute is open and pending review
    Open,
    /// Dispute is under review
    UnderReview,
    /// Dispute has been resolved
    Resolved,
    /// Dispute was dismissed
    Dismissed,
    /// Dispute is in appeal
    Appealed,
}

/// Governance dispute record
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dispute {
    /// Unique dispute identifier
    pub id: u64,
    /// Proposal being disputed
    pub proposal_id: u64,
    /// Address that filed the dispute
    pub disputant: Address,
    /// Reason for the dispute
    pub reason: Bytes,
    /// Current dispute status
    pub status: DisputeStatus,
    /// Timestamp of dispute creation
    pub created_at: u64,
    /// Deadline for resolution
    pub resolution_deadline: u64,
    /// Resolution outcome description
    pub resolution: Option<Bytes>,
    /// Address that resolved the dispute
    pub resolver: Option<Address>,
    /// Votes for upholding the dispute
    pub for_votes: i128,
    /// Votes against the dispute
    pub against_votes: i128,
}

/// Key for dispute storage
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeKey {
    pub dispute_id: u64,
}

/// Appeal record for a resolved dispute
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Appeal {
    /// Dispute being appealed
    pub dispute_id: u64,
    /// Address filing the appeal
    pub appellant: Address,
    /// Reason for the appeal
    pub reason: Bytes,
    /// Timestamp of appeal creation
    pub created_at: u64,
    /// Whether the appeal was granted
    pub granted: bool,
}

// ========== Simulation Types ==========

/// Governance simulation snapshot for prediction
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SimulationSnapshot {
    /// Unique simulation identifier
    pub id: u64,
    /// Proposal ID being simulated
    pub proposal_id: u64,
    /// Creator of the simulation
    pub creator: Address,
    /// Simulated for votes
    pub sim_for_votes: i128,
    /// Simulated against votes
    pub sim_against_votes: i128,
    /// Simulated abstain votes
    pub sim_abstain_votes: i128,
    /// Predicted outcome (true=pass, false=fail)
    pub predicted_pass: bool,
    /// Predicted turnout in basis points
    pub predicted_turnout_bps: u32,
    /// Timestamp of simulation
    pub created_at: u64,
}
