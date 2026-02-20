//! Governance Token Staking Module
//!
//! Implements token staking to amplify voting power. Stakers lock their
//! tokens for a minimum period and receive enhanced voting power as a reward.
//!
//! # Staking Benefits
//!
//! - Amplified voting power based on staking multiplier
//! - Longer lock periods can result in higher effective power
//! - Staked tokens count toward proposal creation threshold
//!
//! # Lock Period
//!
//! Tokens are locked for the configured `lock_period`. During this time:
//! - Tokens cannot be unstaked
//! - Voting power bonus is active
//! - Staker can still vote and delegate

use soroban_sdk::{token, Address, Env};

use crate::events;
use crate::storage::{STAKES, STAKE_CONFIG, TOTAL_STAKED};
use crate::types::{StakeInfo, StakeKey, StakingConfig};

pub struct Staking;

impl Staking {
    /// Initialize staking configuration
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `admin` - Admin address (must authorize)
    /// * `min_stake` - Minimum tokens required to stake
    /// * `lock_period` - Lock-up period in seconds
    /// * `power_multiplier` - Voting power multiplier (basis points, 10000 = 1x)
    pub fn initialize_staking(
        env: &Env,
        admin: Address,
        min_stake: i128,
        lock_period: u64,
        power_multiplier: u32,
    ) {
        admin.require_auth();

        assert!(
            min_stake > 0,
            "ERR_INVALID_CONFIG: Minimum stake must be positive"
        );

        assert!(
            power_multiplier >= 10000,
            "ERR_INVALID_CONFIG: Power multiplier must be at least 10000 (1x)"
        );

        let config = StakingConfig {
            min_stake,
            lock_period,
            power_multiplier,
            enabled: true,
        };

        env.storage().instance().set(&STAKE_CONFIG, &config);
    }

    /// Stake tokens to amplify voting power
    ///
    /// Transfers tokens from the staker to the contract and records the stake.
    /// The staker receives amplified voting power based on the configured multiplier.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `token_address` - Governance token address
    /// * `staker` - Address staking tokens
    /// * `amount` - Amount of tokens to stake
    ///
    /// # Panics
    /// * If staking is not enabled
    /// * If amount is below minimum stake
    pub fn stake(
        env: &Env,
        token_address: &Address,
        staker: Address,
        amount: i128,
    ) {
        staker.require_auth();

        let config: StakingConfig = env
            .storage()
            .instance()
            .get(&STAKE_CONFIG)
            .expect("ERR_STAKING_NOT_INITIALIZED: Staking not initialized");

        assert!(config.enabled, "ERR_STAKING_DISABLED: Staking is disabled");

        assert!(
            amount >= config.min_stake,
            "ERR_INSUFFICIENT_STAKE: Amount below minimum stake"
        );

        let now = env.ledger().timestamp();
        let stake_key = StakeKey {
            staker: staker.clone(),
        };

        // Calculate power bonus
        let power_bonus =
            (amount * i128::from(config.power_multiplier) / 10000) - amount;

        // Check if already staked - add to existing
        let stake_info = if let Some(existing) = env
            .storage()
            .persistent()
            .get::<_, StakeInfo>(&(STAKES, stake_key.clone()))
        {
            let new_amount = existing.amount + amount;
            let new_bonus =
                (new_amount * i128::from(config.power_multiplier) / 10000) - new_amount;

            StakeInfo {
                staker: staker.clone(),
                amount: new_amount,
                staked_at: existing.staked_at,
                lock_until: now + config.lock_period,
                power_bonus: new_bonus,
            }
        } else {
            StakeInfo {
                staker: staker.clone(),
                amount,
                staked_at: now,
                lock_until: now + config.lock_period,
                power_bonus,
            }
        };

        // Transfer tokens to the contract
        let token_client = token::Client::new(env, token_address);
        token_client.transfer(&staker, &env.current_contract_address(), &amount);

        // Store stake info
        env.storage()
            .persistent()
            .set(&(STAKES, stake_key), &stake_info);

        // Update total staked
        let total: i128 = env
            .storage()
            .instance()
            .get(&TOTAL_STAKED)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&TOTAL_STAKED, &(total + amount));

