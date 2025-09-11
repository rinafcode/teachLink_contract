#[starknet::interface]
pub trait IBridge<TContractState> {
    // Admin
    fn set_trusted_relayer(ref self: TContractState, relayer: starknet::ContractAddress, trusted: bool);
    fn set_rate_limit(ref self: TContractState, per_window_limit: u256, window_seconds: u64);
    fn pause(ref self: TContractState);
    fn unpause(ref self: TContractState);

    // Views
    fn is_paused(self: @TContractState) -> bool;
}


