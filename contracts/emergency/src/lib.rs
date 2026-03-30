#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod emergency {
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct EmergencyControl {
        approvals: Mapping<AccountId, bool>,
        required_signatures: u8,
        paused: bool,
    }

    impl EmergencyControl {
        #[ink(constructor)]
        pub fn new(required_signatures: u8) -> Self {
            Self {
                approvals: Mapping::default(),
                required_signatures,
                paused: false,
            }
        }

        #[ink(message)]
        pub fn approve_pause(&mut self) {
            let caller = self.env().caller();
            self.approvals.insert(caller, &true);
        }

        #[ink(message)]
        pub fn execute_pause(&mut self) {
            let mut count = 0;
            for (account, approved) in self.approvals.iter() {
                if approved {
                    count += 1;
                }
            }
            assert!(count >= self.required_signatures, "Not enough approvals");
            self.paused = true;
        }

        #[ink(message)]
        pub fn is_paused(&self) -> bool {
            self.paused
        }
    }
}
