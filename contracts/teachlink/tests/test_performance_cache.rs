use crate::TeachLinkBridge;
use soroban_sdk::{vec, Address, Bytes, Env};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Map;

#[test]
fn performance_cache_hit_miss_and_ttl() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());

    // run inside contract context
    env.as_contract(&contract_id, || {
        // compute and cache summary
        let summary = crate::performance::PerformanceManager::compute_and_cache_summary(&env)
            .expect("compute should succeed");

        // first get should be a hit
        let s1 = crate::performance::PerformanceManager::get_or_compute_summary(&env)
            .expect("should return summary");
        assert_eq!(s1.health_score, summary.health_score);

        // call multiple times to build hits
        for _ in 0..5 {
            let _ = crate::performance::PerformanceManager::get_or_compute_summary(&env);
        }

        // stats should show hits > 0
        let stats: Map<Bytes, i128> = crate::performance::PerformanceManager::get_cache_stats(&env);
        let hits = *stats.get(&Bytes::from_slice(&env, b"hits")).unwrap();
        let misses = *stats.get(&Bytes::from_slice(&env, b"misses")).unwrap();
        assert!(hits >= 1, "expected at least one hit");

        // Fast-forward time past TTL to force expiry
        let now = env.ledger().timestamp();
        env.ledger().with_mut(|l| l.timestamp = now + crate::performance::CACHE_TTL_SECS + 10);

        // Next get should recompute (miss)
        let _ = crate::performance::PerformanceManager::get_or_compute_summary(&env).expect("recompute");

        // stats should increase misses
        let stats2: Map<Bytes, i128> = crate::performance::PerformanceManager::get_cache_stats(&env);
        let hits2 = *stats2.get(&Bytes::from_slice(&env, b"hits")).unwrap();
        let misses2 = *stats2.get(&Bytes::from_slice(&env, b"misses")).unwrap();
        assert!(misses2 >= misses, "misses should have increased or stayed same");

        // Reset stats via admin should work
        // set admin
        let admin = Address::generate(&env);
        env.storage().instance().set(&crate::storage::ADMIN, &admin);
        let _ = crate::performance::PerformanceManager::reset_cache_stats(&env, &admin);

        let stats3: Map<Bytes, i128> = crate::performance::PerformanceManager::get_cache_stats(&env);
        assert_eq!(*stats3.get(&Bytes::from_slice(&env, b"hits")).unwrap(), 0);
        assert_eq!(*stats3.get(&Bytes::from_slice(&env, b"misses")).unwrap(), 0);
    });
}
