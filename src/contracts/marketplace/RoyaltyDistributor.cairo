pub mod RoyaltyDistributor {
    use starknet::ContractAddress;

    pub fn compute_and_split(
        collection: ContractAddress,
        sale_price: u256,
        platform_fee_bps: u16,
        royalty_bps: u16,
        royalty_recipient: ContractAddress,
    ) -> (u256, u256, u256, ContractAddress) {
        let fee = (sale_price * platform_fee_bps.into()) / 10000;
        let royalty = (sale_price * royalty_bps.into()) / 10000;
        let seller_amount = sale_price - fee - royalty;
        (fee, royalty, seller_amount, royalty_recipient)
    }
}


