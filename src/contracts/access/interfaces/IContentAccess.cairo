use starknet::ContractAddress;
use core::array::Array;

#[derive(Drop, Serde, starknet::Store, Clone, Copy, PartialEq)]
pub enum AccessMethod { Token: (), Subscription: () }

#[derive(Drop, Serde, starknet::Store, Clone, Copy, PartialEq)]
pub enum KeyPolicy { None: (), PerCourse: (), PerUser: () }

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct CourseAccessConfig {
    pub course_id: u256,
    pub seller: ContractAddress,
    pub access_method: AccessMethod,
    pub subscription_plan_id: u256,
    pub start_time: u64,
    pub end_time: u64,
    pub key_policy: KeyPolicy,
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct AccessGrant {
    pub user: ContractAddress,
    pub course_id: u256,
    pub expires_at: u64,
    pub created_at: u64,
}

#[starknet::interface]
pub trait IContentAccess<TContractState> {
    // Admin/config
    fn set_subscription_manager(ref self: TContractState, manager: ContractAddress);
    fn set_course_nft(ref self: TContractState, course_nft: ContractAddress);
    fn set_marketplace(ref self: TContractState, marketplace: ContractAddress);
    fn configure_course(ref self: TContractState, config: CourseAccessConfig);
    fn set_course_key(ref self: TContractState, course_id: u256, key_hash: felt252);

    // Purchases/token-gated grants
    fn grant_access(ref self: TContractState, user: ContractAddress, course_id: u256, duration_secs: u64);

    // Views/access checks
    fn has_access(self: @TContractState, user: ContractAddress, course_id: u256) -> bool;
    fn get_access_expiry(self: @TContractState, user: ContractAddress, course_id: u256) -> u64;
    fn get_course_config(self: @TContractState, course_id: u256) -> CourseAccessConfig;

    // Frontend key retrieval (returns encrypted key material as felt252 id)
    fn request_content_key(self: @TContractState, user: ContractAddress, course_id: u256) -> felt252;

    // Events could be emitted by contract; interface left minimal
}


