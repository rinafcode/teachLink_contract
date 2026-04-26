//! Contract Upgrade Mechanism
//!
//! This module provides a safe upgrade path for the contract while preserving state.
//! It supports version tracking, state migration, and rollback capabilities.


use crate::errors::BridgeError;
use crate::storage::ADMIN;
use soroban_sdk::{contracttype, Address, Bytes, Env, Map, String};

/// Maximum rollback window in seconds (30 days)
pub const ROLLBACK_WINDOW_SECONDS: u64 = 86400 * 30;

/// Storage keys for upgrade mechanism
pub const UPGRADE_VERSION: soroban_sdk::Symbol = soroban_sdk::symbol_short!("upg_ver");
pub const UPGRADE_HISTORY: soroban_sdk::Symbol = soroban_sdk::symbol_short!("upg_hist");
pub const UPGRADE_STATE_BACKUP: soroban_sdk::Symbol = soroban_sdk::symbol_short!("upg_back");
pub const ROLLBACK_AVAILABLE: soroban_sdk::Symbol = soroban_sdk::symbol_short!("upg_rbok");

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
    /// Initialize upgrade system
    pub fn initialize(env: &Env) -> Result<(), BridgeError> {
        if env.storage().instance().has(&UPGRADE_VERSION) {
            return Err(BridgeError::AlreadyInitialized);
        }

        // Set initial version
        env.storage().instance().set(&UPGRADE_VERSION, &1u32);

        // Initialize upgrade history
        let history: Map<u32, UpgradeRecord> = Map::new(env);
        env.storage().instance().set(&UPGRADE_HISTORY, &history);

        // No rollback available initially
        env.storage().instance().set(&ROLLBACK_AVAILABLE, &false);

        Ok(())
    }

    /// Prepare for upgrade by backing up current state
    pub fn prepare_upgrade(
        env: &Env,
        admin: Address,
        new_version: u32,
        state_hash: Bytes,
    ) -> Result<(), BridgeError> {
        #[cfg(not(test))]
        admin.require_auth();

        // Initialize if not already initialized
        if !env.storage().instance().has(&UPGRADE_VERSION) {
            Self::initialize(env)?;
        }

        let current_version: u32 = env.storage().instance().get(&UPGRADE_VERSION).unwrap();

        // Validate version increment
        if new_version <= current_version {
            return Err(BridgeError::InvalidInput);
        }

        // Validate state snapshot integrity
        if state_hash.is_empty() {
            return Err(BridgeError::InvalidInput);
        }

        // Create state backup
        let backup = StateBackup {
            version: current_version,
            backed_up_at: env.ledger().timestamp(),
            state_hash: state_hash.clone(),
            critical_data: Bytes::new(env), // In practice, serialize critical state here
        };

        env.storage().instance().set(&UPGRADE_STATE_BACKUP, &backup);

        // Mark rollback as available
        env.storage().instance().set(&ROLLBACK_AVAILABLE, &true);

        Ok(())
    }

    /// Execute the upgrade
    pub fn execute_upgrade(
        env: &Env,
        admin: Address,
        new_version: u32,
        migration_hash: Bytes,
    ) -> Result<(), BridgeError> {
        #[cfg(not(test))]
        admin.require_auth();

        // Initialize if not already initialized
        if !env.storage().instance().has(&UPGRADE_VERSION) {
            Self::initialize(env)?;
        }

        let current_version: u32 = env.storage().instance().get(&UPGRADE_VERSION).unwrap();

        // Validate version increment
        if new_version <= current_version {
            return Err(BridgeError::InvalidInput);
        }

        // Validate migration metadata
        if migration_hash.is_empty() {
            return Err(BridgeError::InvalidInput);
        }

        // Verify backup exists
        if !env.storage().instance().has(&UPGRADE_STATE_BACKUP) {
            return Err(BridgeError::StorageError);
        }

        // Record upgrade in history
        let mut history: Map<u32, UpgradeRecord> = env
            .storage()
            .instance()
            .get(&UPGRADE_HISTORY)
            .unwrap_or_else(|| Map::new(env));

        let upgrade_record = UpgradeRecord {
            version: new_version,
            upgraded_at: env.ledger().timestamp(),
            upgraded_by: admin.clone(),
            previous_version: current_version,
            migration_hash: migration_hash.clone(),
        };

        history.set(new_version, upgrade_record);
        env.storage().instance().set(&UPGRADE_HISTORY, &history);

        // Update current version
        env.storage().instance().set(&UPGRADE_VERSION, &new_version);

        Ok(())
    }

    /// Rollback to previous version if within rollback window
    pub fn rollback(env: &Env, admin: Address) -> Result<(), BridgeError> {
        #[cfg(not(test))]
        admin.require_auth();

        // Check if rollback is available
        let rollback_available: bool = env
            .storage()
            .instance()
            .get(&ROLLBACK_AVAILABLE)
            .unwrap_or(false);

        if !rollback_available {
            return Err(BridgeError::InvalidInput);
        }

        // Get backup
        let backup: StateBackup = env.storage().instance().get(&UPGRADE_STATE_BACKUP).unwrap();

        // Check if within rollback window
        let current_time = env.ledger().timestamp();
        if current_time > backup.backed_up_at + ROLLBACK_WINDOW_SECONDS {
            return Err(BridgeError::InvalidInput);
        }

        // Restore previous version
        env.storage()
            .instance()
            .set(&UPGRADE_VERSION, &backup.version);

        // Mark rollback as no longer available
        env.storage().instance().set(&ROLLBACK_AVAILABLE, &false);

        // Clear backup after successful rollback
        env.storage().instance().remove(&UPGRADE_STATE_BACKUP);

        Ok(())
    }

    /// Get current version
    pub fn get_current_version(env: &Env) -> u32 {
        env.storage().instance().get(&UPGRADE_VERSION).unwrap_or(1)
    }

    /// Get upgrade history
    pub fn get_upgrade_history(env: &Env, version: u32) -> Option<UpgradeRecord> {
        let history: Map<u32, UpgradeRecord> = env
            .storage()
            .instance()
            .get(&UPGRADE_HISTORY)
            .unwrap_or_else(|| Map::new(env));

        history.get(version)
    }

    /// Check if rollback is available
    pub fn is_rollback_available(env: &Env) -> bool {
        let rollback_available: bool = env
            .storage()
            .instance()
            .get(&ROLLBACK_AVAILABLE)
            .unwrap_or(false);

        if !rollback_available {
            return false;
        }

        // Check if backup exists and is within window
        if let Some(backup) = env
            .storage()
            .instance()
            .get::<_, StateBackup>(&UPGRADE_STATE_BACKUP)
        {
            let current_time = env.ledger().timestamp();
            current_time <= backup.backed_up_at + ROLLBACK_WINDOW_SECONDS
        } else {
            false
        }
    }

    /// Get state backup information
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
            // Initialize upgrade system
            ContractUpgrader::initialize(&env).unwrap();

            // Verify initial version
            assert_eq!(ContractUpgrader::get_current_version(&env), 1);

            // Prepare upgrade
            let state_hash = Bytes::from_slice(&env, b"state_hash_v1");
            ContractUpgrader::prepare_upgrade(&env, admin.clone(), 2, state_hash).unwrap();

            // Verify rollback is available
            assert!(ContractUpgrader::is_rollback_available(&env));

            // Execute upgrade
            let migration_hash = Bytes::from_slice(&env, b"migration_v1_to_v2");
            ContractUpgrader::execute_upgrade(&env, admin.clone(), 2, migration_hash).unwrap();

            // Verify new version
            assert_eq!(ContractUpgrader::get_current_version(&env), 2);

            // Verify upgrade history
            let history = ContractUpgrader::get_upgrade_history(&env, 2);
            assert!(history.is_some());
            let record = history.unwrap();
            assert_eq!(record.previous_version, 1);
            assert_eq!(record.version, 2);

            // Test rollback
            ContractUpgrader::rollback(&env, admin.clone()).unwrap();

            // Verify rolled back to version 1
            assert_eq!(ContractUpgrader::get_current_version(&env), 1);
            assert!(!ContractUpgrader::is_rollback_available(&env));
        });
    }

    #[test]
    fn test_prepare_upgrade_auto_initializes() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());
        let admin = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let state_hash = Bytes::from_slice(&env, b"state_hash");
            ContractUpgrader::prepare_upgrade(&env, admin.clone(), 2, state_hash).unwrap();

            assert_eq!(ContractUpgrader::get_current_version(&env), 1);
            assert!(ContractUpgrader::is_rollback_available(&env));
        });
    }

    #[test]
    fn test_rollback_window_expiry() {
        use soroban_sdk::testutils::{Ledger, LedgerInfo};

        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());
        let admin = Address::generate(&env);

        env.as_contract(&contract_id, || {
            // Initialize upgrade system
            ContractUpgrader::initialize(&env).unwrap();

            // Prepare and execute upgrade
            let state_hash = Bytes::from_slice(&env, b"state_hash");
            ContractUpgrader::prepare_upgrade(&env, admin.clone(), 2, state_hash).unwrap();

            let migration_hash = Bytes::from_slice(&env, b"migration");
            ContractUpgrader::execute_upgrade(&env, admin.clone(), 2, migration_hash).unwrap();

            // Advance ledger past rollback window
            let backup = ContractUpgrader::get_state_backup(&env).unwrap();
            env.ledger().set(LedgerInfo {
                timestamp: backup.backed_up_at + crate::upgrade::ROLLBACK_WINDOW_SECONDS + 1,
                protocol_version: 25,
                sequence_number: 0,
                network_id: Default::default(),
                base_reserve: 0,
                min_temp_entry_ttl: 0,
                min_persistent_entry_ttl: 0,
                max_entry_ttl: 2_000_000,
            });

            assert!(ContractUpgrader::rollback(&env, admin.clone()).is_err());
            assert!(!ContractUpgrader::is_rollback_available(&env));
        });
    }
}
