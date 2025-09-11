#[starknet::contract]
pub mod TokenBridge {
    use starknet::{
        ContractAddress, get_caller_address, get_contract_address,
        storage::{Map, StoragePointerReadAccess, StoragePointerWriteAccess},
    };
    use openzeppelin::access::ownable::OwnableComponent;
    use openzeppelin::token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherTrait};

    use super::interfaces::IBridge::IBridge;
    use super::libraries::MessageVerification::MessageVerification;

    component!(path: OwnableComponent, storage: ownable, event: OwnableEvent);

    #[abi(embed_v0)]
    impl OwnableImpl = OwnableComponent::OwnableImpl<ContractState>;
    impl OwnableInternalImpl = OwnableComponent::InternalImpl<ContractState>;

    #[storage]
    struct Storage {
        token: ContractAddress,
        paused: bool,
        locked_total: u256,
        #[substorage(v0)] ownable: OwnableComponent::Storage,
        #[substorage(v0)] verification: MessageVerification::Storage,
        trusted_relayer: Map<ContractAddress, bool>,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        BridgeRequested: BridgeRequested,
        Released: Released,
        Paused: Paused,
        Unpaused: Unpaused,
        #[flat] OwnableEvent: OwnableComponent::Event,
        #[flat] MessageVerificationEvent: MessageVerification::Event,
    }

    #[derive(Drop, starknet::Event)]
    pub struct BridgeRequested { pub message_id: felt252, pub recipient: ContractAddress, pub amount: u256, pub dest_chain_id: felt252 }
    #[derive(Drop, starknet::Event)]
    pub struct Released { pub message_id: felt252, pub recipient: ContractAddress, pub amount: u256 }
    #[derive(Drop, starknet::Event)]
    pub struct Paused {}
    #[derive(Drop, starknet::Event)]
    pub struct Unpaused {}

    #[constructor]
    fn constructor(ref self: ContractState, owner: ContractAddress, token: ContractAddress, per_window_limit: u256, window_seconds: u64) {
        self.ownable.initializer(owner);
        self.token.write(token);
        self.paused.write(false);
        self.locked_total.write(0);
        self.verification.set_rate_limit(per_window_limit, window_seconds);
    }

    #[abi(embed_v0)]
    impl BridgeImpl of IBridge<ContractState> {
        fn set_trusted_relayer(ref self: ContractState, relayer: ContractAddress, trusted: bool) {
            self.ownable.assert_only_owner();
            self.trusted_relayer.write(relayer, trusted);
            self.verification.set_trusted(relayer, trusted);
        }

        fn set_rate_limit(ref self: ContractState, per_window_limit: u256, window_seconds: u64) {
            self.ownable.assert_only_owner();
            self.verification.set_rate_limit(per_window_limit, window_seconds);
        }

        fn pause(ref self: ContractState) {
            self.ownable.assert_only_owner();
            self.paused.write(true);
            self.emit(Paused {});
        }

        fn unpause(ref self: ContractState) {
            self.ownable.assert_only_owner();
            self.paused.write(false);
            self.emit(Unpaused {});
        }

        fn is_paused(self: @ContractState) -> bool { self.paused.read() }
    }

    #[abi]
    impl External of ExternalTrait {
        // User flows
        fn deposit(ref self: ContractState, amount: u256, dest_chain_id: felt252, recipient: ContractAddress, message_id: felt252) {
            assert(!self.paused.read(), 'paused');
            assert(amount > 0, 'amt=0');

            // Pull funds to escrow
            let token = IERC20Dispatcher { contract_address: self.token.read() };
            let ok = token.transfer_from(get_caller_address(), get_contract_address(), amount);
            assert(ok, 'transfer_from');

            self.locked_total.write(self.locked_total.read() + amount);
            // Rate limit on outbound requests as well
            self.verification.assert_and_consume_rate(amount);

            self.emit(BridgeRequested { message_id, recipient, amount, dest_chain_id });
        }

        // Relayer flow on destination chain
        fn release(ref self: ContractState, message_id: felt252, recipient: ContractAddress, amount: u256, relayer: ContractAddress) {
            assert(!self.paused.read(), 'paused');
            assert(self.verification.is_trusted(relayer), 'not trusted');
            assert(!self.verification.is_processed(message_id), 'processed');

            // Rate limit incoming
            self.verification.assert_and_consume_rate(amount);
            self.verification.mark_processed(message_id);

            let token = IERC20Dispatcher { contract_address: self.token.read() };
            let ok2 = token.transfer(recipient, amount);
            assert(ok2, 'transfer');

            let locked = self.locked_total.read();
            self.locked_total.write(if locked > amount { locked - amount } else { 0 });
            self.emit(Released { message_id, recipient, amount });
        }

        // Views
        fn get_token(self: @ContractState) -> ContractAddress { self.token.read() }
        fn get_locked_total(self: @ContractState) -> u256 { self.locked_total.read() }
    }
}


