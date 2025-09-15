#[starknet::contract]
pub mod UsageTracker {
    use starknet::{
        ContractAddress, get_caller_address, get_contract_address, get_block_timestamp,
        storage::{Map, StorageMapReadAccess, StorageMapWriteAccess},
    };
    use core::array::{Array, ArrayTrait};
    use openzeppelin::access::ownable::OwnableComponent;

    use super::interfaces::ISubscriptionManager::{UsageRecord, SubscriptionStatus};

    component!(path: OwnableComponent, storage: ownable, event: OwnableEvent);

    #[abi(embed_v0)]
    impl OwnableImpl = OwnableComponent::OwnableImpl<ContractState>;
    impl OwnableInternalImpl = OwnableComponent::InternalImpl<ContractState>;

    #[storage]
    struct Storage {
        // Usage tracking
        usage_records: Map<u256, UsageRecord>,
        usage_counter: u256,
        
        // Subscription to usage mapping
        subscription_usage_count: Map<u256, u256>,
        subscription_usage_index: Map<(u256, u256), u256>, // (subscription_id, index) -> usage_record_id
        
        // Time-based indexing for efficient queries
        usage_by_timestamp: Map<u64, Array<u256>>,
        usage_by_subscription_and_time: Map<(u256, u64), Array<u256>>,
        
        // Aggregated usage data
        total_usage_by_subscription: Map<u256, u256>,
        usage_by_period: Map<(u256, u64), u256>, // (subscription_id, period_start) -> total_usage
        
        // Access control
        authorized_trackers: Map<ContractAddress, bool>,
        
        #[substorage(v0)]
        ownable: OwnableComponent::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        UsageRecorded: UsageRecorded,
        TrackerAuthorized: TrackerAuthorized,
        TrackerDeauthorized: TrackerDeauthorized,
        #[flat]
        OwnableEvent: OwnableComponent::Event,
    }

    #[derive(Drop, starknet::Event)]
    pub struct UsageRecorded {
        pub subscription_id: u256,
        pub record_id: u256,
        pub amount: u256,
        pub unit: felt252,
        pub timestamp: u64,
    }

    #[derive(Drop, starknet::Event)]
    pub struct TrackerAuthorized {
        pub tracker: ContractAddress,
    }

    #[derive(Drop, starknet::Event)]
    pub struct TrackerDeauthorized {
        pub tracker: ContractAddress,
    }

    #[constructor]
    fn constructor(ref self: ContractState, owner: ContractAddress) {
        self.ownable.initializer(owner);
        self.usage_counter.write(0);
    }

    #[abi(embed_v0)]
    impl UsageTrackerImpl of super::interfaces::ISubscriptionManager::IUsageTracker<ContractState> {
        fn record_usage(
            ref self: ContractState,
            subscription_id: u256,
            amount: u256,
            unit: felt252
        ) {
            // Only authorized trackers or the subscription manager can record usage
            let caller = get_caller_address();
            assert(
                self.authorized_trackers.read(caller) || caller == get_contract_address(),
                'unauthorized'
            );

            let timestamp = get_block_timestamp();
            let record_id = self.usage_counter.read() + 1;
            self.usage_counter.write(record_id);

            let usage_record = UsageRecord {
                subscription_id,
                timestamp,
                amount,
                unit,
            };

            // Store the usage record
            self.usage_records.write(record_id, usage_record);

            // Update subscription usage count and index
            let count = self.subscription_usage_count.read(subscription_id);
            self.subscription_usage_count.write(subscription_id, count + 1);
            self.subscription_usage_index.write((subscription_id, count), record_id);

            // Update time-based indexing
            self._add_to_timestamp_index(record_id, timestamp);
            self._add_to_subscription_time_index(subscription_id, record_id, timestamp);

            // Update aggregated data
            self.total_usage_by_subscription.write(
                subscription_id,
                self.total_usage_by_subscription.read(subscription_id) + amount
            );

            // Update period-based usage (daily periods for simplicity)
            let period_start = timestamp - (timestamp % 86400); // Start of day
            self.usage_by_period.write(
                (subscription_id, period_start),
                self.usage_by_period.read((subscription_id, period_start)) + amount
            );

            self.emit(UsageRecorded {
                subscription_id,
                record_id,
                amount,
                unit,
                timestamp,
            });
        }

