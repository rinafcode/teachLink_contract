#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, Map};
use teachlink_contract::TeachLinkBridge;

mod helpers {
    use super::*;

    pub fn setup() -> (Env, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());
        (env, contract_id)
    }
}

use helpers::*;

// ── Pool Initialization ────────────────────────────────────────────

#[test]
fn test_initialize_liquidity_pool() {
    let (env, cid) = setup();
    let token = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.initialize_liquidity_pool(&1, &token);

    assert_eq!(client.get_available_liquidity(&1), 0);
}

// ── Add / Remove Liquidity ─────────────────────────────────────────

#[test]
fn test_add_liquidity() {
    let (env, cid) = setup();
    let token = Address::generate(&env);
    let provider = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.initialize_liquidity_pool(&1, &token);
    let share = client.add_liquidity(&provider, &1, &500_000);

    // First provider gets 100% share (10000 bps)
    assert_eq!(share, 10000);
    assert_eq!(client.get_available_liquidity(&1), 500_000);
}

#[test]
fn test_add_liquidity_zero_fails() {
    let (env, cid) = setup();
    let token = Address::generate(&env);
    let provider = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.initialize_liquidity_pool(&1, &token);
    let result = client.try_add_liquidity(&provider, &1, &0);
    assert!(result.is_err());
}

#[test]
fn test_add_liquidity_negative_fails() {
    let (env, cid) = setup();
    let token = Address::generate(&env);
    let provider = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.initialize_liquidity_pool(&1, &token);
    let result = client.try_add_liquidity(&provider, &1, &-100);
    assert!(result.is_err());
}

#[test]
fn test_add_liquidity_nonexistent_pool_fails() {
    let (env, cid) = setup();
    let provider = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    let result = client.try_add_liquidity(&provider, &99, &1000);
    assert!(result.is_err());
}

#[test]
fn test_remove_liquidity() {
    let (env, cid) = setup();
    let token = Address::generate(&env);
    let provider = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.initialize_liquidity_pool(&1, &token);
    client.add_liquidity(&provider, &1, &500_000);

    let returned = client.remove_liquidity(&provider, &1, &200_000);
    // returned = amount + rewards
    assert!(returned >= 200_000);
    assert_eq!(client.get_available_liquidity(&1), 300_000);
}

#[test]
fn test_remove_liquidity_exceeds_position_fails() {
    let (env, cid) = setup();
    let token = Address::generate(&env);
    let provider = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.initialize_liquidity_pool(&1, &token);
    client.add_liquidity(&provider, &1, &100_000);

    let result = client.try_remove_liquidity(&provider, &1, &200_000);
    assert!(result.is_err());
}

#[test]
fn test_remove_liquidity_no_position_fails() {
    let (env, cid) = setup();
    let token = Address::generate(&env);
    let stranger = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.initialize_liquidity_pool(&1, &token);
    let result = client.try_remove_liquidity(&stranger, &1, &1000);
    assert!(result.is_err());
}

// ── Fee Calculation ────────────────────────────────────────────────

#[test]
fn test_calculate_bridge_fee_default() {
    let (env, cid) = setup();
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    let fee = client.calculate_bridge_fee(&1, &1_000_000, &0);
    // Base fee is 10 bps = 0.1%, so fee on 1M = 1000
    // Clamped between MIN_FEE_BPS (1 bps) and MAX_FEE_BPS (500 bps)
    assert!(fee > 0);
    assert!(fee <= 1_000_000 * 500 / 10000); // max 5%
}

#[test]
fn test_calculate_bridge_fee_with_congestion() {
    let (env, cid) = setup();
    let token = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.initialize_liquidity_pool(&1, &token);
    // No congestion when pool is empty (returns 100 multiplier)
    let fee_no_pool = client.calculate_bridge_fee(&1, &1_000_000, &0);
    assert!(fee_no_pool > 0);
}

// ── Update Fee Structure ───────────────────────────────────────────

#[test]
fn test_update_fee_structure() {
    let (env, cid) = setup();
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    let tiers: Map<u32, u32> = Map::new(&env);
    client.update_fee_structure(&20, &150, &tiers);

    let fs = client.get_fee_structure();
    assert_eq!(fs.base_fee, 20);
    assert_eq!(fs.dynamic_multiplier, 150);
}

#[test]
fn test_update_fee_structure_out_of_range_fails() {
    let (env, cid) = setup();
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    let tiers: Map<u32, u32> = Map::new(&env);
    // base_fee > MAX_FEE_BPS (500)
    let result = client.try_update_fee_structure(&600, &100, &tiers);
    assert!(result.is_err());
}

// ── Sufficient Liquidity Check ─────────────────────────────────────

#[test]
fn test_has_sufficient_liquidity() {
    let (env, cid) = setup();
    let token = Address::generate(&env);
    let provider = Address::generate(&env);
    let client = teachlink_contract::TeachLinkBridgeClient::new(&env, &cid);

    client.initialize_liquidity_pool(&1, &token);
    client.add_liquidity(&provider, &1, &500_000);

    assert!(client.has_sufficient_liquidity(&1, &400_000));
    assert!(!client.has_sufficient_liquidity(&1, &600_000));
    assert!(!client.has_sufficient_liquidity(&99, &1)); // nonexistent pool
}
