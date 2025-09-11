#[starknet::contract]
pub mod MockERC20 {
    use openzeppelin::token::erc20::{ERC20Component, ERC20MintableComponent};
    use openzeppelin::access::ownable::OwnableComponent;
    use starknet::ContractAddress;

    component!(path: ERC20Component, storage: erc20, event: ERC20Event);
    component!(path: ERC20MintableComponent, storage: mint, event: ERC20MintableEvent);
    component!(path: OwnableComponent, storage: ownable, event: OwnableEvent);

    #[abi(embed_v0)]
    impl ERC20Impl = ERC20Component::ERC20Impl<ContractState>;
    #[abi(embed_v0)]
    impl ERC20CamelOnly = ERC20Component::ERC20CamelOnlyImpl<ContractState>;
    #[abi(embed_v0)]
    impl ERC20MintableImpl = ERC20MintableComponent::ERC20MintableImpl<ContractState>;
    #[abi(embed_v0)]
    impl OwnableImpl = OwnableComponent::OwnableImpl<ContractState>;

    impl ERC20InternalImpl = ERC20Component::InternalImpl<ContractState>;
    impl OwnableInternalImpl = OwnableComponent::InternalImpl<ContractState>;

    #[storage]
    struct Storage {
        #[substorage(v0)] erc20: ERC20Component::Storage,
        #[substorage(v0)] mint: ERC20MintableComponent::Storage,
        #[substorage(v0)] ownable: OwnableComponent::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        #[flat] ERC20Event: ERC20Component::Event,
        #[flat] ERC20MintableEvent: ERC20MintableComponent::Event,
        #[flat] OwnableEvent: OwnableComponent::Event,
    }

    #[constructor]
    fn constructor(ref self: ContractState, owner: ContractAddress, name: felt252, symbol: felt252, decimals: u8) {
        self.ownable.initializer(owner);
        self.erc20.initializer(name, symbol, decimals);
    }

    #[abi]
    impl External of ExternalTrait {
        fn mint_to(ref self: ContractState, to: ContractAddress, amount: u256) {
            self.ownable.assert_only_owner();
            self.erc20._mint(to, amount);
        }
    }
}


