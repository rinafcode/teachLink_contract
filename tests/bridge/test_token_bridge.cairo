use snforge_std::{ declare, ContractClassTrait, start_prank, stop_prank, Deployable, deploy, cheatcodes::CheatcodesTrait, EventAssertion, EventFetcher, assert_eq };
use starknet::{ ContractAddress };

#[test]
fn test_token_bridge_lock_and_release() {
    // Deploy mocks
    let owner = ContractAddress::from(1);
    let user = ContractAddress::from(2);
    let relayer = ContractAddress::from(3);

    let erc20_class = declare('teachlink::src::contracts::mocks::MockERC20');
    let erc20 = erc20_class.deploy(@(owner, 'Mock', 'MOCK', 18_u8));

    let bridge_class = declare('teachlink::src::contracts::bridge::TokenBridge');
    let per_window: u256 = 1_000_000_000_000_000_000_u256; // 1e18
    let bridge = bridge_class.deploy(@(owner, erc20, per_window, 3600_u64));

    // Mint tokens to user and approve bridge
    start_prank(owner);
    let _ = erc20.mint_to(user, 1_000_000_u256);
    stop_prank();

    start_prank(user);
    let _ = erc20.approve(bridge, 500_000_u256);

    // Deposit
    let message_id = 'msg1';
    let dest_chain = 'L2X';
    bridge.deposit(200_000_u256, dest_chain, user, message_id);

    // Verify event
    let mut fetcher = EventFetcher::new(bridge);
    let ev = fetcher.fetch_one();
    ev.assert_emitted('BridgeRequested');

    stop_prank();

    // Owner sets trusted relayer
    start_prank(owner);
    bridge.set_trusted_relayer(relayer, true);
    stop_prank();

    // Release on destination chain by relayer
    start_prank(relayer);
    bridge.release(message_id, user, 200_000_u256, relayer);
    stop_prank();
}


