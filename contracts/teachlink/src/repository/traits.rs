//! Storage Abstraction Layer
//! 
//! This module provides a repository pattern abstraction over Soroban's storage API.
//! It hides storage implementation details from business logic and provides
//! testable interfaces for data access.
//! 
//! # Architecture
//! 
//! - **Traits**: Define storage interface contracts
//! - **Repositories**: Implement data access logic for specific domains
//! - **Storage Backend**: Encapsulates Soroban storage API

use soroban_sdk::{Env, IntoVal, Val, TryFromVal};

/// Error type for storage operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    NotFound,
    AlreadyExists,
    SerializationError,
    DeserializationError,
    Unauthorized,
    InvalidKey,
    StorageFull,
}

/// Instance storage backend (temporary storage)
#[derive(Clone)]
pub struct InstanceStorage<'a> {
    env: &'a Env,
}

impl<'a> InstanceStorage<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self { env }
    }
    
    /// Get a value from storage by key
    pub fn get<T: Clone, V>(&self, key: &T) -> Option<V>
    where
        T: IntoVal<Env, Val>,
        V: IntoVal<Env, Val> + TryFromVal<Env, Val> + Clone,
    {
        self.env.storage().instance().get(key)
    }
    
    /// Set a value in storage
    pub fn set<K, V>(&self, key: &K, value: &V)
    where
        K: IntoVal<Env, Val>,
        V: IntoVal<Env, Val>,
    {
        self.env.storage().instance().set(key, value);
    }
    
    /// Check if a key exists in storage
    pub fn has<K>(&self, key: &K) -> bool
    where
        K: IntoVal<Env, Val>,
    {
        self.env.storage().instance().has(key)
    }
    
    /// Remove a value from storage
    pub fn remove<K>(&self, key: &K)
    where
        K: IntoVal<Env, Val>,
    {
        self.env.storage().instance().remove(key);
    }
    
    /// Get the environment reference
    pub fn env(&self) -> &Env {
        self.env
    }
}

/// Persistent storage backend (permanent storage)
pub struct PersistentStorage<'a> {
    env: &'a Env,
}

impl<'a> PersistentStorage<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self { env }
    }
    
    /// Get a value from storage by key
    pub fn get<T: Clone, V>(&self, key: &T) -> Option<V>
    where
        T: IntoVal<Env, Val>,
        V: IntoVal<Env, Val> + TryFromVal<Env, Val> + Clone,
    {
        self.env.storage().persistent().get(key)
    }
    
    /// Set a value in storage
    pub fn set<K, V>(&self, key: &K, value: &V)
    where
        K: IntoVal<Env, Val>,
        V: IntoVal<Env, Val>,
    {
        self.env.storage().persistent().set(key, value);
    }
    
    /// Check if a key exists in storage
    pub fn has<K>(&self, key: &K) -> bool
    where
        K: IntoVal<Env, Val>,
    {
        self.env.storage().persistent().has(key)
    }
    
    /// Remove a value from storage
    pub fn remove<K>(&self, key: &K)
    where
        K: IntoVal<Env, Val>,
    {
        self.env.storage().persistent().remove(key);
    }
    
    /// Get the environment reference
    pub fn env(&self) -> &Env {
        self.env
    }
}

/// Temporary storage backend (ledger/temporary)
pub struct TemporaryStorage<'a> {
    env: &'a Env,
}

impl<'a> TemporaryStorage<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self { env }
    }
    
    /// Get a value from storage by key
    pub fn get<T: Clone, V>(&self, key: &T) -> Option<V>
    where
        T: IntoVal<Env, Val>,
        V: IntoVal<Env, Val> + TryFromVal<Env, Val> + Clone,
    {
        self.env.storage().temporary().get(key)
    }
    
    /// Set a value in storage
    pub fn set<K, V>(&self, key: &K, value: &V)
    where
        K: IntoVal<Env, Val>,
        V: IntoVal<Env, Val>,
    {
        self.env.storage().temporary().set(key, value);
    }
    
    /// Check if a key exists in storage
    pub fn has<K>(&self, key: &K) -> bool
    where
        K: IntoVal<Env, Val>,
    {
        self.env.storage().temporary().has(key)
    }
    
    /// Remove a value from storage
    pub fn remove<K>(&self, key: &K)
    where
        K: IntoVal<Env, Val>,
    {
        self.env.storage().temporary().remove(key);
    }
    
    /// Get the environment reference
    pub fn env(&self) -> &Env {
        self.env
    }
}

/// Counter repository for managing counters
pub trait CounterRepository {
    type Error;
    
    /// Get current counter value
    fn get(&self) -> Result<u64, Self::Error>;
    
    /// Increment counter and return new value
    fn increment(&self) -> Result<u64, Self::Error>;
    
    /// Reset counter to zero
    fn reset(&self) -> Result<(), Self::Error>;
}

/// Map repository for managing key-value collections
pub trait MapRepository<K, V> {
    type Error;
    
    /// Get value from map
    fn get(&self, key: &K) -> Result<Option<V>, Self::Error>;
    
    /// Set value in map
    fn set(&self, key: &K, value: &V) -> Result<(), Self::Error>;
    
    /// Remove value from map
    fn remove(&self, key: &K) -> Result<(), Self::Error>;
    
    /// Check if key exists in map
    fn contains(&self, key: &K) -> Result<bool, Self::Error>;
    
    /// Get all entries
    fn all(&self) -> Result<soroban_sdk::Map<K, V>, Self::Error>;
}
