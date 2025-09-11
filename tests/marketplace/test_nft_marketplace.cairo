#[cfg(test)]
mod tests {
    use core::traits::TryInto;
    use starknet::{ContractAddress, get_contract_address};
    use snforge_std::{declare, ContractClassTrait, DeclareResultTrait, start_cheat_caller_address, stop_cheat_caller_address, start_cheat_block_timestamp, stop_cheat_block_timestamp};
    use openzeppelin::token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherTrait};
    use openzeppelin::token::erc721::interface::{IERC721Dispatcher, IERC721DispatcherTrait};
    use teachlink::contracts::marketplace::interfaces::INFTMarketplace::{INFTMarketplaceDispatcher, INFTMarketplaceDispatcherTrait, ListingType};
    use teachlink::interfaces::itoken::{ITokenDispatcher, ITokenDispatcherTrait};

    const OWNER: felt252 = 'OWNER';
    const USER1: felt252 = 'USER1';
    const USER2: felt252 = 'USER2';
    const INITIAL: u256 = 1_000_000_000000000000000000; // 1e6 ether

    struct Deployed { market: INFTMarketplaceDispatcher, token: ITokenDispatcher, nft: IERC721Dispatcher }

    fn setup() -> Deployed {
        // Deploy ERC20
        let token_class = declare("TeachLinkToken").unwrap().contract_class();
        let mut calldata: Array<felt252> = array![];
        let name: ByteArray = "TLT"; let symbol: ByteArray = "TLT";
        name.serialize(ref calldata); symbol.serialize(ref calldata); INITIAL.serialize(ref calldata);
        start_cheat_caller_address(get_contract_address(), OWNER.try_into().unwrap());
        let (token_addr, _) = token_class.deploy(@calldata).unwrap();
        stop_cheat_caller_address(get_contract_address());

        // Deploy MockERC721
        let nft_class = declare("MockERC721").unwrap().contract_class();
        let mut ncalldata: Array<felt252> = array![];
        let base: felt252 = 'BASE';
        let owner: ContractAddress = OWNER.try_into().unwrap();
        ncalldata.append(owner.into()); ncalldata.append('NFT'.into()); ncalldata.append('NFT'.into()); ncalldata.append(base);
        start_cheat_caller_address(get_contract_address(), OWNER.try_into().unwrap());
        let (nft_addr, _) = nft_class.deploy(@ncalldata).unwrap();
        stop_cheat_caller_address(get_contract_address());

        // Deploy Marketplace
        let mkt_class = declare("NFTMarketplace").unwrap().contract_class();
        let mut mcalldata: Array<felt252> = array![];
        mcalldata.append(owner.into()); mcalldata.append(token_addr.into()); mcalldata.append(250_u16.into());
        start_cheat_caller_address(get_contract_address(), OWNER.try_into().unwrap());
        let (mkt_addr, _) = mkt_class.deploy(@mcalldata).unwrap();
        stop_cheat_caller_address(get_contract_address());

        let token = ITokenDispatcher { contract_address: token_addr };
        let market = INFTMarketplaceDispatcher { contract_address: mkt_addr };
        let nft = IERC721Dispatcher { contract_address: nft_addr };

        // Fund users and approve
        start_cheat_caller_address(token_addr, OWNER.try_into().unwrap());
        token.mint(USER1.try_into().unwrap(), 1_000_000_000000000000000);
        token.mint(USER2.try_into().unwrap(), 1_000_000_000000000000000);
        stop_cheat_caller_address(token_addr);

        let erc20 = IERC20Dispatcher { contract_address: token_addr };
        start_cheat_caller_address(token_addr, USER1.try_into().unwrap());
        erc20.approve(mkt_addr, 1_000_000_000000000000000);
        stop_cheat_caller_address(token_addr);
        start_cheat_caller_address(token_addr, USER2.try_into().unwrap());
        erc20.approve(mkt_addr, 1_000_000_000000000000000);
        stop_cheat_caller_address(token_addr);

        Deployed { market, token, nft }
    }

    #[test]
    fn test_fixed_price_flow_and_royalties() {
        let d = setup();
        // Owner mints NFT to USER1 and approves marketplace
        start_cheat_caller_address(d.nft.contract_address, OWNER.try_into().unwrap());
        let token_id = d.nft.mint(USER1.try_into().unwrap());
        stop_cheat_caller_address(d.nft.contract_address);

        start_cheat_caller_address(d.nft.contract_address, USER1.try_into().unwrap());
        d.nft.approve(d.market.contract_address, token_id);
        stop_cheat_caller_address(d.nft.contract_address);

        // Set collection royalty 5%
        start_cheat_caller_address(d.market.contract_address, OWNER.try_into().unwrap());
        d.market.set_collection_royalty(d.nft.contract_address, USER1.try_into().unwrap(), 500_u16);
        stop_cheat_caller_address(d.market.contract_address);

        // List and buy
        start_cheat_caller_address(d.market.contract_address, USER1.try_into().unwrap());
        let price = 1000_000000000000000000;
        let id = d.market.list_fixed_price(d.nft.contract_address, token_id, price);
        stop_cheat_caller_address(d.market.contract_address);

        start_cheat_caller_address(d.market.contract_address, USER2.try_into().unwrap());
        d.market.buy(id);
        stop_cheat_caller_address(d.market.contract_address);

        // Verify NFT moved
        assert(d.nft.owner_of(token_id) == USER2.try_into().unwrap(), 'nft owner');
        let analytics = d.market.get_analytics();
        assert(analytics.total_sales == 1, 'sales');
    }

    #[test]
    fn test_offer_accept_flow() {
        let d = setup();
        start_cheat_caller_address(d.nft.contract_address, OWNER.try_into().unwrap());
        let token_id = d.nft.mint(USER1.try_into().unwrap());
        stop_cheat_caller_address(d.nft.contract_address);

        // Buyer makes offer
        start_cheat_caller_address(d.market.contract_address, USER2.try_into().unwrap());
        let offer_id = d.market.make_offer(d.nft.contract_address, token_id, 500_000000000000000000, 999999999_u64);
        stop_cheat_caller_address(d.market.contract_address);

        // Seller approves and accepts
        start_cheat_caller_address(d.nft.contract_address, USER1.try_into().unwrap());
        d.nft.approve(d.market.contract_address, token_id);
        stop_cheat_caller_address(d.nft.contract_address);

        start_cheat_caller_address(d.market.contract_address, USER1.try_into().unwrap());
        d.market.accept_offer(offer_id);
        stop_cheat_caller_address(d.market.contract_address);

        assert(d.nft.owner_of(token_id) == USER2.try_into().unwrap(), 'nft owner');
    }

    #[test]
    fn test_english_auction_flow() {
        let d = setup();
        start_cheat_caller_address(d.nft.contract_address, OWNER.try_into().unwrap());
        let token_id = d.nft.mint(USER1.try_into().unwrap());
        stop_cheat_caller_address(d.nft.contract_address);

        // Approve and create auction
        start_cheat_caller_address(d.nft.contract_address, USER1.try_into().unwrap());
        d.nft.approve(d.market.contract_address, token_id);
        stop_cheat_caller_address(d.nft.contract_address);

        start_cheat_caller_address(d.market.contract_address, USER1.try_into().unwrap());
        let auction_id = d.market.create_english_auction(d.nft.contract_address, token_id, 100, 500_u16, 1000_u64);
        stop_cheat_caller_address(d.market.contract_address);

        // Place bids
        start_cheat_caller_address(d.market.contract_address, USER2.try_into().unwrap());
        d.market.bid(auction_id, 200);
        stop_cheat_caller_address(d.market.contract_address);

        // End and finalize
        start_cheat_block_timestamp(d.market.contract_address, 1_000_000);
        d.market.finalize_english(auction_id);
        stop_cheat_block_timestamp(d.market.contract_address);
    }

    #[test]
    fn test_dutch_auction_flow() {
        let d = setup();
        start_cheat_caller_address(d.nft.contract_address, OWNER.try_into().unwrap());
        let token_id = d.nft.mint(USER1.try_into().unwrap());
        stop_cheat_caller_address(d.nft.contract_address);

        start_cheat_caller_address(d.nft.contract_address, USER1.try_into().unwrap());
        d.nft.approve(d.market.contract_address, token_id);
        stop_cheat_caller_address(d.nft.contract_address);

        start_cheat_caller_address(d.market.contract_address, USER1.try_into().unwrap());
        let auction_id = d.market.create_dutch_auction(d.nft.contract_address, token_id, 1000, 100, 1000_u64);
        stop_cheat_caller_address(d.market.contract_address);

        // Buy dutch
        start_cheat_caller_address(d.market.contract_address, USER2.try_into().unwrap());
        d.market.buy_dutch(auction_id);
        stop_cheat_caller_address(d.market.contract_address);
    }

    #[test]
    fn test_recent_sales_and_collection_stats() {
        let d = setup();
        // mint two tokens to USER1
        start_cheat_caller_address(d.nft.contract_address, OWNER.try_into().unwrap());
        let t1 = d.nft.mint(USER1.try_into().unwrap());
        let t2 = d.nft.mint(USER1.try_into().unwrap());
        stop_cheat_caller_address(d.nft.contract_address);

        start_cheat_caller_address(d.nft.contract_address, USER1.try_into().unwrap());
        d.nft.approve(d.market.contract_address, t1);
        d.nft.approve(d.market.contract_address, t2);
        stop_cheat_caller_address(d.nft.contract_address);

        start_cheat_caller_address(d.market.contract_address, USER1.try_into().unwrap());
        let p1 = 100; let p2 = 200;
        let l1 = d.market.list_fixed_price(d.nft.contract_address, t1, p1);
        let l2 = d.market.list_fixed_price(d.nft.contract_address, t2, p2);
        stop_cheat_caller_address(d.market.contract_address);

        start_cheat_caller_address(d.market.contract_address, USER2.try_into().unwrap());
        d.market.buy(l1);
        d.market.buy(l2);
        stop_cheat_caller_address(d.market.contract_address);

        let (cols, amts, ts) = d.market.get_recent_sales(10_u64);
        assert(cols.len() >= 2, 'recent len');
        let (vol, sales) = d.market.get_collection_stats(d.nft.contract_address);
        assert(sales >= 2, 'sales count');
        assert(vol >= (p1 + p2), 'volume');
    }
}


