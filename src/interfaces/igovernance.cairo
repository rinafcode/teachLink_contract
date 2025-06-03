use starknet::{ClassHash, ContractAddress};

#[derive(Drop, Serde, starknet::Store)]
pub struct Proposal {
    pub id: u256,
    pub title: ByteArray,
    pub description: ByteArray,
    pub proposer: ContractAddress,
    pub target: ContractAddress,
    pub calldata_len: u32,
    pub value: u256,
    pub votes_for: u256,
    pub votes_against: u256,
    pub votes_abstain: u256,
    pub start_time: u64,
    pub end_time: u64,
    pub executed: bool,
    pub canceled: bool,
}

#[derive(Drop, Serde, starknet::Store)]
pub struct Vote {
    pub voter: ContractAddress,
    pub proposal_id: u256,
    pub support: u8, // 0 = against, 1 = for, 2 = abstain
    pub weight: u256,
    pub reason: ByteArray,
}

#[derive(Drop, Serde, starknet::Store)]
pub struct Delegation {
    pub delegator: ContractAddress,
    pub delegate: ContractAddress,
    pub timestamp: u64,
}

#[derive(Drop, Serde, starknet::Store)]
pub struct GovernanceParameters {
    pub voting_delay: u64, // Delay before voting starts (in seconds)
    pub voting_period: u64, // Voting period duration (in seconds)
    pub proposal_threshold: u256, // Minimum tokens needed to create proposal
    pub quorum_threshold: u256, // Minimum participation for valid vote
    pub execution_delay: u64 // Delay before execution after approval
}

#[starknet::interface]
pub trait IGovernance<TContractState> {
    // Proposal Management
    fn create_proposal(
        ref self: TContractState,
        title: ByteArray,
        description: ByteArray,
        target: ContractAddress,
        calldata: Span<felt252>,
        value: u256,
    ) -> u256;

    fn cancel_proposal(ref self: TContractState, proposal_id: u256);
    fn execute_proposal(ref self: TContractState, proposal_id: u256);

    // Voting Functions
    fn cast_vote(ref self: TContractState, proposal_id: u256, support: u8, reason: ByteArray);

    // Delegation Functions
    fn delegate(ref self: TContractState, delegate: ContractAddress);
    fn undelegate(ref self: TContractState);

    // View Functions
    fn get_proposal(self: @TContractState, proposal_id: u256) -> Proposal;
    fn get_proposal_state(self: @TContractState, proposal_id: u256) -> u8;
    fn get_vote(self: @TContractState, proposal_id: u256, voter: ContractAddress) -> Vote;
    fn get_voting_power(self: @TContractState, account: ContractAddress, timestamp: u64) -> u256;
    fn get_delegate(self: @TContractState, account: ContractAddress) -> ContractAddress;
    fn get_delegation(self: @TContractState, delegator: ContractAddress) -> Delegation;
    fn get_governance_parameters(self: @TContractState) -> GovernanceParameters;
    fn get_proposal_count(self: @TContractState) -> u256;
    fn has_voted(self: @TContractState, proposal_id: u256, voter: ContractAddress) -> bool;

    // Parameter Management (only through governance)
    fn update_governance_parameters(
        ref self: TContractState,
        voting_delay: u64,
        voting_period: u64,
        proposal_threshold: u256,
        quorum_threshold: u256,
        execution_delay: u64,
    );

    // Admin Functions
    fn initialize(
        ref self: TContractState,
        token_address: ContractAddress,
        initial_params: GovernanceParameters,
    );

    fn upgrade(ref self: TContractState, new_class_hash: ClassHash);
}
