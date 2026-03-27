#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod access_control {
    use ink::storage::Mapping;

    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Role {
        Admin,
        Governor,
        Emergency,
    }

    #[ink(storage)]
    pub struct AccessControl {
        roles: Mapping<AccountId, Role>,
    }

    impl AccessControl {
        #[ink(constructor)]
        pub fn new(admin: AccountId) -> Self {
            let mut roles = Mapping::default();
            roles.insert(admin, &Role::Admin);
            Self { roles }
        }

        #[ink(message)]
        pub fn assign_role(&mut self, account: AccountId, role: Role) {
            let caller = self.env().caller();
            let caller_role = self.roles.get(caller).unwrap_or(Role::Governor);
            assert!(caller_role == Role::Admin, "Only Admin can assign roles");
            self.roles.insert(account, &role);
        }

        #[ink(message)]
        pub fn get_role(&self, account: AccountId) -> Option<Role> {
            self.roles.get(account)
        }

        pub fn ensure_role(&self, account: AccountId, required: Role) {
            let role = self.roles.get(account).expect("No role assigned");
            assert!(role == required, "Access denied");
        }
    }
}
