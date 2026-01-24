use crate::errors::RewardsError;
use crate::events::{RewardClaimedEvent, RewardIssuedEvent, RewardPoolFundedEvent};
use crate::storage::{REWARD_POOL, REWARD_RATES, REWARDS_ADMIN, TOKEN, TOTAL_REWARDS_ISSUED, USER_REWARDS};
use crate::types::{RewardRate, UserReward};
use soroban_sdk::{symbol_short, vec, Address, Env, IntoVal, Map, String};

pub struct Rewards;

impl Rewards {
    /// Initialize the rewards system
    pub fn initialize_rewards(env: &Env, token: Address, rewards_admin: Address) -> Result<(), RewardsError> {
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

    /// Fund the reward pool
    pub fn fund_reward_pool(env: &Env, funder: Address, amount: i128) -> Result<(), RewardsError> {
        funder.require_auth();

        if amount <= 0 {
            return Err(RewardsError::AmountMustBePositive);
        }

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

        let mut pool_balance: i128 = env.storage().instance().get(&REWARD_POOL).unwrap_or(0i128);
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

        if amount <= 0 {
            return Err(RewardsError::AmountMustBePositive);
        }

        let pool_balance: i128 = env.storage().instance().get(&REWARD_POOL).unwrap_or(0i128);
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
            .unwrap_or(0i128);
        total_issued += amount;
        env.storage().instance().set(&TOTAL_REWARDS_ISSUED, &total_issued);

        RewardIssuedEvent {
            recipient,
            amount,
            reward_type,
            timestamp: env.ledger().timestamp(),
        }
        .publish(env);
        
        Ok(())
    }

    /// Claim pending rewards
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

        let pool_balance: i128 = env.storage().instance().get(&REWARD_POOL).unwrap_or(0i128);
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

        let mut pool_balance = pool_balance;
        pool_balance -= amount_to_claim;
        env.storage().instance().set(&REWARD_POOL, &pool_balance);

        RewardClaimedEvent {
            user,
            amount: amount_to_claim,
            timestamp: env.ledger().timestamp(),
        }
        .publish(env);
        
        Ok(())
    }

    // ========== Admin Functions ==========

    /// Set reward rate for a specific reward type
    pub fn set_reward_rate(env: &Env, reward_type: String, rate: i128, enabled: bool) -> Result<(), RewardsError> {
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

        let reward_rate = RewardRate {
            reward_type: reward_type.clone(),
            rate,
            enabled,
        };

        reward_rates.set(reward_type, reward_rate);
        env.storage().instance().set(&REWARD_RATES, &reward_rates);
        
        Ok(())
    }

    /// Update rewards admin
    pub fn update_rewards_admin(env: &Env, new_admin: Address) {
        let rewards_admin: Address = env.storage().instance().get(&REWARDS_ADMIN).unwrap();
        rewards_admin.require_auth();

        env.storage().instance().set(&REWARDS_ADMIN, &new_admin);
    }

    // ========== View Functions ==========

    /// Get user reward information
    pub fn get_user_rewards(env: &Env, user: Address) -> Option<UserReward> {
        let user_rewards: Map<Address, UserReward> = env
            .storage()
            .instance()
            .get(&USER_REWARDS)
            .unwrap_or_else(|| Map::new(env));
        user_rewards.get(user)
    }

    /// Get reward pool balance
    pub fn get_reward_pool_balance(env: &Env) -> i128 {
        env.storage().instance().get(&REWARD_POOL).unwrap_or(0i128)
    }

    /// Get total rewards issued
    pub fn get_total_rewards_issued(env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&TOTAL_REWARDS_ISSUED)
            .unwrap_or(0i128)
    }

    /// Get reward rate for a specific type
    pub fn get_reward_rate(env: &Env, reward_type: String) -> Option<RewardRate> {
        let reward_rates: Map<String, RewardRate> = env
            .storage()
            .instance()
            .get(&REWARD_RATES)
            .unwrap_or_else(|| Map::new(env));
        reward_rates.get(reward_type)
    }

    /// Get rewards admin address
    pub fn get_rewards_admin(env: &Env) -> Address {
        env.storage().instance().get(&REWARDS_ADMIN).unwrap()
    }
}
