#[starknet::contract]
pub mod MockERC721 {
    use openzeppelin::token::erc721::{ERC721Component, ERC721HooksEmptyImpl};
    use openzeppelin::introspection::src5::SRC5Component;
    use openzeppelin::access::ownable::OwnableComponent;
    use starknet::{ContractAddress};

    component!(path: ERC721Component, storage: erc721, event: ERC721Event);
    component!(path: SRC5Component, storage: src5, event: SRC5Event);
    component!(path: OwnableComponent, storage: ownable, event: OwnableEvent);

    #[abi(embed_v0)]
    impl ERC721Impl = ERC721Component::ERC721Impl<ContractState>;
    #[abi(embed_v0)]
    impl ERC721MetadataImpl = ERC721Component::ERC721MetadataImpl<ContractState>;
    #[abi(embed_v0)]
    impl ERC721CamelOnly = ERC721Component::ERC721CamelOnlyImpl<ContractState>;
    #[abi(embed_v0)]
    impl SRC5Impl = SRC5Component::SRC5Impl<ContractState>;
    #[abi(embed_v0)]
    impl OwnableImpl = OwnableComponent::OwnableImpl<ContractState>;

    impl ERC721InternalImpl = ERC721Component::InternalImpl<ContractState>;
    impl OwnableInternalImpl = OwnableComponent::InternalImpl<ContractState>;

    #[storage]
    struct Storage {
        #[substorage(v0)] erc721: ERC721Component::Storage,
        #[substorage(v0)] src5: SRC5Component::Storage,
        #[substorage(v0)] ownable: OwnableComponent::Storage,
        next_id: u256,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        #[flat] ERC721Event: ERC721Component::Event,
        #[flat] SRC5Event: SRC5Component::Event,
        #[flat] OwnableEvent: OwnableComponent::Event,
    }

    #[constructor]
    fn constructor(ref self: ContractState, owner: ContractAddress, name: felt252, symbol: felt252, base_uri: felt252) {
        self.ownable.initializer(owner);
        self.erc721.initializer(name, symbol, base_uri);
        self.next_id.write(1);
    }

    #[abi]
    impl External of ExternalTrait {
        fn mint_to(ref self: ContractState, to: ContractAddress) -> u256 {
            self.ownable.assert_only_owner();
            let id = self.next_id.read();
            self.next_id.write(id + 1);
            self.erc721._mint(to, id);
            id
        }
    }
}


