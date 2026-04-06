//! Repository Pattern Implementation
//! 
//! This module provides a storage abstraction layer using the repository pattern.
//! It separates data access logic from business logic, making the codebase:
//! 
//! - **Testable**: Repositories can be mocked for unit tests
//! - **Maintainable**: Storage changes are isolated to repository layer
//! - **Flexible**: Easy to swap storage implementations
//! 
//! # Architecture
//! 
//! ```text
//! Business Logic (e.g., Bridge, Escrow)
//!         ↓
//! Repository Layer (e.g., BridgeRepository)
//!         ↓
//! Storage Backend (Instance/Persistent/Temporary)
//! ```
//! 
//! # Usage
//! 
//! ```rust,no_run
//! use crate::repository::bridge::BridgeRepository;
//! 
//! pub fn some_business_logic(env: &Env) {
//!     let bridge_repo = BridgeRepository::new(env);
//!     
//!     // Use repository instead of direct storage access
//!     let token = bridge_repo.config.get_token()?;
//!     let is_valid = bridge_repo.validators.is_validator(&address);
//! }
//! ```

pub mod traits;
pub mod generic;
pub mod bridge_repository;
pub mod escrow_repository;
pub mod facade;

#[cfg(test)]
mod tests;

// Re-export for convenience
pub use traits::{
    StorageError, CounterRepository, MapRepository,
    InstanceStorage, PersistentStorage, TemporaryStorage,
};
pub use generic::{
    SingleValueRepository, GenericCounterRepository, GenericMapRepository,
    RepositoryBuilder,
};
pub use bridge_repository::{
    BridgeRepository, BridgeConfigRepository, ValidatorRepository,
    ChainRepository, BridgeTransactionRepository, BridgeRetryRepository,
};
pub use escrow_repository::{
    EscrowRepository, EscrowApprovalRepository, EscrowAggregateRepository,
};
pub use facade::{StorageFacade, StorageBuilder};
