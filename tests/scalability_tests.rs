//! Integration tests for scalability improvements
//! 
//! This module tests the scalability improvements with large datasets to ensure
//! the optimizations work correctly and handle edge cases.

use soroban_sdk::{testutils::Ledger, Env, Address, Bytes, Map, Vec};
use teachlink_contract::atomic_swap::AtomicSwapManager;
use teachlink_contract::analytics::AnalyticsManager;
use teachlink_contract::types::{AtomicSwap, SwapStatus};

fn create_test_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

#[test]
fn test_scalability_swap_search_performance() {
    let env = create_test_env();
    let initiator = Address::random(&env);
    let counterparty = Address::random(&env);
    
    // Create a large number of swaps
    let num_swaps = 5000;
    let mut created_swap_ids = Vec::new(&env);
    
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
        created_swap_ids.push_back(swap_id);
    }
    
    // Test that indexed lookup returns correct results
    let initiator_swaps = AtomicSwapManager::get_swaps_by_initiator(&env, initiator.clone());
    assert_eq!(initiator_swaps.len(), num_swaps as u32);
    
    // Verify all created swap IDs are returned
    for &swap_id in created_swap_ids.iter() {
        assert!(initiator_swaps.iter().any(|&id| id == swap_id));
    }
    
    // Test counterparty lookup
    let counterparty_swaps = AtomicSwapManager::get_swaps_by_counterparty(&env, counterparty.clone());
    assert_eq!(counterparty_swaps.len(), num_swaps as u32);
    
    // Test active swaps lookup
    let active_swaps = AtomicSwapManager::get_active_swaps(&env);
    assert_eq!(active_swaps.len(), num_swaps as u32);
}

#[test]
fn test_scalability_analytics_performance() {
    let env = create_test_env();
    
    // Create many chains with different volumes
    let num_chains = 200;
    for i in 0..num_chains {
        AnalyticsManager::initialize_chain_metrics(&env, i).unwrap();
        
        // Create varying volumes to test sorting
        let volume = (num_chains - i) as i128 * 1000; // Reverse order for testing
        AnalyticsManager::update_chain_metrics(
            &env,
            i,
            volume,
            i % 2 == 0,
            100 + i as i128,
        ).unwrap();
    }
    
    // Test top chains by volume
    let top_chains = AnalyticsManager::get_top_chains_by_volume(&env, 10);
    assert_eq!(top_chains.len(), 10);
    
    // Verify sorting is correct (highest volume first)
    for i in 1..top_chains.len() {
        let (_, vol_prev) = top_chains.get(i - 1).unwrap();
        let (_, vol_curr) = top_chains.get(i).unwrap();
        assert!(vol_prev >= vol_curr);
    }
    
    // Test bounded version
    let top_chains_bounded = AnalyticsManager::get_top_chains_by_volume_bounded(&env, 5);
    assert_eq!(top_chains_bounded.len(), 5);
    
    // Verify bounded version matches first 5 of unbounded
    for i in 0..5 {
        assert_eq!(top_chains.get(i), top_chains_bounded.get(i));
    }
}

#[test]
fn test_scalability_mixed_operations() {
    let env = create_test_env();
    
    // Create multiple users
    let initiators: Vec<Address> = (0..50).map(|_| Address::random(&env)).collect();
    let counterparties: Vec<Address> = (0..50).map(|_| Address::random(&env)).collect();
    
    // Create many swaps across different users
    let num_swaps = 2000;
    for i in 0..num_swaps {
        let initiator = initiators.get(i % 50).unwrap().clone();
        let counterparty = counterparties.get(i % 50).unwrap().clone();
        
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
    
    // Test that each user has correct number of swaps
    for initiator in initiators.iter() {
        let swaps = AtomicSwapManager::get_swaps_by_initiator(&env, initiator.clone());
        // Each initiator should have approximately num_swaps/50 swaps
        assert!(swaps.len() >= 30 && swaps.len() <= 50);
    }
    
    // Test that all swaps are active
    let active_swaps = AtomicSwapManager::get_active_swaps(&env);
    assert_eq!(active_swaps.len(), num_swaps as u32);
}

#[test]
fn test_scalability_status_updates() {
    let env = create_test_env();
    let initiator = Address::random(&env);
    let counterparty = Address::random(&env);
    
    // Create swaps and complete some of them
    let num_swaps = 1000;
    let mut completed_swaps = Vec::new(&env);
    
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
        
        // Complete every 10th swap
        if i % 10 == 0 {
            AtomicSwapManager::accept_swap(
                &env,
                swap_id,
                counterparty.clone(),
                Bytes::from_slice(&env, &format!("{:032}", i).as_bytes()),
            ).unwrap();
            completed_swaps.push_back(swap_id);
        }
    }
    
    // Test that active swaps count is correct
    let active_swaps = AtomicSwapManager::get_active_swaps(&env);
    assert_eq!(active_swaps.len(), (num_swaps - completed_swaps.len()) as u32);
    
    // Verify completed swaps are not in active list
    for &completed_swap_id in completed_swaps.iter() {
        assert!(!active_swaps.iter().any(|&id| id == completed_swap_id));
    }
}

#[test]
fn test_scalability_edge_cases() {
    let env = create_test_env();
    
    // Test with empty datasets
    let empty_swaps = AtomicSwapManager::get_active_swaps(&env);
    assert_eq!(empty_swaps.len(), 0);
    
    let random_address = Address::random(&env);
    let no_swaps = AtomicSwapManager::get_swaps_by_initiator(&env, random_address.clone());
    assert_eq!(no_swaps.len(), 0);
    
    // Test analytics with no chains
    let no_chains = AnalyticsManager::get_top_chains_by_volume(&env, 10);
    assert_eq!(no_chains.len(), 0);
    
    // Test single item
    AnalyticsManager::initialize_chain_metrics(&env, 1).unwrap();
    AnalyticsManager::update_chain_metrics(&env, 1, 1000, true, 100).unwrap();
    
    let single_chain = AnalyticsManager::get_top_chains_by_volume(&env, 10);
    assert_eq!(single_chain.len(), 1);
    assert_eq!(single_chain.get(0).unwrap().0, 1);
    assert_eq!(single_chain.get(0).unwrap().1, 1000);
}

#[test]
fn test_scalability_memory_efficiency() {
    let env = create_test_env();
    
    // Create many swaps and verify indexes are maintained correctly
    let num_swaps = 3000;
    let initiators: Vec<Address> = (0..10).map(|_| Address::random(&env)).collect();
    
    for i in 0..num_swaps {
        let initiator = initiators.get(i % 10).unwrap().clone();
        
        AtomicSwapManager::initiate_swap(
            &env,
            initiator,
            Address::random(&env),
            1000 + i as i128,
            Address::random(&env),
            Address::random(&env),
            2000 + i as i128,
            Bytes::from_slice(&env, &format!("{:032}", i).as_bytes()),
            3600,
        ).unwrap();
    }
    
    // Verify each initiator has correct number of swaps
    for initiator in initiators.iter() {
        let swaps = AtomicSwapManager::get_swaps_by_initiator(&env, initiator.clone());
        assert_eq!(swaps.len(), (num_swaps / 10) as u32);
    }
    
    // Verify total active swaps
    let active_swaps = AtomicSwapManager::get_active_swaps(&env);
    assert_eq!(active_swaps.len(), num_swaps as u32);
}
