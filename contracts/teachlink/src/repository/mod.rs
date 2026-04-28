//! Repository Pattern Implementation
//!
//! This module provides a storage abstraction layer using the repository pattern.
//! It separates data access logic from business logic, making the codebase:
//!
//! - **Testable**: Repositories can be mocked for unit tests
//! - **Maintainable**: Storage changes are isolated to repository layer
//! - **Flexible**: Easy to swap storage implementations

pub mod bridge_repository;
pub mod escrow_repository;
pub mod generic;
pub mod traits;

// Re-export for convenience
pub use bridge_repository::{
    BridgeConfigRepository, BridgeRepository, BridgeRetryRepository, BridgeTransactionRepository,
    ChainRepository, ValidatorRepository,
};
pub use escrow_repository::{
    EscrowAggregateRepository, EscrowApprovalRepository, EscrowRepository,
};
pub use generic::{
    GenericCounterRepository, GenericMapRepository, RepositoryBuilder, SingleValueRepository,
};
pub use traits::{
    CounterRepository, InstanceStorage, MapRepository, PersistentStorage, StorageError,
    TemporaryStorage,
};
