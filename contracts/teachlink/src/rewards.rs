use crate::errors::RewardsError;
use crate::events::{RewardClaimedEvent, RewardIssuedEvent, RewardPoolFundedEvent};
use crate::storage::{
    REWARDS_ADMIN, REWARD_POOL, REWARD_RATES, TOKEN, TOTAL_REWARDS_ISSUED, USER_REWARDS,
};
use crate::types::{RewardRate, UserReward};
use crate::validation::RewardsValidator;

use soroban_sdk::{symbol_short, vec, Address, Env, IntoVal, Map, String};

pub struct Rewards;

impl Rewards {
    /// Initialize the rewards system
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

        let reward_rates: Map<String, RewardRate> = Map::new(env);
        env.storage().instance().set(&REWARD_RATES, &reward_rates);

        let user_rewards: Map<Address, UserReward> = Map::new(env);
        env.storage().instance().set(&USER_REWARDS, &user_rewards);

        Ok(())
    }

    // ==========================
    // Pool Management
    // ==========================

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

        let mut user_rewards: Map<Address, UserReward> = env
            .storage()
            .instance()
            .get(&USER_REWARDS)
            .unwrap_or_else(|| Map::new(env));

        let mut user_reward = user_rewards.get(recipient.clone()).unwrap_or(UserReward {
            user: recipient.clone(),
            total_earned: 0,
            claimed: 0,
            pending: 0,
            last_claim_timestamp: 0,
        });

        user_reward.total_earned += amount;
        user_reward.pending += amount;

        user_rewards.set(recipient.clone(), user_reward);
        env.storage().instance().set(&USER_REWARDS, &user_rewards);

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

    pub fn claim_rewards(env: &Env, user: Address) -> Result<(), RewardsError> {
        user.require_auth();

        let mut user_rewards: Map<Address, UserReward> = env
            .storage()
            .instance()
            .get(&USER_REWARDS)
            .unwrap_or_else(|| Map::new(env));

        let mut user_reward = user_rewards
            .get(user.clone())
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

        user_rewards.set(user.clone(), user_reward);
        env.storage().instance().set(&USER_REWARDS, &user_rewards);

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

        let mut reward_rates: Map<String, RewardRate> = env
            .storage()
            .instance()
            .get(&REWARD_RATES)
            .unwrap_or_else(|| Map::new(env));

        reward_rates.set(
            reward_type.clone(),
            RewardRate {
                reward_type,
                rate,
                enabled,
            },
        );

        env.storage().instance().set(&REWARD_RATES, &reward_rates);

        Ok(())
    }

    pub fn update_rewards_admin(env: &Env, new_admin: Address) {
        let rewards_admin: Address = env.storage().instance().get(&REWARDS_ADMIN).unwrap();
        rewards_admin.require_auth();

        env.storage().instance().set(&REWARDS_ADMIN, &new_admin);
    }

    // ==========================
    // View Functions
    // ==========================

    pub fn get_user_rewards(env: &Env, user: Address) -> Option<UserReward> {
        let user_rewards: Map<Address, UserReward> = env
            .storage()
            .instance()
            .get(&USER_REWARDS)
            .unwrap_or_else(|| Map::new(env));
        user_rewards.get(user)
    }

    pub fn get_reward_pool_balance(env: &Env) -> i128 {
        env.storage().instance().get(&REWARD_POOL).unwrap_or(0)
    }

    pub fn get_total_rewards_issued(env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&TOTAL_REWARDS_ISSUED)
            .unwrap_or(0)
    }

    pub fn get_reward_rate(env: &Env, reward_type: String) -> Option<RewardRate> {
        let reward_rates: Map<String, RewardRate> = env
            .storage()
            .instance()
            .get(&REWARD_RATES)
            .unwrap_or_else(|| Map::new(env));
        reward_rates.get(reward_type)
    }

    pub fn get_rewards_admin(env: &Env) -> Address {
        env.storage().instance().get(&REWARDS_ADMIN).unwrap()
    }
}
