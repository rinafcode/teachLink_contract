#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod content_nft {
    use ink::storage::Mapping;

    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum LicenseType {
        Personal,
        Commercial,
        Exclusive,
    }

    #[ink(storage)]
    pub struct ContentNFT {
        owner: Mapping<u32, AccountId>,        // tokenId → owner
        metadata: Mapping<u32, String>,        // tokenId → metadata URI
        royalties: Mapping<u32, u8>,           // tokenId → royalty percentage
        license: Mapping<u32, LicenseType>,    // tokenId → license type
        total_supply: u32,
    }

    impl ContentNFT {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                owner: Mapping::default(),
                metadata: Mapping::default(),
                royalties: Mapping::default(),
                license: Mapping::default(),
                total_supply: 0,
            }
        }

        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, metadata_uri: String, royalty: u8, license: LicenseType) -> u32 {
            self.total_supply += 1;
            let token_id = self.total_supply;
            self.owner.insert(token_id, &to);
            self.metadata.insert(token_id, &metadata_uri);
            self.royalties.insert(token_id, &royalty);
            self.license.insert(token_id, &license);
            token_id
        }

        #[ink(message)]
        pub fn get_metadata(&self, token_id: u32) -> Option<String> {
            self.metadata.get(token_id)
        }

        #[ink(message)]
        pub fn get_owner(&self, token_id: u32) -> Option<AccountId> {
            self.owner.get(token_id)
        }
    }
}
