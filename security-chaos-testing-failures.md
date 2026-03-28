# Chaos Testing and Failure Scenario Implementation

## Issue Overview

**Severity:** Medium  
**Category:** Testing & Quality Assurance  
**Impact:** Medium - Poor resilience  

### Description
No chaos testing for failure scenarios and recovery. The system lacks comprehensive testing for network partitions, node failures, and other fault conditions that could occur in production.

## Chaos Testing Framework

### Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│              Chaos Testing Framework                     │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   Network    │  │     Node     │  │    System    │  │
│  │ Partition    │  │   Failure    │  │    Faults    │  │
│  │  Simulator   │  │   Injector   │  │   Generator  │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
│                         │                               │
│                  ┌──────▼──────┐                        │
│                  │   Monitor   │                        │
│                  │  & Reporter │                        │
│                  └─────────────┘                        │
└─────────────────────────────────────────────────────────┘
```

## Implementation Strategy

### 1. Chaos Testing Infrastructure

#### Test Environment Setup

```rust
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[contract]
pub struct ChaosTestContract;

/// Chaos scenario types
#[derive(Debug, Clone)]
pub enum ChaosScenario {
    /// Network partition between nodes
    NetworkPartition {
        duration_secs: u64,
        affected_nodes: Vec<Address>,
    },
    
    /// Node crash and restart
    NodeCrash {
        node_id: Address,
        crash_duration_secs: u64,
    },
    
    /// Message delay injection
    MessageDelay {
        delay_ms: u64,
        probability: f64,
    },
    
    /// Message loss simulation
    MessageLoss {
        loss_rate: f64,
    },
    
    /// Resource exhaustion
    ResourceExhaustion {
        resource_type: ResourceType,
        exhaustion_level: u8,
    },
    
    /// Byzantine behavior simulation
    ByzantineBehavior {
        node_id: Address,
        behavior_type: ByzantineType,
    },
}

#[derive(Debug, Clone)]
pub enum ResourceType {
    Cpu,
    Memory,
    Storage,
    NetworkBandwidth,
}

#[derive(Debug, Clone)]
pub enum ByzantineType {
    DoubleSpend,
    InvalidStateTransition,
    MessageTampering,
    ReplayAttack,
}

/// Chaos test results
#[derive(Debug, Clone)]
pub struct ChaosTestResult {
    pub scenario: ChaosScenario,
    pub success: bool,
    pub recovery_time_ms: u64,
    pub data_loss_detected: bool,
    pub state_consistency: bool,
    pub error_messages: Vec<String>,
}

pub struct ChaosFramework;

impl ChaosFramework {
    /// Run a single chaos test scenario
    pub async fn run_chaos_test(
        env: &Env,
        scenario: ChaosScenario,
    ) -> ChaosTestResult {
        println!("🔴 Starting chaos test: {:?}", scenario);
        
        let start_time = std::time::Instant::now();
        let mut result = ChaosTestResult {
            scenario: scenario.clone(),
            success: true,
            recovery_time_ms: 0,
            data_loss_detected: false,
            state_consistency: true,
            error_messages: vec![],
        };
        
        match scenario {
            ChaosScenario::NetworkPartition { duration_secs, affected_nodes } => {
                result = Self::test_network_partition(env, duration_secs, affected_nodes).await;
            }
            
            ChaosScenario::NodeCrash { node_id, crash_duration_secs } => {
                result = Self::test_node_crash(env, node_id, crash_duration_secs).await;
            }
            
            ChaosScenario::MessageDelay { delay_ms, probability } => {
                result = Self::test_message_delay(env, delay_ms, probability).await;
            }
            
            ChaosScenario::MessageLoss { loss_rate } => {
                result = Self::test_message_loss(env, loss_rate).await;
            }
            
            _ => {
                result.error_messages.push("Scenario not implemented".to_string());
                result.success = false;
            }
        }
        
        result.recovery_time_ms = start_time.elapsed().as_millis() as u64;
        
        if result.success {
            println!("✅ Chaos test passed: {:?}", scenario);
        } else {
            println!("❌ Chaos test failed: {:?}", scenario);
        }
        
        result
    }
    
