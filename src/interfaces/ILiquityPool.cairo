use starknet::ContractAddress;

#[starknet::interface]
pub trait ILiquidityPool<TContractState> {
    // Pool Management
    fn initialize(
        ref self: TContractState,
        token_a: ContractAddress,
        token_b: ContractAddress,
        fee_rate: u256,
        protocol_fee_rate: u256
    );
    
    // Liquidity Operations
    fn add_liquidity(
        ref self: TContractState,
        amount_a_desired: u256,
        amount_b_desired: u256,
        amount_a_min: u256,
        amount_b_min: u256,
        to: ContractAddress,
        deadline: u64
    ) -> (u256, u256, u256);
    
    fn remove_liquidity(
        ref self: TContractState,
        liquidity: u256,
        amount_a_min: u256,
        amount_b_min: u256,
        to: ContractAddress,
        deadline: u64
    ) -> (u256, u256);
    
    // Trading Operations
    fn swap_exact_tokens_for_tokens(
        ref self: TContractState,
        amount_in: u256,
        amount_out_min: u256,
        token_in: ContractAddress,
        to: ContractAddress,
        deadline: u64
    ) -> u256;
    
    fn swap_tokens_for_exact_tokens(
        ref self: TContractState,
        amount_out: u256,
        amount_in_max: u256,
        token_in: ContractAddress,
        to: ContractAddress,
        deadline: u64
    ) -> u256;
    
    // View Functions
    fn get_reserves() -> (u256, u256, u64);
    fn get_amounts_out(amount_in: u256, token_in: ContractAddress) -> u256;
    fn get_amounts_in(amount_out: u256, token_out: ContractAddress) -> u256;
    fn quote(amount_a: u256, reserve_a: u256, reserve_b: u256) -> u256;
    
    // Pool Information
    fn token_a() -> ContractAddress;
    fn token_b() -> ContractAddress;
    fn total_supply() -> u256;
    fn fee_rate() -> u256;
    fn protocol_fee_rate() -> u256;
    
    // Yield Farming
    fn stake_lp_tokens(ref self: TContractState, amount: u256);
    fn unstake_lp_tokens(ref self: TContractState, amount: u256);
    fn claim_rewards(ref self: TContractState) -> u256;
    fn get_pending_rewards(user: ContractAddress) -> u256;
    
    // Impermanent Loss Protection
    fn enable_il_protection(ref self: TContractState);
    fn calculate_il_compensation(user: ContractAddress) -> u256;
    fn claim_il_compensation(ref self: TContractState) -> u256;
}
