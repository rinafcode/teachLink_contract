//! Cross-Chain Integration Testing Library
//! 
//! This library provides comprehensive integration testing for all cross-chain operations
//! in the TeachLink contract system.

pub mod bridge_integration;
pub mod atomic_swap_integration;
pub mod message_passing_integration;
pub mod multichain_integration;
pub mod mock_chains;
pub mod test_utils;
pub mod failure_scenarios;

pub use bridge_integration::*;
pub use atomic_swap_integration::*;
pub use message_passing_integration::*;
pub use multichain_integration::*;
pub use mock_chains::*;
pub use test_utils::*;
pub use failure_scenarios::*;

/// Main integration test runner
pub struct IntegrationTestRunner;

impl IntegrationTestRunner {
    /// Run all integration tests
    pub fn run_all_tests() -> Result<(), String> {
        println!("=== Starting Cross-Chain Integration Tests ===\n");
        
        // Basic integration tests
        Self::run_bridge_integration_tests()?;
        Self::run_atomic_swap_integration_tests()?;
        Self::run_message_passing_integration_tests()?;
        Self::run_multichain_integration_tests()?;
        
        // Failure scenario tests
        Self::run_failure_scenario_tests()?;
        
        // Performance tests
        Self::run_performance_tests()?;
        
        println!("=== All Integration Tests Completed Successfully ===");
        Ok(())
    }
    
    /// Run bridge integration tests
    pub fn run_bridge_integration_tests() -> Result<(), String> {
        println!("Running Bridge Integration Tests...");
        
        BridgeIntegrationTests::test_complete_bridge_workflow();
        BridgeIntegrationTests::test_bridge_failure_scenarios();
        
        println!("✅ Bridge Integration Tests Completed\n");
        Ok(())
    }
    
    /// Run atomic swap integration tests
    pub fn run_atomic_swap_integration_tests() -> Result<(), String> {
        println!("Running Atomic Swap Integration Tests...");
        
        AtomicSwapIntegrationTests::test_complete_atomic_swap_workflow();
        AtomicSwapIntegrationTests::test_cross_chain_atomic_swap();
        AtomicSwapIntegrationTests::test_atomic_swap_failure_scenarios();
        
        println!("✅ Atomic Swap Integration Tests Completed\n");
        Ok(())
    }
    
    /// Run message passing integration tests
    pub fn run_message_passing_integration_tests() -> Result<(), String> {
        println!("Running Message Passing Integration Tests...");
        
        MessagePassingIntegrationTests::test_complete_message_passing_workflow();
        MessagePassingIntegrationTests::test_message_passing_failure_scenarios();
        MessagePassingIntegrationTests::test_high_volume_messaging();
        MessagePassingIntegrationTests::test_message_ordering();
        
        println!("✅ Message Passing Integration Tests Completed\n");
        Ok(())
    }
    
    /// Run multichain integration tests
    pub fn run_multichain_integration_tests() -> Result<(), String> {
        println!("Running Multi-Chain Integration Tests...");
        
        MultiChainIntegrationTests::test_complete_multichain_workflow();
        MultiChainIntegrationTests::test_multichain_failure_scenarios();
        MultiChainIntegrationTests::test_high_throughput_multichain();
        
        println!("✅ Multi-Chain Integration Tests Completed\n");
        Ok(())
    }
    
    /// Run failure scenario tests
    pub fn run_failure_scenario_tests() -> Result<(), String> {
        println!("Running Failure Scenario Tests...");
        
        FailureScenarioTests::run_all_failure_tests();
        FailureScenarioTests::test_recovery_mechanisms();
        
        println!("✅ Failure Scenario Tests Completed\n");
        Ok(())
    }
    
    /// Run performance tests
    pub fn run_performance_tests() -> Result<(), String> {
        println!("Running Performance Tests...");
        
        Self::test_bridge_performance()?;
        Self::test_atomic_swap_performance()?;
        Self::test_message_passing_performance()?;
        Self::test_multichain_performance()?;
        
        println!("✅ Performance Tests Completed\n");
        Ok(())
    }
    