        fn get_usage_for_period(
            self: @ContractState,
            subscription_id: u256,
            start_date: u64,
            end_date: u64
        ) -> u256 {
            let mut total_usage = 0_u256;
            let count = self.subscription_usage_count.read(subscription_id);
            let mut i = 0_u256;

            loop {
                if i >= count { break; }
                let record_id = self.subscription_usage_index.read((subscription_id, i));
                let record = self.usage_records.read(record_id);
                
                if record.timestamp >= start_date && record.timestamp <= end_date {
                    total_usage += record.amount;
                }
                i += 1;
            };

            total_usage
        }

        fn get_usage_records(
            self: @ContractState,
            subscription_id: u256,
            limit: u256
        ) -> Array<UsageRecord> {
            let mut records = ArrayTrait::new();
            let count = self.subscription_usage_count.read(subscription_id);
            let mut i = 0_u256;
            let mut added = 0_u256;

            loop {
                if i >= count || added >= limit { break; }
                let record_id = self.subscription_usage_index.read((subscription_id, i));
                let record = self.usage_records.read(record_id);
                records.append(record);
                added += 1;
                i += 1;
            };

            records
        }

        fn get_total_usage(self: @ContractState, subscription_id: u256) -> u256 {
            self.total_usage_by_subscription.read(subscription_id)
        }

        fn get_usage_by_period(
            self: @ContractState,
            subscription_id: u256,
            period_start: u64
        ) -> u256 {
            self.usage_by_period.read((subscription_id, period_start))
        }
    }

    #[abi]
    impl AdminImpl of AdminTrait {
        fn authorize_tracker(ref self: ContractState, tracker: ContractAddress) {
            self.ownable.assert_only_owner();
            self.authorized_trackers.write(tracker, true);
            self.emit(TrackerAuthorized { tracker });
        }

        fn deauthorize_tracker(ref self: ContractState, tracker: ContractAddress) {
            self.ownable.assert_only_owner();
            self.authorized_trackers.write(tracker, false);
            self.emit(TrackerDeauthorized { tracker });
        }

        fn is_authorized(self: @ContractState, tracker: ContractAddress) -> bool {
            self.authorized_trackers.read(tracker)
        }
    }

    #[generate_trait]
    impl InternalImpl of InternalTrait {
        fn _add_to_timestamp_index(ref self: ContractState, record_id: u256, timestamp: u64) {
            let mut records = self.usage_by_timestamp.read(timestamp);
            records.append(record_id);
            self.usage_by_timestamp.write(timestamp, records);
        }

        fn _add_to_subscription_time_index(
            ref self: ContractState,
            subscription_id: u256,
            record_id: u256,
            timestamp: u64
        ) {
            let mut records = self.usage_by_subscription_and_time.read((subscription_id, timestamp));
            records.append(record_id);
            self.usage_by_subscription_and_time.write((subscription_id, timestamp), records);
        }
    }
}

#[starknet::interface]
pub trait IUsageTracker<TContractState> {
    fn record_usage(ref self: TContractState, subscription_id: u256, amount: u256, unit: felt252);
    fn get_usage_for_period(
        self: @TContractState,
        subscription_id: u256,
        start_date: u64,
        end_date: u64
    ) -> u256;
    fn get_usage_records(
        self: @TContractState,
        subscription_id: u256,
        limit: u256
    ) -> Array<UsageRecord>;
    fn get_total_usage(self: @TContractState, subscription_id: u256) -> u256;
    fn get_usage_by_period(
        self: @TContractState,
        subscription_id: u256,
        period_start: u64
    ) -> u256;
}

#[starknet::interface]
pub trait AdminTrait<TContractState> {
    fn authorize_tracker(ref self: TContractState, tracker: ContractAddress);
    fn deauthorize_tracker(ref self: TContractState, tracker: ContractAddress);
    fn is_authorized(self: @TContractState, tracker: ContractAddress) -> bool;
}
