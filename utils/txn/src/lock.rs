use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LockType {
    Shared,
    Exclusive,
}

#[derive(Clone)]
pub struct PessimisticLockManager {
    inner: Arc<Mutex<HashMap<String, Arc<RwLock<()>>>>>,
}

impl PessimisticLockManager {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(HashMap::new())) }
    }

    fn get_lock(&self, key: &str) -> Arc<RwLock<()>> {
        let mut map = self.inner.lock().unwrap();
        map.entry(key.to_string()).or_insert_with(|| Arc::new(RwLock::new(()))).clone()
    }

    /// Run a closure while holding a shared (read) lock for the key.
    pub fn with_shared<F, R>(&self, key: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let lock = self.get_lock(key);
        let _g = lock.read().unwrap();
        f()
    }

    /// Run a closure while holding an exclusive (write) lock for the key.
    pub fn with_exclusive<F, R>(&self, key: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let lock = self.get_lock(key);
        let _g = lock.write().unwrap();
        f()
    }
}

