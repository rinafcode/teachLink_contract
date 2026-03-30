use crate::errors::RewardsError;
use crate::events::{RewardClaimedEvent, RewardIssuedEvent, RewardPoolFundedEvent};
use crate::storage::{
    REWARDS_ADMIN, REWARD_POOL, REWARD_RATES, TOKEN, TOTAL_REWARDS_ISSUED, USER_REWARDS,
};
use crate::types::{RewardRate, UserReward};
use crate::validation::RewardsValidator;

use soroban_sdk::{symbol_short, vec, Address, Env, IntoVal, String};

pub struct Rewards;

impl Rewards {
    /// Initialize the rewards system
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // initialize_rewards(...);
    /// ```
    pub fn initialize_rewards(
        env: &Env,
        token: Address,
        rewards_admin: Address,
    ) -> Result<(), RewardsError> {
        if env.storage().instance().has(&REWARDS_ADMIN) {
            return Err(RewardsError::AlreadyInitialized);
        }

        env.storage().instance().set(&TOKEN, &token);
        env.storage().instance().set(&REWARDS_ADMIN, &rewards_admin);
        env.storage().instance().set(&REWARD_POOL, &0i128);
        env.storage().instance().set(&TOTAL_REWARDS_ISSUED, &0i128);

        Ok(())
    }

    // ==========================
    // Pool Management
    // ==========================

