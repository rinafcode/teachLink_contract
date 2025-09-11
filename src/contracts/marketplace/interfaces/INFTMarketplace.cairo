use starknet::ContractAddress;
use core::array::Array;

#[derive(Drop, Serde, starknet::Store, Clone, Copy, PartialEq)]
pub enum ListingType { FixedPrice: (), }

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct Listing {
    pub id: u256,
    pub collection: ContractAddress,
    pub token_id: u256,
    pub seller: ContractAddress,
    pub price: u256,
    pub active: bool,
    pub listing_type: ListingType,
    pub created_at: u64,
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct Offer {
    pub id: u256,
    pub collection: ContractAddress,
    pub token_id: u256,
    pub bidder: ContractAddress,
    pub amount: u256,
    pub expiry: u64,
    pub active: bool,
    pub created_at: u64,
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct EnglishAuction {
    pub id: u256,
    pub collection: ContractAddress,
    pub token_id: u256,
    pub seller: ContractAddress,
    pub reserve_price: u256,
    pub min_increment_bps: u16,
    pub highest_bid: u256,
    pub highest_bidder: ContractAddress,
    pub end_time: u64,
    pub settled: bool,
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct DutchAuction {
    pub id: u256,
    pub collection: ContractAddress,
    pub token_id: u256,
    pub seller: ContractAddress,
    pub start_price: u256,
    pub end_price: u256,
    pub start_time: u64,
    pub end_time: u64,
    pub settled: bool,
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct Analytics {
    pub total_volume: u256,
    pub total_sales: u256,
    pub total_auctions: u256,
    pub total_offers: u256,
}

#[starknet::interface]
pub trait INFTMarketplace<TContractState> {
    // Admin
    fn set_platform_fee(ref self: TContractState, bps: u16);
    fn set_collection_royalty(ref self: TContractState, collection: ContractAddress, recipient: ContractAddress, bps: u16);

    // Fixed price
    fn list_fixed_price(ref self: TContractState, collection: ContractAddress, token_id: u256, price: u256) -> u256;
    fn cancel_listing(ref self: TContractState, listing_id: u256);
    fn buy(ref self: TContractState, listing_id: u256);

    // Offers
    fn make_offer(ref self: TContractState, collection: ContractAddress, token_id: u256, amount: u256, expiry: u64) -> u256;
    fn cancel_offer(ref self: TContractState, offer_id: u256);
    fn accept_offer(ref self: TContractState, offer_id: u256);

    // English auction
    fn create_english_auction(ref self: TContractState, collection: ContractAddress, token_id: u256, reserve_price: u256, min_increment_bps: u16, duration_seconds: u64) -> u256;
    fn bid(ref self: TContractState, auction_id: u256, amount: u256);
    fn finalize_english(ref self: TContractState, auction_id: u256);

    // Dutch auction
    fn create_dutch_auction(ref self: TContractState, collection: ContractAddress, token_id: u256, start_price: u256, end_price: u256, duration_seconds: u64) -> u256;
    fn buy_dutch(ref self: TContractState, auction_id: u256);

    // Views
    fn get_listing(self: @TContractState, listing_id: u256) -> Listing;
    fn get_offer(self: @TContractState, offer_id: u256) -> Offer;
    fn get_english(self: @TContractState, auction_id: u256) -> EnglishAuction;
    fn get_dutch(self: @TContractState, auction_id: u256) -> DutchAuction;
    fn get_analytics(self: @TContractState) -> Analytics;
    fn get_collection_stats(self: @TContractState, collection: ContractAddress) -> (u256, u256);
    fn get_platform_fee(self: @TContractState) -> u16;
}


