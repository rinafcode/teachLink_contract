use starknet::{ContractAddress, contract_address_const, get_block_timestamp};
use snforge_std::{
    declare, ContractClassTrait, DeclareResultTrait, start_cheat_caller_address,
    stop_cheat_caller_address, start_cheat_block_timestamp, stop_cheat_block_timestamp
};

use reputation_system::contracts::reputation::ReputationSystem;
use reputation_system::contracts::reputation::interfaces::IReputationSystem::{
    IReputationSystemDispatcher, IReputationSystemDispatcherTrait, Review
};

fn OWNER() -> ContractAddress {
    contract_address_const::<'owner'>()
}

fn INSTRUCTOR1() -> ContractAddress {
    contract_address_const::<'instructor1'>()
}

fn INSTRUCTOR2() -> ContractAddress {
    contract_address_const::<'instructor2'>()
}

fn REVIEWER1() -> ContractAddress {
    contract_address_const::<'reviewer1'>()
}

fn REVIEWER2() -> ContractAddress {
    contract_address_const::<'reviewer2'>()
}

fn deploy_contract() -> IReputationSystemDispatcher {
    let contract = declare("ReputationSystem").unwrap().contract_class();
    let constructor_args = array![
        OWNER().into(),
        'MarketX Reputation',
        'MXREP'
    ];
    let (contract_address, _) = contract.deploy(@constructor_args).unwrap();
    IReputationSystemDispatcher { contract_address }
}

#[test]
fn test_mint_instructor_token() {
    let contract = deploy_contract();
    
    start_cheat_caller_address(contract.contract_address, OWNER());
    
    // Mint token for instructor
    contract.mint_instructor_token(INSTRUCTOR1(), 75);
    

    // Verify instructor is registered
    assert!(contract.is_instructor_registered(INSTRUCTOR1()), "Instructor should be registered");
    
    // Verify token ID
    let token_id = contract.get_instructor_token_id(INSTRUCTOR1());
    assert!(token_id == 1, "Token ID should be 1");
    
    // Verify initial score
    let score = contract.get_weighted_score(INSTRUCTOR1());
    assert!(score == 75, "Initial score should be 75");
    
    stop_cheat_caller_address(contract.contract_address);
}

#[test]
#[should_panic(expected: 'Instructor already registered')]
fn test_mint_duplicate_instructor_token() {
    let contract = deploy_contract();
    
    start_cheat_caller_address(contract.contract_address, OWNER());
    
    // Mint first token
    contract.mint_instructor_token(INSTRUCTOR1(), 75);
    
    // Try to mint duplicate - should panic
    contract.mint_instructor_token(INSTRUCTOR1(), 80);
    
    stop_cheat_caller_address(contract.contract_address);
}

#[test]
fn test_submit_review() {
    let contract = deploy_contract();
    
    // Setup instructor
    start_cheat_caller_address(contract.contract_address, OWNER());
    contract.mint_instructor_token(INSTRUCTOR1(), 75);
    stop_cheat_caller_address(contract.contract_address);
    
    // Submit review as reviewer
    start_cheat_caller_address(contract.contract_address, REVIEWER1());
    contract.submit_review(
        INSTRUCTOR1(),
        1, // course_id
        5, // rating
        'review_hash_123',
        array!['proof1', 'proof2']
    );
    stop_cheat_caller_address(contract.contract_address);
    
    
    // Verify review was created
    let review = contract.get_review(1);
    assert!(review.reviewer == REVIEWER1(), "Reviewer should match");
    assert!(review.instructor == INSTRUCTOR1(), "Instructor should match");
    assert!(review.rating == 5, "Rating should be 5");
    assert!(review.course_id == 1, "Course ID should be 1");
    
    // Verify instructor's review list
    let reviews = contract.get_instructor_reviews(INSTRUCTOR1());
    assert!(reviews.len() == 1, "Should have 1 review");
    assert!(*reviews.at(0) == 1, "Review ID should be 1");
}

#[test]
#[should_panic(expected: 'Rating must be between 1 and 5')]
fn test_submit_invalid_rating() {
    let contract = deploy_contract();
    
    // Setup instructor
    start_cheat_caller_address(contract.contract_address, OWNER());
    contract.mint_instructor_token(INSTRUCTOR1(), 75);
    stop_cheat_caller_address(contract.contract_address);
    
    // Submit invalid review
    start_cheat_caller_address(contract.contract_address, REVIEWER1());
    contract.submit_review(
        INSTRUCTOR1(),
        1,
        6, // Invalid rating > 5
        'review_hash_123',
        array![]
    );
    stop_cheat_caller_address(contract.contract_address);
}

