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
//! Rewards are drawn from the pool's **real fee revenue** (`accumulated_fees`)
//! and weighted by how long the position has been held, so flash liquidity is
//! not rewarded as generously as long-term provision.
//!
//! ```text
//! fee_share      = accumulated_fees * position.amount / total_liquidity
//! time_multiplier = 1.0x, growing +1.0x every 30 days, capped at 3.0x
//! reward         = min(fee_share * time_multiplier, accumulated_fees)
//! ```
//!
//! All arithmetic is scaled (`SCALE = 1_000_000`) to avoid precision loss, and
//! rewards are hard-bounded by the fees the pool has actually collected.
//!
//! # TODO
//! - Add slippage protection for large bridge transactions relative to pool size.

use crate::errors::BridgeError;
use crate::events::{
    FeeRevenueRecordedEvent, FeeUpdatedEvent, LiquidityAddedEvent, LiquidityRemovedEvent,
};
use crate::storage::{ADMIN, FEE_STRUCTURE, LIQUIDITY_POOLS, LP_POSITIONS};
use crate::types::{BridgeFeeStructure, LPPosition, LiquidityPool};
use crate::validation::NumberValidator;
use soroban_sdk::{Address, Env, Map, Vec};

/// Base fee in basis points — re-exported from config.
pub use crate::config::LIQUIDITY_BASE_FEE_BPS as BASE_FEE_BPS;
/// Maximum fee in basis points — re-exported from config.
pub use crate::config::LIQUIDITY_MAX_FEE_BPS as MAX_FEE_BPS;
/// Minimum fee in basis points — re-exported from config.
pub use crate::config::LIQUIDITY_MIN_FEE_BPS as MIN_FEE_BPS;
/// Utilization threshold for dynamic pricing — re-exported from config.
pub use crate::config::LIQUIDITY_UTILIZATION_THRESHOLD as UTILIZATION_THRESHOLD;

/// Congestion multiplier steps
pub const CONGESTION_STEP_1: u32 = 5000; // 50% utilization
pub const CONGESTION_STEP_2: u32 = 7000; // 70% utilization
pub const CONGESTION_STEP_3: u32 = 9000; // 90% utilization

