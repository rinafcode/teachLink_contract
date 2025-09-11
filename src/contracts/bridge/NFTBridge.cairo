#[starknet::contract]
pub mod NFTBridge {
    use starknet::{
        ContractAddress, get_caller_address, get_contract_address,
        storage::{Map, StorageMapReadAccess, StorageMapWriteAccess, StoragePointerReadAccess, StoragePointerWriteAccess},
    };
    use openzeppelin::access::ownable::OwnableComponent;
    use openzeppelin::token::erc721::interface::{IERC721Dispatcher, IERC721DispatcherTrait};

    use super::interfaces::IBridge::IBridge;
    use super::libraries::MessageVerification::MessageVerification;

    component!(path: OwnableComponent, storage: ownable, event: OwnableEvent);

    #[abi(embed_v0)]
    impl OwnableImpl = OwnableComponent::OwnableImpl<ContractState>;
    impl OwnableInternalImpl = OwnableComponent::InternalImpl<ContractState>;

    #[storage]
    struct Storage {
        paused: bool,
        supported_collections: Map<ContractAddress, bool>,
        #[substorage(v0)] ownable: OwnableComponent::Storage,
        #[substorage(v0)] verification: MessageVerification::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        NFTBridgeRequested: NFTBridgeRequested,
        NFTReleased: NFTReleased,
        CollectionUpdated: CollectionUpdated,
        Paused: Paused,
        Unpaused: Unpaused,
        #[flat] OwnableEvent: OwnableComponent::Event,
        #[flat] MessageVerificationEvent: MessageVerification::Event,
    }

    #[derive(Drop, starknet::Event)]
    pub struct NFTBridgeRequested { pub message_id: felt252, pub collection: ContractAddress, pub token_id: u256, pub recipient: ContractAddress, pub dest_chain_id: felt252 }
    #[derive(Drop, starknet::Event)]
    pub struct NFTReleased { pub message_id: felt252, pub collection: ContractAddress, pub token_id: u256, pub recipient: ContractAddress }
    #[derive(Drop, starknet::Event)]
    pub struct CollectionUpdated { pub collection: ContractAddress, pub supported: bool }
    #[derive(Drop, starknet::Event)]
    pub struct Paused {}
    #[derive(Drop, starknet::Event)]
    pub struct Unpaused {}

    #[constructor]
    fn constructor(ref self: ContractState, owner: ContractAddress, per_window_limit: u256, window_seconds: u64) {
        self.ownable.initializer(owner);
        self.paused.write(false);
        self.verification.set_rate_limit(per_window_limit, window_seconds);
    }

    #[abi(embed_v0)]
    impl BridgeImpl of IBridge<ContractState> {
        fn set_trusted_relayer(ref self: ContractState, relayer: ContractAddress, trusted: bool) {
            self.ownable.assert_only_owner();
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
        fn set_collection_supported(ref self: ContractState, collection: ContractAddress, supported: bool) {
            self.ownable.assert_only_owner();
            self.supported_collections.write(collection, supported);
            self.emit(CollectionUpdated { collection, supported });
        }

        fn deposit_nft(ref self: ContractState, collection: ContractAddress, token_id: u256, dest_chain_id: felt252, recipient: ContractAddress, message_id: felt252) {
            assert(!self.paused.read(), 'paused');
            assert(self.supported_collections.read(collection), 'unsupported');
            // Pull NFT to escrow
            let nft = IERC721Dispatcher { contract_address: collection };
            let _ = nft.transfer_from(get_caller_address(), get_contract_address(), token_id);

            self.verification.assert_and_consume_rate(1);
            self.emit(NFTBridgeRequested { message_id, collection, token_id, recipient, dest_chain_id });
        }

        fn release_nft(ref self: ContractState, message_id: felt252, collection: ContractAddress, token_id: u256, recipient: ContractAddress, relayer: ContractAddress) {
            assert(!self.paused.read(), 'paused');
            assert(self.supported_collections.read(collection), 'unsupported');
            assert(self.verification.is_trusted(relayer), 'not trusted');
            assert(!self.verification.is_processed(message_id), 'processed');

            self.verification.assert_and_consume_rate(1);
            self.verification.mark_processed(message_id);

            let nft = IERC721Dispatcher { contract_address: collection };
            let _ = nft.transfer_from(get_contract_address(), recipient, token_id);
            self.emit(NFTReleased { message_id, collection, token_id, recipient });
        }
    }
}


