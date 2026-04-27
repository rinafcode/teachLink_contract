use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct VersionedStore {
    inner: Arc<Mutex<HashMap<String, (u64, Vec<u8>)>>>,
}

impl VersionedStore {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub fn get(&self, key: &str) -> Option<(u64, Vec<u8>)> {
        let map = self.inner.lock().unwrap();
        map.get(key).map(|(v, b)| (*v, b.clone()))
    }

    pub fn set(&self, key: String, value: Vec<u8>) {
        let mut map = self.inner.lock().unwrap();
        let version = map.get(&key).map(|(v,_)| v + 1).unwrap_or(1);
        map.insert(key, (version, value));
    }

    pub fn get_version(&self, key: &str) -> Option<u64> {
        let map = self.inner.lock().unwrap();
        map.get(key).map(|(v,_)| *v)
    }

    /// Validate that for every (key, version) pair the store currently has the same version.
    pub fn validate_versions(&self, read_set: &[(String, u64)]) -> bool {
        let map = self.inner.lock().unwrap();
        for (k, v) in read_set.iter() {
            let cur = map.get(k).map(|(cv, _)| *cv).unwrap_or(0);
            if cur != *v {
                return false;
            }
        }
        true
    }
}
