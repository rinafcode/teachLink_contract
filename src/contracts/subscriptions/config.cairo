use starknet::ContractAddress;

// Configuration constants for the subscription management system
pub mod SubscriptionConfig {
    // Maximum values for validation
    pub const MAX_PLATFORM_FEE_BPS: u16 = 1000; // 10%
    pub const MAX_GRACE_PERIOD_DAYS: u8 = 30;
    pub const MAX_FAILED_PAYMENTS: u8 = 3;
    pub const MAX_USAGE_RECORDS_PER_QUERY: u256 = 1000;
    
    // Default values
    pub const DEFAULT_GRACE_PERIOD_DAYS: u8 = 7;
    pub const DEFAULT_PLATFORM_FEE_BPS: u16 = 250; // 2.5%
    
    // Billing cycle durations in seconds
    pub const DAILY_CYCLE_SECONDS: u64 = 86400; // 24 hours
    pub const WEEKLY_CYCLE_SECONDS: u64 = 604800; // 7 days
    pub const MONTHLY_CYCLE_SECONDS: u64 = 2592000; // ~30 days
    pub const QUARTERLY_CYCLE_SECONDS: u64 = 7776000; // ~90 days
    pub const YEARLY_CYCLE_SECONDS: u64 = 31536000; // ~365 days
    
    // Performance optimization constants
    pub const MAX_BATCH_SIZE: u32 = 50;
    pub const USAGE_COMPRESSION_FACTOR: u256 = 100;
    pub const CHURN_PREDICTION_CACHE_TTL: u64 = 3600; // 1 hour
    
    // Revenue tracking periods
    pub const REVENUE_TRACKING_GRANULARITY: u64 = 3600; // 1 hour
    pub const MAX_REVENUE_HISTORY_DAYS: u64 = 365; // 1 year
    
    // Churn prediction thresholds
    pub const HIGH_CHURN_RISK_THRESHOLD: u256 = 70; // 70% risk score
    pub const MEDIUM_CHURN_RISK_THRESHOLD: u256 = 40; // 40% risk score
    
    // Usage tracking limits
    pub const MAX_USAGE_UNIT_LENGTH: u32 = 32; // Max characters in usage unit
    pub const MAX_PLAN_NAME_LENGTH: u32 = 64; // Max characters in plan name
    pub const MAX_PLAN_DESCRIPTION_LENGTH: u32 = 256; // Max characters in description
    
    // Gas optimization constants
    pub const OPTIMAL_BATCH_SIZE: u32 = 20; // Optimal batch size for gas efficiency
    pub const MAX_ARRAY_SIZE: u32 = 1000; // Maximum array size for operations
    
    // Subscription status codes
    pub const STATUS_ACTIVE: u8 = 0;
    pub const STATUS_PAUSED: u8 = 1;
    pub const STATUS_CANCELLED: u8 = 2;
    pub const STATUS_EXPIRED: u8 = 3;
    pub const STATUS_GRACE_PERIOD: u8 = 4;
    
    // Billing type codes
    pub const BILLING_TYPE_FIXED: u8 = 0;
    pub const BILLING_TYPE_USAGE_BASED: u8 = 1;
    pub const BILLING_TYPE_HYBRID: u8 = 2;
    
    // Billing cycle codes
    pub const CYCLE_DAILY: u8 = 0;
    pub const CYCLE_WEEKLY: u8 = 1;
    pub const CYCLE_MONTHLY: u8 = 2;
    pub const CYCLE_QUARTERLY: u8 = 3;
    pub const CYCLE_YEARLY: u8 = 4;
}

// Validation functions
pub mod Validation {
    use super::SubscriptionConfig;
    
    pub fn validate_platform_fee(fee_bps: u16) -> bool {
        fee_bps <= SubscriptionConfig::MAX_PLATFORM_FEE_BPS
    }
    
    pub fn validate_grace_period(days: u8) -> bool {
        days <= SubscriptionConfig::MAX_GRACE_PERIOD_DAYS
    }
    
    pub fn validate_plan_name(name: felt252) -> bool {
        // In a real implementation, you'd check string length
        true
    }
    
    pub fn validate_price(price: u256) -> bool {
        price > 0
    }
    
    pub fn validate_usage_limit(limit: u256) -> bool {
        // 0 means unlimited, otherwise must be positive
        limit == 0 || limit > 0
    }
}

// Error codes for better debugging
pub mod ErrorCodes {
    pub const INVALID_PLAN_ID: felt252 = 'INVALID_PLAN_ID';
    pub const INVALID_SUBSCRIPTION_ID: felt252 = 'INVALID_SUBSCRIPTION_ID';
    pub const PLAN_NOT_ACTIVE: felt252 = 'PLAN_NOT_ACTIVE';
    pub const SUBSCRIPTION_NOT_ACTIVE: felt252 = 'SUBSCRIPTION_NOT_ACTIVE';
    pub const INSUFFICIENT_FUNDS: felt252 = 'INSUFFICIENT_FUNDS';
    pub const PAYMENT_FAILED: felt252 = 'PAYMENT_FAILED';
    pub const UNAUTHORIZED: felt252 = 'UNAUTHORIZED';
    pub const INVALID_BILLING_CYCLE: felt252 = 'INVALID_BILLING_CYCLE';
    pub const INVALID_BILLING_TYPE: felt252 = 'INVALID_BILLING_TYPE';
    pub const GRACE_PERIOD_EXCEEDED: felt252 = 'GRACE_PERIOD_EXCEEDED';
    pub const MAX_FAILED_PAYMENTS_REACHED: felt252 = 'MAX_FAILED_PAYMENTS_REACHED';
    pub const INVALID_USAGE_AMOUNT: felt252 = 'INVALID_USAGE_AMOUNT';
    pub const INVALID_TIMESTAMP: felt252 = 'INVALID_TIMESTAMP';
    pub const SUBSCRIPTION_ALREADY_EXISTS: felt252 = 'SUBSCRIPTION_ALREADY_EXISTS';
    pub const PLAN_ALREADY_EXISTS: felt252 = 'PLAN_ALREADY_EXISTS';
    pub const INVALID_PLATFORM_FEE: felt252 = 'INVALID_PLATFORM_FEE';
    pub const INVALID_GRACE_PERIOD: felt252 = 'INVALID_GRACE_PERIOD';
}
