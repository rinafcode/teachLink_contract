//! Bridge Liquidity and AMM Module
//!
//! This module implements liquidity pool management and automated market making
//! for optimizing bridge operations and dynamic fee pricing.

use crate::errors::BridgeError;
use crate::events::{FeeUpdatedEvent, LiquidityAddedEvent, LiquidityRemovedEvent};
use crate::storage::{FEE_STRUCTURE, LIQUIDITY_POOLS, LP_POSITIONS};
use crate::types::{BridgeFeeStructure, LPPosition, LiquidityPool};
use soroban_sdk::{Address, Env, Map, Vec};

/// Base fee in basis points (0.1%)
pub const BASE_FEE_BPS: i128 = 10;

/// Maximum fee in basis points (5%)
pub const MAX_FEE_BPS: i128 = 500;

/// Minimum fee in basis points (0.01%)
pub const MIN_FEE_BPS: i128 = 1;

/// Liquidity utilization threshold for dynamic pricing (80%)
pub const UTILIZATION_THRESHOLD: u32 = 8000;

/// Congestion multiplier steps
pub const CONGESTION_STEP_1: u32 = 5000; // 50% utilization
pub const CONGESTION_STEP_2: u32 = 7000; // 70% utilization
pub const CONGESTION_STEP_3: u32 = 9000; // 90% utilization

/// Liquidity Manager
pub struct LiquidityManager;

impl LiquidityManager {
    /// Initialize liquidity pool for a chain
    pub fn initialize_pool(env: &Env, chain_id: u32, token: Address) -> Result<(), BridgeError> {
        let pool = LiquidityPool {
            chain_id,
            token: token.clone(),
            total_liquidity: 0,
            available_liquidity: 0,
            locked_liquidity: 0,
            lp_providers: Map::new(env),
        };

        let mut pools: Map<u32, LiquidityPool> = env
            .storage()
            .instance()
            .get(&LIQUIDITY_POOLS)
            .unwrap_or_else(|| Map::new(env));
        pools.set(chain_id, pool);
        env.storage().instance().set(&LIQUIDITY_POOLS, &pools);

        Ok(())
    }

    /// Add liquidity to a pool
    pub fn add_liquidity(
        env: &Env,
        provider: Address,
        chain_id: u32,
        amount: i128,
    ) -> Result<u32, BridgeError> {
        provider.require_auth();

        if amount <= 0 {
            return Err(BridgeError::AmountMustBePositive);
        }

        // Get pool
        let mut pools: Map<u32, LiquidityPool> = env
            .storage()
            .instance()
            .get(&LIQUIDITY_POOLS)
            .unwrap_or_else(|| Map::new(env));
        let mut pool = pools
            .get(chain_id)
            .ok_or(BridgeError::DestinationChainNotSupported)?;

        // Calculate share percentage
        let share_percentage = if pool.total_liquidity == 0 {
            10000u32 // 100% for first provider
        } else {
            ((amount * 10000) / pool.total_liquidity) as u32
        };

        // Update pool
        pool.total_liquidity += amount;
        pool.available_liquidity += amount;

        // Create or update LP position
        let mut lp_positions: Map<Address, LPPosition> = pool.lp_providers;
        let position = if let Some(mut existing) = lp_positions.get(provider.clone()) {
            existing.amount += amount;
            existing.share_percentage = ((existing.amount * 10000) / pool.total_liquidity) as u32;
            existing
        } else {
            LPPosition {
                provider: provider.clone(),
                amount,
                share_percentage,
                deposited_at: env.ledger().timestamp(),
                rewards_earned: 0,
            }
        };

        lp_positions.set(provider.clone(), position);
        pool.lp_providers = lp_positions;

        // Update pool storage
        pools.set(chain_id, pool);
        env.storage().instance().set(&LIQUIDITY_POOLS, &pools);

        // Emit event
        LiquidityAddedEvent {
            provider: provider.clone(),
            chain_id,
            amount,
            share_percentage,
        }
        .publish(env);

        Ok(share_percentage)
    }

