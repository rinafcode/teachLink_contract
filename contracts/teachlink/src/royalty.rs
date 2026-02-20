pub struct RoyaltySplit {
    pub recipients: Vec<(Address, u16)>, // percentage basis points
}

pub fn distribute(token_id: u64, amount: u128) {
    let splits = Self::get_royalty_split(token_id);

    for (recipient, percentage) in splits {
        let share = amount * percentage as u128 / 10000;
        Self::transfer_platform(recipient, share);
    }
}