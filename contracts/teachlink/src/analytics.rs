#![no_std]

use soroban_sdk::{
    contractevent, Address, Bytes, Env, Map, Symbol, Vec,
};

/// Storage keys used for analytics tracking
#[derive(Clone)]
#[repr(u32)]
pub enum DataKey {
    Views = 0,
    Purchases = 1,
    Revenue = 2,
    BridgeStats = 3,
}

/// Event emitted when content is viewed
#[contractevent]
pub struct ContentViewed {
    pub token_id: Bytes,
}

/// Event emitted when content is purchased
#[contractevent]
pub struct ContentPurchased {
    pub token_id: Bytes,
    pub buyer: Address,
}

/// Record a content view
pub fn record_view(env: &Env, token_id: Bytes) {
    let mut views: Map<Bytes, u64> =
        env.storage().instance().get(&DataKey::Views).unwrap_or(Map::new(env));

    let count = views.get(token_id.clone()).unwrap_or(0);
    views.set(token_id.clone(), count + 1);

    env.storage().instance().set(&DataKey::Views, &views);

    ContentViewed { token_id }.publish(env);
}

/// Record a purchase with amount (used for revenue tracking)
pub fn record_purchase(
    env: &Env,
    token_id: Bytes,
    buyer: Address,
    amount: i128,
) {
    // Update purchase count
    let mut purchases: Map<Bytes, u64> =
        env.storage().instance().get(&DataKey::Purchases).unwrap_or(Map::new(env));

    let count = purchases.get(token_id.clone()).unwrap_or(0);
    purchases.set(token_id.clone(), count + 1);
    env.storage().instance().set(&DataKey::Purchases, &purchases);

    // Update revenue
    let mut revenue: Map<Bytes, i128> =
        env.storage().instance().get(&DataKey::Revenue).unwrap_or(Map::new(env));

    let total = revenue.get(token_id.clone()).unwrap_or(0);
    revenue.set(token_id.clone(), total + amount);
    env.storage().instance().set(&DataKey::Revenue, &revenue);

    ContentPurchased { token_id, buyer }.publish(env);
}

/// Get total views for a specific content
pub fn get_views(env: &Env, token_id: Bytes) -> u64 {
    let views: Map<Bytes, u64> =
        env.storage().instance().get(&DataKey::Views).unwrap_or(Map::new(env));

    views.get(token_id).unwrap_or(0)
}

/// Get total purchases for a specific content
pub fn get_purchases(env: &Env, token_id: Bytes) -> u64 {
    let purchases: Map<Bytes, u64> =
        env.storage().instance().get(&DataKey::Purchases).unwrap_or(Map::new(env));

    purchases.get(token_id).unwrap_or(0)
}

/// Get total revenue for a specific content
pub fn get_revenue(env: &Env, token_id: Bytes) -> i128 {
    let revenue: Map<Bytes, i128> =
        env.storage().instance().get(&DataKey::Revenue).unwrap_or(Map::new(env));

    revenue.get(token_id).unwrap_or(0)
}

/// Store bridge statistics (cross-chain or external integrations)
pub fn set_bridge_stat(env: &Env, key: Bytes, value: i128) {
    let mut stats: Map<Bytes, i128> =
        env.storage().instance().get(&DataKey::BridgeStats).unwrap_or(Map::new(env));

    stats.set(key, value);
    env.storage().instance().set(&DataKey::BridgeStats, &stats);
}

/// Retrieve all bridge statistics
pub fn get_bridge_statistics(env: &Env) -> Map<Bytes, i128> {
    env.storage()
        .instance()
        .get(&DataKey::BridgeStats)
        .unwrap_or(Map::new(env))
}