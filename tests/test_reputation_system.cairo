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
    