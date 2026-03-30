#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod marketplace {
    use ink::storage::Mapping;

    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Status {
        Listed,
        Sold,
        Cancelled,
    }

    #[ink(storage)]
    pub struct Marketplace {
        listings: Mapping<u32, (AccountId, u128, Status)>, // tokenId → (seller, price, status)
    }

    impl Marketplace {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                listings: Mapping::default(),
            }
        }

        #[ink(message)]
        pub fn list(&mut self, token_id: u32, price: u128) {
            let caller = self.env().caller();
            self.listings.insert(token_id, &(caller, price, Status::Listed));
        }

        #[ink(message, payable)]
        pub fn buy(&mut self, token_id: u32) {
            if let Some((seller, price, status)) = self.listings.get(token_id) {
                assert!(status == Status::Listed, "Not available");
                let buyer = self.env().caller();
                let value = self.env().transferred_value();
                assert!(value >= price, "Insufficient funds");

                // Transfer logic would go here (NFT ownership transfer, royalty distribution)
                self.listings.insert(token_id, &(seller, price, Status::Sold));
            }
        }

        #[ink(message)]
        pub fn cancel(&mut self, token_id: u32) {
            if let Some((seller, price, _)) = self.listings.get(token_id) {
                let caller = self.env().caller();
                assert!(caller == seller, "Only seller can cancel");
                self.listings.insert(token_id, &(seller, price, Status::Cancelled));
            }
        }
    }
}
