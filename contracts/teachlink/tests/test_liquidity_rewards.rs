//! LP Reward Integration Tests
//!
//! Verifies end-to-end that LP rewards are drawn from real recorded fee
//! revenue and weighted by time in pool (issue #494):
//! - Rewards are zero until fee revenue is recorded.
//! - Long-held positions earn more than equally-sized flash positions.
//! - Rewards never exceed the fees the pool actually collected.

#![cfg(test)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unreadable_literal)]

mod common;

use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{Address, Env};
use teachlink_contract::{TeachLinkBridge, TeachLinkBridgeClient};

use common::test_env;

const DAY: u64 = 24 * 60 * 60;

fn set_timestamp(env: &Env, timestamp: u64) {
    env.ledger().set(LedgerInfo {
        timestamp,
        protocol_version: 25,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 2_000_000,
    });
}

/// Bridge-initialized environment with a liquidity pool for chain 1.
fn setup_with_pool(env: &Env) -> (TeachLinkBridgeClient<'_>, Address, Address) {
    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(env, &contract_id);
    let token = Address::generate(env);
    let admin = Address::generate(env);
    let fee_recipient = Address::generate(env);
    client.initialize(&token, &admin, &1, &fee_recipient);
    client.add_supported_chain(&1);
    client.initialize_liquidity_pool(&1, &token);
    (client, admin, token)
}

#[test]
fn rewards_are_zero_until_fee_revenue_is_recorded() {
    let env = test_env();
    let (client, _admin, _token) = setup_with_pool(&env);
    let provider = Address::generate(&env);

    client.add_liquidity(&provider, &1, &100_000);
    assert_eq!(client.get_accumulated_fees(&1), 0);

    // No fees recorded: removing liquidity pays out no rewards.
    let returned = client.remove_liquidity(&provider, &1, &100_000);
    assert_eq!(returned, 100_000);
}

#[test]
fn long_term_provider_earns_more_than_flash_provider() {
    let env = test_env();
    let (client, _admin, _token) = setup_with_pool(&env);
    let flash = Address::generate(&env);
    let long = Address::generate(&env);

    // Long-term provider deposits 30 days before the flash provider.
    set_timestamp(&env, 1_000_000);
    client.add_liquidity(&long, &1, &100_000);
    set_timestamp(&env, 1_000_000 + 30 * DAY);
    client.add_liquidity(&flash, &1, &100_000);

    // Pool collects 10_000 in real fee revenue.
    client.record_fee_revenue(&1, &10_000);
    assert_eq!(client.get_accumulated_fees(&1), 10_000);

    // Flash provider (0 seconds in pool, 1.0x): 50% share of fees = 5_000.
    let flash_returned = client.remove_liquidity(&flash, &1, &100_000);
    assert_eq!(flash_returned, 105_000);

    // Long-term provider (30 days in pool, 2.0x): earns more than the flash
    // provider for an equally-sized position.
    let long_returned = client.remove_liquidity(&long, &1, &100_000);
    assert_eq!(long_returned, 110_000);
    assert!(long_returned - 100_000 > flash_returned - 100_000);
}

#[test]
fn rewards_never_exceed_collected_fees() {
    let env = test_env();
    let (client, _admin, _token) = setup_with_pool(&env);
    let provider = Address::generate(&env);

    // Provider holds 100% of the pool for 100 days (max 3.0x multiplier).
    set_timestamp(&env, 1_000_000);
    client.add_liquidity(&provider, &1, &100_000);
    client.record_fee_revenue(&1, &10_000);
    set_timestamp(&env, 1_000_000 + 100 * DAY);

    // The time-weighted reward would be 30_000, but it must be clamped to the
    // 10_000 of fees the pool actually collected.
    let returned = client.remove_liquidity(&provider, &1, &100_000);
    let rewards = returned - 100_000;
    assert_eq!(rewards, 10_000);
    assert!(rewards <= client.get_accumulated_fees(&1));
}