/// Scaling factor for LP reward arithmetic (1_000_000 = 1.0).
pub const REWARD_SCALE: i128 = 1_000_000;
/// Base time-in-pool multiplier (1_000_000 = 1.0x) applied at deposit time.
pub const TIME_SCALE: i128 = 1_000_000;
/// Maximum time-in-pool multiplier (3_000_000 = 3.0x).
pub const MAX_TIME_MULTIPLIER: i128 = 3_000_000;
/// Seconds per full multiplier step: the multiplier grows +1.0x every 30 days.
pub const TIME_MULTIPLIER_STEP_SECS: i128 = 30 * 24 * 60 * 60;

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
            accumulated_fees: 0,
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

        NumberValidator::validate_amount(amount).map_err(|_| BridgeError::AmountMustBePositive)?;

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

        NumberValidator::validate_amount(amount).map_err(|_| BridgeError::AmountMustBePositive)?;

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

        // Calculate rewards from the pool's real fee revenue and time in pool
        let rewards =
            Self::calculate_lp_rewards(env, &position, pool.total_liquidity, pool.accumulated_fees);

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
        NumberValidator::validate_amount(amount).map_err(|_| BridgeError::AmountMustBePositive)?;

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
        NumberValidator::validate_amount(amount).map_err(|_| BridgeError::AmountMustBePositive)?;

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

    /// Record fee revenue collected by a pool (admin only).
    ///
    /// Credits `amount` to the pool's `accumulated_fees` balance. LP rewards
    /// are drawn from this balance, so recorded fees must correspond to fees
    /// actually collected by the bridge for the pool's token.
    pub fn record_fee_revenue(
        env: &Env,
        chain_id: u32,
        amount: i128,
    ) -> Result<i128, BridgeError> {
        Self::require_admin(env);

        NumberValidator::validate_amount(amount).map_err(|_| BridgeError::AmountMustBePositive)?;

        let mut pools: Map<u32, LiquidityPool> = env
            .storage()
            .instance()
            .get(&LIQUIDITY_POOLS)
            .unwrap_or_else(|| Map::new(env));
        let mut pool = pools
            .get(chain_id)
            .ok_or(BridgeError::DestinationChainNotSupported)?;

        pool.accumulated_fees += amount;
        pools.set(chain_id, pool.clone());
        env.storage().instance().set(&LIQUIDITY_POOLS, &pools);

        // Emit event
        FeeRevenueRecordedEvent {
            chain_id,
            amount,
            accumulated_fees: pool.accumulated_fees,
        }
        .publish(env);

        Ok(pool.accumulated_fees)
    }

    /// Get the accumulated fee revenue for a pool.
    pub fn get_accumulated_fees(env: &Env, chain_id: u32) -> i128 {
        if let Some(pool) = Self::get_pool(env, chain_id) {
            pool.accumulated_fees
        } else {
            0
        }
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

    /// Calculate LP rewards based on real fee revenue and time in pool.
    ///
    /// # Algorithm
    ///
    /// Rewards are derived from the pool's actual collected fee revenue rather
    /// than a synthetic proportional formula:
    ///
    /// ```text
    /// fee_share       = accumulated_fees * position.amount / total_liquidity
    /// time_multiplier = 1.0x at deposit, +1.0x per 30 days held, capped at 3.0x
    /// reward          = min(fee_share * time_multiplier, accumulated_fees)
    /// ```
    ///
    /// Scaled integer arithmetic (`REWARD_SCALE = 1_000_000`) avoids precision
    /// loss from the inner division truncating small shares to zero. The final
    /// reward is hard-bounded by the fees the pool has actually collected, so
    /// payouts can never exceed real protocol income.
    fn calculate_lp_rewards(
        env: &Env,
        position: &LPPosition,
        total_liquidity: i128,
        accumulated_fees: i128,
    ) -> i128 {
        if total_liquidity == 0 || position.amount == 0 || accumulated_fees == 0 {
            return 0;
        }

        // Provider's share of the pool, scaled to preserve precision.
        let share_factor = (position.amount * REWARD_SCALE) / total_liquidity;
        let fee_share = (accumulated_fees * share_factor) / REWARD_SCALE;

        // Time-in-pool multiplier so long-term providers earn more than flash
        // liquidity providers of equal size.
        let time_multiplier = Self::time_in_pool_multiplier(env, position);

        let reward = (fee_share * time_multiplier) / TIME_SCALE;

        // Rewards stay bounded by the fees actually collected by the pool.
        reward.min(accumulated_fees)
    }

    /// Time-in-pool multiplier for a position.
    ///
    /// Starts at 1.0x at deposit time and grows linearly by +1.0x for every
    /// [`TIME_MULTIPLIER_STEP_SECS`] (30 days) the position is held, capped at
    /// [`MAX_TIME_MULTIPLIER`] (3.0x).
    fn time_in_pool_multiplier(env: &Env, position: &LPPosition) -> i128 {
        let now = env.ledger().timestamp();
        let elapsed = now.saturating_sub(position.deposited_at) as i128;
        let growth = (elapsed * TIME_SCALE) / TIME_MULTIPLIER_STEP_SECS;
        (TIME_SCALE + growth).min(MAX_TIME_MULTIPLIER)
    }

    /// Require the contract admin to authorize the call.
    fn require_admin(env: &Env) {
        let admin: Address = env.storage().instance().get(&ADMIN).expect("admin not set");
        admin.require_auth();
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

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};

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

    fn position(env: &Env, amount: i128, deposited_at: u64) -> LPPosition {
        LPPosition {
            provider: Address::generate(env),
            amount,
            share_percentage: 0,
            deposited_at,
            rewards_earned: 0,
        }
    }

    #[test]
    fn rewards_are_zero_without_fee_revenue() {
        let env = Env::default();
        set_timestamp(&env, 1_000_000);
        let pos = position(&env, 100_000, 0);

        // No fees collected yet: even a large, long-held position earns nothing,
        // because rewards must be backed by real fee income.
        let rewards = LiquidityManager::calculate_lp_rewards(&env, &pos, 200_000, 0);
        assert_eq!(rewards, 0);
    }

    #[test]
    fn rewards_are_proportional_to_fee_revenue() {
        let env = Env::default();
        set_timestamp(&env, 1_000_000);
        // Provider holds 50% of the pool and deposited at the current time.
        let pos = position(&env, 50_000, 1_000_000);

        // 50% share of 10_000 collected fees at 1.0x time multiplier.
        let rewards = LiquidityManager::calculate_lp_rewards(&env, &pos, 100_000, 10_000);
        assert_eq!(rewards, 5_000);
    }

    #[test]
    fn long_term_position_earns_more_than_flash_position() {
        let env = Env::default();
        let now = 10_000_000u64;
        set_timestamp(&env, now);

        // Two equally-sized positions: one deposited now (flash), one 30 days ago.
        let flash = position(&env, 100_000, now);
        let long = position(&env, 100_000, now - 30 * DAY);
        let total_liquidity = 200_000;
        let accumulated_fees = 10_000;

        let flash_rewards =
            LiquidityManager::calculate_lp_rewards(&env, &flash, total_liquidity, accumulated_fees);
        let long_rewards =
            LiquidityManager::calculate_lp_rewards(&env, &long, total_liquidity, accumulated_fees);

        // Flash position earns its flat 50% share (1.0x); the 30-day position
        // earns the same share scaled by the 2.0x time multiplier.
        assert_eq!(flash_rewards, 5_000);
        assert_eq!(long_rewards, 10_000);
        assert!(long_rewards > flash_rewards);
    }

    #[test]
    fn rewards_stay_bounded_by_collected_fees() {
        let env = Env::default();
        let now = 10_000_000u64;
        set_timestamp(&env, now);

        // 100% share, held 100 days (max 3.0x multiplier): the raw time-weighted
        // reward would be 3x the fees, but it must be clamped to the collected
        // fees so payouts never exceed real protocol income.
        let pos = position(&env, 100_000, now - 100 * DAY);
        let rewards = LiquidityManager::calculate_lp_rewards(&env, &pos, 100_000, 10_000);
        assert_eq!(rewards, 10_000);
        assert!(rewards <= 10_000);
    }

    #[test]
    fn time_multiplier_grows_with_duration_and_caps() {
        let env = Env::default();
        let now = 10_000_000u64;

        // At deposit time: 1.0x
        set_timestamp(&env, now);
        assert_eq!(
            LiquidityManager::time_in_pool_multiplier(&env, &position(&env, 1_000, now)),
            TIME_SCALE
        );

        // 30 days held: 2.0x
        set_timestamp(&env, now + 30 * DAY);
        assert_eq!(
            LiquidityManager::time_in_pool_multiplier(&env, &position(&env, 1_000, now)),
            2 * TIME_SCALE
        );

        // 60 days held: 3.0x
        set_timestamp(&env, now + 60 * DAY);
        assert_eq!(
            LiquidityManager::time_in_pool_multiplier(&env, &position(&env, 1_000, now)),
            3 * TIME_SCALE
        );

        // 100 days held: capped at 3.0x
        set_timestamp(&env, now + 100 * DAY);
        assert_eq!(
            LiquidityManager::time_in_pool_multiplier(&env, &position(&env, 1_000, now)),
            MAX_TIME_MULTIPLIER
        );
    }
}
