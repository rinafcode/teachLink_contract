use starknet::{ContractAddress, contract_address_const};
use snforge_std::{declare, ContractClassTrait, start_cheat_caller_address, stop_cheat_caller_address, start_cheat_block_timestamp, stop_cheat_block_timestamp, start_cheat_caller_address, stop_cheat_caller_address};
use assert_macros::assert;

use src::contracts::subscriptions::interfaces::ISubscriptionManager::{
    ISubscriptionManagerDispatcher, ISubscriptionManagerDispatcherTrait, BillingCycle, BillingType, SubscriptionStatus
};
use src::contracts::subscriptions::UsageTracker::{IUsageTrackerDispatcher, IUsageTrackerDispatcherTrait};
use src::contracts::mocks::MockERC20::{MockERC20Dispatcher, MockERC20DispatcherTrait};
use src::contracts::mocks::MockERC721::{MockERC721Dispatcher, MockERC721DispatcherTrait};

#[starknet::interface]
pub trait IERC20Dispatcher<TContractState> {
    fn transfer(ref self: TContractState, recipient: ContractAddress, amount: u256) -> bool;
    fn transfer_from(ref self: TContractState, sender: ContractAddress, recipient: ContractAddress, amount: u256) -> bool;
    fn balance_of(self: @TContractState, account: ContractAddress) -> u256;
    fn approve(ref self: TContractState, spender: ContractAddress, amount: u256) -> bool;
}

#[starknet::interface]
pub trait IERC721Dispatcher<TContractState> {
    fn transfer_from(ref self: TContractState, from: ContractAddress, to: ContractAddress, token_id: u256) -> bool;
    fn owner_of(self: @TContractState, token_id: u256) -> ContractAddress;
}

fn deploy_mock_erc20() -> MockERC20Dispatcher {
    let contract = declare("MockERC20").unwrap().contract_class();
    let constructor_calldata = array![
        contract_address_const!('owner'), // owner
        'TestToken', // name
        'TT', // symbol
        18 // decimals
    ];
    let contract_address = contract.deploy(@constructor_calldata).unwrap();
    MockERC20Dispatcher { contract_address }
}

fn deploy_usage_tracker() -> IUsageTrackerDispatcher {
    let contract = declare("UsageTracker").unwrap().contract_class();
    let constructor_calldata = array![
        contract_address_const!('owner') // owner
    ];
    let contract_address = contract.deploy(@constructor_calldata).unwrap();
    IUsageTrackerDispatcher { contract_address }
}

fn deploy_subscription_manager() -> ISubscriptionManagerDispatcher {
    let contract = declare("SubscriptionManager").unwrap().contract_class();
    let erc20 = deploy_mock_erc20();
    let usage_tracker = deploy_usage_tracker();
    let constructor_calldata = array![
        contract_address_const!('owner'), // owner
        erc20.contract_address, // payment_token
        250, // platform_fee_bps (2.5%)
        usage_tracker.contract_address // usage_tracker
    ];
    let contract_address = contract.deploy(@constructor_calldata).unwrap();
    ISubscriptionManagerDispatcher { contract_address }
}

#[test]
fn test_create_plan() {
    let subscription_manager = deploy_subscription_manager();
    let plan_id = subscription_manager.create_plan(
        'Basic Plan',
        'A basic subscription plan',
        1000000000000000000, // 1 token
        BillingCycle::Monthly,
        BillingType::Fixed,
        0, // unlimited usage
        7 // 7 days grace period
    );
    
    assert(plan_id == 1, 'plan_id should be 1');
    
    let plan = subscription_manager.get_plan(plan_id);
    assert(plan.id == 1, 'plan id should be 1');
    assert(plan.name == 'Basic Plan', 'plan name should match');
    assert(plan.price == 1000000000000000000, 'plan price should match');
    assert(plan.billing_cycle == BillingCycle::Monthly, 'billing cycle should be monthly');
    assert(plan.billing_type == BillingType::Fixed, 'billing type should be fixed');
    assert(plan.active == true, 'plan should be active');
}