        events::tokens_staked(env, &staker, amount, stake_info.power_bonus);
    }

    /// Unstake tokens after lock period expires
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `token_address` - Governance token address
    /// * `staker` - Address unstaking tokens
    /// * `amount` - Amount to unstake
    ///
    /// # Panics
    /// * If no stake exists
    /// * If lock period has not elapsed
    /// * If amount exceeds staked amount
    pub fn unstake(
        env: &Env,
        token_address: &Address,
        staker: Address,
        amount: i128,
    ) {
        staker.require_auth();

        let stake_key = StakeKey {
            staker: staker.clone(),
        };

        let mut stake_info: StakeInfo = env
            .storage()
            .persistent()
            .get(&(STAKES, stake_key.clone()))
            .expect("ERR_NO_STAKE: No stake found for address");

        let now = env.ledger().timestamp();
        assert!(
            now >= stake_info.lock_until,
            "ERR_STAKE_LOCKED: Tokens are still locked"
        );

        assert!(
            amount <= stake_info.amount,
            "ERR_INSUFFICIENT_STAKE: Amount exceeds staked balance"
        );

        // Transfer tokens back
        let token_client = token::Client::new(env, token_address);
        token_client.transfer(&env.current_contract_address(), &staker, &amount);

        // Update or remove stake
        stake_info.amount -= amount;
        if stake_info.amount > 0 {
            let config: StakingConfig = env
                .storage()
                .instance()
                .get(&STAKE_CONFIG)
                .unwrap();
            stake_info.power_bonus = (stake_info.amount
                * i128::from(config.power_multiplier)
                / 10000)
                - stake_info.amount;

            env.storage()
                .persistent()
                .set(&(STAKES, stake_key), &stake_info);
        } else {
            env.storage()
                .persistent()
                .remove(&(STAKES, stake_key));
        }

        // Update total staked
        let total: i128 = env
            .storage()
            .instance()
            .get(&TOTAL_STAKED)
            .unwrap_or(0);
        let new_total = if total > amount { total - amount } else { 0 };
        env.storage()
            .instance()
            .set(&TOTAL_STAKED, &new_total);

        events::tokens_unstaked(env, &staker, amount);
    }

    /// Get stake info for an address
    pub fn get_stake(env: &Env, staker: &Address) -> Option<StakeInfo> {
        let stake_key = StakeKey {
            staker: staker.clone(),
        };
        env.storage()
            .persistent()
            .get(&(STAKES, stake_key))
    }

    /// Get total staked tokens
    pub fn get_total_staked(env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&TOTAL_STAKED)
            .unwrap_or(0)
    }

    /// Get staking config
    pub fn get_staking_config(env: &Env) -> Option<StakingConfig> {
        env.storage().instance().get(&STAKE_CONFIG)
    }

    /// Get voting power bonus from staking
    pub fn get_staking_bonus(env: &Env, staker: &Address) -> i128 {
        Self::get_stake(env, staker)
            .map(|s| s.power_bonus)
            .unwrap_or(0)
    }

    /// Check if staker's lock period has expired
    pub fn is_unlocked(env: &Env, staker: &Address) -> bool {
        match Self::get_stake(env, staker) {
            Some(stake) => env.ledger().timestamp() >= stake.lock_until,
            None => true,
        }
    }

    /// Enable or disable staking (admin only)
    pub fn set_staking_enabled(env: &Env, admin: Address, enabled: bool) {
        admin.require_auth();

        if let Some(mut config) = env
            .storage()
            .instance()
            .get::<_, StakingConfig>(&STAKE_CONFIG)
        {
            config.enabled = enabled;
            env.storage().instance().set(&STAKE_CONFIG, &config);
        }
    }
}
