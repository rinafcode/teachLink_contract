use starknet::ContractAddress;
use core::array::Array;

#[derive(Drop, Serde, starknet::Store, Clone, Copy, PartialEq)]
pub enum BillingCycle {
    Daily: (),
    Weekly: (),
    Monthly: (),
    Quarterly: (),
    Yearly: (),
}

#[derive(Drop, Serde, starknet::Store, Clone, Copy, PartialEq)]
pub enum SubscriptionStatus {
    Active: (),
    Paused: (),
    Cancelled: (),
    Expired: (),
    GracePeriod: (),
}

#[derive(Drop, Serde, starknet::Store, Clone, Copy, PartialEq)]
pub enum BillingType {
    Fixed: (),
    UsageBased: (),
    Hybrid: (),
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct SubscriptionPlan {
    pub id: u256,
    pub name: felt252,
    pub description: felt252,
    pub price: u256,
    pub billing_cycle: BillingCycle,
    pub billing_type: BillingType,
    pub max_usage: u256, // For usage-based plans, 0 = unlimited
    pub grace_period_days: u8,
    pub active: bool,
    pub created_at: u64,
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct Subscription {
    pub id: u256,
    pub user: ContractAddress,
    pub plan_id: u256,
    pub status: SubscriptionStatus,
    pub start_date: u64,
    pub next_billing_date: u64,
    pub last_payment_date: u64,
    pub total_paid: u256,
    pub failed_payments: u8,
    pub grace_period_until: u64,
    pub created_at: u64,
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct UsageRecord {
    pub subscription_id: u256,
    pub timestamp: u64,
    pub amount: u256,
    pub unit: felt252,
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct BillingPeriod {
    pub subscription_id: u256,
    pub start_date: u64,
    pub end_date: u64,
    pub base_amount: u256,
    pub usage_amount: u256,
    pub total_amount: u256,
    pub paid: bool,
    pub payment_date: u64,
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct SubscriptionAnalytics {
    pub total_subscriptions: u256,
    pub active_subscriptions: u256,
    pub cancelled_subscriptions: u256,
    pub total_revenue: u256,
    pub monthly_recurring_revenue: u256,
    pub churn_rate: u256, // in basis points (100 = 1%)
    pub average_revenue_per_user: u256,
}

#[derive(Drop, Serde, starknet::Store, Clone)]
pub struct ChurnPrediction {
    pub subscription_id: u256,
    pub risk_score: u256, // 0-100, higher = more likely to churn
    pub factors: Array<felt252>,
    pub predicted_churn_date: u64,
}

#[starknet::interface]
pub trait ISubscriptionManager<TContractState> {
    // Admin functions
    fn create_plan(
        ref self: TContractState,
        name: felt252,
        description: felt252,
        price: u256,
        billing_cycle: BillingCycle,
        billing_type: BillingType,
        max_usage: u256,
        grace_period_days: u8
    ) -> u256;
    
    fn update_plan(
        ref self: TContractState,
        plan_id: u256,
        name: felt252,
        description: felt252,
        price: u256,
        max_usage: u256,
        grace_period_days: u8
    );
    
    fn deactivate_plan(ref self: TContractState, plan_id: u256);
    fn set_payment_token(ref self: TContractState, token: ContractAddress);
    fn set_platform_fee(ref self: TContractState, fee_bps: u16);

    // Subscription management
    fn subscribe(ref self: TContractState, plan_id: u256) -> u256;
    fn cancel_subscription(ref self: TContractState, subscription_id: u256);
    fn pause_subscription(ref self: TContractState, subscription_id: u256);
    fn resume_subscription(ref self: TContractState, subscription_id: u256);
    fn update_subscription_plan(ref self: TContractState, subscription_id: u256, new_plan_id: u256);

    // Billing and payments
    fn process_billing(ref self: TContractState, subscription_id: u256) -> bool;
    fn process_all_billing(ref self: TContractState) -> u256; // Returns number of processed subscriptions
    fn retry_failed_payment(ref self: TContractState, subscription_id: u256) -> bool;
    fn extend_grace_period(ref self: TContractState, subscription_id: u256, additional_days: u8);

    // Usage tracking
    fn record_usage(ref self: TContractState, subscription_id: u256, amount: u256, unit: felt252);
    fn get_usage_for_period(
        self: @TContractState,
        subscription_id: u256,
        start_date: u64,
        end_date: u64
    ) -> u256;

    // Analytics and insights
    fn get_analytics(self: @TContractState) -> SubscriptionAnalytics;
    fn get_churn_predictions(self: @TContractState, limit: u256) -> Array<ChurnPrediction>;
    fn get_subscription_history(self: @TContractState, user: ContractAddress) -> Array<u256>;
    fn get_revenue_forecast(self: @TContractState, months: u8) -> u256;

    // Views
    fn get_plan(self: @TContractState, plan_id: u256) -> SubscriptionPlan;
    fn get_subscription(self: @TContractState, subscription_id: u256) -> Subscription;
    fn get_user_subscriptions(self: @TContractState, user: ContractAddress) -> Array<u256>;
    fn get_active_subscriptions(self: @TContractState) -> Array<u256>;
    fn get_billing_periods(self: @TContractState, subscription_id: u256) -> Array<u256>;
    fn get_usage_records(self: @TContractState, subscription_id: u256, limit: u256) -> Array<UsageRecord>;
    fn is_subscription_active(self: @TContractState, subscription_id: u256) -> bool;
    fn get_next_billing_amount(self: @TContractState, subscription_id: u256) -> u256;
}
