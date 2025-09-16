use starknet::{ContractAddress, contract_address_const, get_contract_address};
use snforge_std::{declare, ContractClassTrait, start_cheat_caller_address, stop_cheat_caller_address, start_cheat_block_timestamp, stop_cheat_block_timestamp};
use assert_macros::assert;

use src::contracts::access::interfaces::IContentAccess::{IContentAccessDispatcher, IContentAccessDispatcherTrait, CourseAccessConfig, AccessMethod, KeyPolicy};
use src::contracts::mocks::MockERC721::{MockERC721Dispatcher, MockERC721DispatcherTrait};

fn deploy_course_nft() -> MockERC721Dispatcher {
    let class = declare("MockERC721").unwrap().contract_class();
    let calldata = array![contract_address_const!('owner'), 'NFT', 'NFT', 'BASE'];
    let addr = class.deploy(@calldata).unwrap();
    MockERC721Dispatcher { contract_address: addr }
}

fn deploy_content_access() -> IContentAccessDispatcher {
    let class = declare("ContentAccess").unwrap().contract_class();
    let calldata = array![contract_address_const!('owner')];
    let addr = class.deploy(@calldata).unwrap();
    IContentAccessDispatcher { contract_address: addr }
}

#[test]
fn test_token_gated_access_and_key() {
    let nft = deploy_course_nft();
    let ca = deploy_content_access();

    // configure
    start_cheat_caller_address(ca.contract_address, contract_address_const!('owner'));
    ca.set_course_nft(nft.contract_address);
    let cfg = CourseAccessConfig { course_id: 1, seller: contract_address_const!('seller'), access_method: AccessMethod::Token, subscription_plan_id: 0, start_time: 0, end_time: 0, key_policy: KeyPolicy::PerCourse };
    ca.configure_course(cfg);
    ca.set_course_key(1, 'KEY1');
    stop_cheat_caller_address(ca.contract_address);

    // mint nft to user
    start_cheat_caller_address(nft.contract_address, contract_address_const!('owner'));
    let token_id = nft.mint(contract_address_const!('user'));
    stop_cheat_caller_address(nft.contract_address);
    assert(token_id == 1, 'id');

    // user should have access and key
    assert(ca.has_access(contract_address_const!('user'), 1), 'has');
    let key = ca.request_content_key(contract_address_const!('user'), 1);
    assert(key == 'KEY1', 'key');
}

#[test]
fn test_time_restriction_and_manual_grant() {
    let nft = deploy_course_nft();
    let ca = deploy_content_access();

    start_cheat_caller_address(ca.contract_address, contract_address_const!('owner'));
    ca.set_course_nft(nft.contract_address);
    // allow only in [100, 200]
    let cfg = CourseAccessConfig { course_id: 2, seller: contract_address_const!('seller'), access_method: AccessMethod::Token, subscription_plan_id: 0, start_time: 100, end_time: 200, key_policy: KeyPolicy::PerCourse };
    ca.configure_course(cfg);
    stop_cheat_caller_address(ca.contract_address);

    start_cheat_block_timestamp(ca.contract_address, 50_u64);
    assert(!ca.has_access(contract_address_const!('user'), 2), 'no access before window');
    stop_cheat_block_timestamp(ca.contract_address);

    // grant manual access 150 seconds for user starting at t=120
    start_cheat_caller_address(ca.contract_address, contract_address_const!('owner'));
    start_cheat_block_timestamp(ca.contract_address, 120_u64);
    ca.grant_access(contract_address_const!('user'), 2, 100_u64);
    stop_cheat_block_timestamp(ca.contract_address);
    stop_cheat_caller_address(ca.contract_address);

    start_cheat_block_timestamp(ca.contract_address, 180_u64);
    assert(ca.has_access(contract_address_const!('user'), 2), 'has inside');
    stop_cheat_block_timestamp(ca.contract_address);

    start_cheat_block_timestamp(ca.contract_address, 250_u64);
    assert(!ca.has_access(contract_address_const!('user'), 2), 'expired after window');
    stop_cheat_block_timestamp(ca.contract_address);
}


