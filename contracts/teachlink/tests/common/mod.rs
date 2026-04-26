//! Shared fixtures and helpers for TeachLink contract tests.
//!
//! These helpers standardize environment setup, contract registration, and test data
//! generation for the teachlink test suite.
#![allow(clippy::needless_pass_by_value)]

use soroban_sdk::{testutils::Address as _, Address, Bytes, Env};
use teachlink_contract::{TeachLinkBridge, TeachLinkBridgeClient};

/// Returns a fresh test environment with mocked authentication.
pub fn test_env() -> Env {
    let mut env = Env::default();
    env.mock_all_auths();
    env
}

/// Registers the main TeachLink bridge contract and returns its client.
pub fn register_bridge_client(env: &Env) -> TeachLinkBridgeClient<'_> {
    let contract_id = env.register(TeachLinkBridge, ());
    TeachLinkBridgeClient::new(env, &contract_id)
}

/// Generates a new random address inside the given environment.
pub fn random_address(env: &Env) -> Address {
    Address::generate(env)
}

/// Creates a Bytes payload from a UTF-8 string.
pub fn bytes(env: &Env, value: &str) -> Bytes {
    Bytes::from_slice(env, value.as_bytes())
}

/// Sets up a simple bridge test fixture with a deployer, creator, and student.
pub fn setup_bridge_test(env: &Env) -> (TeachLinkBridgeClient<'_>, Address, Address) {
    let client = register_bridge_client(env);
    let creator = random_address(env);
    let student = random_address(env);
    (client, creator, student)
}
