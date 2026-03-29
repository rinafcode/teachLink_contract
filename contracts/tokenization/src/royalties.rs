#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod royalty_manager {
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct RoyaltyManager {
        shares: Mapping<u32, Vec<(AccountId, u8)>>, // tokenId → [(recipient, percentage)]
    }

    impl RoyaltyManager {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                shares: Mapping::default(),
            }
        }

        #[ink(message)]
        pub fn set_shares(&mut self, token_id: u32, recipients: Vec<(AccountId, u8)>) {
            self.shares.insert(token_id, &recipients);
        }

        #[ink(message)]
        pub fn distribute(&self, token_id: u32, amount: u128) {
            if let Some(recipients) = self.shares.get(token_id) {
                for (recipient, pct) in recipients {
                    let payout = amount * pct as u128 / 100;
                    self.env().transfer(recipient, payout).unwrap();
                }
            }
        }
    }
}