    /// Test network partition scenario
    async fn test_network_partition(
        env: &Env,
        duration_secs: u64,
        affected_nodes: Vec<Address>,
    ) -> ChaosTestResult {
        let mut result = ChaosTestResult {
            scenario: ChaosScenario::NetworkPartition {
                duration_secs,
                affected_nodes: affected_nodes.clone(),
            },
            ..Default::default()
        };
        
        // Record pre-partition state
        let pre_partition_state = Self::capture_global_state(env);
        
        // Simulate partition
        println!("⚡ Injecting network partition for {} seconds", duration_secs);
        Self::enable_network_partition(env, &affected_nodes);
        
        // Attempt operations during partition
        let mut operations_failed = 0;
        let total_operations = 10;
        
        for i in 0..total_operations {
            let operation_result = Self::attempt_cross_node_operation(env, i);
            if !operation_result {
                operations_failed += 1;
            }
        }
        
        // Verify isolated nodes can still operate independently
        for node in &affected_nodes {
            let isolated_operation = Self::attempt_isolated_operation(env, node);
            if !isolated_operation.success {
                result.error_messages.push(format!(
                    "Isolated node {} failed to operate: {:?}",
                    node, isolated_operation.error
                ));
            }
        }
        
        // Heal partition
        println!("🔵 Healing network partition");
        Self::disable_network_partition(env);
        
        // Wait for reconciliation
        sleep(Duration::from_secs(duration_secs)).await;
        
        // Attempt reconciliation
        let reconciliation_success = Self::reconcile_after_partition(env).await;
        
        if !reconciliation_success {
            result.success = false;
            result.error_messages.push("Failed to reconcile after partition".to_string());
        }
        
        // Verify state consistency
        let post_partition_state = Self::capture_global_state(env);
        if !Self::verify_state_consistency(&pre_partition_state, &post_partition_state) {
            result.state_consistency = false;
            result.success = false;
            result.error_messages.push("State inconsistency detected after partition".to_string());
        }
        
        result
    }
    
    /// Test node crash and recovery
    async fn test_node_crash(
        env: &Env,
        node_id: Address,
        crash_duration_secs: u64,
    ) -> ChaosTestResult {
        let mut result = ChaosTestResult {
            scenario: ChaosScenario::NodeCrash {
                node_id: node_id.clone(),
                crash_duration_secs,
            },
            ..Default::default()
        };
        
        // Record pre-crash state
        let pre_crash_state = Self::capture_node_state(env, &node_id);
        
        // Crash the node
        println!("💥 Crashing node: {}", node_id);
        Self::crash_node(env, &node_id);
        
        // Verify other nodes continue operating
        let other_nodes = Self::get_active_nodes(env);
        for node in other_nodes.iter() {
            if node != node_id {
                let operation = Self::attempt_operation_on_node(env, &node);
                if !operation.success {
                    result.error_messages.push(format!(
                        "Node {} failed during peer crash: {:?}",
                        node, operation.error
                    ));
                }
            }
        }
        
        // Attempt to interact with crashed node (should fail gracefully)
        let crashed_node_interaction = Self::interact_with_node(env, &node_id);
        if crashed_node_interaction.success {
            result.error_messages.push("Crashed node should not respond".to_string());
            result.success = false;
        }
        
        // Restart node
        println!("🔄 Restarting node: {}", node_id);
        Self::restart_node(env, &node_id);
        
        // Wait for recovery
        sleep(Duration::from_secs(crash_duration_secs)).await;
        
        // Verify node recovered with correct state
        let post_recovery_state = Self::capture_node_state(env, &node_id);
        
        if !Self::verify_state_recovery(&pre_crash_state, &post_recovery_state) {
            result.state_consistency = false;
            result.success = false;
            result.error_messages.push("State not properly recovered after crash".to_string());
        }
        
        // Verify node reintegrated into network
        let reintegration_success = Self::verify_node_reintegration(env, &node_id).await;
        if !reintegration_success {
            result.success = false;
            result.error_messages.push("Node failed to reintegrate after restart".to_string());
        }
        
        result
    }
    
