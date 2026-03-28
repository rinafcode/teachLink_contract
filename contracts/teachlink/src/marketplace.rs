pub struct Listing {
    pub listing_id: u64,
    pub token_id: u64,
    pub seller: Address,
    pub price: u128,
    pub payment_token: Address,
    pub active: bool,
}

pub fn buy(listing_id: u64, buyer: Address) -> Result<()> {
    let listing = Self::get_listing(listing_id)?;
    let nft = ContentModule::get_nft(listing.token_id)?;

    let royalty = listing.price * nft.royalty_percentage as u128 / 10000;
    let seller_amount = listing.price - royalty;

    // Transfer funds
    Self::transfer(buyer, nft.creator, royalty);
    Self::transfer(buyer, listing.seller, seller_amount);

    // Transfer ownership
    ContentModule::transfer_nft(listing.token_id, buyer)?;

    Ok(())
}

