//! Feature Flag System
//!
//! This module manages feature flags, kill switches, and rollout strategies
//! (such as global, percentage-based, and A/B testing) for the TeachLink contract.

use crate::access_control::AccessControlManager;
use crate::errors::BridgeError;
use crate::storage::FEATURE_FLAGS;
use crate::types::{AccessRole, FeatureFlag, FeatureStatus, RolloutStrategy};
use soroban_sdk::{Address, Bytes, Env, Map, Symbol};

pub struct FeatureFlagManager;

impl FeatureFlagManager {
    /// Internal method to load all feature flags
    fn get_all_flags(env: &Env) -> Map<Symbol, FeatureFlag> {
        env.storage()
            .instance()
            .get(&FEATURE_FLAGS)
            .unwrap_or_else(|| Map::new(env))
    }

    /// Internal method to save all feature flags
    fn save_all_flags(env: &Env, flags: &Map<Symbol, FeatureFlag>) {
        env.storage().instance().set(&FEATURE_FLAGS, flags);
    }

    /// Create or update a feature flag. Requires FeatureManager or Admin role.
    pub fn set_feature_flag(
        env: &Env,
        operator: &Address,
        name: Symbol,
        status: FeatureStatus,
        strategy: RolloutStrategy,
        rollout_percentage: u32,
    ) -> Result<(), BridgeError> {
        operator.require_auth();

        if !AccessControlManager::has_role(env, operator, AccessRole::FeatureManager)
            && !AccessControlManager::has_role(env, operator, AccessRole::Admin)
        {
            return Err(BridgeError::Unauthorized);
        }

        if rollout_percentage > 100 {
            return Err(BridgeError::InvalidParameter);
        }

        let mut flags = Self::get_all_flags(env);
        let timestamp = env.ledger().timestamp();
        
        // Preserve kill_switch and created_at if updating
        let (kill_switch_enabled, created_at) = if let Some(existing) = flags.get(name.clone()) {
            (existing.kill_switch_enabled, existing.created_at)
        } else {
            (false, timestamp)
        };

        let new_flag = FeatureFlag {
            name: name.clone(),
            status,
            strategy,
            rollout_percentage,
            kill_switch_enabled,
            created_at,
            updated_at: timestamp,
        };

        flags.set(name, new_flag);
        Self::save_all_flags(env, &flags);

        Ok(())
    }

    /// Instantly disables a feature flag regardless of its rollout state.
    /// Can be called by EmergencyManager, FeatureManager, or Admin.
    pub fn trigger_kill_switch(
        env: &Env,
        operator: &Address,
        name: Symbol,
        enabled: bool,
    ) -> Result<(), BridgeError> {
        operator.require_auth();

        if !AccessControlManager::has_role(env, operator, AccessRole::EmergencyManager)
            && !AccessControlManager::has_role(env, operator, AccessRole::FeatureManager)
            && !AccessControlManager::has_role(env, operator, AccessRole::Admin)
        {
            return Err(BridgeError::Unauthorized);
        }

        let mut flags = Self::get_all_flags(env);
        let mut flag = flags.get(name.clone()).ok_or(BridgeError::NotFound)?;
        
        flag.kill_switch_enabled = enabled;
        flag.updated_at = env.ledger().timestamp();
        
        flags.set(name, flag);
        Self::save_all_flags(env, &flags);

        Ok(())
    }

    /// Get details of a specific feature flag
    pub fn get_feature_flag(env: &Env, name: Symbol) -> Option<FeatureFlag> {
        Self::get_all_flags(env).get(name)
    }

    /// Determines if a feature is enabled for a specific user.
    pub fn is_feature_enabled(env: &Env, name: Symbol, user: &Address) -> bool {
        let flags = Self::get_all_flags(env);
        let flag = match flags.get(name.clone()) {
            Some(f) => f,
            None => return false, // Features default to false if not found
        };

        if flag.kill_switch_enabled || flag.status == FeatureStatus::Disabled {
            return false;
        }

        if flag.status == FeatureStatus::Enabled {
            return true;
        }

        // Handle FeatureStatus::Rollout
        match flag.strategy {
            RolloutStrategy::Global => {
                // If Rollout and Global, it's effectively enabled for everyone 
                // up to rollout_percentage. If 100%, all pass.
                // Wait, global implies true/false based on flag status.
                // But let's treat Global as "on" if rollout > 0, for simplicity, 
                // or just use rollout percentage for everyone.
                flag.rollout_percentage == 100
            }
            RolloutStrategy::PercentageBased | RolloutStrategy::ABTest => {
                // Determine user's bucket (0-99) deterministically
                let mut data = Bytes::new(env);
                
                // Note: user.to_xdr(env) would be ideal but Bytes::from_slice with string is easier
                // For simplicity, we just use the name and user string representation
                // In a real implementation we'd use XDR or bytes from the Address type directly.
                // Address string representation can be used as unique material.
                let user_str = user.to_string();
                
                data.append(&user_str.into());
                let name_bytes: Bytes = name.to_string().into();
                data.append(&name_bytes);

                let hash = env.crypto().sha256(&data);
                
                // Get the first byte as the hash bucket (0-255)
                // Map to 0-99
                let first_byte = hash.get(0).unwrap_or(0) as u32;
                let bucket = first_byte % 100;

                bucket < flag.rollout_percentage
            }
        }
    }
}