    /// Remove liquidity from a pool
    pub fn remove_liquidity(
        env: &Env,
        provider: Address,
        chain_id: u32,
        amount: i128,
    ) -> Result<i128, BridgeError> {
        provider.require_auth();

        if amount <= 0 {
            return Err(BridgeError::AmountMustBePositive);
        }

        // Get pool
        let mut pools: Map<u32, LiquidityPool> = env
            .storage()
            .instance()
            .get(&LIQUIDITY_POOLS)
            .unwrap_or_else(|| Map::new(env));
        let mut pool = pools
            .get(chain_id)
            .ok_or(BridgeError::DestinationChainNotSupported)?;

        // Get LP position
        let mut lp_positions: Map<Address, LPPosition> = pool.lp_providers.clone();
        let mut position = lp_positions
            .get(provider.clone())
            .ok_or(BridgeError::InvalidInput)?;

        if amount > position.amount {
            return Err(BridgeError::InsufficientBalance);
        }

        // Calculate rewards
        let rewards = Self::calculate_lp_rewards(env, &position, pool.total_liquidity);

        // Update position
        position.amount -= amount;
        position.rewards_earned += rewards;

        if position.amount == 0 {
            lp_positions.remove(provider.clone());
        } else {
            position.share_percentage = ((position.amount * 10000) / pool.total_liquidity) as u32;
            lp_positions.set(provider.clone(), position.clone());
        }

        // Update pool
        pool.total_liquidity -= amount;
        pool.available_liquidity -= amount;
        pool.lp_providers = lp_positions;

        // Update pool storage
        pools.set(chain_id, pool);
        env.storage().instance().set(&LIQUIDITY_POOLS, &pools);

        // Emit event
        LiquidityRemovedEvent {
            provider: provider.clone(),
            chain_id,
            amount,
            rewards,
        }
        .publish(env);

        Ok(amount + rewards)
    }

    /// Lock liquidity for a bridge transaction
    pub fn lock_liquidity(env: &Env, chain_id: u32, amount: i128) -> Result<(), BridgeError> {
        if amount <= 0 {
            return Err(BridgeError::AmountMustBePositive);
        }

        // Get pool
        let mut pools: Map<u32, LiquidityPool> = env
            .storage()
            .instance()
            .get(&LIQUIDITY_POOLS)
            .unwrap_or_else(|| Map::new(env));
        let mut pool = pools
            .get(chain_id)
            .ok_or(BridgeError::DestinationChainNotSupported)?;

        // Check available liquidity
        if amount > pool.available_liquidity {
            return Err(BridgeError::InsufficientLiquidity);
        }

        // Lock liquidity
        pool.available_liquidity -= amount;
        pool.locked_liquidity += amount;

        // Update pool storage
        pools.set(chain_id, pool);
        env.storage().instance().set(&LIQUIDITY_POOLS, &pools);

        Ok(())
    }

    /// Unlock liquidity after bridge completion
    pub fn unlock_liquidity(env: &Env, chain_id: u32, amount: i128) -> Result<(), BridgeError> {
        if amount <= 0 {
            return Err(BridgeError::AmountMustBePositive);
        }

        // Get pool
        let mut pools: Map<u32, LiquidityPool> = env
            .storage()
            .instance()
            .get(&LIQUIDITY_POOLS)
            .unwrap_or_else(|| Map::new(env));
        let mut pool = pools
            .get(chain_id)
            .ok_or(BridgeError::DestinationChainNotSupported)?;

        // Unlock liquidity
        pool.locked_liquidity -= amount;
        pool.available_liquidity += amount;

        // Update pool storage
        pools.set(chain_id, pool);
        env.storage().instance().set(&LIQUIDITY_POOLS, &pools);

        Ok(())
    }

    /// Calculate dynamic bridge fee
    pub fn calculate_bridge_fee(
        env: &Env,
        chain_id: u32,
        amount: i128,
        user_volume_24h: i128,
    ) -> Result<i128, BridgeError> {
        // Get fee structure
        let fee_structure: BridgeFeeStructure = env
            .storage()
            .instance()
            .get(&FEE_STRUCTURE)
            .unwrap_or(BridgeFeeStructure {
                base_fee: BASE_FEE_BPS,
                dynamic_multiplier: 100,    // 1x
                congestion_multiplier: 100, // 1x
                volume_discount_tiers: Self::default_volume_tiers(env),
                last_updated: env.ledger().timestamp(),
            });

        // Get pool for congestion calculation
        let pools: Map<u32, LiquidityPool> = env
            .storage()
            .instance()
            .get(&LIQUIDITY_POOLS)
            .unwrap_or_else(|| Map::new(env));

        let congestion_multiplier = if let Some(pool) = pools.get(chain_id) {
            Self::calculate_congestion_multiplier(&pool)
        } else {
            100u32
        };

        // Calculate volume discount
        let volume_discount =
            Self::calculate_volume_discount(&fee_structure.volume_discount_tiers, user_volume_24h);

        // Calculate final fee
        let base_fee_amount = (amount * fee_structure.base_fee) / 10000;
        let congestion_adjusted = (base_fee_amount * congestion_multiplier as i128) / 100;
        let final_fee = (congestion_adjusted * (10000 - volume_discount as i128)) / 10000;

        // Ensure fee is within bounds
        let min_fee = (amount * MIN_FEE_BPS) / 10000;
        let max_fee = (amount * MAX_FEE_BPS) / 10000;

        Ok(final_fee.clamp(min_fee, max_fee))
    }

