//! Tests for Event Emission
//!
//! This module contains tests to verify that all state-changing operations
//! emit the appropriate events for auditability.

use crate::bridge::Bridge;
use crate::emergency::EmergencyManager;
use crate::escrow::EscrowManager;
use crate::events::*;
use crate::insurance::InsuranceManager;
use crate::reputation::{update_participation, update_course_progress, rate_contribution};
use crate::slashing::SlashingManager;
use crate::tokenization::ContentTokenization;
use crate::TeachLinkBridge;
use soroban_sdk::testutils::{Address as _, Events, Ledger};
use soroban_sdk::{vec, Address, Bytes, Env, Map, Vec};

/// Test that bridge cancellation emits an event
#[test]
fn test_bridge_cancel_emits_event() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());
    
    env.as_contract(&contract_id, || {
        // Setup
        let token = Address::generate(&env);
        let sender = Address::generate(&env);
        let admin = Address::generate(&env);
        
        env.storage().instance().set(&crate::storage::TOKEN, &token);
        env.storage().instance().set(&crate::storage::ADMIN, &admin);
        
        // Create a bridge transaction
        let timestamp = 1000u64;
        env.ledger().with_mut(|li| li.timestamp = timestamp);
        
        let tx = crate::types::BridgeTransaction {
            nonce: 1,
            token: token.clone(),
            amount: 500,
            recipient: sender.clone(),
            destination_chain: 2,
            destination_address: Bytes::from_slice(&env, b"dest"),
            timestamp,
        };
        
        let mut txs: Map<u64, crate::types::BridgeTransaction> = Map::new(&env);
        txs.set(1, tx);
        env.storage().instance().set(&crate::storage::BRIDGE_TXS, &txs);
        
        // Move time forward to allow cancellation
        env.ledger().with_mut(|li| li.timestamp = timestamp + 700_000);
        
        // Cancel the bridge
        Bridge::cancel_bridge(&env, 1).expect("cancel should succeed");
        
        // Verify event was emitted
        let events = env.events().all();
        assert!(events.iter().any(|e| {
            if let Some(event_data) = e.data.get(0).and_then(|v| v.try_into_val::<soroban_sdk::Symbol>(&env).ok()) {
                event_data == soroban_sdk::symbol_short!("bridge_can")
            } else {
                false
            }
        }));
    });
}

/// Test that bridge failure emits an event
#[test]
fn test_bridge_failure_emits_event() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());
    
    env.as_contract(&contract_id, || {
        // Setup
        let token = Address::generate(&env);
        let admin = Address::generate(&env);
        
        env.storage().instance().set(&crate::storage::TOKEN, &token);
        env.storage().instance().set(&crate::storage::ADMIN, &admin);
        
        // Create a bridge transaction
        let tx = crate::types::BridgeTransaction {
            nonce: 1,
            token: token.clone(),
            amount: 500,
            recipient: Address::generate(&env),
            destination_chain: 2,
            destination_address: Bytes::from_slice(&env, b"dest"),
            timestamp: env.ledger().timestamp(),
        };
        
        let mut txs: Map<u64, crate::types::BridgeTransaction> = Map::new(&env);
        txs.set(1, tx);
        env.storage().instance().set(&crate::storage::BRIDGE_TXS, &txs);
        
        // Mark as failed
        Bridge::mark_bridge_failed(&env, 1, Bytes::from_slice(&env, b"timeout"))
            .expect("mark failed should succeed");
        
        // Verify event was emitted
        let events = env.events().all();
        assert!(events.iter().any(|e| {
            if let Some(event_data) = e.data.get(0).and_then(|v| v.try_into_val::<soroban_sdk::Symbol>(&env).ok()) {
                event_data == soroban_sdk::symbol_short!("bridge_fail")
            } else {
                false
            }
        }));
    });
}

