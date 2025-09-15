#[starknet::contract]
pub mod SubscriptionManager {
    use starknet::{
        ContractAddress, get_caller_address, get_contract_address, get_block_timestamp,
        storage::{Map, StorageMapReadAccess, StorageMapWriteAccess, StoragePointerReadAccess, StoragePointerWriteAccess},
    };
    use core::array::{Array, ArrayTrait};
    use openzeppelin::access::ownable::OwnableComponent;
    use openzeppelin::token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherTrait};

    use super::interfaces::ISubscriptionManager::{
        ISubscriptionManager, SubscriptionPlan, Subscription, UsageRecord, BillingPeriod,
        SubscriptionAnalytics, ChurnPrediction, BillingCycle, BillingType, SubscriptionStatus,
    };
    use super::libraries::BillingCalculations::BillingCalculations;
    use super::UsageTracker::{IUsageTrackerDispatcher, IUsageTrackerDispatcherTrait};

    component!(path: OwnableComponent, storage: ownable, event: OwnableEvent);

    #[abi(embed_v0)]
    impl OwnableImpl = OwnableComponent::OwnableImpl<ContractState>;
    impl OwnableInternalImpl = OwnableComponent::InternalImpl<ContractState>;

    #[storage]
    struct Storage {
        // Core data
        plans: Map<u256, SubscriptionPlan>,
        subscriptions: Map<u256, Subscription>,
        billing_periods: Map<u256, BillingPeriod>,
        usage_records: Map<u256, UsageRecord>,
        
        // Counters
        plan_counter: u256,
        subscription_counter: u256,
        billing_period_counter: u256,
        usage_record_counter: u256,
        
        // User to subscription mapping
        user_subscriptions: Map<ContractAddress, Array<u256>>,
        active_subscriptions: Array<u256>,
        
        // Payment and billing
        payment_token: ContractAddress,
        platform_fee_bps: u16,
        usage_tracker: ContractAddress,
        
        // Analytics
        analytics: SubscriptionAnalytics,
        churn_predictions: Map<u256, ChurnPrediction>,
        
        // Billing period indexing
        subscription_billing_periods: Map<u256, Array<u256>>,
        
        // Grace period management
        grace_period_subscriptions: Array<u256>,
        
        // Revenue tracking
        monthly_revenue: Map<u64, u256>, // timestamp -> revenue
        total_revenue: u256,
        
        #[substorage(v0)]
        ownable: OwnableComponent::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        PlanCreated: PlanCreated,
        PlanUpdated: PlanUpdated,
        PlanDeactivated: PlanDeactivated,
        SubscriptionCreated: SubscriptionCreated,
        SubscriptionCancelled: SubscriptionCancelled,
        SubscriptionPaused: SubscriptionPaused,
        SubscriptionResumed: SubscriptionResumed,
        SubscriptionPlanChanged: SubscriptionPlanChanged,
        BillingProcessed: BillingProcessed,
        PaymentFailed: PaymentFailed,
        GracePeriodExtended: GracePeriodExtended,
        UsageRecorded: UsageRecorded,
        AnalyticsUpdated: AnalyticsUpdated,
        #[flat]
        OwnableEvent: OwnableComponent::Event,
    }

    #[derive(Drop, starknet::Event)]
    pub struct PlanCreated { pub plan_id: u256, pub name: felt252, pub price: u256 }
    #[derive(Drop, starknet::Event)]
    pub struct PlanUpdated { pub plan_id: u256, pub name: felt252, pub price: u256 }
    #[derive(Drop, starknet::Event)]
    pub struct PlanDeactivated { pub plan_id: u256 }
    #[derive(Drop, starknet::Event)]
    pub struct SubscriptionCreated { pub subscription_id: u256, pub user: ContractAddress, pub plan_id: u256 }
    #[derive(Drop, starknet::Event)]
    pub struct SubscriptionCancelled { pub subscription_id: u256, pub user: ContractAddress }
    #[derive(Drop, starknet::Event)]
    pub struct SubscriptionPaused { pub subscription_id: u256, pub user: ContractAddress }
    #[derive(Drop, starknet::Event)]
    pub struct SubscriptionResumed { pub subscription_id: u256, pub user: ContractAddress }
    #[derive(Drop, starknet::Event)]
    pub struct SubscriptionPlanChanged { pub subscription_id: u256, pub old_plan_id: u256, pub new_plan_id: u256 }
    #[derive(Drop, starknet::Event)]
    pub struct BillingProcessed { pub subscription_id: u256, pub amount: u256, pub success: bool }
    #[derive(Drop, starknet::Event)]
    pub struct PaymentFailed { pub subscription_id: u256, pub failed_payments: u8 }
    #[derive(Drop, starknet::Event)]
    pub struct GracePeriodExtended { pub subscription_id: u256, pub new_grace_period_until: u64 }
    #[derive(Drop, starknet::Event)]
    pub struct UsageRecorded { pub subscription_id: u256, pub amount: u256, pub unit: felt252 }
    #[derive(Drop, starknet::Event)]
    pub struct AnalyticsUpdated { pub mrr: u256, pub churn_rate: u256 }

    #[constructor]
    fn constructor(
        ref self: ContractState,
        owner: ContractAddress,
        payment_token: ContractAddress,
        platform_fee_bps: u16,
        usage_tracker: ContractAddress
    ) {
        self.ownable.initializer(owner);
        assert(platform_fee_bps <= 1000, 'fee>10%');
        self.payment_token.write(payment_token);
        self.platform_fee_bps.write(platform_fee_bps);
        self.usage_tracker.write(usage_tracker);
        self.plan_counter.write(0);
        self.subscription_counter.write(0);
        self.billing_period_counter.write(0);
        self.usage_record_counter.write(0);
        self.total_revenue.write(0);
        self.analytics.write(SubscriptionAnalytics {
            total_subscriptions: 0,
            active_subscriptions: 0,
            cancelled_subscriptions: 0,
            total_revenue: 0,
            monthly_recurring_revenue: 0,
            churn_rate: 0,
            average_revenue_per_user: 0,
        });
    }

    #[abi(embed_v0)]
    impl SubscriptionManagerImpl of ISubscriptionManager<ContractState> {
        // Admin functions
        fn create_plan(
            ref self: ContractState,
            name: felt252,
            description: felt252,
            price: u256,
            billing_cycle: BillingCycle,
            billing_type: BillingType,
            max_usage: u256,
            grace_period_days: u8
        ) -> u256 {
            self.ownable.assert_only_owner();
            assert(price > 0, 'price=0');
            assert(grace_period_days <= 30, 'grace>30d');

            let plan_id = self.plan_counter.read() + 1;
            self.plan_counter.write(plan_id);

            let plan = SubscriptionPlan {
                id: plan_id,
                name,
                description,
                price,
                billing_cycle,
                billing_type,
                max_usage,
                grace_period_days,
                active: true,
                created_at: get_block_timestamp(),
            };

            self.plans.write(plan_id, plan);
            self.emit(PlanCreated { plan_id, name, price });
            plan_id
        }

        fn update_plan(
            ref self: ContractState,
            plan_id: u256,
            name: felt252,
            description: felt252,
            price: u256,
            max_usage: u256,
            grace_period_days: u8
        ) {
            self.ownable.assert_only_owner();
            let mut plan = self.plans.read(plan_id);
            assert(plan.id != 0, 'no plan');
            assert(plan.active, 'inactive plan');

            plan.name = name;
            plan.description = description;
            plan.price = price;
            plan.max_usage = max_usage;
            plan.grace_period_days = grace_period_days;

            self.plans.write(plan_id, plan);
            self.emit(PlanUpdated { plan_id, name, price });
        }

        fn deactivate_plan(ref self: ContractState, plan_id: u256) {
            self.ownable.assert_only_owner();
            let mut plan = self.plans.read(plan_id);
            assert(plan.id != 0, 'no plan');
            plan.active = false;
            self.plans.write(plan_id, plan);
            self.emit(PlanDeactivated { plan_id });
        }

        fn set_payment_token(ref self: ContractState, token: ContractAddress) {
            self.ownable.assert_only_owner();
            self.payment_token.write(token);
        }

        fn set_platform_fee(ref self: ContractState, fee_bps: u16) {
            self.ownable.assert_only_owner();
            assert(fee_bps <= 1000, 'fee>10%');
            self.platform_fee_bps.write(fee_bps);
        }

        // Subscription management
        fn subscribe(ref self: ContractState, plan_id: u256) -> u256 {
            let plan = self.plans.read(plan_id);
            assert(plan.id != 0, 'no plan');
            assert(plan.active, 'inactive plan');

            let user = get_caller_address();
            let subscription_id = self.subscription_counter.read() + 1;
            self.subscription_counter.write(subscription_id);

            let now = get_block_timestamp();
            let next_billing = BillingCalculations::calculate_next_billing_date(now, plan.billing_cycle, now);

            let subscription = Subscription {
                id: subscription_id,
                user,
                plan_id,
                status: SubscriptionStatus::Active,
                start_date: now,
                next_billing_date: next_billing,
                last_payment_date: now,
                total_paid: 0,
                failed_payments: 0,
                grace_period_until: 0,
                created_at: now,
            };

            self.subscriptions.write(subscription_id, subscription);
            self._add_user_subscription(user, subscription_id);
            self._add_active_subscription(subscription_id);
            self._update_analytics();

            self.emit(SubscriptionCreated { subscription_id, user, plan_id });
            subscription_id
        }

        fn cancel_subscription(ref self: ContractState, subscription_id: u256) {
            let mut subscription = self.subscriptions.read(subscription_id);
            assert(subscription.id != 0, 'no subscription');
            assert(subscription.user == get_caller_address(), 'not owner');

            subscription.status = SubscriptionStatus::Cancelled;
            self.subscriptions.write(subscription_id, subscription);
            self._remove_active_subscription(subscription_id);
            self._update_analytics();

            self.emit(SubscriptionCancelled { subscription_id, user: subscription.user });
        }

        fn pause_subscription(ref self: ContractState, subscription_id: u256) {
            let mut subscription = self.subscriptions.read(subscription_id);
            assert(subscription.id != 0, 'no subscription');
            assert(subscription.user == get_caller_address(), 'not owner');
            assert(subscription.status == SubscriptionStatus::Active, 'not active');

            subscription.status = SubscriptionStatus::Paused;
            self.subscriptions.write(subscription_id, subscription);
            self._remove_active_subscription(subscription_id);
            self._update_analytics();

            self.emit(SubscriptionPaused { subscription_id, user: subscription.user });
        }

        fn resume_subscription(ref self: ContractState, subscription_id: u256) {
            let mut subscription = self.subscriptions.read(subscription_id);
            assert(subscription.id != 0, 'no subscription');
            assert(subscription.user == get_caller_address(), 'not owner');
            assert(subscription.status == SubscriptionStatus::Paused, 'not paused');

            subscription.status = SubscriptionStatus::Active;
            self.subscriptions.write(subscription_id, subscription);
            self._add_active_subscription(subscription_id);
            self._update_analytics();

            self.emit(SubscriptionResumed { subscription_id, user: subscription.user });
        }

        fn update_subscription_plan(ref self: ContractState, subscription_id: u256, new_plan_id: u256) {
            let mut subscription = self.subscriptions.read(subscription_id);
            assert(subscription.id != 0, 'no subscription');
            assert(subscription.user == get_caller_address(), 'not owner');
            assert(subscription.status == SubscriptionStatus::Active, 'not active');

            let old_plan = self.plans.read(subscription.plan_id);
            let new_plan = self.plans.read(new_plan_id);
            assert(new_plan.id != 0, 'no new plan');
            assert(new_plan.active, 'inactive new plan');

            // Calculate proration if needed
            let now = get_block_timestamp();
            let days_remaining = (subscription.next_billing_date - now) / 86400;
            let total_days = match old_plan.billing_cycle {
                BillingCycle::Daily => 1,
                BillingCycle::Weekly => 7,
                BillingCycle::Monthly => 30,
                BillingCycle::Quarterly => 90,
                BillingCycle::Yearly => 365,
            };

            let (refund, charge) = BillingCalculations::calculate_proration(
                old_plan, new_plan, days_remaining, total_days
            );

            // Process proration payment
            if charge > refund {
                let token = IERC20Dispatcher { contract_address: self.payment_token.read() };
                let ok = token.transfer_from(get_caller_address(), get_contract_address(), charge - refund);
                assert(ok, 'proration fail');
            }

            subscription.plan_id = new_plan_id;
            self.subscriptions.write(subscription_id, subscription);

            self.emit(SubscriptionPlanChanged { subscription_id, old_plan_id: subscription.plan_id, new_plan_id });
        }

        // Billing and payments
        fn process_billing(ref self: ContractState, subscription_id: u256) -> bool {
            let mut subscription = self.subscriptions.read(subscription_id);
            assert(subscription.id != 0, 'no subscription');
            assert(subscription.status == SubscriptionStatus::Active, 'not active');

            let now = get_block_timestamp();
            if now < subscription.next_billing_date {
                return false;
            }

            let plan = self.plans.read(subscription.plan_id);
            let billing_result = self._calculate_billing_amount(subscription, plan, subscription.last_payment_date, now);

            // Attempt payment
            let token = IERC20Dispatcher { contract_address: self.payment_token.read() };
            let payment_success = token.transfer_from(subscription.user, get_contract_address(), billing_result.total_amount);

            if payment_success {
                // Successful payment
                subscription.last_payment_date = now;
                subscription.next_billing_date = billing_result.next_billing_date;
                subscription.total_paid += billing_result.total_amount;
                subscription.failed_payments = 0;
                subscription.grace_period_until = 0;
                self.subscriptions.write(subscription_id, subscription);

                // Create billing period record
                self._create_billing_period(subscription_id, subscription.last_payment_date, now, billing_result);

                // Update revenue tracking
                self._update_revenue_tracking(billing_result.total_amount);

                self.emit(BillingProcessed { subscription_id, amount: billing_result.total_amount, success: true });
                true
            } else {
                // Failed payment
                subscription.failed_payments += 1;
                if subscription.failed_payments >= 3 {
                    subscription.status = SubscriptionStatus::Cancelled;
                    self._remove_active_subscription(subscription_id);
                } else {
                    subscription.status = SubscriptionStatus::GracePeriod;
                    subscription.grace_period_until = BillingCalculations::calculate_grace_period_end(
                        now, plan.grace_period_days
                    );
                    self._add_grace_period_subscription(subscription_id);
                }
                self.subscriptions.write(subscription_id, subscription);
                self._update_analytics();

                self.emit(PaymentFailed { subscription_id, failed_payments: subscription.failed_payments });
                false
            }
        }

        fn process_all_billing(ref self: ContractState) -> u256 {
            let mut processed = 0_u256;
            let active_subs = self.active_subscriptions.read();
            let mut i = 0_u32;
            let len = active_subs.len();

            loop {
                if i >= len { break; }
                let subscription_id = *active_subs.at(i);
                if self.process_billing(subscription_id) {
                    processed += 1;
                }
                i += 1;
            };

            processed
        }

        fn retry_failed_payment(ref self: ContractState, subscription_id: u256) -> bool {
            let subscription = self.subscriptions.read(subscription_id);
            assert(subscription.id != 0, 'no subscription');
            assert(subscription.status == SubscriptionStatus::GracePeriod, 'not in grace');

            self.process_billing(subscription_id)
        }

        fn extend_grace_period(ref self: ContractState, subscription_id: u256, additional_days: u8) {
            self.ownable.assert_only_owner();
            let mut subscription = self.subscriptions.read(subscription_id);
            assert(subscription.id != 0, 'no subscription');
            assert(subscription.status == SubscriptionStatus::GracePeriod, 'not in grace');

            subscription.grace_period_until += (additional_days.into() * 86400);
            self.subscriptions.write(subscription_id, subscription);

            self.emit(GracePeriodExtended { subscription_id, new_grace_period_until: subscription.grace_period_until });
        }

        // Usage tracking
        fn record_usage(ref self: ContractState, subscription_id: u256, amount: u256, unit: felt252) {
            let subscription = self.subscriptions.read(subscription_id);
            assert(subscription.id != 0, 'no subscription');
            assert(subscription.status == SubscriptionStatus::Active, 'not active');

            let usage_tracker = IUsageTrackerDispatcher { contract_address: self.usage_tracker.read() };
            usage_tracker.record_usage(subscription_id, amount, unit);

            self.emit(UsageRecorded { subscription_id, amount, unit });
        }

        fn get_usage_for_period(
            self: @ContractState,
            subscription_id: u256,
            start_date: u64,
            end_date: u64
        ) -> u256 {
            let usage_tracker = IUsageTrackerDispatcher { contract_address: self.usage_tracker.read() };
            usage_tracker.get_usage_for_period(subscription_id, start_date, end_date)
        }

        // Analytics and insights
        fn get_analytics(self: @ContractState) -> SubscriptionAnalytics {
            self.analytics.read()
        }

        fn get_churn_predictions(self: @ContractState, limit: u256) -> Array<ChurnPrediction> {
            let mut predictions = ArrayTrait::new();
            let active_subs = self.active_subscriptions.read();
            let mut i = 0_u32;
            let len = active_subs.len();
            let mut added = 0_u256;

            loop {
                if i >= len || added >= limit { break; }
                let subscription_id = *active_subs.at(i);
                let prediction = self.churn_predictions.read(subscription_id);
                if prediction.subscription_id != 0 {
                    predictions.append(prediction);
                    added += 1;
                }
                i += 1;
            };

            predictions
        }

        fn get_subscription_history(self: @ContractState, user: ContractAddress) -> Array<u256> {
            self.user_subscriptions.read(user)
        }

        fn get_revenue_forecast(self: @ContractState, months: u8) -> u256 {
            let mut forecast = 0_u256;
            let active_subs = self.active_subscriptions.read();
            let mut i = 0_u32;
            let len = active_subs.len();

            loop {
                if i >= len { break; }
                let subscription_id = *active_subs.at(i);
                let subscription = self.subscriptions.read(subscription_id);
                let plan = self.plans.read(subscription.plan_id);
                
                // Convert to monthly equivalent
                let monthly_rate = match plan.billing_cycle {
                    BillingCycle::Daily => plan.price * 30_u256,
                    BillingCycle::Weekly => (plan.price * 52_u256) / 12_u256,
                    BillingCycle::Monthly => plan.price,
                    BillingCycle::Quarterly => plan.price / 3_u256,
                    BillingCycle::Yearly => plan.price / 12_u256,
                };

                forecast += monthly_rate * months.into();
                i += 1;
            };

            forecast
        }

        // Views
        fn get_plan(self: @ContractState, plan_id: u256) -> SubscriptionPlan {
            self.plans.read(plan_id)
        }

        fn get_subscription(self: @ContractState, subscription_id: u256) -> Subscription {
            self.subscriptions.read(subscription_id)
        }

        fn get_user_subscriptions(self: @ContractState, user: ContractAddress) -> Array<u256> {
            self.user_subscriptions.read(user)
        }

        fn get_active_subscriptions(self: @ContractState) -> Array<u256> {
            self.active_subscriptions.read()
        }

        fn get_billing_periods(self: @ContractState, subscription_id: u256) -> Array<u256> {
            self.subscription_billing_periods.read(subscription_id)
        }

        fn get_usage_records(self: @ContractState, subscription_id: u256, limit: u256) -> Array<UsageRecord> {
            let usage_tracker = IUsageTrackerDispatcher { contract_address: self.usage_tracker.read() };
            usage_tracker.get_usage_records(subscription_id, limit)
        }

        fn is_subscription_active(self: @ContractState, subscription_id: u256) -> bool {
            let subscription = self.subscriptions.read(subscription_id);
            subscription.status == SubscriptionStatus::Active
        }

        fn get_next_billing_amount(self: @ContractState, subscription_id: u256) -> u256 {
            let subscription = self.subscriptions.read(subscription_id);
            let plan = self.plans.read(subscription.plan_id);
            let billing_result = self._calculate_billing_amount(subscription, plan, subscription.last_payment_date, subscription.next_billing_date);
            billing_result.total_amount
        }
    }

    #[generate_trait]
    impl InternalImpl of InternalTrait {
        fn _add_user_subscription(ref self: ContractState, user: ContractAddress, subscription_id: u256) {
            let mut subs = self.user_subscriptions.read(user);
            subs.append(subscription_id);
            self.user_subscriptions.write(user, subs);
        }

        fn _add_active_subscription(ref self: ContractState, subscription_id: u256) {
            let mut subs = self.active_subscriptions.read();
            subs.append(subscription_id);
            self.active_subscriptions.write(subs);
        }

        fn _remove_active_subscription(ref self: ContractState, subscription_id: u256) {
            let mut subs = self.active_subscriptions.read();
            let mut new_subs = ArrayTrait::new();
            let mut i = 0_u32;
            let len = subs.len();

            loop {
                if i >= len { break; }
                let sub_id = *subs.at(i);
                if sub_id != subscription_id {
                    new_subs.append(sub_id);
                }
                i += 1;
            };

            self.active_subscriptions.write(new_subs);
        }

        fn _add_grace_period_subscription(ref self: ContractState, subscription_id: u256) {
            let mut subs = self.grace_period_subscriptions.read();
            subs.append(subscription_id);
            self.grace_period_subscriptions.write(subs);
        }

        fn _calculate_billing_amount(
            self: @ContractState,
            subscription: Subscription,
            plan: SubscriptionPlan,
            period_start: u64,
            period_end: u64
        ) -> BillingCalculations::BillingResult {
            let usage_tracker = IUsageTrackerDispatcher { contract_address: self.usage_tracker.read() };
            let usage_records = usage_tracker.get_usage_records(subscription.id, 1000); // Get up to 1000 records
            
            BillingCalculations::calculate_billing_amount(plan, usage_records, period_start, period_end)
        }

        fn _create_billing_period(
            ref self: ContractState,
            subscription_id: u256,
            start_date: u64,
            end_date: u64,
            billing_result: BillingCalculations::BillingResult
        ) {
            let period_id = self.billing_period_counter.read() + 1;
            self.billing_period_counter.write(period_id);

            let period = BillingPeriod {
                subscription_id,
                start_date,
                end_date,
                base_amount: billing_result.base_amount,
                usage_amount: billing_result.usage_amount,
                total_amount: billing_result.total_amount,
                paid: true,
                payment_date: get_block_timestamp(),
            };

            self.billing_periods.write(period_id, period);
            
            let mut periods = self.subscription_billing_periods.read(subscription_id);
            periods.append(period_id);
            self.subscription_billing_periods.write(subscription_id, periods);
        }

        fn _update_revenue_tracking(ref self: ContractState, amount: u256) {
            let mut total = self.total_revenue.read();
            total += amount;
            self.total_revenue.write(total);

            let now = get_block_timestamp();
            let month_start = now - (now % 2592000); // Approximate month start
            let mut monthly = self.monthly_revenue.read(month_start);
            monthly += amount;
            self.monthly_revenue.write(month_start, monthly);
        }

        fn _update_analytics(ref self: ContractState) {
            let mut analytics = self.analytics.read();
            let active_subs = self.active_subscriptions.read();
            analytics.active_subscriptions = active_subs.len().into();
            analytics.total_revenue = self.total_revenue.read();
            analytics.monthly_recurring_revenue = self._calculate_mrr();
            analytics.average_revenue_per_user = if analytics.active_subscriptions > 0 {
                analytics.total_revenue / analytics.active_subscriptions
            } else {
                0
            };
            self.analytics.write(analytics);
        }

        fn _calculate_mrr(self: @ContractState) -> u256 {
            let mut mrr = 0_u256;
            let active_subs = self.active_subscriptions.read();
            let mut i = 0_u32;
            let len = active_subs.len();

            loop {
                if i >= len { break; }
                let subscription_id = *active_subs.at(i);
                let subscription = self.subscriptions.read(subscription_id);
                let plan = self.plans.read(subscription.plan_id);
                
                // Convert to monthly equivalent
                let monthly_rate = match plan.billing_cycle {
                    BillingCycle::Daily => plan.price * 30_u256,
                    BillingCycle::Weekly => (plan.price * 52_u256) / 12_u256,
                    BillingCycle::Monthly => plan.price,
                    BillingCycle::Quarterly => plan.price / 3_u256,
                    BillingCycle::Yearly => plan.price / 12_u256,
                };

                mrr += monthly_rate;
                i += 1;
            };

            mrr
        }
    }
}
