#[starknet::contract]
pub mod NFTMarketplace {
    use starknet::{
        ContractAddress, get_caller_address, get_contract_address, get_block_timestamp,
        storage::{Map, StorageMapReadAccess, StorageMapWriteAccess, StoragePointerReadAccess, StoragePointerWriteAccess},
    };
    use core::array::{Array, ArrayTrait};
    use openzeppelin::access::ownable::OwnableComponent;
    use openzeppelin::token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherTrait};
    use openzeppelin::token::erc721::interface::{IERC721Dispatcher, IERC721DispatcherTrait};

    use super::interfaces::INFTMarketplace::{
        INFTMarketplace, Listing, ListingType, Offer, EnglishAuction, DutchAuction, Analytics,
    };
    use super::libraries::AuctionLogic::{AuctionLogic, MinBidResult};
    use super::RoyaltyDistributor::RoyaltyDistributor;

    component!(path: OwnableComponent, storage: ownable, event: OwnableEvent);

    #[abi(embed_v0)]
    impl OwnableImpl = OwnableComponent::OwnableImpl<ContractState>;
    impl OwnableInternalImpl = OwnableComponent::InternalImpl<ContractState>;

    #[storage]
    struct Storage {
        payment_token: ContractAddress,
        platform_fee_bps: u16,

        // Counters
        listing_counter: u256,
        offer_counter: u256,
        english_counter: u256,
        dutch_counter: u256,

        // Core state
        listings: Map<u256, Listing>,
        offers: Map<u256, Offer>,
        english_auctions: Map<u256, EnglishAuction>,
        dutch_auctions: Map<u256, DutchAuction>,

        // Token to active listing mapping
        active_listing_by_token: Map<(ContractAddress, u256), u256>,

        // Royalty defaults per collection
        royalty_recipient_by_collection: Map<ContractAddress, ContractAddress>,
        royalty_bps_by_collection: Map<ContractAddress, u16>,

        // Analytics
        analytics: Analytics,
        collection_volume: Map<ContractAddress, u256>,
        collection_sales_count: Map<ContractAddress, u256>,

        // Recent sales ring buffer (trending support)
        recent_sales_head: u64,
        recent_sales_count: u64,
        recent_sales_collections: Map<u64, ContractAddress>,
        recent_sales_amounts: Map<u64, u256>,
        recent_sales_timestamps: Map<u64, u64>,

        #[substorage(v0)]
        ownable: OwnableComponent::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        Listed: Listed,
        Purchased: Purchased,
        ListingCanceled: ListingCanceled,
        OfferMade: OfferMade,
        OfferCanceled: OfferCanceled,
        OfferAccepted: OfferAccepted,
        EnglishAuctionCreated: EnglishAuctionCreated,
        BidPlaced: BidPlaced,
        EnglishFinalized: EnglishFinalized,
        DutchAuctionCreated: DutchAuctionCreated,
        DutchPurchased: DutchPurchased,
        RoyaltyUpdated: RoyaltyUpdated,
        #[flat]
        OwnableEvent: OwnableComponent::Event,
    }

    #[derive(Drop, starknet::Event)]
    pub struct Listed { pub listing_id: u256, pub collection: ContractAddress, pub token_id: u256, pub seller: ContractAddress, pub price: u256, pub listing_type: ListingType }
    #[derive(Drop, starknet::Event)]
    pub struct Purchased { pub listing_id: u256, pub buyer: ContractAddress, pub price: u256 }
    #[derive(Drop, starknet::Event)]
    pub struct ListingCanceled { pub listing_id: u256 }
    #[derive(Drop, starknet::Event)]
    pub struct OfferMade { pub offer_id: u256, pub collection: ContractAddress, pub token_id: u256, pub bidder: ContractAddress, pub amount: u256 }
    #[derive(Drop, starknet::Event)]
    pub struct OfferCanceled { pub offer_id: u256 }
    #[derive(Drop, starknet::Event)]
    pub struct OfferAccepted { pub offer_id: u256, pub seller: ContractAddress }
    #[derive(Drop, starknet::Event)]
    pub struct EnglishAuctionCreated { pub auction_id: u256, pub collection: ContractAddress, pub token_id: u256, pub seller: ContractAddress, pub end_time: u64 }
    #[derive(Drop, starknet::Event)]
    pub struct BidPlaced { pub auction_id: u256, pub bidder: ContractAddress, pub amount: u256 }
    #[derive(Drop, starknet::Event)]
    pub struct EnglishFinalized { pub auction_id: u256, pub winner: ContractAddress, pub amount: u256 }
    #[derive(Drop, starknet::Event)]
    pub struct DutchAuctionCreated { pub auction_id: u256, pub collection: ContractAddress, pub token_id: u256, pub seller: ContractAddress, pub start_price: u256, pub end_price: u256, pub end_time: u64 }
    #[derive(Drop, starknet::Event)]
    pub struct DutchPurchased { pub auction_id: u256, pub buyer: ContractAddress, pub price: u256 }
    #[derive(Drop, starknet::Event)]
    pub struct RoyaltyUpdated { pub collection: ContractAddress, pub recipient: ContractAddress, pub bps: u16 }

    #[constructor]
    fn constructor(ref self: ContractState, owner: ContractAddress, payment_token: ContractAddress, platform_fee_bps: u16) {
        self.ownable.initializer(owner);
        assert(platform_fee_bps <= 1000, 'fee>10%');
        self.payment_token.write(payment_token);
        self.platform_fee_bps.write(platform_fee_bps);
        self.listing_counter.write(0);
        self.offer_counter.write(0);
        self.english_counter.write(0);
        self.dutch_counter.write(0);
        self.analytics.write(Analytics { total_volume: 0, total_sales: 0, total_auctions: 0, total_offers: 0 });
        self.recent_sales_head.write(0);
        self.recent_sales_count.write(0);
    }

    #[abi(embed_v0)]
    impl MarketplaceImpl of INFTMarketplace<ContractState> {
        // Admin
        fn set_platform_fee(ref self: ContractState, bps: u16) {
            self.ownable.assert_only_owner();
            assert(bps <= 1000, 'fee>10%');
            self.platform_fee_bps.write(bps);
        }
        fn set_collection_royalty(ref self: ContractState, collection: ContractAddress, recipient: ContractAddress, bps: u16) {
            self.ownable.assert_only_owner();
            assert(bps <= 10000, 'royalty>100%');
            self.royalty_recipient_by_collection.write(collection, recipient);
            self.royalty_bps_by_collection.write(collection, bps);
            self.emit(RoyaltyUpdated { collection, recipient, bps });
        }

        // Fixed price listings
        fn list_fixed_price(ref self: ContractState, collection: ContractAddress, token_id: u256, price: u256) -> u256 {
            assert(price > 0, 'price=0');
            let seller = get_caller_address();
            // Require seller to have approved marketplace for transfer
            let listing_id = self.listing_counter.read() + 1;
            self.listing_counter.write(listing_id);
            let listing = Listing { id: listing_id, collection, token_id, seller, price, active: true, listing_type: ListingType::FixedPrice, created_at: get_block_timestamp() };
            self.listings.write(listing_id, listing);
            self.active_listing_by_token.write((collection, token_id), listing_id);
            self.emit(Listed { listing_id, collection, token_id, seller, price, listing_type: ListingType::FixedPrice });
            listing_id
        }

        fn cancel_listing(ref self: ContractState, listing_id: u256) {
            let mut listing = self.listings.read(listing_id);
            assert(listing.id != 0, 'no listing');
            assert(listing.active, 'inactive');
            assert(listing.seller == get_caller_address(), 'not seller');
            listing.active = false;
            self.listings.write(listing_id, listing);
            self.active_listing_by_token.write((listing.collection, listing.token_id), 0);
            self.emit(ListingCanceled { listing_id });
        }

        fn buy(ref self: ContractState, listing_id: u256) {
            let mut listing = self.listings.read(listing_id);
            assert(listing.id != 0, 'no listing');
            assert(listing.active, 'inactive');
            assert(listing.listing_type == ListingType::FixedPrice, 'not fixed');
            let buyer = get_caller_address();
            assert(buyer != listing.seller, 'self buy');

            // Pull funds
            let token = IERC20Dispatcher { contract_address: self.payment_token.read() };
            let ok = token.transfer_from(buyer, get_contract_address(), listing.price);
            assert(ok, 'pay fail');

            // Distribute
            let (platform_fee, royalty_amount, seller_amount, royalty_recipient) = RoyaltyDistributor::compute_and_split(
                listing.collection,
                listing.price,
                self.platform_fee_bps.read(),
                self.royalty_bps_by_collection.read(listing.collection),
                self.royalty_recipient_by_collection.read(listing.collection),
            );
            if platform_fee > 0 { let _ = token.transfer(self.ownable.owner(), platform_fee); }
            if royalty_amount > 0 { let _ = token.transfer(royalty_recipient, royalty_amount); }
            if seller_amount > 0 { let _ = token.transfer(listing.seller, seller_amount); }

            // Transfer NFT to buyer
            let nft = IERC721Dispatcher { contract_address: listing.collection };
            let _ = nft.transfer_from(listing.seller, buyer, listing.token_id);

            // Close listing
            listing.active = false;
            self.listings.write(listing_id, listing);
            self.active_listing_by_token.write((listing.collection, listing.token_id), 0);

            // Analytics
            self._track_sale(listing.collection, listing.price, true);

            self.emit(Purchased { listing_id, buyer, price: listing.price });
        }

        // Offers
        fn make_offer(ref self: ContractState, collection: ContractAddress, token_id: u256, amount: u256, expiry: u64) -> u256 {
            assert(amount > 0, 'amt=0');
            assert(expiry > get_block_timestamp(), 'bad expiry');
            let bidder = get_caller_address();

            // Pull funds to escrow
            let token = IERC20Dispatcher { contract_address: self.payment_token.read() };
            let ok = token.transfer_from(bidder, get_contract_address(), amount);
            assert(ok, 'escrow fail');

            let offer_id = self.offer_counter.read() + 1;
            self.offer_counter.write(offer_id);
            let offer = Offer { id: offer_id, collection, token_id, bidder, amount, expiry, active: true, created_at: get_block_timestamp() };
            self.offers.write(offer_id, offer);

            // Analytics
            let mut a = self.analytics.read(); a.total_offers += 1; self.analytics.write(a);

            self.emit(OfferMade { offer_id, collection, token_id, bidder, amount });
            offer_id
        }

        fn cancel_offer(ref self: ContractState, offer_id: u256) {
            let mut offer = self.offers.read(offer_id);
            assert(offer.id != 0, 'no offer');
            assert(offer.active, 'inactive');
            assert(offer.bidder == get_caller_address(), 'not bidder');
            offer.active = false;
            self.offers.write(offer_id, offer);
            let token = IERC20Dispatcher { contract_address: self.payment_token.read() };
            let _ = token.transfer(offer.bidder, offer.amount);
            self.emit(OfferCanceled { offer_id });
        }

        fn accept_offer(ref self: ContractState, offer_id: u256) {
            let mut offer = self.offers.read(offer_id);
            assert(offer.id != 0, 'no offer');
            assert(offer.active, 'inactive');
            assert(offer.expiry >= get_block_timestamp(), 'expired');

            // Validate ownership
            let seller = get_caller_address();
            let nft = IERC721Dispatcher { contract_address: offer.collection };
            // rely on approval; transfer will fail if not owner/approved
            let _ = nft.transfer_from(seller, offer.bidder, offer.token_id);

            // Distribute
            let token = IERC20Dispatcher { contract_address: self.payment_token.read() };
            let (platform_fee, royalty_amount, seller_amount, royalty_recipient) = RoyaltyDistributor::compute_and_split(
                offer.collection,
                offer.amount,
                self.platform_fee_bps.read(),
                self.royalty_bps_by_collection.read(offer.collection),
                self.royalty_recipient_by_collection.read(offer.collection),
            );
            if platform_fee > 0 { let _ = token.transfer(self.ownable.owner(), platform_fee); }
            if royalty_amount > 0 { let _ = token.transfer(royalty_recipient, royalty_amount); }
            if seller_amount > 0 { let _ = token.transfer(seller, seller_amount); }

            offer.active = false; self.offers.write(offer_id, offer);

            // Analytics
            self._track_sale(offer.collection, offer.amount, true);

            self.emit(OfferAccepted { offer_id, seller });
        }

        // English auction
        fn create_english_auction(ref self: ContractState, collection: ContractAddress, token_id: u256, reserve_price: u256, min_increment_bps: u16, duration_seconds: u64) -> u256 {
            assert(duration_seconds > 0, 'bad duration');
            let seller = get_caller_address();
            // Escrow NFT
            let nft = IERC721Dispatcher { contract_address: collection };
            let _ = nft.transfer_from(seller, get_contract_address(), token_id);

            let auction_id = self.english_counter.read() + 1; self.english_counter.write(auction_id);
            let end = get_block_timestamp() + duration_seconds;
            let auction = EnglishAuction { id: auction_id, collection, token_id, seller, reserve_price, min_increment_bps, highest_bid: 0, highest_bidder: ContractAddress::from(0), end_time: end, settled: false };
            self.english_auctions.write(auction_id, auction);

            // Analytics
            let mut a = self.analytics.read(); a.total_auctions += 1; self.analytics.write(a);

            self.emit(EnglishAuctionCreated { auction_id, collection, token_id, seller, end_time: end });
            auction_id
        }

        fn bid(ref self: ContractState, auction_id: u256, amount: u256) {
            let mut auct = self.english_auctions.read(auction_id);
            assert(auct.id != 0, 'no auction');
            assert(!auct.settled, 'settled');
            assert(get_block_timestamp() < auct.end_time, 'ended');

            // Compute min next bid
            let MinBidResult { min_next_bid } = AuctionLogic::compute_min_next_bid(auct.highest_bid, auct.min_increment_bps);
            assert(amount >= min_next_bid && amount >= auct.reserve_price, 'bid too low');

            // Pull funds
            let token = IERC20Dispatcher { contract_address: self.payment_token.read() };
            let ok = token.transfer_from(get_caller_address(), get_contract_address(), amount);
            assert(ok, 'pay fail');

            // Refund previous highest
            if auct.highest_bid > 0 {
                let _ = token.transfer(auct.highest_bidder, auct.highest_bid);
            }

            auct.highest_bid = amount; auct.highest_bidder = get_caller_address();
            self.english_auctions.write(auction_id, auct);
            self.emit(BidPlaced { auction_id, bidder: auct.highest_bidder, amount });
        }

        fn finalize_english(ref self: ContractState, auction_id: u256) {
            let mut auct = self.english_auctions.read(auction_id);
            assert(auct.id != 0, 'no auction');
            assert(!auct.settled, 'settled');
            assert(get_block_timestamp() >= auct.end_time, 'not ended');

            let nft = IERC721Dispatcher { contract_address: auct.collection };
            let token = IERC20Dispatcher { contract_address: self.payment_token.read() };

            if auct.highest_bid == 0 || auct.highest_bid < auct.reserve_price {
                // Return NFT to seller
                let _ = nft.transfer_from(get_contract_address(), auct.seller, auct.token_id);
            } else {
                // Payouts
                let (platform_fee, royalty_amount, seller_amount, royalty_recipient) = RoyaltyDistributor::compute_and_split(
                    auct.collection,
                    auct.highest_bid,
                    self.platform_fee_bps.read(),
                    self.royalty_bps_by_collection.read(auct.collection),
                    self.royalty_recipient_by_collection.read(auct.collection),
                );
                if platform_fee > 0 { let _ = token.transfer(self.ownable.owner(), platform_fee); }
                if royalty_amount > 0 { let _ = token.transfer(royalty_recipient, royalty_amount); }
                if seller_amount > 0 { let _ = token.transfer(auct.seller, seller_amount); }

                // Transfer NFT to winner
                let _ = nft.transfer_from(get_contract_address(), auct.highest_bidder, auct.token_id);

                // Analytics
                self._track_sale(auct.collection, auct.highest_bid, false);
                self.emit(EnglishFinalized { auction_id, winner: auct.highest_bidder, amount: auct.highest_bid });
            }

            auct.settled = true; self.english_auctions.write(auction_id, auct);
        }

        // Dutch auction
        fn create_dutch_auction(ref self: ContractState, collection: ContractAddress, token_id: u256, start_price: u256, end_price: u256, duration_seconds: u64) -> u256 {
            assert(duration_seconds > 0, 'bad duration');
            assert(start_price > end_price, 'start<=end');
            let seller = get_caller_address();
            let nft = IERC721Dispatcher { contract_address: collection };
            let _ = nft.transfer_from(seller, get_contract_address(), token_id);

            let auction_id = self.dutch_counter.read() + 1; self.dutch_counter.write(auction_id);
            let start = get_block_timestamp();
            let end = start + duration_seconds;
            let au = DutchAuction { id: auction_id, collection, token_id, seller, start_price, end_price, start_time: start, end_time: end, settled: false };
            self.dutch_auctions.write(auction_id, au);
            self.emit(DutchAuctionCreated { auction_id, collection, token_id, seller, start_price, end_price, end_time: end });
            auction_id
        }

        fn buy_dutch(ref self: ContractState, auction_id: u256) {
            let mut au = self.dutch_auctions.read(auction_id);
            assert(au.id != 0, 'no auction');
            assert(!au.settled, 'settled');
            let now = get_block_timestamp();
            assert(now <= au.end_time, 'ended');
            let price = AuctionLogic::current_dutch_price(au.start_price, au.end_price, au.start_time, au.end_time, now);
            let buyer = get_caller_address();

            // Pull funds
            let token = IERC20Dispatcher { contract_address: self.payment_token.read() };
            let ok = token.transfer_from(buyer, get_contract_address(), price);
            assert(ok, 'pay fail');

            let (platform_fee, royalty_amount, seller_amount, royalty_recipient) = RoyaltyDistributor::compute_and_split(
                au.collection,
                price,
                self.platform_fee_bps.read(),
                self.royalty_bps_by_collection.read(au.collection),
                self.royalty_recipient_by_collection.read(au.collection),
            );
            if platform_fee > 0 { let _ = token.transfer(self.ownable.owner(), platform_fee); }
            if royalty_amount > 0 { let _ = token.transfer(royalty_recipient, royalty_amount); }
            if seller_amount > 0 { let _ = token.transfer(au.seller, seller_amount); }

            let nft = IERC721Dispatcher { contract_address: au.collection };
            let _ = nft.transfer_from(get_contract_address(), buyer, au.token_id);

            au.settled = true; self.dutch_auctions.write(auction_id, au);

            // Analytics
            self._track_sale(au.collection, price, false);

            self.emit(DutchPurchased { auction_id, buyer, price });
        }

        // Views
        fn get_listing(self: @ContractState, listing_id: u256) -> Listing { self.listings.read(listing_id) }
        fn get_offer(self: @ContractState, offer_id: u256) -> Offer { self.offers.read(offer_id) }
        fn get_english(self: @ContractState, auction_id: u256) -> EnglishAuction { self.english_auctions.read(auction_id) }
        fn get_dutch(self: @ContractState, auction_id: u256) -> DutchAuction { self.dutch_auctions.read(auction_id) }
        fn get_analytics(self: @ContractState) -> Analytics { self.analytics.read() }
        fn get_collection_stats(self: @ContractState, collection: ContractAddress) -> (u256, u256) { (self.collection_volume.read(collection), self.collection_sales_count.read(collection)) }
        fn get_recent_sales(self: @ContractState, limit: u64) -> (Array<ContractAddress>, Array<u256>, Array<u64>) {
            let cap: u64 = 64_u64;
            let mut ret_addr = ArrayTrait::new();
            let mut ret_amt = ArrayTrait::new();
            let mut ret_ts = ArrayTrait::new();
            let count = self.recent_sales_count.read();
            if count == 0 { return (ret_addr, ret_amt, ret_ts); }
            let mut to_take = if limit == 0 { count } else { if limit > count { count } else { limit } };
            let head = self.recent_sales_head.read();
            // iterate newest to oldest
            let mut i: u64 = 0_u64;
            loop {
                if i >= to_take { break; }
                let idx = (cap + head - 1 - i) % cap;
                ret_addr.append(self.recent_sales_collections.read(idx));
                ret_amt.append(self.recent_sales_amounts.read(idx));
                ret_ts.append(self.recent_sales_timestamps.read(idx));
                i = i + 1_u64;
            };
            (ret_addr, ret_amt, ret_ts)
        }
        fn get_platform_fee(self: @ContractState) -> u16 { self.platform_fee_bps.read() }
    }

    #[generate_trait]
    impl InternalImpl of InternalTrait {
        fn _track_sale(ref self: ContractState, collection: ContractAddress, price: u256, direct_sale: bool) {
            let mut a = self.analytics.read();
            a.total_volume += price; a.total_sales += 1; self.analytics.write(a);
            self.collection_volume.write(collection, self.collection_volume.read(collection) + price);
            self.collection_sales_count.write(collection, self.collection_sales_count.read(collection) + 1);
            if direct_sale { let _ = direct_sale; }

            // push into ring buffer of size 64
            let cap: u64 = 64_u64;
            let head = self.recent_sales_head.read();
            self.recent_sales_collections.write(head, collection);
            self.recent_sales_amounts.write(head, price);
            self.recent_sales_timestamps.write(head, get_block_timestamp());
            let new_head = (head + 1_u64) % cap;
            self.recent_sales_head.write(new_head);
            let cnt = self.recent_sales_count.read();
            if cnt < cap { self.recent_sales_count.write(cnt + 1_u64); }
        }
    }
}


