# Advanced Subscription Management System

A comprehensive, gas-optimized subscription management system built for Starknet/Cairo 2.0, designed for TeachLink and other educational platforms requiring flexible billing and usage tracking.

## Features

### Core Functionality
- **Flexible Subscription Plans**: Support for multiple billing cycles (daily, weekly, monthly, quarterly, yearly)
- **Multiple Billing Types**: Fixed pricing, usage-based billing, and hybrid models
- **Automatic Renewals**: Reliable payment processing with retry mechanisms
- **Grace Periods**: Fair handling of payment failures with configurable grace periods
- **Usage Tracking**: Comprehensive usage monitoring and analytics
- **Churn Prediction**: AI-powered churn risk assessment
- **Revenue Analytics**: Real-time MRR, churn rate, and revenue forecasting

### Advanced Features
- **Proration Support**: Mid-cycle plan changes with automatic proration
- **Batch Processing**: Gas-optimized bulk operations
- **Performance Optimizations**: Bit manipulation and caching for efficiency
- **Comprehensive Analytics**: Detailed subscription and revenue insights
- **Flexible Administration**: Full admin controls for plan and system management

## Architecture

### Core Contracts

#### 1. SubscriptionManager
The main contract handling subscription lifecycle, billing, and analytics.

**Key Functions:**
- `create_plan()` - Create new subscription plans
- `subscribe()` - Subscribe users to plans
- `process_billing()` - Process individual subscription billing
- `process_all_billing()` - Batch process all active subscriptions
- `record_usage()` - Track usage for usage-based billing
- `get_analytics()` - Retrieve comprehensive analytics

#### 2. UsageTracker
Specialized contract for efficient usage tracking and analytics.

**Key Functions:**
- `record_usage()` - Record usage events
- `get_usage_for_period()` - Retrieve usage within time periods
- `get_total_usage()` - Get cumulative usage for subscription

#### 3. BillingCalculations
Library containing optimized billing calculation functions.

**Key Functions:**
- `calculate_billing_amount()` - Compute billing amounts with usage
- `calculate_next_billing_date()` - Determine next billing cycle
- `calculate_churn_risk()` - Assess subscription churn risk
- `calculate_mrr()` - Compute Monthly Recurring Revenue

### Data Structures

#### SubscriptionPlan
```cairo
struct SubscriptionPlan {
    id: u256,
    name: felt252,
    description: felt252,
    price: u256,
    billing_cycle: BillingCycle,
    billing_type: BillingType,
    max_usage: u256,
    grace_period_days: u8,
    active: bool,
    created_at: u64,
}
```

#### Subscription
```cairo
struct Subscription {
    id: u256,
    user: ContractAddress,
    plan_id: u256,
    status: SubscriptionStatus,
    start_date: u64,
    next_billing_date: u64,
    last_payment_date: u64,
    total_paid: u256,
    failed_payments: u8,
    grace_period_until: u64,
    created_at: u64,
}
```

## Usage Examples

### Creating a Subscription Plan

```cairo
// Create a monthly fixed-price plan
let plan_id = subscription_manager.create_plan(
    'Premium Plan',
    'Full access to all features',
    1000000000000000000, // 1 token
    BillingCycle::Monthly,
    BillingType::Fixed,
    0, // unlimited usage
    7  // 7 days grace period
);
```

### Subscribing to a Plan

```cairo
// User subscribes to the plan
let subscription_id = subscription_manager.subscribe(plan_id);
```

### Usage-Based Billing

```cairo
// Create a usage-based plan
let usage_plan_id = subscription_manager.create_plan(
    'API Plan',
    'Pay per API call',
    0, // No base price
    BillingCycle::Monthly,
    BillingType::UsageBased,
    1000, // 1000 calls included
    7
);

// Record usage
subscription_manager.record_usage(subscription_id, 500, 'API_CALLS');
```

### Processing Billing

```cairo
// Process individual subscription billing
let success = subscription_manager.process_billing(subscription_id);

// Process all active subscriptions
let processed_count = subscription_manager.process_all_billing();
```

### Analytics and Insights

```cairo
// Get comprehensive analytics
let analytics = subscription_manager.get_analytics();
// Returns: total_subscriptions, active_subscriptions, mrr, churn_rate, etc.

// Get revenue forecast
let forecast = subscription_manager.get_revenue_forecast(12); // 12 months

// Get churn predictions
let predictions = subscription_manager.get_churn_predictions(10); // Top 10 at risk
```

## Billing Cycles

The system supports five billing cycles:

- **Daily**: 24-hour cycles
- **Weekly**: 7-day cycles  
- **Monthly**: ~30-day cycles
- **Quarterly**: ~90-day cycles
- **Yearly**: ~365-day cycles

## Billing Types

### Fixed Pricing
- Flat monthly/yearly fee
- No usage tracking required
- Simple and predictable

### Usage-Based
- Pay per unit consumed
- Real-time usage tracking
- Overage charges for exceeding limits

### Hybrid
- Base fee + usage charges
- Best of both worlds
- Predictable base + usage flexibility

## Grace Periods

- Configurable grace periods for failed payments
- Automatic retry mechanisms
- Escalation to cancellation after multiple failures
- Admin override capabilities

## Performance Optimizations

### Gas Efficiency
- Batch processing for bulk operations
- Bit manipulation for faster calculations
- Cached values for frequently accessed data
- Optimized array operations

### Scalability
- Efficient indexing for large datasets
- Compressed usage data storage
- Pagination for large result sets
- Background processing capabilities

## Security Features

- Owner-only admin functions
- User authorization for subscription management
- Payment token validation
- Graceful failure handling

## Testing

Comprehensive test suite covering:
- Plan creation and management
- Subscription lifecycle
- Billing calculations
- Usage tracking
- Analytics and reporting
- Error handling and edge cases

Run tests with:
```bash
scarb test
```

## Integration

### With Existing TeachLink System
The subscription system integrates seamlessly with the existing TeachLink marketplace and token systems:

- Uses existing ERC20 payment tokens
- Compatible with marketplace royalty system
- Integrates with user reputation system
- Supports course access control

### With External Systems
- RESTful API endpoints for frontend integration
- Webhook support for real-time notifications
- Export capabilities for external analytics
- Third-party payment processor integration

## Future Enhancements

- **Multi-currency Support**: Support for multiple payment tokens
- **Tiered Pricing**: Volume discounts and tier-based pricing
- **Promotional Codes**: Discount and trial period management
- **White-label Support**: Customizable branding and features
- **Advanced Analytics**: Machine learning-powered insights
- **Mobile SDK**: Native mobile app integration

## Gas Costs

Estimated gas costs for common operations:

- Create Plan: ~50,000 gas
- Subscribe: ~30,000 gas
- Process Billing: ~40,000 gas
- Record Usage: ~15,000 gas
- Batch Process (10 subs): ~200,000 gas

## License

This subscription management system is part of the TeachLink project and follows the same licensing terms.

## Support

For technical support and questions:
- GitHub Issues: [TeachLink Repository]
- Documentation: [TeachLink Docs]
- Community: [TeachLink Discord]

---

*Built with ❤️ for the Starknet ecosystem*