#[test]
fn test_subscribe() {
    let subscription_manager = deploy_subscription_manager();
    let erc20 = deploy_mock_erc20();
    
    // Create a plan
    let plan_id = subscription_manager.create_plan(
        'Basic Plan',
        'A basic subscription plan',
        1000000000000000000,
        BillingCycle::Monthly,
        BillingType::Fixed,
        0,
        7
    );
    
    // Subscribe to the plan
    let subscription_id = subscription_manager.subscribe(plan_id);
    
    assert(subscription_id == 1, 'subscription_id should be 1');
    
    let subscription = subscription_manager.get_subscription(subscription_id);
    assert(subscription.id == 1, 'subscription id should be 1');
    assert(subscription.plan_id == plan_id, 'plan_id should match');
    assert(subscription.status == SubscriptionStatus::Active, 'status should be active');
    assert(subscription.total_paid == 0, 'total_paid should be 0 initially');
}

#[test]
fn test_cancel_subscription() {
    let subscription_manager = deploy_subscription_manager();
    
    // Create a plan and subscribe
    let plan_id = subscription_manager.create_plan(
        'Basic Plan',
        'A basic subscription plan',
        1000000000000000000,
        BillingCycle::Monthly,
        BillingType::Fixed,
        0,
        7
    );
    
    let subscription_id = subscription_manager.subscribe(plan_id);
    
    // Cancel the subscription
    subscription_manager.cancel_subscription(subscription_id);
    
    let subscription = subscription_manager.get_subscription(subscription_id);
    assert(subscription.status == SubscriptionStatus::Cancelled, 'status should be cancelled');
}

#[test]
fn test_pause_and_resume_subscription() {
    let subscription_manager = deploy_subscription_manager();
    
    // Create a plan and subscribe
    let plan_id = subscription_manager.create_plan(
        'Basic Plan',
        'A basic subscription plan',
        1000000000000000000,
        BillingCycle::Monthly,
        BillingType::Fixed,
        0,
        7
    );
    
    let subscription_id = subscription_manager.subscribe(plan_id);
    
    // Pause the subscription
    subscription_manager.pause_subscription(subscription_id);
    
    let subscription = subscription_manager.get_subscription(subscription_id);
    assert(subscription.status == SubscriptionStatus::Paused, 'status should be paused');
    
    // Resume the subscription
    subscription_manager.resume_subscription(subscription_id);
    
    let subscription = subscription_manager.get_subscription(subscription_id);
    assert(subscription.status == SubscriptionStatus::Active, 'status should be active');
}

#[test]
fn test_usage_based_billing() {
    let subscription_manager = deploy_subscription_manager();
    let erc20 = deploy_mock_erc20();
    
    // Create a usage-based plan
    let plan_id = subscription_manager.create_plan(
        'Usage Plan',
        'A usage-based subscription plan',
        100000000000000000, // 0.1 token base
        BillingCycle::Monthly,
        BillingType::UsageBased,
        1000, // 1000 units included
        7
    );
    
    let subscription_id = subscription_manager.subscribe(plan_id);
    
    // Record some usage
    subscription_manager.record_usage(subscription_id, 500, 'API_CALLS');
    subscription_manager.record_usage(subscription_id, 300, 'API_CALLS');
    
    // Check usage tracking
    let usage = subscription_manager.get_usage_for_period(
        subscription_id,
        0, // start from beginning
        9999999999 // far future
    );
    
    assert(usage == 800, 'total usage should be 800');
}

#[test]
fn test_billing_cycle_calculations() {
    let subscription_manager = deploy_subscription_manager();
    
    // Test different billing cycles
    let daily_plan = subscription_manager.create_plan(
        'Daily Plan',
        'Daily billing',
        100000000000000000,
        BillingCycle::Daily,
        BillingType::Fixed,
        0,
        1
    );
    
    let weekly_plan = subscription_manager.create_plan(
        'Weekly Plan',
        'Weekly billing',
        700000000000000000,
        BillingCycle::Weekly,
        BillingType::Fixed,
        0,
        3
    );
    
    let monthly_plan = subscription_manager.create_plan(
        'Monthly Plan',
        'Monthly billing',
        3000000000000000000,
        BillingCycle::Monthly,
        BillingType::Fixed,
        0,
        7
    );
    
    // Subscribe to all plans
    let daily_sub = subscription_manager.subscribe(daily_plan);
    let weekly_sub = subscription_manager.subscribe(weekly_plan);
    let monthly_sub = subscription_manager.subscribe(monthly_plan);
    
    // Check that subscriptions were created
    assert(daily_sub == 1, 'daily subscription should be 1');
    assert(weekly_sub == 2, 'weekly subscription should be 2');
    assert(monthly_sub == 3, 'monthly subscription should be 3');
}

