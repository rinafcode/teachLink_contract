#[starknet::contract]
pub mod MarketplaceContract {
    use starknet::{
        ContractAddress, get_caller_address, get_contract_address, get_block_timestamp,
        storage::{
            StorageMapReadAccess, StorageMapWriteAccess, StoragePointerReadAccess,
            StoragePointerWriteAccess, Map,
        },
    };
    use core::array::{Array, ArrayTrait};
    use openzeppelin::token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherTrait};
    use openzeppelin::access::ownable::OwnableComponent;
    use teachlink::interfaces::IMarketplace::{IMarketplace, Course, Purchase, Dispute};

    component!(path: OwnableComponent, storage: ownable, event: OwnableEvent);

    #[abi(embed_v0)]
    impl OwnableImpl = OwnableComponent::OwnableImpl<ContractState>;
    impl OwnableInternalImpl = OwnableComponent::InternalImpl<ContractState>;

    #[storage]
    struct Storage {
        // Core contract state
        payment_token: ContractAddress,
        platform_fee_percentage: u16, // Basis points (e.g., 250 = 2.5%)
        escrow_period: u64, // Seconds to hold funds in escrow
        platform_earnings: u256,
        // Counters
        course_counter: u256,
        purchase_counter: u256,
        dispute_counter: u256,
        // Core mappings
        courses: Map<u256, Course>,
        purchases: Map<u256, Purchase>,
        disputes: Map<u256, Dispute>,
        // Relationship mappings
        course_purchases: Map<(ContractAddress, u256), bool>, // buyer -> course_id -> has_purchased
        creator_course_count: Map<ContractAddress, u256>, // creator -> course_count
        creator_courses: Map<(ContractAddress, u256), u256>, // (creator, course_count) -> course_id
        buyer_purchase_count: Map<ContractAddress, u256>, // buyer -> purchase_count
        buyer_purchases: Map<
            (ContractAddress, u256), u256,
        >, // (buyer, purchase_count) -> purchase_id
        purchase_dispute_count: Map<u256, u256>, //purchase_id -> dispute_count
        purchase_disputes: Map<(u256, u256), u256>, // (purchase_id, dispute_count) -> dispute_id
        #[substorage(v0)]
        ownable: OwnableComponent::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        CourseCreated: CourseCreated,
        CourseUpdated: CourseUpdated,
        CourseStatusChanged: CourseStatusChanged,
        CoursePurchased: CoursePurchased,
        CourseCompleted: CourseCompleted,
        EscrowReleased: EscrowReleased,
        DisputeCreated: DisputeCreated,
        DisputeResolved: DisputeResolved,
        PlatformFeeUpdated: PlatformFeeUpdated,
        EscrowPeriodUpdated: EscrowPeriodUpdated,
        PlatformFeesWithdrawn: PlatformFeesWithdrawn,
        #[flat]
        OwnableEvent: OwnableComponent::Event,
    }

    #[derive(Drop, starknet::Event)]
    pub struct CourseCreated {
        #[key]
        pub course_id: u256,
        #[key]
        pub creator: ContractAddress,
        pub title: ByteArray,
        pub price: u256,
        pub royalty_percentage: u16,
    }

    #[derive(Drop, starknet::Event)]
    pub struct CourseUpdated {
        #[key]
        pub course_id: u256,
        pub title: ByteArray,
        pub price: u256,
        pub royalty_percentage: u16,
    }

    #[derive(Drop, starknet::Event)]
    pub struct CourseStatusChanged {
        #[key]
        pub course_id: u256,
        pub is_active: bool,
    }

    #[derive(Drop, starknet::Event)]
    pub struct CoursePurchased {
        #[key]
        pub purchase_id: u256,
        #[key]
        pub course_id: u256,
        #[key]
        pub buyer: ContractAddress,
        pub amount_paid: u256,
        pub escrow_release_time: u64,
    }

    #[derive(Drop, starknet::Event)]
    pub struct CourseCompleted {
        #[key]
        pub purchase_id: u256,
        #[key]
        pub course_id: u256,
        #[key]
        pub buyer: ContractAddress,
    }

    #[derive(Drop, starknet::Event)]
    pub struct EscrowReleased {
        #[key]
        pub purchase_id: u256,
        pub creator_amount: u256,
        pub platform_fee: u256,
    }

    #[derive(Drop, starknet::Event)]
    pub struct DisputeCreated {
        #[key]
        pub dispute_id: u256,
        #[key]
        pub purchase_id: u256,
        #[key]
        pub creator: ContractAddress,
        pub reason: ByteArray,
    }

    #[derive(Drop, starknet::Event)]
    pub struct DisputeResolved {
        #[key]
        pub dispute_id: u256,
        #[key]
        pub purchase_id: u256,
        pub refund_buyer: bool,
        pub resolution: ByteArray,
    }

    #[derive(Drop, starknet::Event)]
    pub struct PlatformFeeUpdated {
        pub old_fee: u16,
        pub new_fee: u16,
    }

    #[derive(Drop, starknet::Event)]
    pub struct EscrowPeriodUpdated {
        pub old_period: u64,
        pub new_period: u64,
    }

    #[derive(Drop, starknet::Event)]
    pub struct PlatformFeesWithdrawn {
        pub amount: u256,
        pub recipient: ContractAddress,
    }

    #[constructor]
    fn constructor(
        ref self: ContractState,
        owner: ContractAddress,
        payment_token: ContractAddress,
        platform_fee_percentage: u16,
        escrow_period: u64,
    ) {
        self.ownable.initializer(owner);
        self.payment_token.write(payment_token);
        self.platform_fee_percentage.write(platform_fee_percentage);
        self.escrow_period.write(escrow_period);
        self.course_counter.write(0);
        self.purchase_counter.write(0);
        self.dispute_counter.write(0);
        self.platform_earnings.write(0);
    }

    #[abi(embed_v0)]
    impl MarketplaceImpl of IMarketplace<ContractState> {
        fn create_course(
            ref self: ContractState,
            title: ByteArray,
            description: ByteArray,
            price: u256,
            royalty_percentage: u16,
        ) -> u256 {
            assert(price > 0, 'Price must be greater than 0');
            assert(royalty_percentage <= 10000, 'Royalty exceeds 100%');

            let caller = get_caller_address();
            let course_id = self.course_counter.read() + 1;
            self.course_counter.write(course_id);

            let course = Course {
                id: course_id,
                creator: caller,
                title: title.clone(),
                description,
                price,
                royalty_percentage,
                is_active: true,
                created_at: get_block_timestamp(),
                total_sales: 0,
            };

            self.courses.write(course_id, course);

            // Add to creator's course list
            let creator_course_count = self.creator_course_count.read(caller);
            self.creator_course_count.write(caller, creator_course_count + 1);
            self.creator_courses.write((caller, creator_course_count + 1), course_id);

            self
                .emit(
                    CourseCreated { course_id, creator: caller, title, price, royalty_percentage },
                );

            course_id
        }

        fn update_course(
            ref self: ContractState,
            course_id: u256,
            title: ByteArray,
            description: ByteArray,
            price: u256,
            royalty_percentage: u16,
        ) {
            let mut course = self.courses.read(course_id);
            assert(course.id != 0, 'Course does not exist');
            assert(course.creator == get_caller_address(), 'Not course creator');
            assert(price > 0, 'Price must be greater than 0');
            assert(royalty_percentage <= 10000, 'Royalty exceeds 100%');

            course.title = title.clone();
            course.description = description;
            course.price = price;
            course.royalty_percentage = royalty_percentage;

            self.courses.write(course_id, course);

            self.emit(CourseUpdated { course_id, title, price, royalty_percentage });
        }

        fn deactivate_course(ref self: ContractState, course_id: u256) {
            let mut course = self.courses.read(course_id);
            assert(course.id != 0, 'Course does not exist');
            assert(course.creator == get_caller_address(), 'Not course creator');

            course.is_active = false;
            self.courses.write(course_id, course);

            self.emit(CourseStatusChanged { course_id, is_active: false });
        }

        fn activate_course(ref self: ContractState, course_id: u256) {
            let mut course = self.courses.read(course_id);
            assert(course.id != 0, 'Course does not exist');
            assert(course.creator == get_caller_address(), 'Not course creator');

            course.is_active = true;
            self.courses.write(course_id, course);

            self.emit(CourseStatusChanged { course_id, is_active: true });
        }

        fn purchase_course(ref self: ContractState, course_id: u256) {
            let course = self.courses.read(course_id);
            assert(course.id != 0, 'Course does not exist');
            assert(course.is_active, 'Course is not active');

            let buyer = get_caller_address();
            assert(course.creator != buyer, 'Cannot buy own course');
            assert(!self.has_purchased_course(buyer, course_id), 'Already purchased');

            let purchase_id = self.purchase_counter.read() + 1;
            self.purchase_counter.write(purchase_id);

            let escrow_release_time = get_block_timestamp() + self.escrow_period.read();

            let purchase = Purchase {
                id: purchase_id,
                course_id,
                buyer,
                amount_paid: course.price,
                purchase_time: get_block_timestamp(),
                is_completed: false,
                in_dispute: false,
                escrow_release_time,
            };

            self.purchases.write(purchase_id, purchase);

            // Update mappings
            self.course_purchases.write((buyer, course_id), true);
            let buyer_purchase_count = self.buyer_purchase_count.read(buyer);
            self.buyer_purchase_count.write(buyer, buyer_purchase_count + 1);
            self.buyer_purchases.write((buyer, buyer_purchase_count + 1), purchase_id);

            // Transfer payment from buyer to contract
            let token = IERC20Dispatcher { contract_address: self.payment_token.read() };
            let success = token.transfer_from(buyer, get_contract_address(), course.price);
            assert(success, 'Payment transfer failed');

            // Update course sales
            let mut updated_course = course.clone();
            updated_course.total_sales += course.price;
            self.courses.write(course_id, updated_course);

            self
                .emit(
                    CoursePurchased {
                        purchase_id,
                        course_id,
                        buyer,
                        amount_paid: course.price,
                        escrow_release_time,
                    },
                );
        }

        fn complete_course(ref self: ContractState, purchase_id: u256) {
            let mut purchase = self.purchases.read(purchase_id);
            assert(purchase.id != 0, 'Purchase does not exist');
            assert(purchase.buyer == get_caller_address(), 'Not the buyer');
            assert(!purchase.is_completed, 'Course already completed');
            assert(!purchase.in_dispute, 'Purchase in dispute');

            purchase.is_completed = true;
            self.purchases.write(purchase_id, purchase.clone());

            // Automatically release escrow when course is completed
            self._release_escrow_internal(purchase_id.clone());

            self
                .emit(
                    CourseCompleted {
                        purchase_id, course_id: purchase.course_id, buyer: purchase.buyer,
                    },
                );
        }

        fn release_escrow(ref self: ContractState, purchase_id: u256) {
            let purchase = self.purchases.read(purchase_id);
            assert(purchase.id != 0, 'Purchase does not exist');
            assert(!purchase.in_dispute, 'Purchase in dispute');

            let caller = get_caller_address();
            let course = self.courses.read(purchase.course_id);

            // Either the buyer completed the course, or escrow timeout passed, or creator releases
            // early
            assert(
                purchase.is_completed
                    || get_block_timestamp() >= purchase.escrow_release_time
                    || caller == course.creator,
                'Cannot release escrow yet',
            );

            self._release_escrow_internal(purchase_id);
        }

        fn claim_escrow_after_timeout(ref self: ContractState, purchase_id: u256) {
            let purchase = self.purchases.read(purchase_id);
            assert(purchase.id != 0, 'Purchase does not exist');
            assert(
                get_block_timestamp() >= purchase.escrow_release_time, 'Escrow period not ended',
            );
            assert(!purchase.in_dispute, 'Purchase in dispute');

            self._release_escrow_internal(purchase_id);
        }

        fn create_dispute(ref self: ContractState, purchase_id: u256, reason: ByteArray) -> u256 {
            let mut purchase = self.purchases.read(purchase_id);
            assert(purchase.id != 0, 'Purchase does not exist');
            assert(purchase.buyer == get_caller_address(), 'Not the buyer');
            assert(!purchase.is_completed, 'Course already completed');
            assert(!purchase.in_dispute, 'Already in dispute');

            purchase.in_dispute = true;
            self.purchases.write(purchase_id, purchase.clone());

            let dispute_id = self.dispute_counter.read() + 1;
            self.dispute_counter.write(dispute_id);

            let dispute = Dispute {
                id: dispute_id,
                purchase_id,
                reason: reason.clone(),
                created_at: get_block_timestamp(),
                resolved: false,
                resolution: "",
            };

            self.disputes.write(dispute_id, dispute);

            // Add to purchase disputes list
            let purchase_dispute_count = self.purchase_dispute_count.read(purchase_id);
            self.purchase_dispute_count.write(purchase_id, purchase_dispute_count + 1);
            self.purchase_disputes.write((purchase_id, purchase_dispute_count + 1), dispute_id);

            let course = self.courses.read(purchase.course_id);

            self.emit(DisputeCreated { dispute_id, purchase_id, creator: course.creator, reason });

            dispute_id
        }

        fn resolve_dispute(
            ref self: ContractState, dispute_id: u256, refund_buyer: bool, resolution: ByteArray,
        ) {
            self.ownable.assert_only_owner();

            let mut dispute = self.disputes.read(dispute_id);
            assert(dispute.id != 0, 'Dispute does not exist');
            assert(!dispute.resolved, 'Dispute already resolved');

            dispute.resolved = true;
            dispute.resolution = resolution.clone();
            self.disputes.write(dispute_id, dispute.clone());

            let mut purchase = self.purchases.read(dispute.purchase_id);
            purchase.in_dispute = false;
            self.purchases.write(dispute.purchase_id, purchase.clone());

            if refund_buyer {
                // Refund the buyer
                let token = IERC20Dispatcher { contract_address: self.payment_token.read() };
                let success = token.transfer(purchase.buyer, purchase.amount_paid);
                assert(success, 'Refund transfer failed');
            } else {
                // Release to creator
                self._release_escrow_internal(dispute.purchase_id);
            }

            self
                .emit(
                    DisputeResolved {
                        dispute_id, purchase_id: dispute.purchase_id, refund_buyer, resolution,
                    },
                );
        }

        fn set_platform_fee(ref self: ContractState, fee_percentage: u16) {
            self.ownable.assert_only_owner();
            assert(fee_percentage <= 1000, 'Fee exceeds 10%'); // Max 10% platform fee

            let old_fee = self.platform_fee_percentage.read();
            self.platform_fee_percentage.write(fee_percentage);

            self.emit(PlatformFeeUpdated { old_fee, new_fee: fee_percentage });
        }

        fn set_escrow_period(ref self: ContractState, period_seconds: u64) {
            self.ownable.assert_only_owner();
            assert(period_seconds >= 86400, 'Period too short'); // Minimum 1 day
            assert(period_seconds <= 2592000, 'Period too long'); // Maximum 30 days

            let old_period = self.escrow_period.read();
            self.escrow_period.write(period_seconds);

            self.emit(EscrowPeriodUpdated { old_period, new_period: period_seconds });
        }

        fn withdraw_platform_fees(ref self: ContractState) {
            self.ownable.assert_only_owner();

            let amount = self.platform_earnings.read();
            assert(amount > 0, 'No fees to withdraw');

            self.platform_earnings.write(0);

            let owner = self.ownable.owner();
            let token = IERC20Dispatcher { contract_address: self.payment_token.read() };
            let success = token.transfer(owner, amount);
            assert(success, 'Fee withdrawal failed');

            self.emit(PlatformFeesWithdrawn { amount, recipient: owner });
        }

        // View Functions
        fn get_course(self: @ContractState, course_id: u256) -> Course {
            self.courses.read(course_id)
        }

        fn get_purchase(self: @ContractState, purchase_id: u256) -> Purchase {
            self.purchases.read(purchase_id)
        }

        fn get_dispute(self: @ContractState, dispute_id: u256) -> Dispute {
            self.disputes.read(dispute_id)
        }

        fn get_platform_fee(self: @ContractState) -> u16 {
            self.platform_fee_percentage.read()
        }

        fn get_escrow_period(self: @ContractState) -> u64 {
            self.escrow_period.read()
        }

        fn get_courses_by_creator(self: @ContractState, creator: ContractAddress) -> Array<u256> {
            let creator_course_count = self.creator_course_count.read(creator);
            let mut courses = ArrayTrait::new();
            for i in 0..creator_course_count {
                courses.append(self.creator_courses.read((creator, i)));
            };
            courses
        }

        fn get_purchases_by_buyer(self: @ContractState, buyer: ContractAddress) -> Array<u256> {
            let buyer_purchase_count = self.buyer_purchase_count.read(buyer);

            let mut purchases = ArrayTrait::new();
            for i in 0..buyer_purchase_count {
                purchases.append(self.buyer_purchases.read((buyer, i)));
            };

            purchases
        }

        fn has_purchased_course(
            self: @ContractState, buyer: ContractAddress, course_id: u256,
        ) -> bool {
            self.course_purchases.read((buyer, course_id))
        }

        fn get_platform_earnings(self: @ContractState) -> u256 {
            self.platform_earnings.read()
        }

        fn get_course_count(self: @ContractState) -> u256 {
            self.course_counter.read()
        }

        fn get_purchase_count(self: @ContractState) -> u256 {
            self.purchase_counter.read()
        }
    }

    #[generate_trait]
    impl InternalImpl of InternalTrait {
        fn _release_escrow_internal(ref self: ContractState, purchase_id: u256) {
            let purchase = self.purchases.read(purchase_id);
            let course = self.courses.read(purchase.course_id);

            // Calculate platform fee
            let platform_fee = (purchase.amount_paid * self.platform_fee_percentage.read().into())
                / 10000;
            let creator_amount = purchase.amount_paid - platform_fee;

            // Update platform earnings
            self.platform_earnings.write(self.platform_earnings.read() + platform_fee);

            // Transfer to creator
            let token = IERC20Dispatcher { contract_address: self.payment_token.read() };
            let success = token.transfer(course.creator, creator_amount);
            assert(success, 'Creator payment failed');

            self.emit(EscrowReleased { purchase_id, creator_amount, platform_fee });
        }
    }
}
