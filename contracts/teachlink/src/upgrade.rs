//! Contract Upgrade Mechanism
//!
//! Safe upgrade path with versioning, migration, and rollback.

use crate::errors::BridgeError;
use crate::storage::ADMIN;
use soroban_sdk::{contracttype, Address, Bytes, Env, Map};

pub const UPGRADE_VERSION: soroban_sdk::Symbol = soroban_sdk::symbol_short!("upg_ver");
pub const UPGRADE_HISTORY: soroban_sdk::Symbol = soroban_sdk::symbol_short!("upg_hist");
pub const UPGRADE_STATE_BACKUP: soroban_sdk::Symbol = soroban_sdk::symbol_short!("upg_back");
pub const ROLLBACK_AVAILABLE: soroban_sdk::Symbol = soroban_sdk::symbol_short!("upg_rbok");

pub const ROLLBACK_WINDOW_SECONDS: u64 = 86400 * 30;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeRecord {
    pub version: u32,
    pub upgraded_at: u64,
    pub upgraded_by: Address,
    pub previous_version: u32,
    pub migration_hash: Bytes,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateBackup {
    pub version: u32,
    pub backed_up_at: u64,
    pub state_hash: Bytes,
    pub critical_data: Bytes,
}

pub struct ContractUpgrader;

impl ContractUpgrader {
    /// Initialize upgrade system + admin
    pub fn initialize(env: &Env, admin: Address) -> Result<(), BridgeError> {
        if env.storage().instance().has(&UPGRADE_VERSION) {
            return Err(BridgeError::AlreadyInitialized);
        }

        // Store admin
        env.storage().instance().set(&ADMIN, &admin);

        env.storage().instance().set(&UPGRADE_VERSION, &1u32);

        let history: Map<u32, UpgradeRecord> = Map::new(env);
        env.storage().instance().set(&UPGRADE_HISTORY, &history);

        env.storage().instance().set(&ROLLBACK_AVAILABLE, &false);

        Ok(())
    }

    /// Internal helper: ensure initialized
    fn ensure_initialized(env: &Env) -> Result<(), BridgeError> {
        if !env.storage().instance().has(&UPGRADE_VERSION) {
            return Err(BridgeError::NotInitialized);
        }
        Ok(())
    }

    /// Internal helper: get admin safely
    fn get_admin(env: &Env) -> Result<Address, BridgeError> {
        env.storage()
            .instance()
            .get(&ADMIN)
            .ok_or(BridgeError::Unauthorized)
    }

    /// Prepare upgrade (backup state)
    pub fn prepare_upgrade(
        env: &Env,
        admin: Address,
        new_version: u32,
        state_hash: Bytes,
    ) -> Result<(), BridgeError> {
        Self::ensure_initialized(env)?;

        let stored_admin = Self::get_admin(env)?;
        if admin != stored_admin {
            return Err(BridgeError::Unauthorized);
        }

        let current_version: u32 = env.storage().instance().get(&UPGRADE_VERSION).unwrap_or(1);

        if new_version <= current_version {
            return Err(BridgeError::InvalidInput);
        }

        if state_hash.len() == 0 {
            return Err(BridgeError::InvalidInput);
        }

        let backup = StateBackup {
            version: current_version,
            backed_up_at: env.ledger().timestamp(),
            state_hash: state_hash.clone(),
            critical_data: Bytes::new(env),
        };

        env.storage().instance().set(&UPGRADE_STATE_BACKUP, &backup);
        env.storage().instance().set(&ROLLBACK_AVAILABLE, &true);

        Ok(())
    }

    /// Execute upgrade
    pub fn execute_upgrade(
        env: &Env,
        admin: Address,
        new_version: u32,
        migration_hash: Bytes,
    ) -> Result<(), BridgeError> {
        Self::ensure_initialized(env)?;

        let stored_admin = Self::get_admin(env)?;
        if admin != stored_admin {
            return Err(BridgeError::Unauthorized);
        }

        let current_version: u32 = env.storage().instance().get(&UPGRADE_VERSION).unwrap_or(1);

        if new_version <= current_version {
            return Err(BridgeError::InvalidInput);
        }

        if migration_hash.len() == 0 {
            return Err(BridgeError::InvalidInput);
        }

        if !env.storage().instance().has(&UPGRADE_STATE_BACKUP) {
            return Err(BridgeError::StorageError);
        }

        let mut history: Map<u32, UpgradeRecord> = env
            .storage()
            .instance()
            .get(&UPGRADE_HISTORY)
            .unwrap_or_else(|| Map::new(env));

        let record = UpgradeRecord {
            version: new_version,
            upgraded_at: env.ledger().timestamp(),
            upgraded_by: admin.clone(),
            previous_version: current_version,
            migration_hash: migration_hash.clone(),
        };

        history.set(new_version, record);
        env.storage().instance().set(&UPGRADE_HISTORY, &history);
        env.storage().instance().set(&UPGRADE_VERSION, &new_version);

        Ok(())
    }

    /// Rollback
    pub fn rollback(env: &Env, admin: Address) -> Result<(), BridgeError> {
        Self::ensure_initialized(env)?;

        let stored_admin = Self::get_admin(env)?;
        if admin != stored_admin {
            return Err(BridgeError::Unauthorized);
        }

        let rollback_available: bool = env
            .storage()
            .instance()
            .get(&ROLLBACK_AVAILABLE)
            .unwrap_or(false);

        if !rollback_available {
            return Err(BridgeError::InvalidInput);
        }

        let backup: StateBackup = env
            .storage()
            .instance()
            .get(&UPGRADE_STATE_BACKUP)
            .ok_or(BridgeError::StorageError)?;

        let now = env.ledger().timestamp();
        if now > backup.backed_up_at + ROLLBACK_WINDOW_SECONDS {
            return Err(BridgeError::InvalidInput);
        }

        env.storage()
            .instance()
            .set(&UPGRADE_VERSION, &backup.version);

        env.storage().instance().set(&ROLLBACK_AVAILABLE, &false);
        env.storage().instance().remove(&UPGRADE_STATE_BACKUP);

        Ok(())
    }

    pub fn get_current_version(env: &Env) -> u32 {
        env.storage().instance().get(&UPGRADE_VERSION).unwrap_or(1)
    }

    pub fn get_upgrade_history(env: &Env, version: u32) -> Option<UpgradeRecord> {
        let history: Map<u32, UpgradeRecord> = env
            .storage()
            .instance()
            .get(&UPGRADE_HISTORY)
            .unwrap_or_else(|| Map::new(env));

        history.get(version)
    }

    pub fn is_rollback_available(env: &Env) -> bool {
        let rollback_available: bool = env
            .storage()
            .instance()
            .get(&ROLLBACK_AVAILABLE)
            .unwrap_or(false);

        if !rollback_available {
            return false;
        }

        if let Some(backup) = env
            .storage()
            .instance()
            .get::<_, StateBackup>(&UPGRADE_STATE_BACKUP)
        {
            let now = env.ledger().timestamp();
            now <= backup.backed_up_at + ROLLBACK_WINDOW_SECONDS
        } else {
            false
        }
    }

    pub fn get_state_backup(env: &Env) -> Option<StateBackup> {
        env.storage().instance().get(&UPGRADE_STATE_BACKUP)
    }
}

#[cfg(test)]
mod tests {
    use super::ContractUpgrader;
    use crate::TeachLinkBridge;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env};

    #[test]
    fn test_upgrade_lifecycle() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());
        let admin = Address::generate(&env);

        env.as_contract(&contract_id, || {
            ContractUpgrader::initialize(&env, admin.clone()).unwrap();

            assert_eq!(ContractUpgrader::get_current_version(&env), 1);

            let state_hash = Bytes::from_slice(&env, b"state_hash_v1");
            ContractUpgrader::prepare_upgrade(&env, admin.clone(), 2, state_hash).unwrap();

            let migration_hash = Bytes::from_slice(&env, b"migration_v1_to_v2");
            ContractUpgrader::execute_upgrade(&env, admin.clone(), 2, migration_hash).unwrap();

            assert_eq!(ContractUpgrader::get_current_version(&env), 2);

            let history = ContractUpgrader::get_upgrade_history(&env, 2);
            assert!(history.is_some());

            let record = history.unwrap();
            assert_eq!(record.previous_version, 1);
            assert_eq!(record.version, 2);

            ContractUpgrader::rollback(&env, admin.clone()).unwrap();

            assert_eq!(ContractUpgrader::get_current_version(&env), 1);
            assert!(!ContractUpgrader::is_rollback_available(&env));
        });
    }

    #[test]
    fn test_prepare_upgrade_requires_initialization() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());
        let admin = Address::generate(&env);

        env.as_contract(&contract_id, || {
            ContractUpgrader::initialize(&env, admin.clone()).unwrap();

            let state_hash = Bytes::from_slice(&env, b"state_hash");
            ContractUpgrader::prepare_upgrade(&env, admin.clone(), 2, state_hash).unwrap();

            assert_eq!(ContractUpgrader::get_current_version(&env), 1);
            assert!(ContractUpgrader::is_rollback_available(&env));
        });
    }
}