#[test]
fn test_analytics() {
    let subscription_manager = deploy_subscription_manager();
    
    // Create multiple plans and subscriptions
    let plan1 = subscription_manager.create_plan(
        'Plan 1',
        'First plan',
        1000000000000000000,
        BillingCycle::Monthly,
        BillingType::Fixed,
        0,
        7
    );
    
    let plan2 = subscription_manager.create_plan(
        'Plan 2',
        'Second plan',
        2000000000000000000,
        BillingCycle::Monthly,
        BillingType::Fixed,
        0,
        7
    );
    
    // Create multiple subscriptions
    let sub1 = subscription_manager.subscribe(plan1);
    let sub2 = subscription_manager.subscribe(plan2);
    let sub3 = subscription_manager.subscribe(plan1);
    
    // Cancel one subscription
    subscription_manager.cancel_subscription(sub2);
    
    let analytics = subscription_manager.get_analytics();
    assert(analytics.total_subscriptions == 3, 'total subscriptions should be 3');
    assert(analytics.active_subscriptions == 2, 'active subscriptions should be 2');
    assert(analytics.cancelled_subscriptions == 1, 'cancelled subscriptions should be 1');
}

#[test]
fn test_revenue_forecast() {
    let subscription_manager = deploy_subscription_manager();
    
    // Create a monthly plan
    let plan_id = subscription_manager.create_plan(
        'Monthly Plan',
        'Monthly billing plan',
        1000000000000000000, // 1 token per month
        BillingCycle::Monthly,
        BillingType::Fixed,
        0,
        7
    );
    
    // Create 5 subscriptions
    let mut i = 0;
    loop {
        if i >= 5 { break; }
        subscription_manager.subscribe(plan_id);
        i += 1;
    };
    
    // Forecast for 12 months
    let forecast = subscription_manager.get_revenue_forecast(12);
    // 5 subscriptions * 1 token * 12 months = 60 tokens
    assert(forecast == 60000000000000000000, 'forecast should be 60 tokens');
}

#[test]
fn test_grace_period() {
    let subscription_manager = deploy_subscription_manager();
    
    // Create a plan with 7-day grace period
    let plan_id = subscription_manager.create_plan(
        'Grace Plan',
        'Plan with grace period',
        1000000000000000000,
        BillingCycle::Monthly,
        BillingType::Fixed,
        0,
        7
    );
    
    let subscription_id = subscription_manager.subscribe(plan_id);
    
    // Extend grace period (admin function)
    subscription_manager.extend_grace_period(subscription_id, 3);
    
    let subscription = subscription_manager.get_subscription(subscription_id);
    // Grace period should be extended by 3 days (3 * 86400 seconds)
    assert(subscription.grace_period_until == 259200, 'grace period should be extended');
}

#[test]
fn test_plan_updates() {
    let subscription_manager = deploy_subscription_manager();
    
    // Create a plan
    let plan_id = subscription_manager.create_plan(
        'Original Plan',
        'Original description',
        1000000000000000000,
        BillingCycle::Monthly,
        BillingType::Fixed,
        0,
        7
    );
    
    // Update the plan
    subscription_manager.update_plan(
        plan_id,
        'Updated Plan',
        'Updated description',
        2000000000000000000, // Double the price
        1000, // Add usage limit
        14 // Extend grace period
    );
    
    let plan = subscription_manager.get_plan(plan_id);
    assert(plan.name == 'Updated Plan', 'name should be updated');
    assert(plan.description == 'Updated description', 'description should be updated');
    assert(plan.price == 2000000000000000000, 'price should be updated');
    assert(plan.max_usage == 1000, 'max_usage should be updated');
    assert(plan.grace_period_days == 14, 'grace_period_days should be updated');
}

#[test]
fn test_deactivate_plan() {
    let subscription_manager = deploy_subscription_manager();
    
    // Create a plan
    let plan_id = subscription_manager.create_plan(
        'Test Plan',
        'Test description',
        1000000000000000000,
        BillingCycle::Monthly,
        BillingType::Fixed,
        0,
        7
    );
    
    // Deactivate the plan
    subscription_manager.deactivate_plan(plan_id);
    
    let plan = subscription_manager.get_plan(plan_id);
    assert(plan.active == false, 'plan should be deactivated');
}

