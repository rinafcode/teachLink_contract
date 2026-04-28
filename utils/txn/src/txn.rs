use crate::store::VersionedStore;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IsolationLevel {
    ReadCommitted,
    Serializable,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConflictResolution {
    Abort,
    Retry(u32),
}

#[derive(Debug)]
pub enum TxnError {
    Conflict,
}

pub struct Transaction {
    store: Arc<VersionedStore>,
    isolation: IsolationLevel,
    conflict: ConflictResolution,
    read_set: Vec<(String, u64)>,
    write_set: HashMap<String, Vec<u8>>,
}

impl Transaction {
    pub fn new(store: &VersionedStore, isolation: IsolationLevel, conflict: ConflictResolution) -> Self {
        Self {
            store: Arc::new(store.clone()),
            isolation,
            conflict,
            read_set: Vec::new(),
            write_set: HashMap::new(),
        }
    }

    pub fn read(&mut self, key: &str) -> Option<Vec<u8>> {
        let res = self.store.get(key);
        if let Some((ver, val)) = res {
            self.read_set.push((key.to_string(), ver));
            Some(val)
        } else {
            self.read_set.push((key.to_string(), 0));
            None
        }
    }

    pub fn write(&mut self, key: &str, value: Vec<u8>) {
        self.write_set.insert(key.to_string(), value);
    }

    pub fn commit(&mut self) -> Result<(), TxnError> {
        match self.isolation {
            IsolationLevel::ReadCommitted => {
                // apply writes without full validation
                for (k, v) in self.write_set.drain() {
                    self.store.set(k, v);
                }
                Ok(())
            }
            IsolationLevel::Serializable => {
                let mut attempts = 0u32;
                loop {
                    if self.store.validate_versions(&self.read_set) {
                        // apply writes
                        for (k, v) in self.write_set.drain() {
                            self.store.set(k, v);
                        }
                        return Ok(());
                    } else {
                        match self.conflict {
                            ConflictResolution::Abort => return Err(TxnError::Conflict),
                            ConflictResolution::Retry(max) => {
                                attempts += 1;
                                if attempts > max {
                                    return Err(TxnError::Conflict);
                                }
                                // reload read_set versions
                                let mut new_rs = Vec::new();
                                for (k, _) in self.read_set.iter() {
                                    let ver = self.store.get_version(k).unwrap_or(0);
                                    new_rs.push((k.clone(), ver));
                                }
                                self.read_set = new_rs;
                                continue;
                            }
                        }
                    }
                }
            }
        }
    }
}
