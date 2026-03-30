#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::too_many_lines)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    vec, Address, Bytes, Env, Map, Symbol, Vec,
};

use teachlink_contract::{
    CircuitBreakerConfig, EmergencyAction, EmergencyError, EmergencyStatus, TeachLinkBridge,
    TeachLinkBridgeClient,
};

fn create_circuit_breaker_config(
    env: &Env,
    failure_threshold: u32,
    recovery_threshold: u32,
    timeout_period: u64,
) -> CircuitBreakerConfig {
    CircuitBreakerConfig {
        failure_threshold,
        recovery_threshold,
        timeout_period,
        auto_pause_enabled: true,
        max_pause_duration: 86400, // 24 hours
    }
}

#[test]
fn test_emergency_initialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let config = create_circuit_breaker_config(&env, 5, 3, 3600);

    // Test successful initialization
    client.initialize_emergency(&admin, &config);

    // Test double initialization
    let result = client.try_initialize_emergency(&admin, &config);
    assert_eq!(result.error(), Some(Ok(EmergencyError::AlreadyInitialized)));
}

#[test]
fn test_emergency_pause() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let config = create_circuit_breaker_config(&env, 5, 3, 3600);

    client.initialize_emergency(&admin, &config);

    // Test authorized emergency pause
    client.emergency_pause(&admin);

    let status = client.get_emergency_status();
    assert_eq!(status, EmergencyStatus::Paused);

    // Test unauthorized emergency pause
    let unauthorized_user = Address::generate(&env);
    let result = client.try_emergency_pause(&unauthorized_user);
    assert_eq!(result.error(), Some(Ok(EmergencyError::Unauthorized)));
}

#[test]
fn test_emergency_resume() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let config = create_circuit_breaker_config(&env, 5, 3, 3600);

    client.initialize_emergency(&admin, &config);

    // Pause first
    client.emergency_pause(&admin);
    assert_eq!(client.get_emergency_status(), EmergencyStatus::Paused);

    // Test authorized resume
    client.emergency_resume(&admin);
    assert_eq!(client.get_emergency_status(), EmergencyStatus::Active);

    // Test unauthorized resume
    client.emergency_pause(&admin);
    let unauthorized_user = Address::generate(&env);
    let result = client.try_emergency_resume(&unauthorized_user);
    assert_eq!(result.error(), Some(Ok(EmergencyError::Unauthorized)));
}

#[test]
fn test_circuit_breaker_trigger() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let config = create_circuit_breaker_config(&env, 3, 2, 3600); // Trigger after 3 failures

    client.initialize_emergency(&admin, &config);

    // Report failures to trigger circuit breaker
    for i in 0..3 {
        let error_data = Bytes::from_slice(&env, &format!("failure_{}", i).as_bytes());
        client.report_operation_failure(&error_data);
    }

    // Circuit breaker should be triggered
    let status = client.get_emergency_status();
    assert_eq!(status, EmergencyStatus::CircuitBreakerTriggered);
}

#[test]
fn test_circuit_breaker_recovery() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let config = create_circuit_breaker_config(&env, 3, 2, 1); // 1 second timeout for testing

    client.initialize_emergency(&admin, &config);

    // Trigger circuit breaker
    for i in 0..3 {
        let error_data = Bytes::from_slice(&env, &format!("failure_{}", i).as_bytes());
        client.report_operation_failure(&error_data);
    }

    assert_eq!(
        client.get_emergency_status(),
        EmergencyStatus::CircuitBreakerTriggered
    );

    // Report successes for recovery
    for i in 0..2 {
        let success_data = Bytes::from_slice(&env, &format!("success_{}", i).as_bytes());
        client.report_operation_success(&success_data);
    }

    // Fast forward past timeout
    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + 2,
        protocol_version: 20,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Check if recovered
    let status = client.get_emergency_status();
    assert_eq!(status, EmergencyStatus::Active);
}

