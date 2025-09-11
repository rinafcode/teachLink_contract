use snforge_std::{ declare, ContractClassTrait, start_prank, stop_prank, Deployable, deploy, cheatcodes::CheatcodesTrait, EventFetcher };
use starknet::{ ContractAddress };

#[test]
fn test_nft_bridge_roundtrip() {
    let owner = ContractAddress::from(1);
    let user = ContractAddress::from(2);
    let relayer = ContractAddress::from(3);

    let nft_class = declare('teachlink::src::contracts::mocks::MockERC721');
    let nft = nft_class.deploy(@(owner, 'MockNFT', 'MNFT', 'ipfs://base'));

    let bridge_class = declare('teachlink::src::contracts::bridge::NFTBridge');
    let bridge = bridge_class.deploy(@(owner, 100_u256, 3600_u64));

    // Configure
    start_prank(owner);
    bridge.set_trusted_relayer(relayer, true);
    bridge.set_collection_supported(nft, true);
    stop_prank();

    // Mint NFT to user
    start_prank(owner);
    let token_id = nft.mint_to(user);
    stop_prank();

    // User approves bridge and deposits NFT
    start_prank(user);
    let _ = nft.approve(bridge, token_id);
    let msg = 'nmsg1';
    let dest = 'L2X';
    bridge.deposit_nft(nft, token_id, dest, user, msg);
    stop_prank();

    // Release by relayer
    start_prank(relayer);
    bridge.release_nft(msg, nft, token_id, user, relayer);
    stop_prank();
}


