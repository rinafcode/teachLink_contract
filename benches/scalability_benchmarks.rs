//! Performance benchmarks for scalability improvements
//! 
//! This module benchmarks the performance improvements from replacing linear searches
//! with indexed lookups and implementing efficient data structures.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use soroban_sdk::{testutils::Ledger, Env, Address, Bytes, Map, Vec};
use teachlink_contract::atomic_swap::AtomicSwapManager;
use teachlink_contract::analytics::AnalyticsManager;
use teachlink_contract::types::{AtomicSwap, ChainMetrics, SwapStatus};

fn create_test_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

fn benchmark_linear_vs_indexed_swap_search(c: &mut Criterion) {
    let env = create_test_env();
    let initiator = Address::random(&env);
    let counterparty = Address::random(&env);
    
    // Create test data
    let num_swaps = 1000;
    for i in 0..num_swaps {
        let swap_id = AtomicSwapManager::initiate_swap(
            &env,
            initiator.clone(),
            Address::random(&env),
            1000 + i as i128,
            counterparty.clone(),
            Address::random(&env),
            2000 + i as i128,
            Bytes::from_slice(&env, &format!("{:032}", i).as_bytes()),
            3600,
        ).unwrap();
    }

    c.bench_function("linear_search_swaps_by_initiator", |b| {
        b.iter(|| {
            // Simulate linear search (old implementation)
            let swaps: soroban_sdk::Map<u64, AtomicSwap> = env
                .storage()
                .instance()
                .get(&soroban_sdk::symbol_short!("swaps"))
                .unwrap_or_else(|| soroban_sdk::Map::new(&env));
            
            let mut result = Vec::new(&env);
            for (swap_id, swap) in swaps.iter() {
                if swap.initiator == initiator {
                    result.push_back(swap_id);
                }
            }
            black_box(result)
        })
    });

    c.bench_function("indexed_lookup_swaps_by_initiator", |b| {
        b.iter(|| {
            // Use indexed lookup (new implementation)
            let result = AtomicSwapManager::get_swaps_by_initiator(&env, initiator.clone());
            black_box(result)
        })
    });
}

fn benchmark_analytics_sorting(c: &mut Criterion) {
    let env = create_test_env();
    
    // Create test chain metrics
    let num_chains = 100;
    for i in 0..num_chains {
        AnalyticsManager::initialize_chain_metrics(&env, i).unwrap();
        AnalyticsManager::update_chain_metrics(
            &env,
            i,
            (i as i128) * 1000, // Volume increases with chain ID
            i % 2 == 0,
            100,
        ).unwrap();
    }

    c.bench_function("bubble_sort_chains_by_volume", |b| {
        b.iter(|| {
            // Simulate old bubble sort implementation
            let chain_metrics: Map<u32, ChainMetrics> = env
                .storage()
                .instance()
                .get(&soroban_sdk::symbol_short!("ch_mets"))
                .unwrap_or_else(|| Map::new(&env));

            let mut chains: Vec<(u32, i128)> = Vec::new(&env);
            for (chain_id, metrics) in chain_metrics.iter() {
                let total_volume = metrics.volume_in + metrics.volume_out;
                chains.push_back((chain_id, total_volume));
            }

            // Bubble sort (O(n²))
            let len = chains.len();
            for i in 0..len {
                for j in 0..(len - i - 1) {
                    let (_, vol_a) = chains.get(j).unwrap();
                    let (_, vol_b) = chains.get(j + 1).unwrap();
                    if vol_a < vol_b {
                        let temp = chains.get(j).unwrap();
                        chains.set(j, chains.get(j + 1).unwrap());
                        chains.set(j + 1, temp);
                    }
                }
            }
            black_box(chains)
        })
    });

    c.bench_function("indexed_sort_chains_by_volume", |b| {
        b.iter(|| {
            // Use indexed lookup with efficient sort (new implementation)
            let result = AnalyticsManager::get_top_chains_by_volume(&env, 10);
            black_box(result)
        })
    });
}

fn benchmark_large_dataset_operations(c: &mut Criterion) {
    let env = create_test_env();
    
    // Create large dataset
    let num_swaps = 10000;
    let initiators: Vec<Address> = (0..100).map(|_| Address::random(&env)).collect();
    let counterparties: Vec<Address> = (0..100).map(|_| Address::random(&env)).collect();
    
    for i in 0..num_swaps {
        let initiator = initiators.get(i % 100).unwrap().clone();
        let counterparty = counterparties.get(i % 100).unwrap().clone();
        
        AtomicSwapManager::initiate_swap(
            &env,
            initiator,
            Address::random(&env),
            1000 + i as i128,
            counterparty,
            Address::random(&env),
            2000 + i as i128,
            Bytes::from_slice(&env, &format!("{:032}", i).as_bytes()),
            3600,
        ).unwrap();
    }

    c.bench_function("get_active_swaps_large_dataset", |b| {
        b.iter(|| {
            let result = AtomicSwapManager::get_active_swaps(&env);
            black_box(result)
        })
    });

    c.bench_function("get_swaps_by_initiator_large_dataset", |b| {
        b.iter(|| {
            let initiator = initiators.get(50).unwrap().clone();
            let result = AtomicSwapManager::get_swaps_by_initiator(&env, initiator);
            black_box(result)
        })
    });
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let env = create_test_env();
    
    // Measure memory overhead of indexes
    let num_swaps = 1000;
    let initiator = Address::random(&env);
    let counterparty = Address::random(&env);
    
    c.bench_function("create_swaps_with_indexes", |b| {
        b.iter(|| {
            for i in 0..num_swaps {
                AtomicSwapManager::initiate_swap(
                    &env,
                    initiator.clone(),
                    Address::random(&env),
                    1000 + i as i128,
                    counterparty.clone(),
                    Address::random(&env),
                    2000 + i as i128,
                    Bytes::from_slice(&env, &format!("{:032}", i).as_bytes()),
                    3600,
                ).unwrap();
            }
        })
    });
}

criterion_group!(
    benches,
    benchmark_linear_vs_indexed_swap_search,
    benchmark_analytics_sorting,
    benchmark_large_dataset_operations,
    benchmark_memory_usage
);
criterion_main!(benches);