#[test]
fn test_user_subscription_history() {
    let subscription_manager = deploy_subscription_manager();
    
    // Create multiple plans
    let plan1 = subscription_manager.create_plan(
        'Plan 1',
        'First plan',
        1000000000000000000,
        BillingCycle::Monthly,
        BillingType::Fixed,
        0,
        7
    );
    
    let plan2 = subscription_manager.create_plan(
        'Plan 2',
        'Second plan',
        2000000000000000000,
        BillingCycle::Monthly,
        BillingType::Fixed,
        0,
        7
    );
    
    // Subscribe to both plans
    let sub1 = subscription_manager.subscribe(plan1);
    let sub2 = subscription_manager.subscribe(plan2);
    
    // Get user subscription history
    let user_subs = subscription_manager.get_subscription_history(contract_address_const!('user'));
    assert(user_subs.len() == 2, 'user should have 2 subscriptions');
}

#[test]
fn test_hybrid_billing() {
    let subscription_manager = deploy_subscription_manager();
    
    // Create a hybrid plan (fixed + usage-based)
    let plan_id = subscription_manager.create_plan(
        'Hybrid Plan',
        'Fixed base + usage-based overage',
        1000000000000000000, // 1 token base
        BillingCycle::Monthly,
        BillingType::Hybrid,
        1000, // 1000 units included
        7
    );
    
    let subscription_id = subscription_manager.subscribe(plan_id);
    
    // Record usage within limit
    subscription_manager.record_usage(subscription_id, 500, 'API_CALLS');
    
    // Record usage over limit
    subscription_manager.record_usage(subscription_id, 600, 'API_CALLS');
    
    let total_usage = subscription_manager.get_usage_for_period(
        subscription_id,
        0,
        9999999999
    );
    
    assert(total_usage == 1100, 'total usage should be 1100');
}

#[test]
fn test_churn_prediction() {
    let subscription_manager = deploy_subscription_manager();
    
    // Create a plan
    let plan_id = subscription_manager.create_plan(
        'Test Plan',
        'Test description',
        1000000000000000000,
        BillingCycle::Monthly,
        BillingType::Fixed,
        0,
        7
    );
    
    let subscription_id = subscription_manager.subscribe(plan_id);
    
    // Get churn predictions (should be empty initially as no predictions are generated)
    let predictions = subscription_manager.get_churn_predictions(10);
    assert(predictions.len() == 0, 'no predictions should exist initially');
}

#[test]
fn test_usage_tracker_integration() {
    let subscription_manager = deploy_subscription_manager();
    let usage_tracker = deploy_usage_tracker();
    
    // Create a plan
    let plan_id = subscription_manager.create_plan(
        'Usage Plan',
        'Usage-based plan',
        1000000000000000000,
        BillingCycle::Monthly,
        BillingType::UsageBased,
        0,
        7
    );
    
    let subscription_id = subscription_manager.subscribe(plan_id);
    
    // Record usage through the subscription manager
    subscription_manager.record_usage(subscription_id, 100, 'API_CALLS');
    subscription_manager.record_usage(subscription_id, 200, 'API_CALLS');
    
    // Verify usage was recorded
    let usage = subscription_manager.get_usage_for_period(
        subscription_id,
        0,
        9999999999
    );
    
    assert(usage == 300, 'total usage should be 300');
}

#[test]
fn test_subscription_plan_change() {
    let subscription_manager = deploy_subscription_manager();
    
    // Create two plans
    let plan1 = subscription_manager.create_plan(
        'Plan 1',
        'First plan',
        1000000000000000000,
        BillingCycle::Monthly,
        BillingType::Fixed,
        0,
        7
    );
    
    let plan2 = subscription_manager.create_plan(
        'Plan 2',
        'Second plan',
        2000000000000000000,
        BillingCycle::Monthly,
        BillingType::Fixed,
        0,
        7
    );
    
    // Subscribe to plan 1
    let subscription_id = subscription_manager.subscribe(plan1);
    
    // Change to plan 2
    subscription_manager.update_subscription_plan(subscription_id, plan2);
    
    let subscription = subscription_manager.get_subscription(subscription_id);
    assert(subscription.plan_id == plan2, 'plan_id should be updated to plan2');
}
