pub mod lock;
pub mod store;
pub mod txn;

pub use lock::{PessimisticLockManager, LockType};
pub use store::VersionedStore;
pub use txn::{Transaction, IsolationLevel, ConflictResolution};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pessimistic_lock_basic() {
        let lm = PessimisticLockManager::new();
        let key = "k1".to_string();

        // acquire exclusive
        let _guard = lm.acquire(&key, LockType::Exclusive);

        // trying to acquire shared will block if done concurrently; here it should still succeed when dropped
        drop(_guard);
    }

    #[test]
    fn optimistic_txn_conflict() {
        let store = VersionedStore::new();
        store.set("x".into(), b"1".to_vec());

        let mut t1 = Transaction::new(&store, IsolationLevel::Serializable, ConflictResolution::Abort);
        let mut t2 = Transaction::new(&store, IsolationLevel::Serializable, ConflictResolution::Abort);

        t1.read("x");
        t2.read("x");

        t1.write("x", b"2".to_vec());
        assert!(t1.commit().is_ok());

        t2.write("x", b"3".to_vec());
        assert!(t2.commit().is_err());
    }
}
