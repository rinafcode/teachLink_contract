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
}
