//! Generic Repository Implementations for Insurance Contract

use crate::repository::traits::{CounterRepository, MapRepository, StorageBackend, StorageError};
use soroban_sdk::{IntoVal, Map, Val};

/// Generic repository for single-value storage
pub struct SingleValueRepository<'a, K, V> {
    storage: &'a dyn StorageBackend,
    _key: std::marker::PhantomData<K>,
    _value: std::marker::PhantomData<V>,
}

impl<'a, K, V> SingleValueRepository<'a, K, V>
where
    K: IntoVal<Val> + Clone,
    V: IntoVal<Val> + Clone,
{
    pub fn new(storage: &'a dyn StorageBackend) -> Self {
        Self {
            storage,
            _key: std::marker::PhantomData,
            _value: std::marker::PhantomData,
        }
    }
    
    pub fn get(&self, key: &K) -> Result<Option<V>, StorageError> {
        Ok(self.storage.get(key))
    }
    
    pub fn set(&self, key: &K, value: &V) -> Result<(), StorageError> {
        self.storage.set(key, value);
        Ok(())
    }
    
    pub fn exists(&self, key: &K) -> Result<bool, StorageError> {
        Ok(self.storage.has(key))
    }
    
    pub fn remove(&self, key: &K) -> Result<(), StorageError> {
        self.storage.remove(key);
        Ok(())
    }
}

/// Generic counter repository
pub struct GenericCounterRepository<'a, K> {
    storage: &'a dyn StorageBackend,
    key: K,
}

impl<'a, K> GenericCounterRepository<'a, K>
where
    K: IntoVal<Val> + Clone,
{
    pub fn new(storage: &'a dyn StorageBackend, key: K) -> Self {
        Self { storage, key }
    }
    
    pub fn get(&self) -> Result<u64, StorageError> {
        Ok(self.storage.get(&self.key).unwrap_or(0))
    }
    
    pub fn increment(&self) -> Result<u64, StorageError> {
        let current = self.get()?;
        let new_value = current + 1;
        self.storage.set(&self.key, &new_value);
        Ok(new_value)
    }
    
    pub fn reset(&self) -> Result<(), StorageError> {
        self.storage.set(&self.key, &0u64);
        Ok(())
    }
}

impl<'a, K> CounterRepository for GenericCounterRepository<'a, K>
where
    K: IntoVal<Val> + Clone,
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

/// Generic map repository
pub struct GenericMapRepository<'a, K, V> {
    storage: &'a dyn StorageBackend,
    key: K,
    _value: std::marker::PhantomData<V>,
}

impl<'a, K, V> GenericMapRepository<'a, K, V>
where
    K: IntoVal<Val> + Clone,
    V: IntoVal<Val> + Clone,
{
    pub fn new(storage: &'a dyn StorageBackend, key: K) -> Self {
        Self {
            storage,
            key,
            _value: std::marker::PhantomData,
        }
    }
    
    fn get_map(&self) -> Map<K, V> {
        self.storage.get(&self.key).unwrap_or_else(|| Map::new(self.storage.env()))
    }
    
    pub fn get(&self, map_key: &K) -> Result<Option<V>, StorageError> {
        let map = self.get_map();
        Ok(map.get(map_key.clone()))
    }
    
    pub fn set(&self, map_key: &K, value: &V) -> Result<(), StorageError> {
        let mut map = self.get_map();
        map.set(map_key.clone(), value.clone());
        self.storage.set(&self.key, &map);
        Ok(())
    }
    
    pub fn remove(&self, map_key: &K) -> Result<(), StorageError> {
        let mut map = self.get_map();
        if map.contains_key(map_key.clone()) {
            map.remove(map_key.clone());
            self.storage.set(&self.key, &map);
        }
        Ok(())
    }
    
    pub fn contains(&self, map_key: &K) -> Result<bool, StorageError> {
        let map = self.get_map();
        Ok(map.contains_key(map_key.clone()))
    }
    
    pub fn all(&self) -> Result<Map<K, V>, StorageError> {
        Ok(self.get_map())
    }
}

impl<'a, K, V> MapRepository<K, V> for GenericMapRepository<'a, K, V>
where
    K: IntoVal<Val> + Clone,
    V: IntoVal<Val> + Clone,
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
