//! Generic Repository Implementations
//! 
//! This module provides generic repository implementations that can be used
//! across different domains. These repositories encapsulate common storage
//! patterns like counters, maps, and single-value storage.

use crate::repository::traits::{CounterRepository, MapRepository, StorageError};
use crate::repository::traits::{InstanceStorage, PersistentStorage, TemporaryStorage};
use soroban_sdk::{IntoVal, Map, Env, Val, TryFromVal};
use core::marker::PhantomData;

/// Generic repository for single-value storage
pub struct SingleValueRepository<'a, K, V> {
    storage: InstanceStorage<'a>,
    _key: PhantomData<K>,
    _value: PhantomData<V>,
}

impl<'a, K, V> SingleValueRepository<'a, K, V>
where
    K: IntoVal<Env, Val> + Clone,
    V: IntoVal<Env, Val> + TryFromVal<Env, Val> + Clone,
{
    pub fn new(storage: InstanceStorage<'a>) -> Self {
        Self {
            storage,
            _key: PhantomData,
            _value: PhantomData,
        }
    }
    
    /// Get the stored value
    pub fn get(&self, key: &K) -> Result<Option<V>, StorageError> {
        Ok(self.storage.get::<K, V>(key))
    }
    
    /// Set the stored value
    pub fn set(&self, key: &K, value: &V) -> Result<(), StorageError> {
        self.storage.set(key, value);
        Ok(())
    }
    
    /// Check if value exists
    pub fn exists(&self, key: &K) -> Result<bool, StorageError> {
        Ok(self.storage.has(key))
    }
    
    /// Remove the stored value
    pub fn remove(&self, key: &K) -> Result<(), StorageError> {
        self.storage.remove(key);
        Ok(())
    }
    
    /// Get or insert with default value
    pub fn get_or_insert<F>(&self, key: &K, default: F) -> Result<V, StorageError>
    where
        F: FnOnce() -> V,
    {
        if let Some(value) = self.storage.get::<K, V>(key) {
            Ok(value)
        } else {
            let value = default();
            self.storage.set(key, &value);
            Ok(value)
        }
    }
}

/// Generic repository for counter operations
pub struct GenericCounterRepository<'a, K> {
    storage: InstanceStorage<'a>,
    key: K,
}

impl<'a, K> GenericCounterRepository<'a, K>
where
    K: IntoVal<Env, Val> + Clone,
{
    pub fn new(storage: InstanceStorage<'a>, key: K) -> Self {
        Self { storage, key }
    }
    
    /// Get current counter value
    pub fn get(&self) -> Result<u64, StorageError> {
        Ok(self.storage.get::<K, u64>(&self.key).unwrap_or(0))
    }
    
    /// Increment counter and return new value
    pub fn increment(&self) -> Result<u64, StorageError> {
        let current = self.get()?;
        let new_value = current + 1;
        self.storage.set(&self.key, &new_value);
        Ok(new_value)
    }
    
    /// Increment counter by a specific amount
    pub fn increment_by(&self, amount: u64) -> Result<u64, StorageError> {
        let current = self.get()?;
        let new_value = current + amount;
        self.storage.set(&self.key, &new_value);
        Ok(new_value)
    }
    
    /// Decrement counter and return new value
    pub fn decrement(&self) -> Result<u64, StorageError> {
        let current = self.get()?;
        let new_value = current.saturating_sub(1);
        self.storage.set(&self.key, &new_value);
        Ok(new_value)
    }
    
    /// Reset counter to zero
    pub fn reset(&self) -> Result<(), StorageError> {
        self.storage.set(&self.key, &0u64);
        Ok(())
    }
    
    /// Set counter to specific value
    pub fn set(&self, value: u64) -> Result<(), StorageError> {
        self.storage.set(&self.key, &value);
        Ok(())
    }
}

impl<'a, K> CounterRepository for GenericCounterRepository<'a, K>
where
    K: IntoVal<Env, Val> + Clone,
{
    type Error = StorageError;
    
    fn get(&self) -> Result<u64, Self::Error> {
        self.get()
    }
    
    fn increment(&self) -> Result<u64, Self::Error> {
        self.increment()
    }
    
    fn reset(&self) -> Result<(), Self::Error> {
        self.reset()
    }
}

/// Generic repository for map operations
pub struct GenericMapRepository<'a, K, V> {
    storage: InstanceStorage<'a>,
    key: K,
    _value: PhantomData<V>,
}

