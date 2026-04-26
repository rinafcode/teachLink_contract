use crate::errors::BridgeError;
use crate::storage::{ADMIN, INTERFACE_VERSION, MIN_COMPAT_INTERFACE_VERSION};
use crate::types::{
    ContractSemVer, DeprecatedFunction, DeprecationPolicy, InterfaceVersionStatus, MigrationPath,
};
use soroban_sdk::{symbol_short, Address, Bytes, Env, Map, Symbol, Vec};

pub const DEFAULT_INTERFACE_VERSION: ContractSemVer = ContractSemVer::new(1, 0, 0);
pub const DEFAULT_MIN_COMPAT_INTERFACE_VERSION: ContractSemVer = ContractSemVer::new(1, 0, 0);

/// Storage key for the deprecation registry
const DEPRECATION_REGISTRY: Symbol = symbol_short!("dep_reg");

/// Storage key for the migration path registry
const MIGRATION_PATHS: Symbol = symbol_short!("mig_path");

/// Storage key for version upgrade history
const VERSION_HISTORY: Symbol = symbol_short!("ver_hist");

pub struct InterfaceVersioning;

impl InterfaceVersioning {
    // -----------------------------------------------------------------------
    // Initialization
    // -----------------------------------------------------------------------

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

    // -----------------------------------------------------------------------
    // Semantic Versioning — read
    // -----------------------------------------------------------------------

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

    // -----------------------------------------------------------------------
    // Semantic Versioning — write (admin only)
    // -----------------------------------------------------------------------