    /// Update fee structure
    pub fn update_fee_structure(
        env: &Env,
        base_fee: i128,
        dynamic_multiplier: u32,
        volume_discount_tiers: Map<u32, u32>,
    ) -> Result<(), BridgeError> {
        if base_fee < MIN_FEE_BPS || base_fee > MAX_FEE_BPS {
            return Err(BridgeError::FeeCannotBeNegative);
        }

        let old_fee_structure: BridgeFeeStructure = env
            .storage()
            .instance()
            .get(&FEE_STRUCTURE)
            .unwrap_or(BridgeFeeStructure {
                base_fee: BASE_FEE_BPS,
                dynamic_multiplier: 100,
                congestion_multiplier: 100,
                volume_discount_tiers: Self::default_volume_tiers(env),
                last_updated: env.ledger().timestamp(),
            });

        let new_fee_structure = BridgeFeeStructure {
            base_fee,
            dynamic_multiplier,
            congestion_multiplier: old_fee_structure.congestion_multiplier,
            volume_discount_tiers,
            last_updated: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&FEE_STRUCTURE, &new_fee_structure);

        // Emit event
        FeeUpdatedEvent {
            old_fee: old_fee_structure.base_fee,
            new_fee: base_fee,
            multiplier: dynamic_multiplier,
        }
        .publish(env);

        Ok(())
    }

    /// Calculate congestion multiplier based on pool utilization
    fn calculate_congestion_multiplier(pool: &LiquidityPool) -> u32 {
        if pool.total_liquidity == 0 {
            return 100;
        }

        let utilization = ((pool.locked_liquidity * 10000) / pool.total_liquidity) as u32;

        if utilization < CONGESTION_STEP_1 {
            100 // 1x
        } else if utilization < CONGESTION_STEP_2 {
            150 // 1.5x
        } else if utilization < CONGESTION_STEP_3 {
            200 // 2x
        } else {
            300 // 3x
        }
    }

    /// Calculate volume discount based on 24h volume
    fn calculate_volume_discount(volume_tiers: &Map<u32, u32>, user_volume_24h: i128) -> u32 {
        let mut discount = 0u32;

        for (threshold, tier_discount) in volume_tiers.iter() {
            if user_volume_24h >= threshold as i128 && tier_discount > discount {
                discount = tier_discount;
            }
        }

        discount
    }

    /// Calculate LP rewards based on position and pool performance
    fn calculate_lp_rewards(_env: &Env, position: &LPPosition, total_liquidity: i128) -> i128 {
        if total_liquidity == 0 {
            return 0;
        }

        // Simple reward calculation based on share and time
        let time_factor = 1i128; // Could be based on time in pool
        let share_factor = (position.amount * 10000) / total_liquidity;

        (position.amount * share_factor * time_factor) / 1000000
    }

    /// Default volume discount tiers
    fn default_volume_tiers(env: &Env) -> Map<u32, u32> {
        let mut tiers = Map::new(env);
        tiers.set(10000u32, 0u32); // $0-10k: 0% discount
        tiers.set(100000u32, 500u32); // $10k-100k: 5% discount
        tiers.set(500000u32, 1000u32); // $100k-500k: 10% discount
        tiers.set(1000000u32, 2000u32); // $500k+: 20% discount
        tiers
    }

    /// Get pool information
    pub fn get_pool(env: &Env, chain_id: u32) -> Option<LiquidityPool> {
        let pools: Map<u32, LiquidityPool> = env
            .storage()
            .instance()
            .get(&LIQUIDITY_POOLS)
            .unwrap_or_else(|| Map::new(env));
        pools.get(chain_id)
    }

    /// Get LP position
    pub fn get_lp_position(env: &Env, chain_id: u32, provider: Address) -> Option<LPPosition> {
        if let Some(pool) = Self::get_pool(env, chain_id) {
            pool.lp_providers.get(provider)
        } else {
            None
        }
    }

    /// Get available liquidity for a chain
    pub fn get_available_liquidity(env: &Env, chain_id: u32) -> i128 {
        if let Some(pool) = Self::get_pool(env, chain_id) {
            pool.available_liquidity
        } else {
            0
        }
    }

    /// Get fee structure
    pub fn get_fee_structure(env: &Env) -> BridgeFeeStructure {
        env.storage()
            .instance()
            .get(&FEE_STRUCTURE)
            .unwrap_or(BridgeFeeStructure {
                base_fee: BASE_FEE_BPS,
                dynamic_multiplier: 100,
                congestion_multiplier: 100,
                volume_discount_tiers: Self::default_volume_tiers(env),
                last_updated: env.ledger().timestamp(),
            })
    }

    /// Check if pool has sufficient liquidity
    pub fn has_sufficient_liquidity(env: &Env, chain_id: u32, amount: i128) -> bool {
        Self::get_available_liquidity(env, chain_id) >= amount
    }
}
