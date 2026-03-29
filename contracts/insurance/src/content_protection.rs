pub struct ContentInsurancePolicy {
    pub token_id: u64,
    pub coverage_amount: u128,
    pub premium_paid: u128,
    pub active: bool,
}

pub struct Order {
    pub token_id: u64,
    pub trader: Address,
    pub price: u128,
    pub is_buy: bool,
}