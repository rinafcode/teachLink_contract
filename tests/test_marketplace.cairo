#[cfg(test)]
mod tests {
    use core::result::ResultTrait;
    use core::option::OptionTrait;
    use core::traits::TryInto;
    use starknet::{ContractAddress, get_contract_address};
    use snforge_std::{
        declare, ContractClassTrait, DeclareResultTrait, start_cheat_caller_address,
        stop_cheat_caller_address, start_cheat_block_timestamp, stop_cheat_block_timestamp,
    };

    use teachlink::interfaces::IMarketplace::{IMarketplaceDispatcher, IMarketplaceDispatcherTrait};
    use teachlink::interfaces::itoken::{ITokenDispatcher, ITokenDispatcherTrait};
    use openzeppelin::token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherTrait};

    // Constants
    const INITIAL_TOKEN_SUPPLY: u256 = 1000000_000000000000000000; // 1M tokens with 18 decimals
    const COURSE_PRICE: u256 = 1000_000000000000000000; // 1K tokens
    const PLATFORM_FEE_PERCENTAGE: u16 = 250; // 2.5%
    const ESCROW_PERIOD: u64 = 604800; // 7 days

    // Address constants
    const OWNER: felt252 = 'OWNER';
    const CREATOR: felt252 = 'CREATOR';
    const BUYER: felt252 = 'BUYER';
    const BUYER_TWO: felt252 = 'BUYER_TWO';

    fn __setup__() -> (IMarketplaceDispatcher, ITokenDispatcher) {
        // Deploy token contract first
        let token_contract = declare("TeachLinkToken").unwrap().contract_class();

        let mut token_constructor_calldata: Array<felt252> = array![];
        let token_name: ByteArray = "TeachLink Token";
        let token_symbol: ByteArray = "TLT";

        token_name.serialize(ref token_constructor_calldata);
        token_symbol.serialize(ref token_constructor_calldata);
        INITIAL_TOKEN_SUPPLY.serialize(ref token_constructor_calldata);

        start_cheat_caller_address(get_contract_address(), OWNER.try_into().unwrap());
        let (token_address, _) = token_contract.deploy(@token_constructor_calldata).unwrap();
        stop_cheat_caller_address(get_contract_address());

        // Deploy marketplace contract
        let marketplace_contract = declare("MarketplaceContract").unwrap().contract_class();

        let mut marketplace_constructor_args: Array<felt252> = array![];
        let owner_address: ContractAddress = OWNER.try_into().unwrap();
        marketplace_constructor_args.append(owner_address.into());
        marketplace_constructor_args.append(token_address.into());
        marketplace_constructor_args.append(PLATFORM_FEE_PERCENTAGE.into());
        marketplace_constructor_args.append(ESCROW_PERIOD.into());

        start_cheat_caller_address(get_contract_address(), OWNER.try_into().unwrap());
        let (marketplace_address, _) = marketplace_contract
            .deploy(@marketplace_constructor_args)
            .unwrap();
        stop_cheat_caller_address(get_contract_address());

        let marketplace_dispatcher = IMarketplaceDispatcher {
            contract_address: marketplace_address,
        };
        let token_dispatcher = ITokenDispatcher { contract_address: token_address };

        // Setup initial token balances and approvals
        setup_token_balances(token_dispatcher, marketplace_address);

        (marketplace_dispatcher, token_dispatcher)
    }

    fn setup_token_balances(
        token_dispatcher: ITokenDispatcher, marketplace_address: ContractAddress,
    ) {
        let erc20_dispatcher = IERC20Dispatcher {
            contract_address: token_dispatcher.contract_address,
        };

        // Mint tokens to test users
        start_cheat_caller_address(token_dispatcher.contract_address, OWNER.try_into().unwrap());
        token_dispatcher.mint(CREATOR.try_into().unwrap(), 10000_000000000000000000); // 10K tokens
        token_dispatcher.mint(BUYER.try_into().unwrap(), 10000_000000000000000000); // 10K tokens
        token_dispatcher
            .mint(BUYER_TWO.try_into().unwrap(), 10000_000000000000000000); // 10K tokens
        stop_cheat_caller_address(token_dispatcher.contract_address);

        // Approve marketplace to spend tokens
        start_cheat_caller_address(token_dispatcher.contract_address, CREATOR.try_into().unwrap());
        erc20_dispatcher.approve(marketplace_address, 10000_000000000000000000);
        stop_cheat_caller_address(token_dispatcher.contract_address);

        start_cheat_caller_address(token_dispatcher.contract_address, BUYER.try_into().unwrap());
        erc20_dispatcher.approve(marketplace_address, 10000_000000000000000000);
        stop_cheat_caller_address(token_dispatcher.contract_address);

        start_cheat_caller_address(
            token_dispatcher.contract_address, BUYER_TWO.try_into().unwrap(),
        );
        erc20_dispatcher.approve(marketplace_address, 10000_000000000000000000);
        stop_cheat_caller_address(token_dispatcher.contract_address);
    }

    // *************************************************************************
    //                              COURSE CREATION TESTS
    // *************************************************************************

    #[test]
    fn test_course_creation_with_full_metadata() {
        let (marketplace, _token) = __setup__();

        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());

        let title = "Advanced Cairo Programming";
        let description =
            "Master advanced Cairo concepts including components, storage, and optimization techniques";
        let price = 5000_000000000000000000; // 5K tokens
        let royalty = 750_u16; // 7.5%

        let course_id = marketplace
            .create_course(title.clone(), description.clone(), price, royalty);

        stop_cheat_caller_address(marketplace.contract_address);

        assert(course_id == 1, 'Invalid course ID');

        let course = marketplace.get_course(course_id);
        assert(course.creator == CREATOR.try_into().unwrap(), 'Invalid creator');
        assert(course.title == title, 'Invalid title');
        assert(course.description == description, 'Invalid description');
        assert(course.price == price, 'Invalid price');
        assert(course.royalty_percentage == royalty, 'Invalid royalty');
        assert(course.is_active, 'Course should be active');
        assert(course.total_sales == 0, 'Initial sales should be 0');
        assert(course.created_at > 0, 'Created timestamp should be set');

        let creator_courses = marketplace.get_courses_by_creator(CREATOR.try_into().unwrap());
        assert(creator_courses.len() == 1, 'Creator should have 1 course');
        assert(*creator_courses.at(0) == course_id, 'Invalid course in creator list');
    }

    #[test]
    fn test_course_update() {
        let (marketplace, _token) = __setup__();

        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());

        let course_id = marketplace
            .create_course(
                "Introduction to Cairo",
                "Learn Cairo programming from scratch",
                COURSE_PRICE,
                500_u16,
            );

        marketplace
            .update_course(
                course_id,
                "Advanced Cairo Programming",
                "Master advanced Cairo concepts",
                2000_000000000000000000, // 2K tokens
                750_u16 // 7.5% royalty
            );

        stop_cheat_caller_address(marketplace.contract_address);

        let course = marketplace.get_course(course_id);
        assert(course.price == 2000_000000000000000000, 'Price not updated');
        assert(course.royalty_percentage == 750, 'Royalty not updated');
    }

    #[test]
    fn test_course_activation_deactivation() {
        let (marketplace, _token) = __setup__();

        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());
        let course_id = marketplace
            .create_course(
                "Introduction to Cairo",
                "Learn Cairo programming from scratch",
                COURSE_PRICE,
                500_u16,
            );

        // Deactivate course
        marketplace.deactivate_course(course_id);
        let course = marketplace.get_course(course_id);
        assert(!course.is_active, 'Course should be deactivated');

        // Reactivate course
        marketplace.activate_course(course_id);
        let course = marketplace.get_course(course_id);
        assert(course.is_active, 'Course should be activated');

        stop_cheat_caller_address(marketplace.contract_address);
    }

    // *************************************************************************
    //                              COURSE PURCHASE TESTS
    // *************************************************************************

    #[test]
    fn test_course_purchase_and_fund_transfer() {
        let (marketplace, token) = __setup__();
        let erc20_dispatcher = IERC20Dispatcher { contract_address: token.contract_address };

        // Create course
        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());
        let course_price = 2000_000000000000000000; // 2K tokens
        let course_id = marketplace
            .create_course(
                "Cairo Fundamentals",
                "Learn the basics of Cairo programming",
                course_price,
                500_u16 // 5% royalty
            );
        stop_cheat_caller_address(marketplace.contract_address);

        // Check initial balances
        let buyer_balance_before = erc20_dispatcher.balance_of(BUYER.try_into().unwrap());
        let marketplace_balance_before = erc20_dispatcher.balance_of(marketplace.contract_address);

        // Purchase course
        start_cheat_caller_address(marketplace.contract_address, BUYER.try_into().unwrap());
        start_cheat_block_timestamp(marketplace.contract_address, 1000);

        marketplace.purchase_course(course_id);

        stop_cheat_block_timestamp(marketplace.contract_address);
        stop_cheat_caller_address(marketplace.contract_address);

        // Verify balances changed
        let buyer_balance_after = erc20_dispatcher.balance_of(BUYER.try_into().unwrap());
        let marketplace_balance_after = erc20_dispatcher.balance_of(marketplace.contract_address);

        assert(
            buyer_balance_after == buyer_balance_before - course_price, 'Buyer balance incorrect',
        );
        assert(
            marketplace_balance_after == marketplace_balance_before + course_price,
            'Marketplace balance incorrect',
        );

        // Verify purchase was created
        let purchase = marketplace.get_purchase(1);
        assert(purchase.id == 1, 'Invalid purchase ID');
        assert(purchase.course_id == course_id, 'Invalid course ID in purchase');
        assert(purchase.buyer == BUYER.try_into().unwrap(), 'Invalid buyer');
        assert(purchase.amount_paid == course_price, 'Invalid amount paid');
        assert(purchase.purchase_time == 1000, 'Invalid purchase time');
        assert(!purchase.is_completed, 'Purchase not completed');
        assert(!purchase.in_dispute, 'Purchase not in dispute');
        assert(purchase.escrow_release_time == 1000 + ESCROW_PERIOD, 'Invalid escrow release time');

        // Verify buyer has purchased the course
        assert(
            marketplace.has_purchased_course(BUYER.try_into().unwrap(), course_id),
            'Buyer should have purchased',
        );

        // Verify course sales were updated
        let updated_course = marketplace.get_course(course_id);
        assert(updated_course.total_sales == course_price, 'Course sales not updated');
    }

    #[test]
    fn test_multiple_purchases_different_buyers() {
        let (marketplace, token) = __setup__();
        let erc20_dispatcher = IERC20Dispatcher { contract_address: token.contract_address };

        // Create course
        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());
        let course_price = COURSE_PRICE;
        let course_id = marketplace
            .create_course(
                "Introduction to Blockchain",
                "Basic blockchain concepts and Cairo",
                course_price,
                200_u16 // 2% royalty
            );
        stop_cheat_caller_address(marketplace.contract_address);

        // First buyer purchases
        start_cheat_caller_address(marketplace.contract_address, BUYER.try_into().unwrap());
        marketplace.purchase_course(course_id);
        stop_cheat_caller_address(marketplace.contract_address);

        // Second buyer purchases
        start_cheat_caller_address(marketplace.contract_address, BUYER_TWO.try_into().unwrap());
        marketplace.purchase_course(course_id);
        stop_cheat_caller_address(marketplace.contract_address);

        // Verify both purchases
        assert(
            marketplace.has_purchased_course(BUYER.try_into().unwrap(), course_id),
            'First buyer purchased',
        );
        assert(
            marketplace.has_purchased_course(BUYER_TWO.try_into().unwrap(), course_id),
            'Second buyer purchased',
        );

        // Verify course total sales
        let course = marketplace.get_course(course_id);
        assert(course.total_sales == course_price * 2, 'Total sales 2x price');

        // Verify purchase counts
        assert(marketplace.get_purchase_count() == 2, 'Should have 2 total purchases');

        // Verify marketplace balance
        let marketplace_balance = erc20_dispatcher.balance_of(marketplace.contract_address);
        assert(marketplace_balance == course_price * 2, 'Marketplace balance incorrect');
    }

    // *************************************************************************
    //                              COURSE COMPLETION TESTS
    // *************************************************************************

    #[test]
    fn test_course_completion_and_escrow_release() {
        let (marketplace, token) = __setup__();
        let erc20_dispatcher = IERC20Dispatcher { contract_address: token.contract_address };

        // Create and purchase course
        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());
        let course_price = 3000_000000000000000000; // 3K tokens
        let course_id = marketplace
            .create_course(
                "Advanced Starknet",
                "Deep dive into Starknet development",
                course_price,
                600_u16 // 6% royalty
            );
        stop_cheat_caller_address(marketplace.contract_address);

        start_cheat_caller_address(marketplace.contract_address, BUYER.try_into().unwrap());
        marketplace.purchase_course(course_id);
        stop_cheat_caller_address(marketplace.contract_address);

        let creator_balance_before = erc20_dispatcher.balance_of(CREATOR.try_into().unwrap());
        let marketplace_balance_before = erc20_dispatcher.balance_of(marketplace.contract_address);

        // Complete the course
        start_cheat_caller_address(marketplace.contract_address, BUYER.try_into().unwrap());
        marketplace.complete_course(1);
        stop_cheat_caller_address(marketplace.contract_address);

        // Verify course completion
        let purchase = marketplace.get_purchase(1);
        assert(purchase.is_completed, 'Course should be completed');

        // Verify fund transfers
        let creator_balance_after = erc20_dispatcher.balance_of(CREATOR.try_into().unwrap());
        let marketplace_balance_after = erc20_dispatcher.balance_of(marketplace.contract_address);

        let expected_platform_fee = (course_price * PLATFORM_FEE_PERCENTAGE.into()) / 10000;
        let expected_creator_payment = course_price - expected_platform_fee;

        assert(
            creator_balance_after == creator_balance_before + expected_creator_payment,
            'Creator balance incorrect',
        );
        assert(
            marketplace_balance_after == marketplace_balance_before - course_price,
            'Marketplace balance incorrect',
        );

        // Verify platform earnings
        let platform_earnings = marketplace.get_platform_earnings();
        assert(platform_earnings == expected_platform_fee, 'Invalid platform earnings');
    }

    #[test]
    fn test_escrow_release_after_timeout() {
        let (marketplace, token) = __setup__();
        let erc20_dispatcher = IERC20Dispatcher { contract_address: token.contract_address };

        // Create and purchase course
        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());
        let course_price = 1500_000000000000000000; // 1.5K tokens
        let course_id = marketplace
            .create_course(
                "Smart Contract Security",
                "Learn to write secure smart contracts",
                course_price,
                400_u16 // 4% royalty
            );
        stop_cheat_caller_address(marketplace.contract_address);

        start_cheat_caller_address(marketplace.contract_address, BUYER.try_into().unwrap());
        start_cheat_block_timestamp(marketplace.contract_address, 1000);
        marketplace.purchase_course(course_id);
        stop_cheat_block_timestamp(marketplace.contract_address);
        stop_cheat_caller_address(marketplace.contract_address);

        let creator_balance_before = erc20_dispatcher.balance_of(CREATOR.try_into().unwrap());

        // Fast forward past escrow period
        start_cheat_block_timestamp(marketplace.contract_address, 1000 + ESCROW_PERIOD + 1);

        // Claim escrow after timeout
        marketplace.claim_escrow_after_timeout(1);

        stop_cheat_block_timestamp(marketplace.contract_address);

        // Verify fund transfers
        let creator_balance_after = erc20_dispatcher.balance_of(CREATOR.try_into().unwrap());
        let expected_platform_fee = (course_price * PLATFORM_FEE_PERCENTAGE.into()) / 10000;
        let expected_creator_payment = course_price - expected_platform_fee;

        assert(
            creator_balance_after == creator_balance_before + expected_creator_payment,
            'Creator balance incorrect',
        );

        // Verify platform earnings
        let platform_earnings = marketplace.get_platform_earnings();
        assert(platform_earnings == expected_platform_fee, 'Invalid platform earnings');
    }

    #[test]
    fn test_creator_early_escrow_release() {
        let (marketplace, token) = __setup__();
        let erc20_dispatcher = IERC20Dispatcher { contract_address: token.contract_address };

        // Create and purchase course
        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());
        let course_price = 2500_000000000000000000; // 2.5K tokens
        let course_id = marketplace
            .create_course(
                "DeFi Development",
                "Build decentralized finance applications",
                course_price,
                300_u16 // 3% royalty
            );
        stop_cheat_caller_address(marketplace.contract_address);

        start_cheat_caller_address(marketplace.contract_address, BUYER.try_into().unwrap());
        start_cheat_block_timestamp(marketplace.contract_address, 1000);
        marketplace.purchase_course(course_id);
        stop_cheat_block_timestamp(marketplace.contract_address);
        stop_cheat_caller_address(marketplace.contract_address);

        let creator_balance_before = erc20_dispatcher.balance_of(CREATOR.try_into().unwrap());

        // Creator releases escrow early
        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());
        start_cheat_block_timestamp(marketplace.contract_address, 1000 + 100000); // Before timeout
        marketplace.release_escrow(1);
        stop_cheat_block_timestamp(marketplace.contract_address);
        stop_cheat_caller_address(marketplace.contract_address);

        // Verify fund transfers
        let creator_balance_after = erc20_dispatcher.balance_of(CREATOR.try_into().unwrap());
        let expected_platform_fee = (course_price * PLATFORM_FEE_PERCENTAGE.into()) / 10000;
        let expected_creator_payment = course_price - expected_platform_fee;

        assert(
            creator_balance_after == creator_balance_before + expected_creator_payment,
            'Creator balance incorrect',
        );

        // Verify platform earnings
        let platform_earnings = marketplace.get_platform_earnings();
        assert(platform_earnings == expected_platform_fee, 'Invalid platform earnings');
    }

    // *************************************************************************
    //                              DISPUTE TESTS
    // *************************************************************************

    #[test]
    fn test_dispute_creation_and_resolution_refund() {
        let (marketplace, token) = __setup__();
        let erc20_dispatcher = IERC20Dispatcher { contract_address: token.contract_address };

        // Create and purchase course
        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());
        let course_price = 4000_000000000000000000; // 4K tokens
        let course_id = marketplace
            .create_course(
                "Advanced Cairo Patterns",
                "Master advanced Cairo programming patterns",
                course_price,
                800_u16 // 8% royalty
            );
        stop_cheat_caller_address(marketplace.contract_address);

        start_cheat_caller_address(marketplace.contract_address, BUYER.try_into().unwrap());
        start_cheat_block_timestamp(marketplace.contract_address, 1000);
        marketplace.purchase_course(course_id);
        stop_cheat_block_timestamp(marketplace.contract_address);
        stop_cheat_caller_address(marketplace.contract_address);

        // Create dispute
        let dispute_reason = "Course content does not match description";
        start_cheat_caller_address(marketplace.contract_address, BUYER.try_into().unwrap());
        let dispute_id = marketplace.create_dispute(1, dispute_reason.clone());
        stop_cheat_caller_address(marketplace.contract_address);

        // Verify dispute creation
        assert(dispute_id == 1, 'Invalid dispute ID');
        let dispute = marketplace.get_dispute(dispute_id);
        assert(dispute.id == dispute_id, 'Invalid dispute ID in struct');
        assert(dispute.purchase_id == 1, 'Invalid purchase ID in dispute');
        assert(dispute.reason == dispute_reason, 'Invalid dispute reason');
        assert(!dispute.resolved, 'Dispute not resolved initially');
        assert(dispute.created_at == 1000, 'Invalid dispute creation time');

        // Verify purchase is marked as in dispute
        let purchase = marketplace.get_purchase(1);
        assert(purchase.in_dispute, 'Purchase should be in dispute');

        let buyer_balance_before = erc20_dispatcher.balance_of(BUYER.try_into().unwrap());

        // Resolve dispute with refund to buyer
        start_cheat_caller_address(marketplace.contract_address, OWNER.try_into().unwrap());
        let resolution = "Refunding buyer due to misleading course description";
        marketplace.resolve_dispute(dispute_id, true, resolution.clone());
        stop_cheat_caller_address(marketplace.contract_address);

        // Verify dispute resolution
        let resolved_dispute = marketplace.get_dispute(dispute_id);
        assert(resolved_dispute.resolved, 'Dispute should be resolved');
        assert(resolved_dispute.resolution == resolution, 'Invalid dispute resolution');

        // Verify purchase is no longer in dispute
        let updated_purchase = marketplace.get_purchase(1);
        assert(!updated_purchase.in_dispute, 'Purchase not in dispute');

        // Verify refund was processed
        let buyer_balance_after = erc20_dispatcher.balance_of(BUYER.try_into().unwrap());
        assert(
            buyer_balance_after == buyer_balance_before + course_price,
            'Buyer should receive refund',
        );

        // Platform earnings should be 0 since refund was given
        let platform_earnings = marketplace.get_platform_earnings();
        assert(platform_earnings == 0, 'Platform earnings should be 0');
    }

    #[test]
    fn test_dispute_creation_and_resolution_no_refund() {
        let (marketplace, token) = __setup__();
        let erc20_dispatcher = IERC20Dispatcher { contract_address: token.contract_address };

        // Create and purchase course
        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());
        let course_price = 3500_000000000000000000; // 3.5K tokens
        let course_id = marketplace
            .create_course(
                "Cairo Testing Strategies",
                "Comprehensive guide to testing Cairo contracts",
                course_price,
                450_u16 // 4.5% royalty
            );
        stop_cheat_caller_address(marketplace.contract_address);

        start_cheat_caller_address(marketplace.contract_address, BUYER.try_into().unwrap());
        marketplace.purchase_course(course_id);
        stop_cheat_caller_address(marketplace.contract_address);

        // Create dispute
        let dispute_reason = "Difficulty level too high";
        start_cheat_caller_address(marketplace.contract_address, BUYER.try_into().unwrap());
        let dispute_id = marketplace.create_dispute(1, dispute_reason.clone());
        stop_cheat_caller_address(marketplace.contract_address);

        let creator_balance_before = erc20_dispatcher.balance_of(CREATOR.try_into().unwrap());

        // Resolve dispute without refund (creator keeps payment)
        start_cheat_caller_address(marketplace.contract_address, OWNER.try_into().unwrap());
        let resolution = "Dispute resolved in favor of creator - course content as advertised";
        marketplace.resolve_dispute(dispute_id, false, resolution.clone());
        stop_cheat_caller_address(marketplace.contract_address);

        // Verify dispute resolution
        let resolved_dispute = marketplace.get_dispute(dispute_id);
        assert(resolved_dispute.resolved, 'Dispute should be resolved');
        assert(resolved_dispute.resolution == resolution, 'Invalid dispute resolution');

        // Verify creator received payment
        let creator_balance_after = erc20_dispatcher.balance_of(CREATOR.try_into().unwrap());
        let expected_platform_fee = (course_price * PLATFORM_FEE_PERCENTAGE.into()) / 10000;
        let expected_creator_payment = course_price - expected_platform_fee;

        assert(
            creator_balance_after == creator_balance_before + expected_creator_payment,
            'Creator should receive payment',
        );

        // Verify platform earnings
        let platform_earnings = marketplace.get_platform_earnings();
        assert(platform_earnings == expected_platform_fee, 'Invalid platform earnings');
    }

    // *************************************************************************
    //                              PLATFORM MANAGEMENT TESTS
    // *************************************************************************

    #[test]
    fn test_platform_fee_withdrawal() {
        let (marketplace, token) = __setup__();
        let erc20_dispatcher = IERC20Dispatcher { contract_address: token.contract_address };

        // Create and purchase course to generate platform fees
        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());
        let course_price = 10000_000000000000000000; // 10K tokens
        let course_id = marketplace
            .create_course(
                "Expert Cairo Development",
                "Professional-level Cairo programming course",
                course_price,
                1000_u16 // 10% royalty
            );
        stop_cheat_caller_address(marketplace.contract_address);

        start_cheat_caller_address(marketplace.contract_address, BUYER.try_into().unwrap());
        marketplace.purchase_course(course_id);
        marketplace.complete_course(1);
        stop_cheat_caller_address(marketplace.contract_address);

        // Calculate expected platform fee
        let expected_platform_fee = (course_price * PLATFORM_FEE_PERCENTAGE.into()) / 10000;
        let platform_earnings = marketplace.get_platform_earnings();
        assert(platform_earnings == expected_platform_fee, 'Invalid platform earnings');

        let owner_balance_before = erc20_dispatcher.balance_of(OWNER.try_into().unwrap());

        // Withdraw platform fees
        start_cheat_caller_address(marketplace.contract_address, OWNER.try_into().unwrap());
        marketplace.withdraw_platform_fees();
        stop_cheat_caller_address(marketplace.contract_address);

        // Verify platform earnings are reset to 0
        let platform_earnings_after = marketplace.get_platform_earnings();
        assert(platform_earnings_after == 0, 'Platform earnings reset to 0');

        // Verify owner received the fees
        let owner_balance_after = erc20_dispatcher.balance_of(OWNER.try_into().unwrap());
        assert(
            owner_balance_after == owner_balance_before + expected_platform_fee,
            'Owner should receive fees',
        );
    }

    #[test]
    fn test_platform_fee_management() {
        let (marketplace, _token) = __setup__();

        start_cheat_caller_address(marketplace.contract_address, OWNER.try_into().unwrap());

        // Check initial fee
        let initial_fee = marketplace.get_platform_fee();
        assert(initial_fee == PLATFORM_FEE_PERCENTAGE, 'Invalid initial fee');

        // Update platform fee
        marketplace.set_platform_fee(300_u16); // 3%
        let updated_fee = marketplace.get_platform_fee();
        assert(updated_fee == 300, 'Fee not updated');

        stop_cheat_caller_address(marketplace.contract_address);
    }

    #[test]
    fn test_escrow_period_management() {
        let (marketplace, _token) = __setup__();

        start_cheat_caller_address(marketplace.contract_address, OWNER.try_into().unwrap());

        // Check initial period
        let initial_period = marketplace.get_escrow_period();
        assert(initial_period == ESCROW_PERIOD, 'Invalid initial period');

        // Update escrow period
        marketplace.set_escrow_period(1209600_u64); // 14 days
        let updated_period = marketplace.get_escrow_period();
        assert(updated_period == 1209600, 'Period not updated');

        stop_cheat_caller_address(marketplace.contract_address);
    }

    // *************************************************************************
    //                              NEGATIVE TESTS
    // *************************************************************************

    #[test]
    #[should_panic(expected: ('Cannot buy own course',))]
    fn test_creator_cannot_buy_own_course() {
        let (marketplace, _token) = __setup__();

        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());
        let course_id = marketplace
            .create_course("My Course", "Course description", COURSE_PRICE, 500_u16);

        // Try to purchase own course - should fail
        marketplace.purchase_course(course_id);
        stop_cheat_caller_address(marketplace.contract_address);
    }

    #[test]
    #[should_panic(expected: ('Already purchased',))]
    fn test_cannot_purchase_same_course_twice() {
        let (marketplace, _token) = __setup__();

        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());
        let course_id = marketplace
            .create_course("Unique Course", "Course description", COURSE_PRICE, 500_u16);
        stop_cheat_caller_address(marketplace.contract_address);

        start_cheat_caller_address(marketplace.contract_address, BUYER.try_into().unwrap());
        marketplace.purchase_course(course_id);

        // Try to purchase again - should fail
        marketplace.purchase_course(course_id);
        stop_cheat_caller_address(marketplace.contract_address);
    }

    #[test]
    #[should_panic(expected: ('Course is not active',))]
    fn test_cannot_purchase_inactive_course() {
        let (marketplace, _token) = __setup__();

        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());
        let course_id = marketplace
            .create_course("Inactive Course", "Course description", COURSE_PRICE, 500_u16);

        // Deactivate course
        marketplace.deactivate_course(course_id);
        stop_cheat_caller_address(marketplace.contract_address);

        start_cheat_caller_address(marketplace.contract_address, BUYER.try_into().unwrap());
        // Try to purchase inactive course - should fail
        marketplace.purchase_course(course_id);
        stop_cheat_caller_address(marketplace.contract_address);
    }

    #[test]
    #[should_panic(expected: ('Not course creator',))]
    fn test_course_update_unauthorized() {
        let (marketplace, _token) = __setup__();

        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());
        let course_id = marketplace
            .create_course(
                "Introduction to Cairo",
                "Learn Cairo programming from scratch",
                COURSE_PRICE,
                500_u16,
            );
        stop_cheat_caller_address(marketplace.contract_address);

        start_cheat_caller_address(marketplace.contract_address, BUYER.try_into().unwrap());
        marketplace
            .update_course(
                course_id, "Unauthorized Update", "This should fail", COURSE_PRICE, 500_u16,
            );
        stop_cheat_caller_address(marketplace.contract_address);
    }

    #[test]
    #[should_panic(expected: ('Fee exceeds 10%',))]
    fn test_platform_fee_too_high() {
        let (marketplace, _token) = __setup__();

        start_cheat_caller_address(marketplace.contract_address, OWNER.try_into().unwrap());
        marketplace.set_platform_fee(1100_u16); // 11% - should fail
        stop_cheat_caller_address(marketplace.contract_address);
    }

    #[test]
    #[should_panic(expected: ('Period too short',))]
    fn test_escrow_period_too_short() {
        let (marketplace, _token) = __setup__();

        start_cheat_caller_address(marketplace.contract_address, OWNER.try_into().unwrap());
        marketplace.set_escrow_period(3600_u64); // 1 hour - should fail
        stop_cheat_caller_address(marketplace.contract_address);
    }

    #[test]
    #[should_panic(expected: ('Period too long',))]
    fn test_escrow_period_too_long() {
        let (marketplace, _token) = __setup__();

        start_cheat_caller_address(marketplace.contract_address, OWNER.try_into().unwrap());
        marketplace.set_escrow_period(2592001_u64); // > 30 days - should fail
        stop_cheat_caller_address(marketplace.contract_address);
    }

    #[test]
    #[should_panic(expected: ('Price must be greater than 0',))]
    fn test_create_course_zero_price() {
        let (marketplace, _token) = __setup__();

        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());
        marketplace.create_course("Free Course", "This should fail", 0_u256, 500_u16);
        stop_cheat_caller_address(marketplace.contract_address);
    }

    #[test]
    #[should_panic(expected: ('Royalty exceeds 100%',))]
    fn test_create_course_invalid_royalty() {
        let (marketplace, _token) = __setup__();

        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());
        marketplace
            .create_course(
                "Invalid Royalty Course",
                "This should fail",
                COURSE_PRICE,
                10001_u16 // 100.01% royalty should fail
            );
        stop_cheat_caller_address(marketplace.contract_address);
    }

    #[test]
    #[should_panic(expected: ('Course does not exist',))]
    fn test_get_nonexistent_course() {
        let (marketplace, _token) = __setup__();
        marketplace.get_course(999_u256);
    }

    #[test]
    #[should_panic(expected: ('Purchase does not exist',))]
    fn test_get_nonexistent_purchase() {
        let (marketplace, _token) = __setup__();
        marketplace.get_purchase(999_u256);
    }

    #[test]
    #[should_panic(expected: ('Dispute does not exist',))]
    fn test_get_nonexistent_dispute() {
        let (marketplace, _token) = __setup__();
        marketplace.get_dispute(999_u256);
    }

    // *************************************************************************
    //                              GETTER TESTS
    // *************************************************************************

    #[test]
    fn test_get_courses_by_creator() {
        let (marketplace, _token) = __setup__();

        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());

        let course_id1 = marketplace
            .create_course("Course 1", "Description 1", COURSE_PRICE, 500_u16);
        let course_id2 = marketplace
            .create_course("Course 2", "Description 2", COURSE_PRICE * 2, 750_u16);

        stop_cheat_caller_address(marketplace.contract_address);

        let creator_courses = marketplace.get_courses_by_creator(CREATOR.try_into().unwrap());
        assert(creator_courses.len() == 2, 'Creator should have 2 courses');
        assert(*creator_courses.at(0) == course_id1, 'Invalid first course');
        assert(*creator_courses.at(1) == course_id2, 'Invalid second course');
    }

    #[test]
    fn test_course_counter() {
        let (marketplace, _token) = __setup__();

        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());

        assert(marketplace.get_course_count() == 0, 'Initial count should be 0');

        marketplace.create_course("Course 1", "Description 1", COURSE_PRICE, 500_u16);
        assert(marketplace.get_course_count() == 1, 'Count should be 1');

        marketplace.create_course("Course 2", "Description 2", COURSE_PRICE * 2, 750_u16);
        assert(marketplace.get_course_count() == 2, 'Count should be 2');

        stop_cheat_caller_address(marketplace.contract_address);
    }

    #[test]
    fn test_has_purchased_course() {
        let (marketplace, _token) = __setup__();

        start_cheat_caller_address(marketplace.contract_address, CREATOR.try_into().unwrap());
        let course_id = marketplace
            .create_course(
                "Introduction to Cairo",
                "Learn Cairo programming from scratch",
                COURSE_PRICE,
                500_u16,
            );
        stop_cheat_caller_address(marketplace.contract_address);

        // Initially, buyer hasn't purchased
        assert(
            !marketplace.has_purchased_course(BUYER.try_into().unwrap(), course_id),
            'Should not have purchased',
        );

        // After purchase, should return true
        start_cheat_caller_address(marketplace.contract_address, BUYER.try_into().unwrap());
        marketplace.purchase_course(course_id);
        stop_cheat_caller_address(marketplace.contract_address);

        assert(
            marketplace.has_purchased_course(BUYER.try_into().unwrap(), course_id),
            'Should have purchased',
        );
    }
}
