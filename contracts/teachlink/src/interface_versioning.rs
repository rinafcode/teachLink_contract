use crate::errors::BridgeError;
use crate::storage::{ADMIN, INTERFACE_VERSION, MIN_COMPAT_INTERFACE_VERSION};
use crate::types::{ContractSemVer, InterfaceVersionStatus};
use soroban_sdk::{Address, Env};

pub const DEFAULT_INTERFACE_VERSION: ContractSemVer = ContractSemVer::new(1, 0, 0);
pub const DEFAULT_MIN_COMPAT_INTERFACE_VERSION: ContractSemVer = ContractSemVer::new(1, 0, 0);

pub struct InterfaceVersioning;

impl InterfaceVersioning {
    pub fn initialize(env: &Env) {
        if !env.storage().instance().has(&INTERFACE_VERSION) {
            env.storage()
                .instance()
                .set(&INTERFACE_VERSION, &DEFAULT_INTERFACE_VERSION);
        }

        if !env.storage().instance().has(&MIN_COMPAT_INTERFACE_VERSION) {
            env.storage().instance().set(
                &MIN_COMPAT_INTERFACE_VERSION,
                &DEFAULT_MIN_COMPAT_INTERFACE_VERSION,
            );
        }
    }

    pub fn get_interface_version(env: &Env) -> ContractSemVer {
        env.storage()
            .instance()
            .get(&INTERFACE_VERSION)
            .unwrap_or(DEFAULT_INTERFACE_VERSION)
    }

    pub fn get_minimum_compatible_interface_version(env: &Env) -> ContractSemVer {
        env.storage()
            .instance()
            .get(&MIN_COMPAT_INTERFACE_VERSION)
            .unwrap_or(DEFAULT_MIN_COMPAT_INTERFACE_VERSION)
    }

    pub fn get_interface_version_status(env: &Env) -> InterfaceVersionStatus {
        InterfaceVersionStatus {
            current: Self::get_interface_version(env),
            minimum_compatible: Self::get_minimum_compatible_interface_version(env),
        }
    }

    pub fn set_interface_versions(
        env: &Env,
        current: ContractSemVer,
        minimum_compatible: ContractSemVer,
    ) -> Result<(), BridgeError> {
        Self::require_admin_auth(env);
        Self::validate_range(&current, &minimum_compatible)?;

        env.storage().instance().set(&INTERFACE_VERSION, &current);
        env.storage()
            .instance()
            .set(&MIN_COMPAT_INTERFACE_VERSION, &minimum_compatible);

        Ok(())
    }

    #[must_use]
    pub fn is_interface_compatible(env: &Env, client_version: ContractSemVer) -> bool {
        let status = Self::get_interface_version_status(env);

        client_version.major == status.current.major
            && !client_version.is_lower_than(&status.minimum_compatible)
            && !client_version.is_greater_than(&status.current)
    }

    pub fn assert_interface_compatible(
        env: &Env,
        client_version: ContractSemVer,
    ) -> Result<(), BridgeError> {
        if Self::is_interface_compatible(env, client_version) {
            Ok(())
        } else {
            Err(BridgeError::IncompatibleInterfaceVersion)
        }
    }

    fn require_admin_auth(env: &Env) {
        // SAFETY: ADMIN is always set during contract initialization
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();
    }

    fn validate_range(
        current: &ContractSemVer,
        minimum_compatible: &ContractSemVer,
    ) -> Result<(), BridgeError> {
        if minimum_compatible.major != current.major {
            return Err(BridgeError::InvalidInterfaceVersionRange);
        }

        if minimum_compatible.is_greater_than(current) {
            return Err(BridgeError::InvalidInterfaceVersionRange);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{InterfaceVersioning, DEFAULT_INTERFACE_VERSION};
    use crate::types::ContractSemVer;
    use crate::TeachLinkBridge;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    #[test]
    fn compatibility_requires_same_major_and_supported_range() {
        let env = Env::default();
        let contract_id = env.register(TeachLinkBridge, ());

        let token_admin = Address::generate(&env);
        let sac = env.register_stellar_asset_contract_v2(token_admin);
        let token = sac.address();
        let admin = Address::generate(&env);
        let fee_recipient = Address::generate(&env);

        let client = crate::TeachLinkBridgeClient::new(&env, &contract_id);
        env.mock_all_auths();
        client.initialize(&token, &admin, &1, &fee_recipient);

        assert!(env.as_contract(&contract_id, || {
            InterfaceVersioning::is_interface_compatible(&env, ContractSemVer::new(1, 0, 0))
        }));
        assert!(!env.as_contract(&contract_id, || {
            InterfaceVersioning::is_interface_compatible(&env, ContractSemVer::new(2, 0, 0))
        }));
    }

    #[test]
    fn defaults_are_available_before_explicit_initialization() {
        let env = Env::default();
        let contract_id = env.register(TeachLinkBridge, ());

        let version = env.as_contract(&contract_id, || {
            InterfaceVersioning::get_interface_version(&env)
        });

        assert_eq!(version, DEFAULT_INTERFACE_VERSION);
    }
}