impl<'a, K, V> GenericMapRepository<'a, K, V>
where
    K: IntoVal<Env, Val> + TryFromVal<Env, Val> + Clone,
    V: IntoVal<Env, Val> + TryFromVal<Env, Val> + Clone,
{
    pub fn new(storage: InstanceStorage<'a>, key: K) -> Self {
        Self {
            storage,
            key,
            _value: PhantomData,
        }
    }
    
    /// Get the entire map
    fn get_map(&self) -> Map<K, V> {
        self.storage.get::<K, Map<K, V>>(&self.key).unwrap_or_else(|| Map::new(self.storage.env()))
    }
    
    /// Get value from map
    pub fn get(&self, map_key: &K) -> Result<Option<V>, StorageError> {
        let map = self.get_map();
        Ok(map.get(map_key.clone()))
    }
    
    /// Set value in map
    pub fn set(&self, map_key: &K, value: &V) -> Result<(), StorageError> {
        let mut map = self.get_map();
        map.set(map_key.clone(), value.clone());
        self.storage.set(&self.key, &map);
        Ok(())
    }
    
    /// Remove value from map
    pub fn remove(&self, map_key: &K) -> Result<(), StorageError> {
        let mut map = self.get_map();
        if map.contains_key(map_key.clone()) {
            map.remove(map_key.clone());
            self.storage.set(&self.key, &map);
        }
        Ok(())
    }
    
    /// Check if key exists in map
    pub fn contains(&self, map_key: &K) -> Result<bool, StorageError> {
        let map = self.get_map();
        Ok(map.contains_key(map_key.clone()))
    }
    
    /// Get all entries
    pub fn all(&self) -> Result<Map<K, V>, StorageError> {
        Ok(self.get_map())
    }
    
    /// Get map length
    pub fn len(&self) -> Result<u32, StorageError> {
        Ok(self.get_map().len())
    }
    
    /// Check if map is empty
    pub fn is_empty(&self) -> Result<bool, StorageError> {
        Ok(self.get_map().is_empty())
    }
}

impl<'a, K, V> MapRepository<K, V> for GenericMapRepository<'a, K, V>
where
    K: IntoVal<Env, Val> + TryFromVal<Env, Val> + Clone,
    V: IntoVal<Env, Val> + TryFromVal<Env, Val> + Clone,
{
    type Error = StorageError;
    
    fn get(&self, key: &K) -> Result<Option<V>, Self::Error> {
        self.get(key)
    }
    
    fn set(&self, key: &K, value: &V) -> Result<(), Self::Error> {
        self.set(key, value)
    }
    
    fn remove(&self, key: &K) -> Result<(), Self::Error> {
        self.remove(key)
    }
    
    fn contains(&self, key: &K) -> Result<bool, Self::Error> {
        self.contains(key)
    }
    
    fn all(&self) -> Result<Map<K, V>, Self::Error> {
        self.all()
    }
}

/// Builder for creating repositories with specific storage backends
pub struct RepositoryBuilder<'a> {
    env: &'a soroban_sdk::Env,
}

impl<'a> RepositoryBuilder<'a> {
    pub fn new(env: &'a soroban_sdk::Env) -> Self {
        Self { env }
    }
    
    /// Create an instance storage backend
    pub fn instance(&'a self) -> InstanceStorage<'a> {
        InstanceStorage::new(self.env)
    }
    
    /// Create a persistent storage backend
    pub fn persistent(&'a self) -> PersistentStorage<'a> {
        PersistentStorage::new(self.env)
    }
    
    /// Create a temporary storage backend
    pub fn temporary(&'a self) -> TemporaryStorage<'a> {
        TemporaryStorage::new(self.env)
    }
    
    /// Create a single value repository with instance storage
    pub fn single_value_instance<K, V>(&'a self) -> SingleValueRepository<'a, K, V>
    where
        K: IntoVal<Env, Val> + Clone,
        V: IntoVal<Env, Val> + TryFromVal<Env, Val> + Clone,
    {
        SingleValueRepository::new(self.instance())
    }
    
    /// Create a counter repository with instance storage
    pub fn counter_instance<K>(&'a self, key: K) -> GenericCounterRepository<'a, K>
    where
        K: IntoVal<Env, Val> + Clone,
    {
        GenericCounterRepository::new(self.instance(), key)
    }
    
    /// Create a map repository with instance storage
    pub fn map_instance<K, V>(&'a self, key: K) -> GenericMapRepository<'a, K, V>
    where
        K: IntoVal<Env, Val> + TryFromVal<Env, Val> + Clone,
        V: IntoVal<Env, Val> + TryFromVal<Env, Val> + Clone,
    {
        GenericMapRepository::new(self.instance(), key)
    }
}
