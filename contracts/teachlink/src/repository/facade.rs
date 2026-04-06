//! Storage Facade
//! 
//! This module provides a unified facade for accessing all repositories.
//! It serves as a single entry point for storage operations, simplifying
//! dependency injection and making the API more ergonomic.

use crate::repository::bridge_repository::BridgeRepository;
use crate::repository::escrow_repository::EscrowAggregateRepository;
use soroban_sdk::Env;

/// Storage Facade - Single entry point for all storage operations
/// 
/// This struct provides access to all domain repositories through
/// a single interface, following the facade pattern.
/// 
/// # Example
/// 
/// ```rust,no_run
/// use crate::repository::facade::StorageFacade;
/// 
/// pub fn business_logic(env: &Env) {
///     let storage = StorageFacade::new(env);
///     
///     // Access different repositories through facade
///     let token = storage.bridge().config.get_token()?;
///     let escrow = storage.escrow().escrows.get_escrow(1);
/// }
/// ```
pub struct StorageFacade<'a> {
    env: &'a Env,
}

impl<'a> StorageFacade<'a> {
    /// Create a new storage facade
    pub fn new(env: &'a Env) -> Self {
        Self { env }
    }
    
    /// Get bridge repository
    pub fn bridge(&'a self) -> BridgeRepository<'a> {
        BridgeRepository::new(self.env)
    }
    
    /// Get escrow repository
    pub fn escrow(&'a self) -> EscrowAggregateRepository<'a> {
        EscrowAggregateRepository::new(self.env)
    }
    
    /// Get the environment reference
    pub fn env(&self) -> &Env {
        self.env
    }
}

/// Builder for creating storage components with specific configurations
pub struct StorageBuilder<'a> {
    env: &'a Env,
}

impl<'a> StorageBuilder<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self { env }
    }
    
    /// Build a storage facade
    pub fn build(self) -> StorageFacade<'a> {
        StorageFacade::new(self.env)
    }
    
    /// Build a specific repository directly
    pub fn bridge_repository(self) -> BridgeRepository<'a> {
        BridgeRepository::new(self.env)
    }
    
    /// Build escrow repository directly
    pub fn escrow_repository(self) -> EscrowAggregateRepository<'a> {
        EscrowAggregateRepository::new(self.env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_storage_facade_creation() {
        let env = Env::default();
        let facade = StorageFacade::new(&env);
        
        // Should be able to access repositories
        let _bridge = facade.bridge();
        let _escrow = facade.escrow();
        
        // Environment should be accessible
        assert_eq!(facade.env(), &env);
    }
    
    #[test]
    fn test_storage_builder() {
        let env = Env::default();
        let builder = StorageBuilder::new(&env);
        
        // Build facade
        let _facade = builder.build();
        
        // Or build repositories directly
        let _bridge = StorageBuilder::new(&env).bridge_repository();
        let _escrow = StorageBuilder::new(&env).escrow_repository();
    }
}