    /// Standard API for fund_reward_pool
    ///
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // fund_reward_pool(...);
    /// ```
    pub fn fund_reward_pool(env: &Env, funder: Address, amount: i128) -> Result<(), RewardsError> {
        funder.require_auth();

        RewardsValidator::validate_pool_funding(env, &funder, amount)?;

        let token: Address = env.storage().instance().get(&TOKEN).unwrap();

        env.invoke_contract::<()>(
            &token,
            &symbol_short!("transfer"),
            vec![
                env,
                funder.clone().into_val(env),
                env.current_contract_address().into_val(env),
                amount.into_val(env),
            ],
        );

        let mut pool_balance: i128 = env.storage().instance().get(&REWARD_POOL).unwrap_or(0);
        pool_balance += amount;
        env.storage().instance().set(&REWARD_POOL, &pool_balance);

        RewardPoolFundedEvent {
            funder,
            amount,
            timestamp: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    /// Issue rewards to a user
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // issue_reward(...);
    /// ```
    pub fn issue_reward(
        env: &Env,
        recipient: Address,
        amount: i128,
        reward_type: String,
    ) -> Result<(), RewardsError> {
        let rewards_admin: Address = env.storage().instance().get(&REWARDS_ADMIN).unwrap();
        rewards_admin.require_auth();

        RewardsValidator::validate_reward_issuance(env, &recipient, amount, &reward_type)?;

        let pool_balance: i128 = env.storage().instance().get(&REWARD_POOL).unwrap_or(0);
        if pool_balance < amount {
            return Err(RewardsError::InsufficientRewardPoolBalance);
        }

        let mut user_reward: UserReward = env
            .storage()
            .persistent()
            .get(&(USER_REWARDS, recipient.clone()))
            .unwrap_or(UserReward {
                user: recipient.clone(),
                total_earned: 0,
                claimed: 0,
                pending: 0,
                last_claim_timestamp: 0,
            });

        user_reward.total_earned += amount;
        user_reward.pending += amount;

        env.storage()
            .persistent()
            .set(&(USER_REWARDS, recipient.clone()), &user_reward);

        let mut total_issued: i128 = env
            .storage()
            .instance()
            .get(&TOTAL_REWARDS_ISSUED)
            .unwrap_or(0);
        total_issued += amount;
        env.storage()
            .instance()
            .set(&TOTAL_REWARDS_ISSUED, &total_issued);

        RewardIssuedEvent {
            recipient,
            amount,
            reward_type,
            timestamp: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    // ==========================
    // Claiming
    // ==========================

    /// Standard API for claim_rewards
    ///
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // claim_rewards(...);
    /// ```
    pub fn claim_rewards(env: &Env, user: Address) -> Result<(), RewardsError> {
        user.require_auth();

        let mut user_reward: UserReward = env
            .storage()
            .persistent()
            .get(&(USER_REWARDS, user.clone()))
            .ok_or(RewardsError::NoRewardsAvailable)?;

        if user_reward.pending <= 0 {
            return Err(RewardsError::NoPendingRewards);
        }

        let amount_to_claim = user_reward.pending;

        let pool_balance: i128 = env.storage().instance().get(&REWARD_POOL).unwrap_or(0);
        if pool_balance < amount_to_claim {
            return Err(RewardsError::InsufficientRewardPoolBalance);
        }

        let token: Address = env.storage().instance().get(&TOKEN).unwrap();

        env.invoke_contract::<()>(
            &token,
            &symbol_short!("transfer"),
            vec![
                env,
                env.current_contract_address().into_val(env),
                user.clone().into_val(env),
                amount_to_claim.into_val(env),
            ],
        );

        user_reward.claimed += amount_to_claim;
        user_reward.pending = 0;
        user_reward.last_claim_timestamp = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&(USER_REWARDS, user.clone()), &user_reward);

        let new_pool_balance = pool_balance - amount_to_claim;
        env.storage()
            .instance()
            .set(&REWARD_POOL, &new_pool_balance);

        RewardClaimedEvent {
            user,
            amount: amount_to_claim,
            timestamp: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    // ==========================
    // Admin Functions
    // ==========================

    /// Set reward rate for a specific reward type
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // set_reward_rate(...);
    /// ```
    pub fn set_reward_rate(
        env: &Env,
        reward_type: String,
        rate: i128,
        enabled: bool,
    ) -> Result<(), RewardsError> {
        let rewards_admin: Address = env.storage().instance().get(&REWARDS_ADMIN).unwrap();
        rewards_admin.require_auth();

        if rate < 0 {
            return Err(RewardsError::RateCannotBeNegative);
        }

        env.storage().persistent().set(
            &(REWARD_RATES, reward_type.clone()),
            &RewardRate {
                reward_type,
                rate,
                enabled,
            },
        );

        Ok(())
    }

    /// Standard API for update_rewards_admin
    ///
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // update_rewards_admin(...);
    /// ```
    pub fn update_rewards_admin(env: &Env, new_admin: Address) {
        let rewards_admin: Address = env.storage().instance().get(&REWARDS_ADMIN).unwrap();
        rewards_admin.require_auth();

        env.storage().instance().set(&REWARDS_ADMIN, &new_admin);
    }

    // ==========================
    // View Functions
    // ==========================

    /// Standard API for get_user_rewards
    ///
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // get_user_rewards(...);
    /// ```
    pub fn get_user_rewards(env: &Env, user: Address) -> Option<UserReward> {
        env.storage().persistent().get(&(USER_REWARDS, user))
    }

    /// Standard API for get_reward_pool_balance
    ///
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // get_reward_pool_balance(...);
    /// ```
    pub fn get_reward_pool_balance(env: &Env) -> i128 {
        env.storage().instance().get(&REWARD_POOL).unwrap_or(0)
    }

    /// Standard API for get_total_rewards_issued
    ///
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // get_total_rewards_issued(...);
    /// ```
    pub fn get_total_rewards_issued(env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&TOTAL_REWARDS_ISSUED)
            .unwrap_or(0)
    }

    /// Standard API for get_reward_rate
    ///
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // get_reward_rate(...);
    /// ```
    pub fn get_reward_rate(env: &Env, reward_type: String) -> Option<RewardRate> {
        env.storage()
            .persistent()
            .get(&(REWARD_RATES, reward_type))
    }

    /// Standard API for get_rewards_admin
    ///
    /// # Arguments
    ///
    /// * `env` - The environment (if applicable).
    ///
    /// # Returns
    ///
    /// * The return value of the function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage
    /// // get_rewards_admin(...);
    /// ```
    pub fn get_rewards_admin(env: &Env) -> Address {
        env.storage().instance().get(&REWARDS_ADMIN).unwrap()
    }
}
