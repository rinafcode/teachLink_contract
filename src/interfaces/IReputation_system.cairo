use starknet::ContractAddress;

#[starknet::interface]
trait IReputationSystem<TContractState> {
    // Soulbound Token Management
    fn mint_instructor_token(ref self: TContractState, instructor: ContractAddress, initial_score: u256);
    fn get_instructor_token_id(self: @TContractState, instructor: ContractAddress) -> u256;
    fn is_instructor_registered(self: @TContractState, instructor: ContractAddress) -> bool;
    
    // Review System
    fn submit_review(
        ref self: TContractState,
        instructor: ContractAddress,
        course_id: u256,
        rating: u8,
        review_hash: felt252,
        proof: Array<felt252>
    );
    fn get_review(self: @TContractState, review_id: u256) -> Review;
    fn get_instructor_reviews(self: @TContractState, instructor: ContractAddress) -> Array<u256>;
    
    // Reputation Scoring
    fn calculate_reputation_score(self: @TContractState, instructor: ContractAddress) -> u256;
    fn get_weighted_score(self: @TContractState, instructor: ContractAddress) -> u256;
    fn update_reputation_score(ref self: TContractState, instructor: ContractAddress);
    
    // Anti-Manipulation
    fn report_suspicious_activity(ref self: TContractState, target: ContractAddress, evidence_hash: felt252);
    fn flag_review(ref self: TContractState, review_id: u256, reason: u8);
    fn get_reviewer_credibility(self: @TContractState, reviewer: ContractAddress) -> u256;
    
    // Query Functions
    fn get_reputation_history(self: @TContractState, instructor: ContractAddress) -> Array<ReputationSnapshot>;
    fn get_top_instructors(self: @TContractState, limit: u32) -> Array<ContractAddress>;
    fn verify_reputation_score(self: @TContractState, instructor: ContractAddress, claimed_score: u256) -> bool;
    
    // Admin Functions
    fn set_minimum_credibility(ref self: TContractState, threshold: u256);
    fn pause_contract(ref self: TContractState);
    fn unpause_contract(ref self: TContractState);
}

#[derive(Drop, Serde, starknet::Store)]
struct Review {
    id: u256,
    reviewer: ContractAddress,
    instructor: ContractAddress,
    course_id: u256,
    rating: u8,
    review_hash: felt252,
    timestamp: u64,
    weight: u256,
    is_flagged: bool,
    credibility_score: u256,
}

#[derive(Drop, Serde, starknet::Store)]
struct InstructorReputation {
    token_id: u256,
    instructor: ContractAddress,
    total_score: u256,
    weighted_score: u256,
    review_count: u32,
    last_updated: u64,
    is_active: bool,
}

#[derive(Drop, Serde, starknet::Store)]
struct ReputationSnapshot {
    timestamp: u64,
    score: u256,
    review_count: u32,
    event_type: u8, // 0: review_added, 1: score_updated, 2: flagged
}

#[derive(Drop, Serde, starknet::Store)]
struct ReviewerProfile {
    address: ContractAddress,
    credibility_score: u256,
    total_reviews: u32,
    accurate_reviews: u32,
    flagged_reviews: u32,
    registration_time: u64,
}
