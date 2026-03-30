#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod governance {
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct Governance {
        pending_actions: Mapping<u32, (AccountId, u64)>, // actionId → (initiator, timestamp)
        delay: u64,
    }

    impl Governance {
        #[ink(constructor)]
        pub fn new(delay: u64) -> Self {
            Self {
                pending_actions: Mapping::default(),
                delay,
            }
        }

        #[ink(message)]
        pub fn propose_action(&mut self, action_id: u32) {
            let caller = self.env().caller();
            let now = self.env().block_timestamp();
            self.pending_actions.insert(action_id, &(caller, now));
        }

        #[ink(message)]
        pub fn execute_action(&mut self, action_id: u32) {
            if let Some((initiator, timestamp)) = self.pending_actions.get(action_id) {
                let now = self.env().block_timestamp();
                assert!(now >= timestamp + self.delay, "Action delay not met");
                // Execute governance action here
                self.pending_actions.remove(action_id);
            }
        }
    }
}
