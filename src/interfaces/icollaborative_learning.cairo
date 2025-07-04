use starknet::ContractAddress;

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct StudyGroup {
    pub id: u256,
    pub name: ByteArray,
    pub description: ByteArray,
    pub creator: ContractAddress,
    pub max_members: u32,
    pub current_members: u32,
    pub goals: ByteArray,
    pub creation_time: u64,
    pub is_active: bool,
    pub achievement_threshold: u256,
    pub reward_pool: u256,
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct GroupMember {
    pub user: ContractAddress,
    pub group_id: u256,
    pub join_time: u64,
    pub contribution_score: u256,
    pub peer_review_score: u256,
    pub is_active: bool,
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct PeerReview {
    pub id: u256,
    pub reviewer: ContractAddress,
    pub reviewee: ContractAddress,
    pub group_id: u256,
    pub content_hash: felt252, // IPFS hash or similar
    pub rating: u8, // 1-10 scale
    pub feedback: ByteArray,
    pub submission_time: u64,
    pub is_verified: bool,
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct Contribution {
    pub id: u256,
    pub contributor: ContractAddress,
    pub group_id: u256,
    pub content_hash: felt252,
    pub contribution_type: u8, // 0=assignment, 1=research, 2=presentation, 3=other
    pub submission_time: u64,
    pub peer_votes: u32,
    pub total_reviews: u32,
    pub average_rating: u256 // scaled by 100 for precision
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct Dispute {
    pub id: u256,
    pub disputer: ContractAddress,
    pub disputed_against: ContractAddress,
    pub group_id: u256,
    pub reason: ByteArray,
    pub evidence_hash: felt252,
    pub creation_time: u64,
    pub votes_for: u32,
    pub votes_against: u32,
    pub total_voters: u32,
    pub resolved: bool,
    pub resolution: ByteArray,
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct GroupAchievement {
    pub group_id: u256,
    pub achievement_type: u8, // 0=completion, 1=excellence, 2=collaboration
    pub description: ByteArray,
    pub earned_time: u64,
    pub reward_amount: u256,
    pub participants: u32,
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct LearningParameters {
    pub min_peer_reviews: u32,
    pub review_period: u64, // seconds
    pub dispute_voting_period: u64,
    pub min_contribution_score: u256,
    pub achievement_multiplier: u256,
}

#[starknet::interface]
pub trait ICollaborativeLearning<TContractState> {
    // Study Group Management
    fn create_study_group(
        ref self: TContractState,
        name: ByteArray,
        description: ByteArray,
        goals: ByteArray,
        max_members: u32,
        achievement_threshold: u256,
    ) -> u256;

    fn join_study_group(ref self: TContractState, group_id: u256);
    fn leave_study_group(ref self: TContractState, group_id: u256);
    fn update_group_goals(ref self: TContractState, group_id: u256, new_goals: ByteArray);
    fn deactivate_group(ref self: TContractState, group_id: u256);

    // Contribution System
    fn submit_contribution(
        ref self: TContractState, group_id: u256, content_hash: felt252, contribution_type: u8,
    ) -> u256;

    fn vote_on_contribution(ref self: TContractState, contribution_id: u256, is_positive: bool);

    // Peer Review System
    fn submit_peer_review(
        ref self: TContractState,
        reviewee: ContractAddress,
        group_id: u256,
        content_hash: felt252,
        rating: u8,
        feedback: ByteArray,
    ) -> u256;

    fn verify_peer_review(ref self: TContractState, review_id: u256);

    // Dispute Resolution
    fn create_dispute(
        ref self: TContractState,
        disputed_against: ContractAddress,
        group_id: u256,
        reason: ByteArray,
        evidence_hash: felt252,
    ) -> u256;

    fn vote_on_dispute(ref self: TContractState, dispute_id: u256, support: bool);
    fn resolve_dispute(ref self: TContractState, dispute_id: u256);

    // Achievement & Rewards
    fn claim_group_achievement(ref self: TContractState, group_id: u256);
    fn distribute_rewards(ref self: TContractState, group_id: u256);

    // View Functions
    fn get_study_group(self: @TContractState, group_id: u256) -> StudyGroup;
    fn get_group_member(
        self: @TContractState, group_id: u256, user: ContractAddress,
    ) -> GroupMember;
    fn get_peer_review(self: @TContractState, review_id: u256) -> PeerReview;
    fn get_contribution(self: @TContractState, contribution_id: u256) -> Contribution;
    fn get_dispute(self: @TContractState, dispute_id: u256) -> Dispute;
    fn get_group_achievement(self: @TContractState, group_id: u256) -> GroupAchievement;
    fn get_user_groups(self: @TContractState, user: ContractAddress) -> Array<u256>;
    fn get_group_members(self: @TContractState, group_id: u256) -> Array<ContractAddress>;
    fn get_user_contribution_score(
        self: @TContractState, user: ContractAddress, group_id: u256,
    ) -> u256;
    fn get_user_peer_review_score(
        self: @TContractState, user: ContractAddress, group_id: u256,
    ) -> u256;
    fn get_learning_parameters(self: @TContractState) -> LearningParameters;
    fn get_group_count(self: @TContractState) -> u256;

    // Admin Functions
    fn initialize(
        ref self: TContractState, token_address: ContractAddress, params: LearningParameters,
    );
    fn update_learning_parameters(ref self: TContractState, new_params: LearningParameters);
}
