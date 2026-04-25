//! Bridge Liquidity and AMM Module
//!
//! This module implements liquidity pool management and automated market making
//! for optimizing bridge operations and dynamic fee pricing.
//!
//! # Dynamic Fee Algorithm
//!
//! The bridge fee is computed in three stages:
//!
//! ```text
//! 1. base_fee_amount   = amount * base_fee_bps / 10_000
//! 2. congestion_adj    = base_fee_amount * congestion_multiplier / 100
//! 3. final_fee         = congestion_adj * (10_000 - volume_discount_bps) / 10_000
//! ```
//!
//! The result is clamped to `[MIN_FEE_BPS, MAX_FEE_BPS]` of the transfer amount.
//!
//! ## Congestion Multiplier
//!
//! Derived from pool utilisation (`locked / total`):
//!
//! | Utilisation | Multiplier |
//! |-------------|-----------|
//! | < 50 %      | 1.0×      |
//! | 50 – 70 %   | 1.5×      |
//! | 70 – 90 %   | 2.0×      |
//! | ≥ 90 %      | 3.0×      |
//!
//! ## Volume Discount
//!
//! Users with higher 24-hour trading volume receive a fee discount.  The
//! discount is looked up from a configurable tier map (threshold → discount_bps).
//! The highest matching tier wins.
//!
//! Default tiers:
//! | 24h Volume  | Discount |
//! |-------------|---------|
//! | < $10k      |  0 %    |
//! | $10k–$100k  |  5 %    |
//! | $100k–$500k | 10 %    |
//! | > $500k     | 20 %    |
//!
//! # LP Share Calculation
//!
//! LP share percentages are stored in basis points (10 000 = 100 %).
//! When a provider adds liquidity, their share is recalculated against the
//! *post-deposit* total to avoid inflating the percentage:
//!
//! ```text
//! share_bps = (provider_amount * 10_000) / new_total_liquidity
//! ```
//!
//! # LP Reward Calculation
//!
//! Rewards use scaled integer arithmetic to avoid precision loss:
//!
//! ```text
//! reward = (position.amount * SCALE / total_liquidity) * position.amount / SCALE
//!        ≈ position.amount² / total_liquidity
//! ```
//!
//! where `SCALE = 1_000_000`.  This is equivalent to `amount * (amount / total)`
//! but avoids the inner division truncating to zero for small positions.
//!
//! # TODO
//! - Implement time-weighted LP rewards so long-term providers earn more than
//!   flash liquidity providers.
//! - Add slippage protection for large bridge transactions relative to pool size.

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

        // Update pool totals first so share percentages are calculated against the correct total
        pool.total_liquidity += amount;
        pool.available_liquidity += amount;

        // Calculate share percentage against updated total
        let share_percentage = if pool.total_liquidity == 0 {
            10000u32 // 100% for first provider (unreachable after adding amount, but safe fallback)
        } else {
            ((amount * 10000) / pool.total_liquidity) as u32
        };

        // Create or update LP position
        let mut lp_positions: Map<Address, LPPosition> = pool.lp_providers;
        let position = if let Some(mut existing) = lp_positions.get(provider.clone()) {
            existing.amount += amount;
            // Recalculate share against updated total
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

        // Update pool totals before recalculating share so the percentage reflects the new total
        pool.total_liquidity -= amount;
        // Only reduce available_liquidity by what is actually available (guard against locked funds)
        let deduct_available = amount.min(pool.available_liquidity);
        pool.available_liquidity -= deduct_available;

        if position.amount == 0 {
            lp_positions.remove(provider.clone());
        } else {
            // Recalculate share against the post-removal total
            position.share_percentage = if pool.total_liquidity == 0 {
                0
            } else {
                ((position.amount * 10000) / pool.total_liquidity) as u32
            };
            lp_positions.set(provider.clone(), position.clone());
        }
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

    /// Calculate dynamic bridge fee.
    ///
    /// # Algorithm
    ///
    /// Three-stage computation (all arithmetic in basis points):
    ///
    /// ```text
    /// base_fee_amount = amount * base_fee_bps / 10_000
    /// congestion_adj  = base_fee_amount * congestion_multiplier / 100
    /// final_fee       = congestion_adj * (10_000 - volume_discount_bps) / 10_000
    /// ```
    ///
    /// The result is clamped to `[MIN_FEE_BPS, MAX_FEE_BPS]` of `amount` to
    /// prevent fees from being zero (dust attacks) or excessively large.
    ///
    /// If no pool exists for `chain_id`, the congestion multiplier defaults to
    /// 1× (100) so the fee degrades gracefully to the base rate.
    ///
    /// # Parameters
    /// - `chain_id`       – Target chain; used to look up pool utilisation.
    /// - `amount`         – Transfer amount in token base units.
    /// - `user_volume_24h`– Caller's 24-hour trading volume for discount lookup.
    ///
    /// # TODO
    /// - Cache the fee structure in a performance layer to avoid repeated
    ///   storage reads on high-frequency bridging.
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

    /// Calculate congestion multiplier based on pool utilization.
    ///
    /// # Algorithm
    ///
    /// Computes utilisation as `locked_liquidity * 10_000 / total_liquidity`
    /// (basis points), then maps it to a step-function multiplier:
    ///
    /// | Utilisation (bp) | Multiplier (%) | Effective fee multiplier |
    /// |-----------------|---------------|--------------------------|
    /// | < 5 000 (50 %)  | 100           | 1.0×                     |
    /// | 5 000 – 7 000   | 150           | 1.5×                     |
    /// | 7 000 – 9 000   | 200           | 2.0×                     |
    /// | ≥ 9 000 (90 %)  | 300           | 3.0×                     |
    ///
    /// Returns 100 (1×) when the pool is empty to avoid division by zero.
    ///
    /// # TODO
    /// - Replace the step function with a smooth curve (e.g., linear or
    ///   exponential) to reduce fee cliff effects at utilisation boundaries.
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

    /// Calculate volume discount based on 24h volume.
    ///
    /// Iterates all configured discount tiers and returns the highest discount
    /// whose threshold the user's 24-hour volume meets or exceeds.
    ///
    /// # Algorithm
    ///
    /// ```text
    /// discount = max { tier_discount | threshold ≤ user_volume_24h }
    /// ```
    ///
    /// Returns 0 if no tier is matched (volume below the lowest threshold).
    ///
    /// # Note
    /// The tier map key is a `u32` threshold cast to `i128` for comparison.
    /// Ensure tier thresholds are set in the same units as `user_volume_24h`
    /// (token base units, not USD) to avoid mismatches.
    fn calculate_volume_discount(volume_tiers: &Map<u32, u32>, user_volume_24h: i128) -> u32 {
        let mut discount = 0u32;

        for (threshold, tier_discount) in volume_tiers.iter() {
            if user_volume_24h >= threshold as i128 && tier_discount > discount {
                discount = tier_discount;
            }
        }

        discount
    }

    /// Calculate LP rewards based on position and pool performance.
    ///
    /// # Algorithm
    ///
    /// Uses scaled integer arithmetic to avoid precision loss from integer
    /// division truncating `share_factor` to zero for small positions:
    ///
    /// ```text
    /// reward = (position.amount * SCALE / total_liquidity) * position.amount / SCALE
    ///        ≈ position.amount² / total_liquidity
    /// ```
    ///
    /// where `SCALE = 1_000_000`.  This is mathematically equivalent to
    /// `amount * (amount / total)` but avoids the inner division flooring to
    /// zero when `amount << total`.
    ///
    /// # Note
    /// This is a simplified proportional reward model.  In production, rewards
    /// should also factor in time-in-pool and accumulated fee revenue.
    ///
    /// # TODO
    /// - Integrate actual fee revenue collected by the pool so LP rewards
    ///   reflect real earnings rather than a synthetic proportional amount.
    /// - Add time-weighting: providers who stay longer earn a multiplier.
    fn calculate_lp_rewards(_env: &Env, position: &LPPosition, total_liquidity: i128) -> i128 {
        if total_liquidity == 0 || position.amount == 0 {
            return 0;
        }

        // Scale numerator before dividing to preserve precision:
        // reward = (position.amount^2 * SCALE) / (total_liquidity * SCALE_DIVISOR)
        // equivalent to: position.amount * (position.amount / total_liquidity)
        // but avoids share_factor flooring to 0 for small positions.
        const SCALE: i128 = 1_000_000;
        let reward = (position.amount * SCALE) / total_liquidity * position.amount / SCALE;

        reward
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