    fn test_bridge_performance() -> Result<(), String> {
        println!("Testing bridge performance...");
        
        let test_env = IntegrationTestEnv::new();
        let mut perf = PerformanceMeasurements::new();
        
        // Test multiple concurrent bridge operations
        for i in 0..50 {
            perf.measure(&format!("bridge_op_{}", i), || {
                let user1 = test_env.get_user("user1");
                let user2 = test_env.get_user("user2");
                BridgeIntegrationTests::test_bridge_transfer(&test_env, user1, user2);
            });
        }
        
        perf.print_summary();
        Ok(())
    }
    
    fn test_atomic_swap_performance() -> Result<(), String> {
        println!("Testing atomic swap performance...");
        
        let test_env = IntegrationTestEnv::new();
        let mut perf = PerformanceMeasurements::new();
        
        // Test multiple concurrent atomic swaps
        for i in 0..25 {
            perf.measure(&format!("swap_op_{}", i), || {
                let user1 = test_env.get_user("user1");
                let user2 = test_env.get_user("user2");
                let swap_id = AtomicSwapIntegrationTests::test_swap_initiation(&test_env, user1, user2);
                AtomicSwapIntegrationTests::test_swap_participation(&test_env, swap_id, user2);
            });
        }
        
        perf.print_summary();
        Ok(())
    }
    
    fn test_message_passing_performance() -> Result<(), String> {
        println!("Testing message passing performance...");
        
        let test_env = IntegrationTestEnv::new();
        let mut perf = PerformanceMeasurements::new();
        
        // Test high volume messaging
        perf.measure("high_volume_messaging", || {
            MessagePassingIntegrationTests::test_high_volume_messaging();
        });
        
        perf.print_summary();
        Ok(())
    }
    
    fn test_multichain_performance() -> Result<(), String> {
        println!("Testing multi-chain performance...");
        
        let test_env = IntegrationTestEnv::new();
        let mut perf = PerformanceMeasurements::new();
        
        // Test high throughput multi-chain operations
        perf.measure("high_throughput_multichain", || {
            MultiChainIntegrationTests::test_high_throughput_multichain();
        });
        
        perf.print_summary();
        Ok(())
    }
    
    /// Run smoke tests for quick validation
    pub fn run_smoke_tests() -> Result<(), String> {
        println!("=== Running Smoke Tests ===\n");
        
        // Quick validation tests
        Self::test_basic_bridge_operation()?;
        Self::test_basic_atomic_swap()?;
        Self::test_basic_message_passing()?;
        Self::test_basic_multichain()?;
        
        println!("=== Smoke Tests Completed ===");
        Ok(())
    }
    
    fn test_basic_bridge_operation() -> Result<(), String> {
        println!("Testing basic bridge operation...");
        
        let test_env = IntegrationTestEnv::new();
        let user1 = test_env.get_user("user1");
        let user2 = test_env.get_user("user2");
        
        BridgeIntegrationTests::test_bridge_transfer(&test_env, user1, user2);
        
        println!("✅ Basic bridge operation test passed");
        Ok(())
    }
    
    fn test_basic_atomic_swap() -> Result<(), String> {
        println!("Testing basic atomic swap...");
        
        let test_env = IntegrationTestEnv::new();
        let user1 = test_env.get_user("user1");
        let user2 = test_env.get_user("user2");
        
        let swap_id = AtomicSwapIntegrationTests::test_swap_initiation(&test_env, user1, user2);
        AtomicSwapIntegrationTests::test_swap_participation(&test_env, swap_id, user2);
        
        println!("✅ Basic atomic swap test passed");
        Ok(())
    }
    
    fn test_basic_message_passing() -> Result<(), String> {
        println!("Testing basic message passing...");
        
        let test_env = IntegrationTestEnv::new();
        let packet_id = MessagePassingIntegrationTests::test_message_sending(&test_env);
        MessagePassingIntegrationTests::test_message_delivery(&test_env, packet_id);
        
        println!("✅ Basic message passing test passed");
        Ok(())
    }
    
    fn test_basic_multichain() -> Result<(), String> {
        println!("Testing basic multichain operation...");
        
        let test_env = IntegrationTestEnv::new();
        MultiChainIntegrationTests::test_chain_configuration(&test_env);
        
        println!("✅ Basic multichain test passed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_integration_runner() {
        // Quick test to verify the integration test runner works
        assert!(IntegrationTestRunner::run_smoke_tests().is_ok());
    }
}
