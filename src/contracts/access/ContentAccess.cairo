#[starknet::contract]
mod ContentAccess {
    use starknet::{
        ContractAddress, get_caller_address, get_block_timestamp,
    };
    use core::array::ArrayTrait;
    use openzeppelin::access::ownable::OwnableComponent;
    use super::interfaces::IContentAccess::{IContentAccess, CourseAccessConfig, AccessMethod, KeyPolicy};
    use super::libraries::AccessControl::{AccessControl, TimeWindow};

    // External contracts used
    use super::subscriptions::interfaces::ISubscriptionManager::{ISubscriptionManagerDispatcher, ISubscriptionManagerDispatcherTrait, SubscriptionStatus};
    use openzeppelin::token::erc721::interface::{IERC721Dispatcher, IERC721DispatcherTrait};

    component!(path: OwnableComponent, storage: ownable, event: OwnableEvent);
    #[abi(embed_v0)]
    impl OwnableImpl = OwnableComponent::OwnableImpl<ContractState>;
    impl OwnableInternalImpl = OwnableComponent::InternalImpl<ContractState>;

    #[storage]
    struct Storage {
        #[substorage(v0)]
        ownable: OwnableComponent::Storage,

        subscription_manager: ContractAddress,
        course_nft: ContractAddress,
        marketplace: ContractAddress,

        // course configuration
        course_configs: LegacyMap<u256, CourseAccessConfig>,

        // per-course key hash (e.g., IPFS CID hash or symmetric key hash)
        course_key_hashes: LegacyMap<u256, felt252>,

        // access grants: user->course->expiry
        access_expiry: LegacyMap<(ContractAddress, u256), u64>,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        #[flat]
        OwnableEvent: OwnableComponent::Event,
        AccessGranted: AccessGranted,
        CourseConfigured: CourseConfigured,
        CourseKeyUpdated: CourseKeyUpdated,
    }

    #[derive(Drop, starknet::Event)]
    struct AccessGranted { user: ContractAddress, course_id: u256, expires_at: u64 }
    #[derive(Drop, starknet::Event)]
    struct CourseConfigured { course_id: u256 }
    #[derive(Drop, starknet::Event)]
    struct CourseKeyUpdated { course_id: u256 }

    #[constructor]
    fn constructor(ref self: ContractState, owner: ContractAddress) {
        self.ownable.initializer(owner);
    }

    #[abi(embed_v0)]
    impl ContentAccessImpl of IContentAccess<ContractState> {
        fn set_subscription_manager(ref self: ContractState, manager: ContractAddress) {
            self.ownable.assert_only_owner();
            self.subscription_manager.write(manager);
        }

        fn set_course_nft(ref self: ContractState, course_nft: ContractAddress) {
            self.ownable.assert_only_owner();
            self.course_nft.write(course_nft);
        }

        fn set_marketplace(ref self: ContractState, marketplace: ContractAddress) {
            self.ownable.assert_only_owner();
            self.marketplace.write(marketplace);
        }

        fn configure_course(ref self: ContractState, config: CourseAccessConfig) {
            self.ownable.assert_only_owner();
            assert(config.course_id != 0, 'bad course');
            // require a seller for token-gated configurations
            if config.access_method == AccessMethod::Token { assert(config.seller != ContractAddress::from(0), 'no seller'); }
            self.course_configs.write(config.course_id, config);
            self.emit(CourseConfigured { course_id: config.course_id });
        }

        fn set_course_key(ref self: ContractState, course_id: u256, key_hash: felt252) {
            self.ownable.assert_only_owner();
            assert(course_id != 0, 'bad course');
            self.course_key_hashes.write(course_id, key_hash);
            self.emit(CourseKeyUpdated { course_id });
        }

        fn grant_access(ref self: ContractState, user: ContractAddress, course_id: u256, duration_secs: u64) {
            // Can be called by marketplace after successful purchase or by owner for manual grant
            let caller = get_caller_address();
            let cfg = self.course_configs.read(course_id);
            assert(cfg.course_id != 0, 'no cfg');
            if caller != self.ownable.owner() {
                // Only marketplace or course seller can grant
                assert(caller == self.marketplace.read() || caller == cfg.seller, 'unauth');
            }
            let now = get_block_timestamp();
            let expires = AccessControl::compute_expiry(now, duration_secs, cfg.end_time);
            self.access_expiry.write((user, course_id), expires);
            self.emit(AccessGranted { user, course_id, expires_at: expires });
        }

        fn has_access(self: @ContractState, user: ContractAddress, course_id: u256) -> bool { Self::_has_access(self, user, course_id) }

        fn get_access_expiry(self: @ContractState, user: ContractAddress, course_id: u256) -> u64 {
            self.access_expiry.read((user, course_id))
        }

        fn get_course_config(self: @ContractState, course_id: u256) -> CourseAccessConfig {
            self.course_configs.read(course_id)
        }

        fn request_content_key(self: @ContractState, user: ContractAddress, course_id: u256) -> felt252 {
            // Frontend queries with user address; contract verifies and returns key hash
            if !Self::_has_access(self, user, course_id) { return 0; }
            let cfg = self.course_configs.read(course_id);
            let key = self.course_key_hashes.read(course_id);
            if key == 0 { return 0; }
            // For KeyPolicy::PerUser one would derive a per-user hash off-chain; on-chain we expose course key id only
            key
        }
    }

    #[generate_trait]
    impl InternalImpl of InternalTrait {
        fn _has_access(self: @ContractState, user: ContractAddress, course_id: u256) -> bool {
            let cfg = self.course_configs.read(course_id);
            if cfg.course_id == 0 { return false; }
            let now = get_block_timestamp();
            if !AccessControl::is_within_time_window(TimeWindow { start: cfg.start_time, end: cfg.end_time }, now) { return false; }

            // Token-gated: owner or granted
            if cfg.access_method == AccessMethod::Token {
                let nft = IERC721Dispatcher { contract_address: self.course_nft.read() };
                let owner = nft.owner_of(course_id);
                if owner == user { return true; }
                let exp = self.access_expiry.read((user, course_id));
                return exp != 0 && now <= exp;
            }

            // Subscription-based: check user's subscriptions for matching plan and Active/GracePeriod
            if cfg.access_method == AccessMethod::Subscription {
                let mgr = ISubscriptionManagerDispatcher { contract_address: self.subscription_manager.read() };
                let subs = mgr.get_user_subscriptions(user);
                let mut i = 0;
                loop {
                    if i >= subs.len() { break; }
                    let sid = *subs.at(i);
                    let s = mgr.get_subscription(sid);
                    if s.plan_id == cfg.subscription_plan_id {
                        if s.status == SubscriptionStatus::Active || s.status == SubscriptionStatus::GracePeriod {
                            break true;
                        }
                    }
                    i += 1;
                };
                let exp = self.access_expiry.read((user, course_id));
                return exp != 0 && now <= exp;
            }
            false
        }
    }
}


