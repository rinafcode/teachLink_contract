use starknet::ContractAddress;
use super::interfaces::ISubscriptionManager::{BillingCycle, BillingType, SubscriptionPlan, UsageRecord};

pub mod BillingCalculations {
    use super::*;

    #[derive(Drop, Serde, starknet::Store, Clone, Copy)]
    pub struct BillingResult {
        pub base_amount: u256,
        pub usage_amount: u256,
        pub total_amount: u256,
        pub next_billing_date: u64,
    }

    #[derive(Drop, Serde, starknet::Store, Clone, Copy)]
    pub struct UsageCalculation {
        pub total_usage: u256,
        pub overage_amount: u256,
        pub overage_cost: u256,
    }

    /// Calculate the next billing date based on billing cycle
    pub fn calculate_next_billing_date(
        current_date: u64,
        billing_cycle: BillingCycle,
        start_date: u64
    ) -> u64 {
        match billing_cycle {
            BillingCycle::Daily => current_date + 86400, // 24 hours in seconds
            BillingCycle::Weekly => current_date + 604800, // 7 days in seconds
            BillingCycle::Monthly => {
                // Approximate 30 days, in production you'd want more precise month calculation
                current_date + 2592000
            },
            BillingCycle::Quarterly => {
                // Approximate 90 days
                current_date + 7776000
            },
            BillingCycle::Yearly => {
                // Approximate 365 days
                current_date + 31536000
            },
        }
    }

    /// Calculate billing amount for a subscription
    pub fn calculate_billing_amount(
        plan: SubscriptionPlan,
        usage_records: Array<UsageRecord>,
        period_start: u64,
        period_end: u64
    ) -> BillingResult {
        let base_amount = plan.price;
        let mut usage_amount = 0_u256;

        // Calculate usage-based billing if applicable
        if plan.billing_type == BillingType::UsageBased || plan.billing_type == BillingType::Hybrid {
            let usage_calc = calculate_usage_cost(plan, usage_records, period_start, period_end);
            usage_amount = usage_calc.total_amount;
        }

        let total_amount = base_amount + usage_amount;
        let next_billing_date = calculate_next_billing_date(period_end, plan.billing_cycle, period_start);

        BillingResult {
            base_amount,
            usage_amount,
            total_amount,
            next_billing_date,
        }
    }

    /// Calculate usage-based costs
    pub fn calculate_usage_cost(
        plan: SubscriptionPlan,
        usage_records: Array<UsageRecord>,
        period_start: u64,
        period_end: u64
    ) -> UsageCalculation {
        let mut total_usage = 0_u256;
        let mut i = 0_u32;
        let len = usage_records.len();

        // Sum up usage within the period
        loop {
            if i >= len { break; }
            let record = *usage_records.at(i);
            if record.timestamp >= period_start && record.timestamp <= period_end {
                total_usage += record.amount;
            }
            i += 1;
        }

        // Calculate overage
        let overage_amount = if plan.max_usage > 0 && total_usage > plan.max_usage {
            total_usage - plan.max_usage
        } else {
            0_u256
        };

        // For simplicity, overage cost is 2x the base price per unit
        // In production, this would be configurable per plan
        let overage_cost = if overage_amount > 0 {
            (overage_amount * plan.price) / plan.max_usage
        } else {
            0_u256
        };

        let total_amount = if plan.billing_type == BillingType::UsageBased {
            // For pure usage-based, base amount is 0
            overage_cost
        } else {
            // For hybrid, include base amount
            plan.price + overage_cost
        };

        UsageCalculation {
            total_usage,
            overage_amount,
            overage_cost: total_amount,
        }
    }

    /// Calculate grace period end date
    pub fn calculate_grace_period_end(
        last_payment_date: u64,
        grace_period_days: u8
    ) -> u64 {
        last_payment_date + (grace_period_days.into() * 86400) // Convert days to seconds
    }

    /// Check if subscription is in grace period
    pub fn is_in_grace_period(
        current_time: u64,
        grace_period_until: u64
    ) -> bool {
        current_time <= grace_period_until
    }

    /// Calculate prorated amount for mid-cycle plan changes
    pub fn calculate_proration(
        old_plan: SubscriptionPlan,
        new_plan: SubscriptionPlan,
        days_remaining: u64,
        total_days_in_cycle: u64
    ) -> (u256, u256) {
        // Calculate refund for old plan
        let old_refund = if days_remaining > 0 {
            (old_plan.price * days_remaining.into()) / total_days_in_cycle.into()
        } else {
            0_u256
        };

        // Calculate charge for new plan
        let new_charge = if days_remaining > 0 {
            (new_plan.price * days_remaining.into()) / total_days_in_cycle.into()
        } else {
            0_u256
        };

        (old_refund, new_charge)
    }

    /// Calculate churn risk score based on various factors
    pub fn calculate_churn_risk(
        failed_payments: u8,
        days_since_last_payment: u64,
        usage_trend: u256, // 0 = decreasing, 1 = stable, 2 = increasing
        subscription_age_days: u64
    ) -> u256 {
        let mut risk_score = 0_u256;

        // Failed payments increase risk
        risk_score += failed_payments.into() * 20_u256; // Each failed payment adds 20 points

        // Days since last payment
        if days_since_last_payment > 30 {
            risk_score += 30_u256;
        } else if days_since_last_payment > 14 {
            risk_score += 15_u256;
        }

        // Usage trend
        if usage_trend == 0 { // Decreasing usage
            risk_score += 25_u256;
        }

        // Subscription age (newer subscriptions are riskier)
        if subscription_age_days < 30 {
            risk_score += 15_u256;
        }

        // Cap at 100
        if risk_score > 100 {
            risk_score = 100;
        }

        risk_score
    }

    /// Calculate monthly recurring revenue (MRR)
    pub fn calculate_mrr(
        subscriptions: Array<SubscriptionPlan>,
        active_subscription_counts: Array<u256>
    ) -> u256 {
        let mut mrr = 0_u256;
        let mut i = 0_u32;
        let len = subscriptions.len();

        loop {
            if i >= len { break; }
            let plan = *subscriptions.at(i);
            let count = *active_subscription_counts.at(i);
            
            // Convert to monthly equivalent
            let monthly_rate = match plan.billing_cycle {
                BillingCycle::Daily => plan.price * 30_u256,
                BillingCycle::Weekly => (plan.price * 52_u256) / 12_u256,
                BillingCycle::Monthly => plan.price,
                BillingCycle::Quarterly => plan.price / 3_u256,
                BillingCycle::Yearly => plan.price / 12_u256,
            };

            mrr += monthly_rate * count;
            i += 1;
        }

        mrr
    }

    /// Calculate churn rate in basis points
    pub fn calculate_churn_rate(
        cancelled_this_period: u256,
        active_at_start: u256
    ) -> u256 {
        if active_at_start == 0 {
            return 0;
        }
        (cancelled_this_period * 10000_u256) / active_at_start
    }
}
