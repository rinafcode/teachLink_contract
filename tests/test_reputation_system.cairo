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
    
