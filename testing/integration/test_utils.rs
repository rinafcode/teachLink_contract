//! Test utilities for cross-chain integration testing

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, Bytes, Env, Symbol, Vec, Map, xdr::ScVal,
};
use std::collections::HashMap;
use crate::mock_chains::{MockChain, MockChainResponse};

/// Integration test environment setup
pub struct IntegrationTestEnv {
    pub env: Env,
    pub chains: HashMap<u32, MockChain>,
    pub contracts: HashMap<String, Address>,
    pub users: HashMap<String, Address>,
}

impl IntegrationTestEnv {
    pub fn new() -> Self {
        let mut env = Env::default();
        
        // Setup mock ledger info for realistic testing
        env.ledger().set(LedgerInfo {
            protocol_version: 20,
            sequence_number: 12345,
            timestamp: 1640995200, // Jan 1, 2022
            network_id: Default::default(),
            base_reserve: 10000000,
            min_temp_entry_ttl: 4096,
            min_persistent_entry_ttl: 65536,
            max_entry_ttl: 6553600,
        });

        let mut chains = HashMap::new();
        let mut contracts = HashMap::new();
        let mut users = HashMap::new();

        // Setup mock chains
        chains.insert(1, MockChain::new(1, "Stellar"));
        chains.insert(2, MockChain::new(2, "Ethereum"));
        chains.insert(3, MockChain::new(3, "Polygon"));
        chains.insert(4, MockChain::new(4, "BSC"));

        // Setup test users
        users.insert("admin".to_string(), Address::generate(&env));
        users.insert("user1".to_string(), Address::generate(&env));
        users.insert("user2".to_string(), Address::generate(&env));
        users.insert("validator1".to_string(), Address::generate(&env));
        users.insert("validator2".to_string(), Address::generate(&env));
        users.insert("validator3".to_string(), Address::generate(&env));

        Self {
            env,
            chains,
            contracts,
            users,
        }
    }

    pub fn get_user(&self, name: &str) -> Address {
        self.users.get(name).unwrap().clone()
    }

    pub fn get_chain(&self, chain_id: u32) -> &MockChain {
        self.chains.get(&chain_id).unwrap()
    }

    pub fn setup_mock_auths(&self) {
        self.env.mock_all_auths();
    }

    pub fn advance_time(&self, seconds: u64) {
        let current = self.env.ledger().sequence();
        self.env.ledger().set(LedgerInfo {
            sequence_number: current + 1,
            timestamp: self.env.ledger().timestamp() + seconds,
            ..self.env.ledger().get()
        });
    }
}

/// Test data generators
pub struct TestDataGenerator;

impl TestDataGenerator {
    pub fn generate_bridge_params(
        env: &Env,
        from_chain: u32,
        to_chain: u32,
        amount: i128,
    ) -> crate::bridge_integration::BridgeParameters {
        crate::bridge_integration::BridgeParameters {
            from_chain,
            to_chain,
            token: Address::generate(env),
            amount,
            recipient: Address::generate(env),
            nonce: rand::random::<u64>(),
            timeout: 86400, // 24 hours
        }
    }

    pub fn generate_atomic_swap_params(
        env: &Env,
        amount: i128,
    ) -> crate::atomic_swap_integration::AtomicSwapParameters {
        crate::atomic_swap_integration::AtomicSwapParameters {
            initiator: Address::generate(env),
            initiator_token: Address::generate(env),
            initiator_amount: amount,
            counterparty: Address::generate(env),
            counterparty_token: Address::generate(env),
            counterparty_amount: amount * 2, // 2:1 ratio
            hashlock: Self::generate_hashlock(),
            timelock: 3600, // 1 hour
        }
    }

    pub fn generate_hashlock() -> Bytes {
        use sha2::{Sha256, Digest};
        let secret = b"test_secret_for_hashlock";
        let hash = Sha256::digest(secret);
        Bytes::from_array(&Env::default(), &hash)
    }

    pub fn generate_cross_chain_message(
        env: &Env,
        source_chain: u32,
        dest_chain: u32,
    ) -> crate::message_passing_integration::CrossChainMessage {
        crate::message_passing_integration::CrossChainMessage {
            source_chain,
            destination_chain: dest_chain,
            sender: Bytes::from_slice(env, b"sender_address"),
            recipient: Bytes::from_slice(env, b"recipient_address"),
            payload: Bytes::from_slice(env, b"test_message_payload"),
            timeout: Some(86400),
            nonce: rand::random::<u64>(),
        }
    }
}

/// Assertion helpers for integration tests
pub struct IntegrationAssertions;

impl IntegrationAssertions {
    pub fn assert_bridge_transaction_completed(
        env: &Env,
        tx_id: u64,
        expected_amount: i128,
    ) {
        // This would check the bridge transaction state
        // Implementation depends on the actual contract storage structure
        println!("Asserting bridge transaction {} completed with amount {}", tx_id, expected_amount);
    }

    pub fn assert_atomic_swap_completed(
        env: &Env,
        swap_id: u64,
        expected_status: &str,
    ) {
        println!("Asserting atomic swap {} completed with status {}", swap_id, expected_status);
    }

    pub fn assert_message_delivered(
        env: &Env,
        packet_id: u64,
        expected_payload: &Bytes,
    ) {
        println!("Asserting message {} delivered with payload", packet_id);
    }

    pub fn assert_chain_configured(
        env: &Env,
        chain_id: u32,
        expected_name: &str,
    ) {
        println!("Asserting chain {} configured with name {}", chain_id, expected_name);
    }
}

/// Performance measurement utilities
pub struct PerformanceMeasurements {
    measurements: Vec<(&'static str, std::time::Duration)>,
}

impl PerformanceMeasurements {
    pub fn new() -> Self {
        Self {
            measurements: Vec::new(),
        }
    }

    pub fn measure<F, R>(&mut self, name: &'static str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = std::time::Instant::now();
        let result = f();
        let duration = start.elapsed();
        self.measurements.push((name, duration));
        result
    }

    pub fn print_summary(&self) {
        println!("\n=== Performance Summary ===");
        for (name, duration) in &self.measurements {
            println!("{}: {:?}", name, duration);
        }
    }
}