/// Test that validator management emits events
#[test]
fn test_validator_management_emits_events() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());
    
    env.as_contract(&contract_id, || {
        // Setup
        let admin = Address::generate(&env);
        let validator = Address::generate(&env);
        
        env.storage().instance().set(&crate::storage::ADMIN, &admin);
        
        let mut validators: Map<Address, bool> = Map::new(&env);
        env.storage().instance().set(&crate::storage::VALIDATORS, &validators);
        
        // Add validator
        Bridge::add_validator(&env, validator.clone()).expect("add validator should succeed");
        
        // Verify event was emitted
        let events = env.events().all();
        assert!(events.iter().any(|e| {
            if let Some(event_data) = e.data.get(0).and_then(|v| v.try_into_val::<soroban_sdk::Symbol>(&env).ok()) {
                event_data == soroban_sdk::symbol_short!("val_add")
            } else {
                false
            }
        }));
        
        // Remove validator
        Bridge::remove_validator(&env, validator.clone()).expect("remove validator should succeed");
        
        // Verify event was emitted
        let events = env.events().all();
        assert!(events.iter().any(|e| {
            if let Some(event_data) = e.data.get(0).and_then(|v| v.try_into_val::<soroban_sdk::Symbol>(&env).ok()) {
                event_data == soroban_sdk::symbol_short!("val_rem")
            } else {
                false
            }
        }));
    });
}

/// Test that circuit breaker initialization emits an event
#[test]
fn test_circuit_breaker_init_emits_event() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());
    
    env.as_contract(&contract_id, || {
        // Initialize circuit breaker
        EmergencyManager::initialize_circuit_breaker(&env, 1, 1000000, 100000)
            .expect("init should succeed");
        
        // Verify event was emitted
        let events = env.events().all();
        assert!(events.iter().any(|e| {
            if let Some(event_data) = e.data.get(0).and_then(|v| v.try_into_val::<soroban_sdk::Symbol>(&env).ok()) {
                event_data == soroban_sdk::symbol_short!("cb_init")
            } else {
                false
            }
        }));
    });
}

/// Test that circuit breaker reset emits an event
#[test]
fn test_circuit_breaker_reset_emits_event() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());
    
    env.as_contract(&contract_id, || {
        let admin = Address::generate(&env);
        
        // Initialize circuit breaker
        EmergencyManager::initialize_circuit_breaker(&env, 1, 1000000, 100000)
            .expect("init should succeed");
        
        // Trigger it by exceeding limit
        EmergencyManager::check_circuit_breaker(&env, 1, 200000)
            .expect_err("should trigger");
        
        // Reset circuit breaker
        EmergencyManager::reset_circuit_breaker(&env, 1, admin.clone())
            .expect("reset should succeed");
        
        // Verify event was emitted
        let events = env.events().all();
        assert!(events.iter().any(|e| {
            if let Some(event_data) = e.data.get(0).and_then(|v| v.try_into_val::<soroban_sdk::Symbol>(&env).ok()) {
                event_data == soroban_sdk::symbol_short!("cb_reset")
            } else {
                false
            }
        }));
    });
}

/// Test that insurance pool operations emit events
#[test]
fn test_insurance_pool_emits_events() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());
    
    env.as_contract(&contract_id, || {
        let token = Address::generate(&env);
        let funder = Address::generate(&env);
        
        // Initialize pool
        InsuranceManager::initialize_pool(&env, token.clone(), 100)
            .expect("init should succeed");
        
        // Verify init event
        let events = env.events().all();
        assert!(events.iter().any(|e| {
            if let Some(event_data) = e.data.get(0).and_then(|v| v.try_into_val::<soroban_sdk::Symbol>(&env).ok()) {
                event_data == soroban_sdk::symbol_short!("ins_init")
            } else {
                false
            }
        }));
        
        // Fund pool
        InsuranceManager::fund_pool(&env, funder.clone(), 10000)
            .expect("fund should succeed");
        
        // Verify fund event
        let events = env.events().all();
        assert!(events.iter().any(|e| {
            if let Some(event_data) = e.data.get(0).and_then(|v| v.try_into_val::<soroban_sdk::Symbol>(&env).ok()) {
                event_data == soroban_sdk::symbol_short!("ins_fund")
            } else {
                false
            }
        }));
    });
}