    /// Update current and minimum compatible versions.
    ///
    /// Semantic versioning rules enforced:
    /// - `minimum_compatible.major` must equal `current.major` (no cross-major compat)
    /// - `minimum_compatible` must not be greater than `current`
    /// - A major bump resets `minimum_compatible` to the new major baseline
    /// - A minor bump is backward compatible; patch bumps never break compat
    pub fn set_interface_versions(
        env: &Env,
        current: ContractSemVer,
        minimum_compatible: ContractSemVer,
    ) -> Result<(), BridgeError> {
        Self::require_admin_auth(env);
        Self::validate_semver_bump(env, &current)?;
        Self::validate_range(&current, &minimum_compatible)?;

        // Record previous version in history before overwriting
        let previous = Self::get_interface_version(env);
        Self::record_version_history(env, previous, current.clone());

        env.storage().instance().set(&INTERFACE_VERSION, &current);
        env.storage()
            .instance()
            .set(&MIN_COMPAT_INTERFACE_VERSION, &minimum_compatible);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Backward Compatibility
    // -----------------------------------------------------------------------

    /// Returns true if `client_version` is within the supported compatibility
    /// window: same major, >= minimum_compatible, <= current.
    #[must_use]
    pub fn is_interface_compatible(env: &Env, client_version: ContractSemVer) -> bool {
        let status = Self::get_interface_version_status(env);

        client_version.major == status.current.major
            && !client_version.is_lower_than(&status.minimum_compatible)
            && !client_version.is_greater_than(&status.current)
    }

    /// Assert compatibility or return `IncompatibleInterfaceVersion`.
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

    /// Returns true if a minor or patch upgrade from `from` to `to` is
    /// backward compatible (same major, `to` >= `from`).
    #[must_use]
    pub fn is_backward_compatible(from: &ContractSemVer, to: &ContractSemVer) -> bool {
        from.major == to.major && !to.is_lower_than(from)
    }

    // -----------------------------------------------------------------------
    // Deprecation Policy
    // -----------------------------------------------------------------------

    /// Register a function as deprecated.
    ///
    /// - `function_name`: the symbol name of the deprecated entry point
    /// - `deprecated_in`: version where deprecation was announced
    /// - `removal_in`: version where the function will be removed
    /// - `replacement`: optional symbol name of the replacement function
    /// - `reason`: human-readable deprecation reason
    pub fn deprecate_function(
        env: &Env,
        caller: Address,
        function_name: Symbol,
        deprecated_in: ContractSemVer,
        removal_in: ContractSemVer,
        replacement: Option<Symbol>,
        reason: Bytes,
    ) -> Result<(), BridgeError> {
        caller.require_auth();
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        if caller != admin {
            return Err(BridgeError::Unauthorized);
        }

        // removal_in must be strictly greater than deprecated_in
        if !removal_in.is_greater_than(&deprecated_in) {
            return Err(BridgeError::InvalidInterfaceVersionRange);
        }

        let entry = DeprecatedFunction {
            function_name: function_name.clone(),
            deprecated_in,
            removal_in,
            replacement,
            reason,
        };

        let mut registry: Map<Symbol, DeprecatedFunction> = env
            .storage()
            .instance()
            .get(&DEPRECATION_REGISTRY)
            .unwrap_or_else(|| Map::new(env));
        registry.set(function_name, entry);
        env.storage()
            .instance()
            .set(&DEPRECATION_REGISTRY, &registry);

        Ok(())
    }

    /// Returns the deprecation record for a function, or `None` if not deprecated.
    pub fn get_deprecation(env: &Env, function_name: Symbol) -> Option<DeprecatedFunction> {
        let registry: Map<Symbol, DeprecatedFunction> = env
            .storage()
            .instance()
            .get(&DEPRECATION_REGISTRY)
            .unwrap_or_else(|| Map::new(env));
        registry.get(function_name)
    }

    /// Returns all currently deprecated functions.
    pub fn get_all_deprecations(env: &Env) -> Vec<DeprecatedFunction> {
        let registry: Map<Symbol, DeprecatedFunction> = env
            .storage()
            .instance()
            .get(&DEPRECATION_REGISTRY)
            .unwrap_or_else(|| Map::new(env));
        let mut result = Vec::new(env);
        for (_, entry) in registry.iter() {
            result.push_back(entry);
        }
        result
    }

    /// Returns the full deprecation policy: current version + all deprecations.
    pub fn get_deprecation_policy(env: &Env) -> DeprecationPolicy {
        DeprecationPolicy {
            current_version: Self::get_interface_version(env),
            deprecated_functions: Self::get_all_deprecations(env),
        }
    }

    // -----------------------------------------------------------------------
    // Migration Paths
    // -----------------------------------------------------------------------

    /// Register a migration path between two versions.
    ///
    /// A migration path documents what callers must do to upgrade from
    /// `from_version` to `to_version`.
    pub fn register_migration_path(
        env: &Env,
        caller: Address,
        from_version: ContractSemVer,
        to_version: ContractSemVer,
        description: Bytes,
        breaking_changes: Vec<Bytes>,
        migration_steps: Vec<Bytes>,
    ) -> Result<(), BridgeError> {
        caller.require_auth();
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        if caller != admin {
            return Err(BridgeError::Unauthorized);
        }

        if !to_version.is_greater_than(&from_version) {
            return Err(BridgeError::InvalidInterfaceVersionRange);
        }

        let path = MigrationPath {
            from_version: from_version.clone(),
            to_version: to_version.clone(),
            description,
            breaking_changes,
            migration_steps,
        };

        // Key: (from_major, from_minor, from_patch, to_major, to_minor, to_patch)
        // Encoded as a u128 for compact storage key
        let key = Self::migration_key(&from_version, &to_version);

        let mut paths: Map<u128, MigrationPath> = env
            .storage()
            .instance()
            .get(&MIGRATION_PATHS)
            .unwrap_or_else(|| Map::new(env));
        paths.set(key, path);
        env.storage().instance().set(&MIGRATION_PATHS, &paths);

        Ok(())
    }

    /// Retrieve the migration path between two specific versions.
    pub fn get_migration_path(
        env: &Env,
        from_version: ContractSemVer,
        to_version: ContractSemVer,
    ) -> Option<MigrationPath> {
        let paths: Map<u128, MigrationPath> = env
            .storage()
            .instance()
            .get(&MIGRATION_PATHS)
            .unwrap_or_else(|| Map::new(env));
        let key = Self::migration_key(&from_version, &to_version);
        paths.get(key)
    }

    /// Return all registered migration paths.
    pub fn get_all_migration_paths(env: &Env) -> Vec<MigrationPath> {
        let paths: Map<u128, MigrationPath> = env
            .storage()
            .instance()
            .get(&MIGRATION_PATHS)
            .unwrap_or_else(|| Map::new(env));
        let mut result = Vec::new(env);
        for (_, path) in paths.iter() {
            result.push_back(path);
        }
        result
    }

    // -----------------------------------------------------------------------
    // Version History
    // -----------------------------------------------------------------------

    /// Return the full version upgrade history (oldest first).
    pub fn get_version_history(env: &Env) -> Vec<ContractSemVer> {
        env.storage()
            .instance()
            .get(&VERSION_HISTORY)
            .unwrap_or_else(|| Vec::new(env))
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn require_admin_auth(env: &Env) {
        // SAFETY: ADMIN is always set during contract initialization
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();
    }

    /// Enforce that a new version is strictly greater than the current one.
    fn validate_semver_bump(env: &Env, new_version: &ContractSemVer) -> Result<(), BridgeError> {
        let current = Self::get_interface_version(env);
        if !new_version.is_greater_than(&current) {
            return Err(BridgeError::InvalidInterfaceVersionRange);
        }
        Ok(())
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

    /// Append `previous` to the version history log before a version bump.
    fn record_version_history(env: &Env, previous: ContractSemVer, _new: ContractSemVer) {
        let mut history: Vec<ContractSemVer> = env
            .storage()
            .instance()
            .get(&VERSION_HISTORY)
            .unwrap_or_else(|| Vec::new(env));
        history.push_back(previous);
        env.storage().instance().set(&VERSION_HISTORY, &history);
    }

    /// Encode a (from, to) version pair as a single u128 key.
    /// Layout: [from_major(16) | from_minor(16) | from_patch(16) | to_major(16) | to_minor(16) | to_patch(16)]
    fn migration_key(from: &ContractSemVer, to: &ContractSemVer) -> u128 {
        ((from.major as u128) << 80)
            | ((from.minor as u128) << 64)
            | ((from.patch as u128) << 48)
            | ((to.major as u128) << 32)
            | ((to.minor as u128) << 16)
            | (to.patch as u128)
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
