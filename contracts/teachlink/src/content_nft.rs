#[derive(Serialize, Deserialize, Clone)]
pub struct ContentMetadata {
    pub title: String,
    pub description: String,
    pub content_uri: String,
    pub preview_uri: Option<String>,
    pub category: String,
    pub tags: Vec<String>,
    pub license_type: LicenseType,
    pub version: u32,
    pub quality_score: u32,
}

pub struct ContentNFT {
    pub token_id: u64,
    pub creator: Address,
    pub co_owners: Vec<Address>,
    pub metadata: ContentMetadata,
    pub royalty_percentage: u16,
    pub fractionalized: bool,
    pub created_at: u64,
}

pub fn mint_content(
    creator: Address,
    metadata: ContentMetadata,
    royalty_percentage: u16,
) -> Result<u64> {
    assert!(royalty_percentage <= 2000); // max 20%

    let token_id = Self::next_token_id();
    
    let nft = ContentNFT {
        token_id,
        creator,
        co_owners: vec![],
        metadata,
        royalty_percentage,
        fractionalized: false,
        created_at: block_timestamp(),
    };

    Self::store_nft(token_id, nft);
    Ok(token_id)
}