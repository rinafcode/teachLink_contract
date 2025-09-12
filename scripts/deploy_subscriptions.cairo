use starknet::{ContractAddress, contract_address_const};
use snforge_std::{declare, ContractClassTrait};

// Deployment script for the subscription management system
// This script demonstrates how to deploy and configure the subscription system

fn deploy_subscription_system() -> (ContractAddress, ContractAddress, ContractAddress) {
    // 1. Deploy MockERC20 for testing
    let erc20_contract = declare("MockERC20").unwrap().contract_class();
    let erc20_constructor_calldata = array![
        contract_address_const!('admin'), // owner
        'TeachLinkToken', // name
        'TLT', // symbol
        18 // decimals
    ];
    let erc20_address = erc20_contract.deploy(@erc20_constructor_calldata).unwrap();

    // 2. Deploy UsageTracker
    let usage_tracker_contract = declare("UsageTracker").unwrap().contract_class();
    let usage_tracker_constructor_calldata = array![
        contract_address_const!('admin') // owner
    ];
    let usage_tracker_address = usage_tracker_contract.deploy(@usage_tracker_constructor_calldata).unwrap();

    // 3. Deploy SubscriptionManager
    let subscription_manager_contract = declare("SubscriptionManager").unwrap().contract_class();
    let subscription_manager_constructor_calldata = array![
        contract_address_const!('admin'), // owner
        erc20_address, // payment_token
        250, // platform_fee_bps (2.5%)
        usage_tracker_address // usage_tracker
    ];
    let subscription_manager_address = subscription_manager_contract.deploy(@subscription_manager_constructor_calldata).unwrap();

    (erc20_address, usage_tracker_address, subscription_manager_address)
}

fn setup_initial_plans(subscription_manager: ContractAddress) {
    // This would be called after deployment to set up initial subscription plans
    // In a real deployment, this would be done through admin functions
    
    // Example plans that could be created:
    // - Free Plan: $0/month, limited features
    // - Basic Plan: $10/month, standard features  
    // - Premium Plan: $25/month, all features
    // - Enterprise Plan: $100/month, custom features + support
}

fn configure_usage_tracker(usage_tracker: ContractAddress, subscription_manager: ContractAddress) {
    // Authorize the subscription manager to record usage
    // This would be done through the UsageTracker's admin functions
}

// Example deployment configuration
fn main() {
    let (erc20, usage_tracker, subscription_manager) = deploy_subscription_system();
    
    // Configure the system
    configure_usage_tracker(usage_tracker, subscription_manager);
    setup_initial_plans(subscription_manager);
    
    // The system is now ready for use
}
