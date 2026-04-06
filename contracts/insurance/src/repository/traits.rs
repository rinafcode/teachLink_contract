//! Storage Traits for Insurance Contract
//! 
//! This module defines the storage interface contracts for the insurance contract.

use soroban_sdk::{Env, IntoVal, Val};

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

/// Trait for basic storage operations (CRUD)
pub trait StorageBackend {
    fn get<T: IntoVal<Val> + Clone>(&self, key: &T) -> Option<T>;
    fn set<K: IntoVal<Val>, V: IntoVal<Val>>(&self, key: &K, value: &V);
    fn has<K: IntoVal<Val>>(&self, key: &K) -> bool;
    fn remove<K: IntoVal<Val>>(&self, key: &K);
    fn env(&self) -> &Env;
}

/// Instance storage backend
pub struct InstanceStorage<'a> {
    env: &'a Env,
}

impl<'a> InstanceStorage<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self { env }
    }
}

impl<'a> StorageBackend for InstanceStorage<'a> {
    fn get<T: IntoVal<Val> + Clone>(&self, key: &T) -> Option<T> {
        self.env.storage().instance().get(key)
    }
    
    fn set<K: IntoVal<Val>, V: IntoVal<Val>>(&self, key: &K, value: &V) {
        self.env.storage().instance().set(key, value);
    }
    
    fn has<K: IntoVal<Val>>(&self, key: &K) -> bool {
        self.env.storage().instance().has(key)
    }
    
    fn remove<K: IntoVal<Val>>(&self, key: &K) {
        self.env.storage().instance().remove(key)
    }
    
    fn env(&self) -> &Env {
        self.env
    }
}

/// Persistent storage backend
pub struct PersistentStorage<'a> {
    env: &'a Env,
}

impl<'a> PersistentStorage<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self { env }
    }
}

impl<'a> StorageBackend for PersistentStorage<'a> {
    fn get<T: IntoVal<Val> + Clone>(&self, key: &T) -> Option<T> {
        self.env.storage().persistent().get(key)
    }
    
    fn set<K: IntoVal<Val>, V: IntoVal<Val>>(&self, key: &K, value: &V) {
        self.env.storage().persistent().set(key, value);
    }
    
    fn has<K: IntoVal<Val>>(&self, key: &K) -> bool {
        self.env.storage().persistent().has(key)
    }
    
    fn remove<K: IntoVal<Val>>(&self, key: &K) {
        self.env.storage().persistent().remove(key)
    }
    
    fn env(&self) -> &Env {
        self.env
    }
}

/// Repository trait for entity-specific operations
pub trait Repository<K, V> {
    type Error;
    fn find(&self, key: &K) -> Result<Option<V>, Self::Error>;
    fn save(&self, key: &K, value: &V) -> Result<(), Self::Error>;
    fn delete(&self, key: &K) -> Result<(), Self::Error>;
    fn exists(&self, key: &K) -> Result<bool, Self::Error>;
}

/// Counter repository trait
pub trait CounterRepository {
    type Error;
    fn get(&self) -> Result<u64, Self::Error>;
    fn increment(&self) -> Result<u64, Self::Error>;
    fn reset(&self) -> Result<(), Self::Error>;
}

/// Map repository trait
pub trait MapRepository<K, V> {
    type Error;
    fn get(&self, key: &K) -> Result<Option<V>, Self::Error>;
    fn set(&self, key: &K, value: &V) -> Result<(), Self::Error>;
    fn remove(&self, key: &K) -> Result<(), Self::Error>;
    fn contains(&self, key: &K) -> Result<bool, Self::Error>;
    fn all(&self) -> Result<soroban_sdk::Map<K, V>, Self::Error>;
}