#[test]
fn test_reputation_score_calculation() {
    let contract = deploy_contract();
    
    // Setup instructor
    start_cheat_caller_address(contract.contract_address, OWNER());
    contract.mint_instructor_token(INSTRUCTOR1(), 50);
    stop_cheat_caller_address(contract.contract_address);
    
    // Submit multiple reviews
    start_cheat_caller_address(contract.contract_address, REVIEWER1());
    contract.submit_review(INSTRUCTOR1(), 1, 5, 'review1', array![]);
    stop_cheat_caller_address(contract.contract_address);
    
    
    start_cheat_caller_address(contract.contract_address, REVIEWER2());
    contract.submit_review(INSTRUCTOR1(), 2, 4, 'review2', array![]);
    stop_cheat_caller_address(contract.contract_address);
    
    // Check updated reputation score
    let score = contract.get_weighted_score(INSTRUCTOR1());
    assert!(score > 50, "Score should be updated and higher than initial");
    
    // Verify score calculation
    let calculated_score = contract.calculate_reputation_score(INSTRUCTOR1());
    assert!(calculated_score == score, "Calculated score should match stored score");
}

#[test]
fn test_flag_review() {
    let contract = deploy_contract();
    
    // Setup instructor and submit review
    start_cheat_caller_address(contract.contract_address, OWNER());
    contract.mint_instructor_token(INSTRUCTOR1(), 75);
    stop_cheat_caller_address(contract.contract_address);
    
    start_cheat_caller_address(contract.contract_address, REVIEWER1());
    contract.submit_review(INSTRUCTOR1(), 1, 5, 'review1', array![]);
    stop_cheat_caller_address(contract.contract_address);
    
    // Flag the review
    start_cheat_caller_address(contract.contract_address, OWNER());
    contract.flag_review(1, 1); // reason: 1 (spam)
    stop_cheat_caller_address(contract.contract_address);
    
    // Verify review is flagged
    let review = contract.get_review(1);
    assert!(review.is_flagged, "Review should be flagged");
}

#[test]
fn test_reviewer_credibility() {
    let contract = deploy_contract();
    
    // Setup instructor
    start_cheat_caller_address(contract.contract_address, OWNER());
    contract.mint_instructor_token(INSTRUCTOR1(), 75);
    stop_cheat_caller_address(contract.contract_address);
    
    // Submit review to establish reviewer profile
    start_cheat_caller_address(contract.contract_address, REVIEWER1());
    contract.submit_review(INSTRUCTOR1(), 1, 5, 'review1', array![]);
    stop_cheat_caller_address(contract.contract_address);
    
    
    // Check reviewer credibility
    let credibility = contract.get_reviewer_credibility(REVIEWER1());
    assert!(credibility >= 50, "New reviewer should have at least 50% credibility");
}

#[test]
fn test_anti_manipulation_measures() {
    let contract = deploy_contract();
    
    // Setup instructor
    start_cheat_caller_address(contract.contract_address, OWNER());
    contract.mint_instructor_token(INSTRUCTOR1(), 50);
    stop_cheat_caller_address(contract.contract_address);
    
    // Submit multiple reviews from same reviewer (suspicious pattern)
    start_cheat_caller_address(contract.contract_address, REVIEWER1());
    contract.submit_review(INSTRUCTOR1(), 1, 5, 'review1', array![]);
    contract.submit_review(INSTRUCTOR1(), 2, 5, 'review2', array![]);
    contract.submit_review(INSTRUCTOR1(), 3, 5, 'review3', array![]);
    contract.submit_review(INSTRUCTOR1(), 4, 5, 'review4', array![]);
    stop_cheat_caller_address(contract.contract_address);
    
    // Score should be penalized for suspicious patterns
    let score = contract.get_weighted_score(INSTRUCTOR1());
    let calculated_score = contract.calculate_reputation_score(INSTRUCTOR1());
    
    // The calculated score should be lower due to anti-gaming penalties
    assert!(calculated_score < 100, "Score should be penalized for suspicious patterns");
}

#[test]
fn test_reputation_verification() {
    let contract = deploy_contract();
    
    // Setup instructor
    start_cheat_caller_address(contract.contract_address, OWNER());
    contract.mint_instructor_token(INSTRUCTOR1(), 75);
    stop_cheat_caller_address(contract.contract_address);
    
    // Get current score
    let actual_score = contract.get_weighted_score(INSTRUCTOR1());
    
    // Verify correct score
    assert!(contract.verify_reputation_score(INSTRUCTOR1(), actual_score), "Should verify correct score");
    
    // Verify incorrect score
    assert!(!contract.verify_reputation_score(INSTRUCTOR1(), actual_score + 10), "Should reject incorrect score");
}

#[test]
fn test_minimum_credibility_threshold() {
    let contract = deploy_contract();
    
    // Set high minimum credibility
    start_cheat_caller_address(contract.contract_address, OWNER());
    contract.set_minimum_credibility(80);
    contract.mint_instructor_token(INSTRUCTOR1(), 75);
    stop_cheat_caller_address(contract.contract_address);
    
    // New reviewer with default 50% credibility should not be able to review
    start_cheat_caller_address(contract.contract_address, REVIEWER1());
    
    // This should fail due to insufficient credibility
    let result = std::panic::catch_unwind(|| {
        contract.submit_review(INSTRUCTOR1(), 1, 5, 'review1', array![]);
    });
    
    assert!(result.is_err(), "Should fail due to insufficient credibility");
    
    stop_cheat_caller_address(contract.contract_address);
}