    /// Test message delay injection
    async fn test_message_delay(
        env: &Env,
        delay_ms: u64,
        probability: f64,
    ) -> ChaosTestResult {
        let mut result = ChaosTestResult {
            scenario: ChaosScenario::MessageDelay { delay_ms, probability },
            ..Default::default()
        };
        
        println!("🐌 Injecting message delays: {}ms with {:.2}% probability", delay_ms, probability * 100.0);
        
        // Enable delay injection
        Self::enable_message_delay(env, delay_ms, probability);
        
        // Send multiple messages and measure latency
        let mut latencies = Vec::new();
        let message_count = 100;
        
        for _ in 0..message_count {
            let start = std::time::Instant::now();
            let operation = Self::send_test_message(env);
            let latency = start.elapsed().as_millis() as u64;
            
            if operation.success {
                latencies.push(latency);
            } else {
                result.error_messages.push(format!("Message failed: {:?}", operation.error));
            }
        }
        
        // Analyze latency distribution
        let avg_latency = latencies.iter().sum::<u64>() / latencies.len() as u64;
        let max_latency = *latencies.iter().max().unwrap();
        
        println!("📊 Latency stats - Avg: {}ms, Max: {}ms", avg_latency, max_latency);
        
        // Verify no timeouts occurred
        let timeout_threshold = delay_ms * 2;
        let timeouts = latencies.iter().filter(|&&l| l > timeout_threshold).count();
        
        if timeouts > 0 {
            result.error_messages.push(format!("{} messages timed out", timeouts));
        }
        
        // Disable delay injection
        Self::disable_message_delay(env);
        
        result
    }
    
    /// Test message loss scenario
    async fn test_message_loss(env: &Env, loss_rate: f64) -> ChaosTestResult {
        let mut result = ChaosTestResult {
            scenario: ChaosScenario::MessageLoss { loss_rate },
            ..Default::default()
        };
        
        println!("📉 Injecting message loss: {:.2}%", loss_rate * 100.0);
        
        Self::enable_message_loss(env, loss_rate);
        
        // Send messages with retry logic
        let mut sent = 0;
        let mut successful = 0;
        let max_retries = 3;
        
        for _ in 0..100 {
            sent += 1;
            
            for attempt in 0..max_retries {
                let operation = Self::send_test_message(env);
                
                if operation.success {
                    successful += 1;
                    break;
                }
                
                if attempt == max_retries - 1 {
                    result.error_messages.push(format!(
                        "Message failed after {} retries",
                        max_retries
                    ));
                }
            }
        }
        
        let success_rate = successful as f64 / sent as f64;
        let expected_success_rate = 1.0 - loss_rate.powf(max_retries as f64);
        
        println!(
            "📊 Success rate: {:.2}% (expected: {:.2}%)",
            success_rate * 100.0,
            expected_success_rate * 100.0
        );
        
        // Verify retry mechanism effectiveness
        if success_rate < expected_success_rate * 0.9 {
            result.success = false;
            result.error_messages.push("Retry mechanism underperforming".to_string());
        }
        
        Self::disable_message_loss(env);
        
        result
    }
    
    // Helper methods for chaos injection
    
    fn enable_network_partition(env: &Env, affected_nodes: &[Address]) {
        // Implementation would use Soroban's test utilities to isolate nodes
        env.host().set_network_partition(affected_nodes);
    }
    
    fn disable_network_partition(env: &Env) {
        env.host().clear_network_partition();
    }
    
    fn crash_node(env: &Env, node_id: &Address) {
        env.host().crash_node(node_id);
    }
    
    fn restart_node(env: &Env, node_id: &Address) {
        env.host().restart_node(node_id);
    }
    
    fn enable_message_delay(env: &Env, delay_ms: u64, probability: f64) {
        env.host().set_message_delay(delay_ms, probability);
    }
    
    fn disable_message_delay(env: &Env) {
        env.host().clear_message_delay();
    }
    
    fn enable_message_loss(env: &Env, loss_rate: f64) {
        env.host().set_message_loss(loss_rate);
    }
    
    fn disable_message_loss(env: &Env) {
        env.host().clear_message_loss();
    }
    
    fn capture_global_state(env: &Env) -> GlobalState {
        // Capture complete contract state
        GlobalState {
            balances: Self::get_all_balances(env),
            escrows: Self::get_all_escrows(env),
            pending_operations: Self::get_pending_operations(env),
        }
    }
    
