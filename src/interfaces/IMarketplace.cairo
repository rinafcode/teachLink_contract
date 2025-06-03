use starknet::ContractAddress;
use core::array::Array;

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct Course {
    pub id: u256,
    pub creator: ContractAddress,
    pub title: ByteArray,
    pub description: ByteArray,
    pub price: u256,
    pub royalty_percentage: u16, // Basis points (e.g., 500 = 5%)
    pub is_active: bool,
    pub created_at: u64,
    pub total_sales: u256,
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct Purchase {
    pub id: u256,
    pub course_id: u256,
    pub buyer: ContractAddress,
    pub amount_paid: u256,
    pub purchase_time: u64,
    pub is_completed: bool,
    pub in_dispute: bool,
    pub escrow_release_time: u64,
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct Dispute {
    pub id: u256,
    pub purchase_id: u256,
    pub reason: ByteArray,
    pub created_at: u64,
    pub resolved: bool,
    pub resolution: ByteArray,
}

#[starknet::interface]
pub trait IMarketplace<TContractState> {
    // Course Management
    fn create_course(
        ref self: TContractState,
        title: ByteArray,
        description: ByteArray,
        price: u256,
        royalty_percentage: u16,
    ) -> u256;

    fn update_course(
        ref self: TContractState,
        course_id: u256,
        title: ByteArray,
        description: ByteArray,
        price: u256,
        royalty_percentage: u16,
    );

    fn deactivate_course(ref self: TContractState, course_id: u256);
    fn activate_course(ref self: TContractState, course_id: u256);

    // Purchase Flow
    fn purchase_course(ref self: TContractState, course_id: u256);
    fn complete_course(ref self: TContractState, purchase_id: u256);

    // Escrow Management
    fn release_escrow(ref self: TContractState, purchase_id: u256);
    fn claim_escrow_after_timeout(ref self: TContractState, purchase_id: u256);

    // Dispute Resolution
    fn create_dispute(ref self: TContractState, purchase_id: u256, reason: ByteArray) -> u256;
    fn resolve_dispute(
        ref self: TContractState, dispute_id: u256, refund_buyer: bool, resolution: ByteArray,
    );

    // Platform Management
    fn set_platform_fee(ref self: TContractState, fee_percentage: u16);
    fn set_escrow_period(ref self: TContractState, period_seconds: u64);
    fn withdraw_platform_fees(ref self: TContractState);

    // View Functions
    fn get_course(self: @TContractState, course_id: u256) -> Course;
    fn get_purchase(self: @TContractState, purchase_id: u256) -> Purchase;
    fn get_dispute(self: @TContractState, dispute_id: u256) -> Dispute;
    fn get_platform_fee(self: @TContractState) -> u16;
    fn get_escrow_period(self: @TContractState) -> u64;
    fn get_courses_by_creator(self: @TContractState, creator: ContractAddress) -> Array<u256>;
    fn get_purchases_by_buyer(self: @TContractState, buyer: ContractAddress) -> Array<u256>;
    fn has_purchased_course(self: @TContractState, buyer: ContractAddress, course_id: u256) -> bool;
    fn get_platform_earnings(self: @TContractState) -> u256;
    fn get_course_count(self: @TContractState) -> u256;
    fn get_purchase_count(self: @TContractState) -> u256;
}