/// Test that reputation updates emit events
#[test]
fn test_reputation_updates_emit_events() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());
    
    env.as_contract(&contract_id, || {
        let user = Address::generate(&env);
        
        // Update participation
        update_participation(&env, user.clone(), 100);
        
        // Verify event was emitted
        let events = env.events().all();
        assert!(events.iter().any(|e| {
            if let Some(event_data) = e.data.get(0).and_then(|v| v.try_into_val::<soroban_sdk::Symbol>(&env).ok()) {
                event_data == soroban_sdk::symbol_short!("part_upd")
            } else {
                false
            }
        }));
        
        // Update course progress
        update_course_progress(&env, user.clone(), false);
        
        // Verify event was emitted
        let events = env.events().all();
        assert!(events.iter().any(|e| {
            if let Some(event_data) = e.data.get(0).and_then(|v| v.try_into_val::<soroban_sdk::Symbol>(&env).ok()) {
                event_data == soroban_sdk::symbol_short!("course_upd")
            } else {
                false
            }
        }));
        
        // Rate contribution
        rate_contribution(&env, user.clone(), 4);
        
        // Verify event was emitted
        let events = env.events().all();
        assert!(events.iter().any(|e| {
            if let Some(event_data) = e.data.get(0).and_then(|v| v.try_into_val::<soroban_sdk::Symbol>(&env).ok()) {
                event_data == soroban_sdk::symbol_short!("contrib_rate")
            } else {
                false
            }
        }));
    });
}

/// Test that tokenization transferability update emits an event
#[test]
fn test_transferability_update_emits_event() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());
    
    env.as_contract(&contract_id, || {
        let creator = Address::generate(&env);
        
        // Mint a token
        let token_id = ContentTokenization::mint(
            &env,
            creator.clone(),
            Bytes::from_slice(&env, b"Title"),
            Bytes::from_slice(&env, b"Desc"),
            crate::types::ContentType::Video,
            Bytes::from_slice(&env, b"hash"),
            Bytes::from_slice(&env, b"MIT"),
            vec![&env],
            true,
            500,
        );
        
        // Update transferability
        ContentTokenization::set_transferable(&env, creator.clone(), token_id, false);
        
        // Verify event was emitted
        let events = env.events().all();
        assert!(events.iter().any(|e| {
            if let Some(event_data) = e.data.get(0).and_then(|v| v.try_into_val::<soroban_sdk::Symbol>(&env).ok()) {
                event_data == soroban_sdk::symbol_short!("transf_upd")
            } else {
                false
            }
        }));
    });
}

/// Test that escrow cancellation emits an event
#[test]
fn test_escrow_cancel_emits_event() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());
    
    env.as_contract(&contract_id, || {
        let depositor = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        
        // Create escrow
        let escrow_id = EscrowManager::create_escrow(
            &env,
            depositor.clone(),
            beneficiary.clone(),
            token.clone(),
            1000,
            vec![&env],
            1,
            None,
            None,
            arbitrator.clone(),
        ).expect("create should succeed");
        
        // Cancel escrow
        EscrowManager::cancel(&env, escrow_id, depositor.clone())
            .expect("cancel should succeed");
        
        // Verify event was emitted
        let events = env.events().all();
        assert!(events.iter().any(|e| {
            if let Some(event_data) = e.data.get(0).and_then(|v| v.try_into_val::<soroban_sdk::Symbol>(&env).ok()) {
                event_data == soroban_sdk::symbol_short!("esc_cancel")
            } else {
                false
            }
        }));
    });
}

/// Test that bridge fee update emits an event
#[test]
fn test_bridge_fee_update_emits_event() {
    let env = Env::default();
    let contract_id = env.register(TeachLinkBridge, ());
    
    env.as_contract(&contract_id, || {
        let admin = Address::generate(&env);
        let fee_recipient = Address::generate(&env);
        let token = Address::generate(&env);
        
        // Setup
        env.storage().instance().set(&crate::storage::ADMIN, &admin);
        env.storage().instance().set(&crate::storage::TOKEN, &token);
        env.storage().instance().set(&crate::storage::FEE_RECIPIENT, &fee_recipient);
        env.storage().instance().set(&crate::storage::BRIDGE_FEE, &0i128);
        env.storage().instance().set(&crate::storage::MIN_VALIDATORS, &1u32);
        
        let mut validators: Map<Address, bool> = Map::new(&env);
        env.storage().instance().set(&crate::storage::VALIDATORS, &validators);
        
        let mut chains: Map<u32, bool> = Map::new(&env);
        env.storage().instance().set(&crate::storage::SUPPORTED_CHAINS, &chains);
        
        // Update fee
        Bridge::set_bridge_fee(&env, 100).expect("set fee should succeed");
        
        // Verify event was emitted
        let events = env.events().all();
        assert!(events.iter().any(|e| {
            if let Some(event_data) = e.data.get(0).and_then(|v| v.try_into_val::<soroban_sdk::Symbol>(&env).ok()) {
                event_data == soroban_sdk::symbol_short!("fee_upd")
            } else {
                false
            }
        }));
    });
}