    fn verify_state_consistency(before: &GlobalState, after: &GlobalState) -> bool {
        // Verify critical invariants maintained
        before.total_balance() == after.total_balance() &&
        before.escrow_sum() == after.escrow_sum() &&
        before.pending_operations.len() <= after.pending_operations.len()
    }
}
```

### 2. Network Partition Test Scenarios

```rust
mod network_partition_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_partition_during_escrow_creation() {
        let env = Env::default();
        let nodes = setup_multi_node_environment(&env);
        
        // Start escrow creation
        let escrow_initiator = nodes[0].clone();
        let escrow_receiver = nodes[1].clone();
        
        // Initiate escrow
        let escrow_id = initiate_escrow(&env, &escrow_initiator, &escrow_receiver, 1000);
        
        // Inject partition mid-transaction
        let affected_nodes = vec![escrow_receiver.clone()];
        ChaosFramework::enable_network_partition(&env, &affected_nodes);
        
        // Try to complete escrow during partition
        let result = env.try_invoke_contract::<_, ()>(
            &nodes[0],
            &Symbol::new(&env, "complete_escrow"),
            vec![&env, escrow_id.into_val(&env)],
        );
        
        // Should handle gracefully
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("network_unavailable"));
        
        // Heal partition
        ChaosFramework::disable_network_partition(&env);
        
        // Retry should succeed
        let retry_result = env.try_invoke_contract::<_, ()>(
            &nodes[0],
            &Symbol::new(&env, "complete_escrow"),
            vec![&env, escrow_id.into_val(&env)],
        );
        
        assert!(retry_result.is_ok());
    }
    
    #[tokio::test]
    async fn test_partition_split_brain_scenario() {
        let env = Env::default();
        
        // Create two isolated node groups
        let group_a = vec![Address::generate(&env), Address::generate(&env)];
        let group_b = vec![Address::generate(&env), Address::generate(&env)];
        
        // Partition network
        env.host().set_network_partition(&group_b);
        
        // Both groups continue operating independently
        let op_a = perform_operation_in_group(&env, &group_a);
        let op_b = perform_operation_in_group(&env, &group_b);
        
        assert!(op_a.success);
        assert!(op_b.success);
        
        // Heal partition
        env.host().clear_network_partition();
        
        // Verify conflict resolution
        let reconciliation = reconcile_partitions(&env).await;
        assert!(reconciliation.success);
        assert!(!reconciliation.data_loss_detected);
    }
    
    #[tokio::test]
    async fn test_partition_during_reward_distribution() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let recipients: Vec<Address> = (0..10).map(|_| Address::generate(&env)).collect();
        
        // Setup batch reward distribution
        let amounts: Vec<i128> = vec![100; 10];
        
        // Partition affects some recipients
        let partitioned_nodes = recipients[5..].to_vec();
        env.host().set_network_partition(&partitioned_nodes);
        
        // Attempt batch distribution
        let result = env.try_invoke_contract::<_, ()>(
            &admin,
            &Symbol::new(&env, "batch_distribute_rewards"),
            vec![
                &env,
                recipients.clone().into_val(&env),
                amounts.clone().into_val(&env),
            ],
        );
        
        // Should either:
        // 1. Complete for reachable nodes, queue for others
        // 2. Fail atomically and rollback
        
        // Verify state consistency
        let distribution_state = get_distribution_state(&env);
        assert!(distribution_state.consistent);
    }
}
```

### 3. Node Failure Recovery Tests

```rust
mod node_failure_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_validator_node_crash_during_consensus() {
        let env = Env::default();
        let validators = setup_validator_set(&env);
        
        // Start consensus round
        let proposal_id = start_consensus_round(&env, &validators);
        
        // Crash primary validator mid-consensus
        let primary_validator = validators[0].clone();
        ChaosFramework::crash_node(&env, &primary_validator);
        
        // Verify consensus continues without primary
        let consensus_result = wait_for_consensus(&env, proposal_id).await;
        
        assert!(consensus_result.success);
        assert_eq!(consensus_result.participating_validators, validators.len() - 1);
        
        // Restart crashed validator
        ChaosFramework::restart_node(&env, &primary_validator);
        
