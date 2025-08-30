use starknet::{ContractAddress, contract_address_const, get_block_timestamp};
use snforge_std::{declare, ContractClassTrait, start_prank, stop_prank, CheatTarget};
use super::super::contracts::identity::MarketXIdentity;
use super::super::contracts::identity::interfaces::IMarketXIdentity::{
    IMarketXIdentityDispatcher, IMarketXIdentityDispatcherTrait, DID, VerifiableCredential
};

fn deploy_contract() -> (IMarketXIdentityDispatcher, ContractAddress) {
    let contract = declare("MarketXIdentity");
    let owner = contract_address_const::<'owner'>();
    let constructor_calldata = array![owner.into()];
    let contract_address = contract.deploy(@constructor_calldata).unwrap();
    (IMarketXIdentityDispatcher { contract_address }, owner)
}

#[test]
fn test_create_did() {
    let (contract, owner) = deploy_contract();
    let user = contract_address_const::<'user'>();
    
    start_prank(CheatTarget::One(contract.contract_address), user);
    
    let did_id = contract.create_did(user);
    assert(did_id == 1, 'DID ID should be 1');
    
    let did = contract.get_did(did_id);
    assert(did.controller == user, 'Controller should be user');
    assert(did.is_active == true, 'DID should be active');
    
    stop_prank(CheatTarget::One(contract.contract_address));
}

#[test]
fn test_issue_credential() {
    let (contract, owner) = deploy_contract();
    let user = contract_address_const::<'user'>();
    
    // Create DID first
    start_prank(CheatTarget::One(contract.contract_address), user);
    let did_id = contract.create_did(user);
    stop_prank(CheatTarget::One(contract.contract_address));
    
    // Issue credential as owner (authorized issuer)
    start_prank(CheatTarget::One(contract.contract_address), owner);
    let credential_id = contract.issue_credential(
        did_id,
        'degree',
        'computer_science_bs',
        get_block_timestamp() + 31536000 // 1 year from now
    );