#[test]
fn test_emergency_action_execution() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let config = create_circuit_breaker_config(&env, 5, 3, 3600);

    client.initialize_emergency(&admin, &config);

    // Test emergency action: Pause specific chain
    let action_data = Bytes::from_slice(&env, b"pause_chain_1");
    client.execute_emergency_action(&EmergencyAction::PauseChain, &action_data);

    // Test emergency action: Revoke validator access
    let validator = Address::generate(&env);
    let revoke_data = Bytes::from_slice(&env, &validator.serialize());
    client.execute_emergency_action(&EmergencyAction::RevokeValidatorAccess, &revoke_data);

    // Test emergency action: Freeze assets
    let freeze_data = Bytes::from_slice(&env, b"freeze_all_assets");
    client.execute_emergency_action(&EmergencyAction::FreezeAssets, &freeze_data);
}

#[test]
fn test_emergency_time_limits() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let config = create_circuit_breaker_config(&env, 5, 3, 2); // 2 second max pause

    client.initialize_emergency(&admin, &config);

    // Emergency pause
    client.emergency_pause(&admin);
    assert_eq!(client.get_emergency_status(), EmergencyStatus::Paused);

    // Fast forward past max pause duration
    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + 3,
        protocol_version: 20,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Try to keep paused (should fail due to time limit)
    let result = client.try_emergency_pause(&admin);
    assert_eq!(
        result.error(),
        Some(Ok(EmergencyError::MaxPauseDurationExceeded))
    );
}

#[test]
fn test_emergency_logging() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let config = create_circuit_breaker_config(&env, 5, 3, 3600);

    client.initialize_emergency(&admin, &config);

    // Generate some emergency events
    client.emergency_pause(&admin);

    let error_data = Bytes::from_slice(&env, b"test_error");
    client.report_operation_failure(&error_data);

    let success_data = Bytes::from_slice(&env, b"test_success");
    client.report_operation_success(&success_data);

    // Check emergency logs
    let logs = client.get_emergency_logs();
    assert!(logs.len() >= 3); // Pause + failure + success
}

#[test]
fn test_emergency_configuration_update() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let config = create_circuit_breaker_config(&env, 5, 3, 3600);

    client.initialize_emergency(&admin, &config);

    // Update configuration
    let new_config = create_circuit_breaker_config(&env, 10, 5, 7200);
    client.update_emergency_config(&admin, &new_config);

    // Verify configuration updated
    let current_config = client.get_emergency_config();
    assert_eq!(current_config.failure_threshold, 10);
    assert_eq!(current_config.recovery_threshold, 5);
    assert_eq!(current_config.timeout_period, 7200);

    // Test unauthorized config update
    let unauthorized_user = Address::generate(&env);
    let result = client.try_update_emergency_config(&unauthorized_user, &new_config);
    assert_eq!(result.error(), Some(Ok(EmergencyError::Unauthorized)));
}

#[test]
fn test_emergency_status_transitions() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let config = create_circuit_breaker_config(&env, 3, 2, 3600);

    client.initialize_emergency(&admin, &config);

    // Initial state should be Active
    assert_eq!(client.get_emergency_status(), EmergencyStatus::Active);

    // Transition to Paused
    client.emergency_pause(&admin);
    assert_eq!(client.get_emergency_status(), EmergencyStatus::Paused);

    // Transition back to Active
    client.emergency_resume(&admin);
    assert_eq!(client.get_emergency_status(), EmergencyStatus::Active);

    // Transition to CircuitBreakerTriggered
    for i in 0..3 {
        let error_data = Bytes::from_slice(&env, &format!("failure_{}", i).as_bytes());
        client.report_operation_failure(&error_data);
    }
    assert_eq!(
        client.get_emergency_status(),
        EmergencyStatus::CircuitBreakerTriggered
    );
}

#[test]
fn test_emergency_notification_system() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let notifier = Address::generate(&env);
    let config = create_circuit_breaker_config(&env, 5, 3, 3600);

    client.initialize_emergency(&admin, &config);

    // Register emergency notifier
    client.register_emergency_notifier(&notifier);

    // Trigger emergency event
    client.emergency_pause(&admin);

    // Check if notification was sent (implementation dependent)
    let notifications = client.get_emergency_notifications();
    assert!(notifications.len() > 0);
}