        // Verify validator syncs and rejoins
        let rejoin_result = wait_for_validator_rejoin(&env, &primary_validator).await;
        assert!(rejoin_result.success);
    }
    
    #[tokio::test]
    async fn test_storage_node_failure_data_recovery() {
        let env = Env::default();
        let storage_nodes = setup_storage_cluster(&env);
        
        // Store data with replication
        let data = vec![1, 2, 3, 4, 5];
        let replication_factor = 3;
        
        let storage_result = store_data_with_replication(&env, &storage_nodes, &data, replication_factor);
        assert!(storage_result.success);
        
        // Crash one storage node
        let failed_node = storage_nodes[0].clone();
        ChaosFramework::crash_node(&env, &failed_node);
        
        // Verify data still accessible from replicas
        let retrieval_result = retrieve_data(&env, &storage_nodes[1..]);
        assert!(retrieval_result.success);
        assert_eq!(retrieval_result.data, data);
        
        // Replace failed node
        let new_node = Address::generate(&env);
        ChaosFramework::restart_node(&env, &new_node);
        
        // Verify data rebalancing
        let rebalance_result = rebalance_storage_cluster(&env, &new_node).await;
        assert!(rebalance_result.success);
    }
    
    #[tokio::test]
    async fn test_cascading_failure_prevention() {
        let env = Env::default();
        let interconnected_nodes = setup_interconnected_services(&env);
        
        // Crash one node
        let initial_failure = interconnected_nodes[0].clone();
        ChaosFramework::crash_node(&env, &initial_failure);
        
        // Monitor for cascading failures
        let cascade_detected = monitor_for_cascading_failures(&env, &interconnected_nodes[1..]);
        
        // System should isolate failure
        assert!(!cascade_detected, "Cascading failure should be prevented");
        
        // Verify circuit breakers activated
        let circuit_breaker_status = check_circuit_breakers(&env);
        assert!(circuit_breaker_status.activated_appropriately);
    }
}
```

### 4. Fault Injection Tests

```rust
mod fault_injection_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_random_message_delays() {
        let env = Env::default();
        
        // Configure random delays
        let delay_config = DelayConfig {
            min_delay_ms: 100,
            max_delay_ms: 5000,
            probability: 0.3,
        };
        
        ChaosFramework::enable_random_delays(&env, &delay_config);
        
        // Run stress test with delays
        let stress_result = run_transaction_stress_test(&env, 1000).await;
        
        // All transactions should eventually complete
        assert!(stress_result.success_rate > 0.95);
        assert!(stress_result.avg_completion_time < Duration::from_secs(10));
        
        ChaosFramework::disable_random_delays(&env);
    }
    
    #[tokio::test]
    async fn test_random_message_corruption() {
        let env = Env::default();
        
        // Enable message corruption
        ChaosFramework::enable_message_corruption(&env, 0.05); // 5% corruption rate
        
        // Send messages with checksums
        let mut corrupted_detected = 0;
        let total_messages = 100;
        
        for _ in 0..total_messages {
            let message = create_test_message();
            let result = send_message_with_checksum(&env, &message);
            
            if result.checksum_mismatch {
                corrupted_detected += 1;
            }
        }
        
        // Verify corruption detection works
        assert!(corrupted_detected > 0, "Should detect some corrupted messages");
        
        // Verify retry mechanism handles corruption
        let retry_success_rate = calculate_retry_success_rate(&env);
        assert!(retry_success_rate > 0.90);
        
        ChaosFramework::disable_message_corruption(&env);
    }
    
    #[tokio::test]
    async fn test_memory_pressure_fault() {
        let env = Env::default();
        
        // Inject memory pressure
        ChaosFramework::inject_memory_pressure(&env, 0.8); // 80% memory usage
        
        // System should continue operating with degraded performance
        let performance_under_pressure = measure_performance(&env);
        
        assert!(performance_under_pressure.operations_per_second > 100);
        assert!(!performance_under_pressure.out_of_memory);
        
        // Verify garbage collection triggers appropriately
        let gc_stats = get_gc_statistics(&env);
        assert!(gc_stats.gc_triggered);
        assert!(gc_stats.memory_recovered > 0);
        
        ChaosFramework::clear_memory_pressure(&env);
    }
    
    #[tokio::test]
    async fn test_cpu_throttling_fault() {
        let env = Env::default();
        
        // Inject CPU throttling
        ChaosFramework::inject_cpu_throttling(&env, 0.5); // 50% CPU reduction
        
        // Verify system adapts
        let adaptation_result = monitor_cpu_adaptation(&env).await;
        
        assert!(adaptation_result.queue_size_stable);
        assert!(adaptation_result.no_timeout_errors);
        
        ChaosFramework::clear_cpu_throttling(&env);
    }
}
```

### 5. Byzantine Fault Tolerance Tests

```rust
mod byzantine_fault_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_double_spend_attempt_detection() {
        let env = Env::default();
        let validators = setup_validator_set(&env);
        let malicious_node = validators[0].clone();
        
