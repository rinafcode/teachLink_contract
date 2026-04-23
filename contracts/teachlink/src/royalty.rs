pub struct RoyaltySplit {
    pub recipients: Vec<(Address, u16)>, // percentage basis points
}

pub fn distribute(token_id: u64, amount: u128) {
    let splits = Self::get_royalty_split(token_id);

    // Guard: ensure total basis points do not exceed 10000 to prevent over-distribution
    let total_bps: u32 = splits
        .iter()
        .map(|(_, pct)| pct as u32)
        .sum();
    assert!(
        total_bps <= 10000,
        "royalty split exceeds 100% ({}bps)",
        total_bps
    );

    let mut distributed: u128 = 0;
    let mut last_recipient: Option<Address> = None;

    for (recipient, percentage) in splits.iter() {
        let share = amount * percentage as u128 / 10000;
        distributed += share;
        Self::transfer_platform(recipient.clone(), share);
        last_recipient = Some(recipient);
    }

    // Send any rounding dust to the first recipient to ensure full distribution
    let remainder = amount.saturating_sub(distributed);
    if remainder > 0 {
        if let Some(recipient) = last_recipient {
            Self::transfer_platform(recipient, remainder);
        }
    }
}
