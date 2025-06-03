#[cfg(test)]
mod tests {
    use core::result::ResultTrait;
    use starknet::{ContractAddress, get_contract_address};
    use snforge_std::{
        declare, ContractClassTrait, DeclareResultTrait, start_cheat_caller_address,
        stop_cheat_caller_address, spy_events, EventSpyAssertionsTrait,
    };

    use teachlink::interfaces::itoken::{ITokenDispatcher, ITokenDispatcherTrait};
    use teachlink::token::TeachLinkToken::{
        Event as TokenEvent, MinterAdded, TokensMinted, ContractPaused,
    };
    use openzeppelin::token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherTrait};

    const INITIAL_SUPPLY: u256 = 1000000_000000000000000000; // 1M tokens with 18 decimals
    const MINT_AMOUNT: u256 = 1000_000000000000000000; // 1K tokens
    const BURN_AMOUNT: u256 = 500_000000000000000000; // 500 tokens

    const OWNER: felt252 = 'OWNER';
    const USER1: felt252 = 'USER1';
    const USER2: felt252 = 'USER2';
    const MINTER: felt252 = 'MINTER';
    const BURNER: felt252 = 'BURNER';

    fn __setup__() -> ContractAddress {
        let contract = declare("TeachLinkToken").unwrap().contract_class();

        let mut constructor_calldata: Array<felt252> = array![];
        let token_name: ByteArray = "TeachLink Token";
        let token_symbol: ByteArray = "TLT";

        token_name.serialize(ref constructor_calldata);
        token_symbol.serialize(ref constructor_calldata);
        INITIAL_SUPPLY.serialize(ref constructor_calldata);

        start_cheat_caller_address(get_contract_address(), OWNER.try_into().unwrap());
        let (contract_address, _) = contract.deploy(@constructor_calldata).unwrap();
        stop_cheat_caller_address(get_contract_address());

        contract_address
    }

    // *************************************************************************
    //                              ROLE MANAGEMENT TESTS
    // *************************************************************************

    #[test]
    fn test_initial_roles() {
        let contract_address = __setup__();
        let dispatcher = ITokenDispatcher { contract_address };

        // Owner should have both minter and burner roles initially
        assert(dispatcher.is_minter(OWNER.try_into().unwrap()), 'Owner should be minter');
        assert(dispatcher.is_burner(OWNER.try_into().unwrap()), 'Owner should be burner');
        assert(!dispatcher.is_minter(USER1.try_into().unwrap()), 'User1 should not be minter');
        assert(!dispatcher.is_burner(USER1.try_into().unwrap()), 'User1 should not be burner');
    }

    #[test]
    fn test_set_minter_role() {
        let contract_address = __setup__();
        let dispatcher = ITokenDispatcher { contract_address };

        start_cheat_caller_address(contract_address, OWNER.try_into().unwrap());
        dispatcher.set_minter(MINTER.try_into().unwrap(), true);
        stop_cheat_caller_address(contract_address);

        assert(dispatcher.is_minter(MINTER.try_into().unwrap()), 'Minter role not set');
    }

    #[test]
    fn test_revoke_minter_role() {
        let contract_address = __setup__();
        let dispatcher = ITokenDispatcher { contract_address };

        start_cheat_caller_address(contract_address, OWNER.try_into().unwrap());
        dispatcher.set_minter(MINTER.try_into().unwrap(), true);
        dispatcher.set_minter(MINTER.try_into().unwrap(), false);
        stop_cheat_caller_address(contract_address);

        assert(!dispatcher.is_minter(MINTER.try_into().unwrap()), 'Minter role not revoked');
    }

    #[test]
    fn test_set_burner_role() {
        let contract_address = __setup__();
        let dispatcher = ITokenDispatcher { contract_address };

        start_cheat_caller_address(contract_address, OWNER.try_into().unwrap());
        dispatcher.set_burner(BURNER.try_into().unwrap(), true);
        stop_cheat_caller_address(contract_address);

        assert(dispatcher.is_burner(BURNER.try_into().unwrap()), 'Burner role not set');
    }

    // *************************************************************************
    //                              MINTING TESTS
    // *************************************************************************

    #[test]
    fn test_mint_by_owner() {
        let contract_address = __setup__();
        let dispatcher = ITokenDispatcher { contract_address };
        let erc20_dispatcher = IERC20Dispatcher { contract_address };

        let initial_balance = erc20_dispatcher.balance_of(USER1.try_into().unwrap());
        let initial_supply = erc20_dispatcher.total_supply();

        start_cheat_caller_address(contract_address, OWNER.try_into().unwrap());
        dispatcher.mint(USER1.try_into().unwrap(), MINT_AMOUNT);
        stop_cheat_caller_address(contract_address);

        assert(
            erc20_dispatcher.balance_of(USER1.try_into().unwrap()) == initial_balance + MINT_AMOUNT,
            'Mint failed',
        );
        assert(
            erc20_dispatcher.total_supply() == initial_supply + MINT_AMOUNT,
            'Total supply not updated',
        );
    }

    #[test]
    fn test_mint_by_authorized_minter() {
        let contract_address = __setup__();
        let dispatcher = ITokenDispatcher { contract_address };
        let erc20_dispatcher = IERC20Dispatcher { contract_address };

        start_cheat_caller_address(contract_address, OWNER.try_into().unwrap());
        dispatcher.set_minter(MINTER.try_into().unwrap(), true);
        stop_cheat_caller_address(contract_address);

        let initial_balance = erc20_dispatcher.balance_of(USER1.try_into().unwrap());
        let initial_supply = erc20_dispatcher.total_supply();

        start_cheat_caller_address(contract_address, MINTER.try_into().unwrap());
        dispatcher.mint(USER1.try_into().unwrap(), MINT_AMOUNT);
        stop_cheat_caller_address(contract_address);

        assert(
            erc20_dispatcher.balance_of(USER1.try_into().unwrap()) == initial_balance + MINT_AMOUNT,
            'Mint failed',
        );
        assert(
            erc20_dispatcher.total_supply() == initial_supply + MINT_AMOUNT,
            'Total supply not updated',
        );
    }

    // *************************************************************************
    //                              BURNING TESTS
    // *************************************************************************

    #[test]
    fn test_burn_by_owner() {
        let contract_address = __setup__();
        let dispatcher = ITokenDispatcher { contract_address };
        let erc20_dispatcher = IERC20Dispatcher { contract_address };

        let initial_balance = erc20_dispatcher.balance_of(OWNER.try_into().unwrap());
        let initial_supply = erc20_dispatcher.total_supply();

        start_cheat_caller_address(contract_address, OWNER.try_into().unwrap());
        dispatcher.burn(OWNER.try_into().unwrap(), BURN_AMOUNT);
        stop_cheat_caller_address(contract_address);

        assert(
            erc20_dispatcher.balance_of(OWNER.try_into().unwrap()) == initial_balance - BURN_AMOUNT,
            'Burn failed',
        );
        assert(
            erc20_dispatcher.total_supply() == initial_supply - BURN_AMOUNT,
            'Total supply not updated',
        );
    }

    #[test]
    fn test_burn_by_authorized_burner() {
        let contract_address = __setup__();
        let dispatcher = ITokenDispatcher { contract_address };
        let erc20_dispatcher = IERC20Dispatcher { contract_address };

        // Setup: mint tokens to USER1 and set BURNER role
        start_cheat_caller_address(contract_address, OWNER.try_into().unwrap());
        dispatcher.mint(USER1.try_into().unwrap(), MINT_AMOUNT);
        dispatcher.set_burner(BURNER.try_into().unwrap(), true);
        stop_cheat_caller_address(contract_address);

        // USER1 approves BURNER to burn their tokens
        start_cheat_caller_address(contract_address, USER1.try_into().unwrap());
        erc20_dispatcher.approve(BURNER.try_into().unwrap(), BURN_AMOUNT);
        stop_cheat_caller_address(contract_address);

        let initial_balance = erc20_dispatcher.balance_of(USER1.try_into().unwrap());
        let initial_supply = erc20_dispatcher.total_supply();

        start_cheat_caller_address(contract_address, BURNER.try_into().unwrap());
        dispatcher.burn(USER1.try_into().unwrap(), BURN_AMOUNT);
        stop_cheat_caller_address(contract_address);

        assert(
            erc20_dispatcher.balance_of(USER1.try_into().unwrap()) == initial_balance - BURN_AMOUNT,
            'Burn failed',
        );
        assert(
            erc20_dispatcher.total_supply() == initial_supply - BURN_AMOUNT,
            'Total supply not updated',
        );
    }

    #[test]
    fn test_burn_from_caller() {
        let contract_address = __setup__();
        let dispatcher = ITokenDispatcher { contract_address };
        let erc20_dispatcher = IERC20Dispatcher { contract_address };

        // Mint tokens to USER1
        start_cheat_caller_address(contract_address, OWNER.try_into().unwrap());
        dispatcher.mint(USER1.try_into().unwrap(), MINT_AMOUNT);
        stop_cheat_caller_address(contract_address);

        let initial_balance = erc20_dispatcher.balance_of(USER1.try_into().unwrap());
        let initial_supply = erc20_dispatcher.total_supply();

        start_cheat_caller_address(contract_address, USER1.try_into().unwrap());
        dispatcher.burn_from_caller(BURN_AMOUNT);
        stop_cheat_caller_address(contract_address);

        assert(
            erc20_dispatcher.balance_of(USER1.try_into().unwrap()) == initial_balance - BURN_AMOUNT,
            'Self burn failed',
        );
        assert(
            erc20_dispatcher.total_supply() == initial_supply - BURN_AMOUNT,
            'Total supply not updated',
        );
    }

    // *************************************************************************
    //                              PAUSE/UNPAUSE TESTS
    // *************************************************************************

    #[test]
    fn test_pause_unpause() {
        let contract_address = __setup__();
        let dispatcher = ITokenDispatcher { contract_address };

        assert(!dispatcher.is_paused(), 'Should not be paused initially');

        start_cheat_caller_address(contract_address, OWNER.try_into().unwrap());
        dispatcher.pause();
        stop_cheat_caller_address(contract_address);

        assert(dispatcher.is_paused(), 'Should be paused');

        start_cheat_caller_address(contract_address, OWNER.try_into().unwrap());
        dispatcher.unpause();
        stop_cheat_caller_address(contract_address);

        assert(!dispatcher.is_paused(), 'Should be unpaused');
    }

    // *************************************************************************
    //                              ACCOUNT FREEZING TESTS
    // *************************************************************************

    #[test]
    fn test_freeze_unfreeze_account() {
        let contract_address = __setup__();
        let dispatcher = ITokenDispatcher { contract_address };

        assert(!dispatcher.is_frozen(USER1.try_into().unwrap()), 'Should not be frozen initially');

        start_cheat_caller_address(contract_address, OWNER.try_into().unwrap());
        dispatcher.freeze_account(USER1.try_into().unwrap());
        stop_cheat_caller_address(contract_address);

        assert(dispatcher.is_frozen(USER1.try_into().unwrap()), 'Should be frozen');

        start_cheat_caller_address(contract_address, OWNER.try_into().unwrap());
        dispatcher.unfreeze_account(USER1.try_into().unwrap());
        stop_cheat_caller_address(contract_address);

        assert(!dispatcher.is_frozen(USER1.try_into().unwrap()), 'Should be unfrozen');
    }

    // *************************************************************************
    //                              EMERGENCY BURN TESTS
    // *************************************************************************

    #[test]
    fn test_emergency_burn_works_when_paused() {
        let contract_address = __setup__();
        let dispatcher = ITokenDispatcher { contract_address };
        let erc20_dispatcher = IERC20Dispatcher { contract_address };

        // Mint tokens to USER1
        start_cheat_caller_address(contract_address, OWNER.try_into().unwrap());
        dispatcher.mint(USER1.try_into().unwrap(), MINT_AMOUNT);
        stop_cheat_caller_address(contract_address);

        let initial_balance = erc20_dispatcher.balance_of(USER1.try_into().unwrap());
        let initial_supply = erc20_dispatcher.total_supply();

        // Pause contract and perform emergency burn
        start_cheat_caller_address(contract_address, OWNER.try_into().unwrap());
        dispatcher.pause();
        dispatcher.emergency_burn(USER1.try_into().unwrap(), BURN_AMOUNT);
        stop_cheat_caller_address(contract_address);

        assert(
            erc20_dispatcher.balance_of(USER1.try_into().unwrap()) == initial_balance - BURN_AMOUNT,
            'Emergency burn failed',
        );
        assert(
            erc20_dispatcher.total_supply() == initial_supply - BURN_AMOUNT,
            'Total supply not updated',
        );
    }

    // *************************************************************************
    //                              EVENT TESTS
    // *************************************************************************

    #[test]
    fn test_minter_added_event() {
        let contract_address = __setup__();
        let dispatcher = ITokenDispatcher { contract_address };

        let mut spy = spy_events();

        start_cheat_caller_address(contract_address, OWNER.try_into().unwrap());
        dispatcher.set_minter(MINTER.try_into().unwrap(), true);
        stop_cheat_caller_address(contract_address);

        let expected_event = TokenEvent::MinterAdded(
            MinterAdded { minter: MINTER.try_into().unwrap() },
        );

        spy.assert_emitted(@array![(contract_address, expected_event)]);
    }

    #[test]
    fn test_tokens_minted_event() {
        let contract_address = __setup__();
        let dispatcher = ITokenDispatcher { contract_address };

        let mut spy = spy_events();

        start_cheat_caller_address(contract_address, OWNER.try_into().unwrap());
        dispatcher.mint(USER1.try_into().unwrap(), MINT_AMOUNT);
        stop_cheat_caller_address(contract_address);

        let expected_event = TokenEvent::TokensMinted(
            TokensMinted {
                to: USER1.try_into().unwrap(),
                amount: MINT_AMOUNT,
                minter: OWNER.try_into().unwrap(),
            },
        );

        spy.assert_emitted(@array![(contract_address, expected_event)]);
    }

    #[test]
    fn test_contract_paused_event() {
        let contract_address = __setup__();
        let dispatcher = ITokenDispatcher { contract_address };

        let mut spy = spy_events();

        start_cheat_caller_address(contract_address, OWNER.try_into().unwrap());
        dispatcher.pause();
        stop_cheat_caller_address(contract_address);

        let expected_event = TokenEvent::ContractPaused(
            ContractPaused { by: OWNER.try_into().unwrap() },
        );

        spy.assert_emitted(@array![(contract_address, expected_event)]);
    }
}
