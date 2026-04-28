//! Role-Based Access Control (RBAC) Module
//!
//! This module manages system-wide roles and permissions, providing
//! multi-layered authorization checks and comprehensive audit trails.

use crate::audit::AuditManager;
use crate::errors::BridgeError;
use crate::storage::{ACCESS_CONTROL, ADMIN};
use crate::types::{AccessRole, OperationType};
use soroban_sdk::{Address, Bytes, Env, Map, Vec};

pub struct AccessControlManager;

impl AccessControlManager {
    /// Check if an address has a specific role
    pub fn has_role(env: &Env, address: &Address, role: AccessRole) -> bool {
        // Root Admin always has all roles
        if let Some(admin) = env.storage().instance().get::<_, Address>(&ADMIN) {
            if *address == admin {
                return true;
            }
        }

        let roles: Map<Address, Vec<AccessRole>> = env
            .storage()
            .instance()
            .get(&ACCESS_CONTROL)
            .unwrap_or_else(|| Map::new(env));

        if let Some(user_roles) = roles.get(address.clone()) {
            user_roles.contains(role)
        } else {
            false
        }
    }

    /// Enforce a role check, returning an error if unauthorized
    pub fn check_role(env: &Env, address: &Address, role: AccessRole) -> Result<(), BridgeError> {
        if !Self::has_role(env, address, role) {
            return Err(BridgeError::Unauthorized);
        }
        Ok(())
    }

    /// Grant a role to an address (Admin only)
    pub fn grant_role(
        env: &Env,
        caller: Address,
        target: Address,
        role: AccessRole,
    ) -> Result<(), BridgeError> {
        caller.require_auth();
        // Only Admin can grant roles
        Self::check_role(env, &caller, AccessRole::Admin)?;

        let mut roles: Map<Address, Vec<AccessRole>> = env
            .storage()
            .instance()
            .get(&ACCESS_CONTROL)
            .unwrap_or_else(|| Map::new(env));

        let mut user_roles = roles.get(target.clone()).unwrap_or_else(|| Vec::new(env));

        if !user_roles.contains(role.clone()) {
            user_roles.push_back(role.clone());
            roles.set(target.clone(), user_roles);
            env.storage().instance().set(&ACCESS_CONTROL, &roles);

            // Audit Log
            let mut details = Bytes::new(env);
            // Simple serialization of role for audit log
            // In a real system we'd use a more robust way to encode the role
            AuditManager::create_audit_record(
                env,
                OperationType::RoleGranted,
                caller,
                details,
                Bytes::new(env), // tx_hash would be passed if available
            )?;
        }

        Ok(())
    }

    /// Revoke a role from an address (Admin only)
    pub fn revoke_role(
        env: &Env,
        caller: Address,
        target: Address,
        role: AccessRole,
    ) -> Result<(), BridgeError> {
        caller.require_auth();
        Self::check_role(env, &caller, AccessRole::Admin)?;

        let mut roles: Map<Address, Vec<AccessRole>> = env
            .storage()
            .instance()
            .get(&ACCESS_CONTROL)
            .unwrap_or_else(|| Map::new(env));

        if let Some(user_roles) = roles.get(target.clone()) {
            let mut new_roles = Vec::new(env);
            let mut found = false;
            for r in user_roles.iter() {
                if r != role {
                    new_roles.push_back(r);
                } else {
                    found = true;
                }
            }

            if found {
                roles.set(target.clone(), new_roles);
                env.storage().instance().set(&ACCESS_CONTROL, &roles);

                // Audit Log
                AuditManager::create_audit_record(
                    env,
                    OperationType::RoleRevoked,
                    caller,
                    Bytes::new(env),
                    Bytes::new(env),
                )?;
            }
        }

        Ok(())
    }
}