        // Attempt double spend
        let tx1 = create_transfer_tx(&malicious_node, 100);
        let tx2 = create_transfer_tx(&malicious_node, 100); // Same inputs
        
        // Broadcast both transactions to different parts of network
        broadcast_to_subset(&env, &tx1, &validators[1..3]);
        broadcast_to_subset(&env, &tx2, &validators[3..5]);
        
        // Wait for consensus
        let consensus_result = wait_for_consensus(&env, tx1.id).await;
        
        // Only one transaction should be accepted
        let tx1_accepted = is_transaction_accepted(&env, tx1.id);
        let tx2_accepted = is_transaction_accepted(&env, tx2.id);
        
        assert!(tx1_accepted ^ tx2_accepted, "Exactly one TX should be accepted (XOR)");
        
        // Malicious node should be flagged
        assert!(is_node_flagged_as_malicious(&env, &malicious_node));
    }
    
    #[tokio::test]
    async fn test_invalid_state_transition_rejection() {
        let env = Env::default();
        let malicious_node = Address::generate(&env);
        
        // Create invalid state transition
        let invalid_transition = create_invalid_state_transition(&malicious_node);
        
        // Submit to network
        let result = submit_state_transition(&env, &invalid_transition);
        
        // Network should reject invalid transition
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("invalid_state_transition"));
        
        // State should remain unchanged
        let current_state = get_contract_state(&env);
        assert_eq!(current_state, get_contract_state(&env));
    }
    
    #[tokio::test]
    async fn test_message_tampering_detection() {
        let env = Env::default();
        
        // Enable message tampering
        ChaosFramework::enable_message_tampering(&env, 0.1);
        
        // Send signed messages
        let mut tampered_detected = 0;
        let total_messages = 100;
        
        for _ in 0..total_messages {
            let message = create_signed_message();
            let result = send_signed_message(&env, &message);
            
            if result.signature_invalid {
                tampered_detected += 1;
            }
        }
        
        // Some tampering should be detected
        assert!(tampered_detected > 0);
        
        // Verify signature verification working
        let valid_messages = count_valid_signatures(&env);
        assert!(valid_messages > total_messages * 80 / 100);
        
        ChaosFramework::disable_message_tampering(&env);
    }
    
    #[tokio::test]
    async fn test_replay_attack_prevention() {
        let env = Env::default();
        
        // Capture valid transaction
        let original_tx = create_valid_transaction(&env);
        execute_transaction(&env, &original_tx);
        
        // Attempt replay
        let replay_result = execute_transaction(&env, &original_tx);
        
        // Should be rejected as duplicate
        assert!(replay_result.is_err());
        assert!(replay_result.err().unwrap().to_string().contains("duplicate_transaction"));
        
        // Verify nonce mechanism
        let sender_nonce = get_sender_nonce(&env, &original_tx.sender);
        assert_eq!(sender_nonce, 1);
    }
}
```

### 6. Chaos Monkey Framework

```rust
pub struct ChaosMonkey {
    config: ChaosMonkeyConfig,
    active_scenarios: Vec<ChaosScenario>,
}

#[derive(Clone)]
pub struct ChaosMonkeyConfig {
    pub enabled: bool,
    pub min_interval_secs: u64,
    pub max_interval_secs: u64,
    pub max_concurrent_scenarios: usize,
    pub allowed_scenarios: Vec<ChaosScenarioType>,
    pub blackout_periods: Vec<TimeRange>,
}

