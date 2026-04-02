//! Repository Pattern Implementation for Insurance Contract
//! 
//! This module provides a storage abstraction layer for the insurance contract
//! using the repository pattern.

pub mod traits;
pub mod generic;
pub mod insurance_repository;

// Re-export for convenience
pub use traits::{StorageBackend, StorageError, Repository, CounterRepository, MapRepository};
pub use generic::{SingleValueRepository, GenericCounterRepository, GenericMapRepository};
pub use insurance_repository::InsuranceRepository;
