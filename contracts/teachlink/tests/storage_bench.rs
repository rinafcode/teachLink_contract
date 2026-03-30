#![cfg(test)]

use crate::{
    types::{LPPosition, LiquidityPool},
    TeachLinkBridge,
};
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, Map};

#[test]
fn test_storage_efficiency() {
    let env = Env::default();

    // Scenario 1: Optimized Approach (Granular Keys)
    // We already refactored the contract to use this.
    let contract_id = env.register(TeachLinkBridge, ());
    let client = crate::TeachLinkBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let chain_id = 1u32;
    let provider = Address::generate(&env);

    // Initialize
    client.initialize(&token, &admin, &1, &admin);

    // Add first liquidity
    env.budget().reset_unlimited();
    client.add_liquidity(&provider, &chain_id, &1000);
    let initial_gas = env.budget().cpu_instruction_count();

    // Add more liquidity to same provider (Granular Key update)
    env.budget().reset_unlimited();
    client.add_liquidity(&provider, &chain_id, &1000);
    let update_gas_optimized = env.budget().cpu_instruction_count();

    // Scenario 2: Simulation of Old Approach (Entire Map update)
    // Let's simulate a large map update to show the overhead.
    env.budget().reset_unlimited();
    let mut large_map: Map<Address, LPPosition> = Map::new(&env);
    for _ in 0..50 {
        large_map.set(
            Address::generate(&env),
            LPPosition {
                provider: Address::generate(&env),
                amount: 100,
                share_percentage: 1,
                deposited_at: 0,
                rewards_earned: 0,
            },
        );
    }

    // "Loading" and "Saving" a simulate large map-in-key
    env.storage()
        .instance()
        .set(&symbol_short!("L_MAP"), &large_map);
    env.budget().reset_unlimited();
    let mut loaded: Map<Address, LPPosition> = env
        .storage()
        .instance()
        .get(&symbol_short!("L_MAP"))
        .unwrap();
    loaded.set(
        provider.clone(),
        LPPosition {
            provider: provider.clone(),
            amount: 100,
            share_percentage: 1,
            deposited_at: 0,
            rewards_earned: 0,
        },
    );
    env.storage()
        .instance()
        .set(&symbol_short!("L_MAP"), &loaded);
    let update_gas_old = env.budget().cpu_instruction_count();

    println!("Optimized update gas: {}", update_gas_optimized);
    println!(
        "Old approach simulated gas (50 entries): {}",
        update_gas_old
    );

    assert!(
        update_gas_optimized < update_gas_old,
        "Optimized approach should be much cheaper"
    );
}