impl ChaosMonkey {
    pub fn new(config: ChaosMonkeyConfig) -> Self {
        Self {
            config,
            active_scenarios: vec![],
        }
    }
    
    /// Run chaos monkey in background
    pub async fn run_background(&mut self, env: Arc<Env>) {
        while self.config.enabled {
            let interval = rand::gen_range(
                self.config.min_interval_secs,
                self.config.max_interval_secs,
            );
            
            sleep(Duration::from_secs(interval)).await;
            
            if self.is_blackout_period() {
                continue;
            }
            
            // Select random chaos scenario
            let scenario = self.select_random_scenario();
            
            // Check if we can run more concurrent scenarios
            if self.active_scenarios.len() >= self.config.max_concurrent_scenarios {
                continue;
            }
            
            // Inject chaos!
            self.inject_chaos(env.clone(), scenario).await;
        }
    }
    
    async fn inject_chaos(&mut self, env: Arc<Env>, scenario: ChaosScenario) {
        println!("🐵 Chaos Monkey injecting: {:?}", scenario);
        
        let result = ChaosFramework::run_chaos_test(&env, scenario.clone()).await;
        
        // Record results
        self.record_chaos_event(&scenario, &result);
        
        // Alert if critical failure
        if !result.success {
            self.send_alert(&scenario, &result);
        }
    }
    
    fn select_random_scenario(&self) -> ChaosScenario {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        
        self.config.allowed_scenarios
            .choose(&mut rng)
            .unwrap()
            .create_random_instance()
    }
    
    fn is_blackout_period(&self) -> bool {
        let now = chrono::Local::now();
        self.config.blackout_periods.iter().any(|period| {
            period.contains(&now)
        })
    }
    
    fn record_chaos_event(&self, scenario: &ChaosScenario, result: &ChaosTestResult) {
        // Log to monitoring system
        println!("Chaos Event: {:?} - Success: {}", scenario, result.success);
    }
    
    fn send_alert(&self, scenario: &ChaosScenario, result: &ChaosTestResult) {
        // Send alert to monitoring dashboard
        eprintln!("🚨 CHAOS ALERT: {:?} failed!", scenario);
    }
}
```

## Monitoring and Metrics

### Key Metrics to Track

```rust
pub struct ChaosMetrics {
    /// Total chaos tests run
    pub total_tests: u64,
    
    /// Success rate
    pub success_rate: f64,
    
    /// Average recovery time
    pub avg_recovery_time_ms: u64,
    
    /// Data loss incidents
    pub data_loss_incidents: u64,
    
    /// State consistency violations
    pub consistency_violations: u64,
    
    /// Most common failure modes
    pub top_failures: Vec<FailureMode>,
}

pub fn generate_chaos_report(metrics: &ChaosMetrics) -> String {
    format!(
        r#"
# Chaos Testing Report

## Summary
- **Total Tests**: {}
- **Success Rate**: {:.2}%
- **Average Recovery Time**: {}ms

## Critical Issues
- **Data Loss Incidents**: {}
- **Consistency Violations**: {}

## Top Failure Modes
{:?}

## Recommendations
{}
        "#,
        metrics.total_tests,
        metrics.success_rate * 100.0,
        metrics.avg_recovery_time_ms,
        metrics.data_loss_incidents,
        metrics.consistency_violations,
        metrics.top_failures,
        generate_recommendations(metrics)
    )
}
```

## Deployment Checklist

- [ ] Setup chaos testing infrastructure
- [ ] Implement network partition tests
- [ ] Implement node failure tests
- [ ] Implement fault injection tests
- [ ] Implement Byzantine fault tests
- [ ] Deploy chaos monkey for continuous testing
- [ ] Setup monitoring and alerting
- [ ] Document failure recovery procedures
- [ ] Train team on chaos engineering principles
- [ ] Schedule regular chaos testing runs
- [ ] Review and act on chaos test results

## References

- [Principles of Chaos Engineering](https://principlesofchaos.org)
- [Netflix Chaos Monkey](https://github.com/Netflix/chaosmonkey)
- [Stellar Network Resilience Patterns](https://soroban.stellar.org/docs)
