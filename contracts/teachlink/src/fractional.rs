pub struct FractionalVault {
    pub token_id: u64,
    pub total_shares: u128,
    pub price_per_share: u128,
    pub shareholders: Map<Address, u128>,
}

