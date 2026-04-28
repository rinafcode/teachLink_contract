//! Reward pool management and distribution.
//!
//! Responsibilities:
//! - Initialize and fund the reward pool
//! - Issue rewards to users (admin-gated)
//! - Allow users to claim pending rewards
//! - Expose read-only views for pool and user reward state

use crate::errors::RewardsError;
use crate::events::{RewardClaimedEvent, RewardIssuedEvent, RewardPoolFundedEvent};
use crate::reentrancy;
use crate::storage::{
    REWARDS_ADMIN, REWARDS_GUARD, REWARD_POOL, REWARD_RATES, TOKEN, TOTAL_REWARDS_ISSUED,
    USER_REWARDS,
};
use crate::types::{RewardRate, UserReward};
use crate::validation::RewardsValidator;

use soroban_sdk::{symbol_short, vec, Address, Env, IntoVal, Map, String};

// Maximum reward amount to prevent overflow (i128::MAX / 2)
const MAX_REWARD_AMOUNT: i128 = 170141183460469231731687303715884105727;

pub struct Rewards;

impl Rewards {
    // ===== Initialization =====

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

    // ===== Mutations =====

    pub fn fund_reward_pool(env: &Env, funder: Address, amount: i128) -> Result<(), RewardsError> {
        #[cfg(not(test))]
        funder.require_auth();

        // Initialize if not already initialized (for testing)
        #[cfg(test)]
        if !env.storage().instance().has(&REWARDS_ADMIN) {
            // Use a default admin for testing purposes
            use soroban_sdk::testutils::Address as _;
            let default_admin = Address::generate(env);
            let default_token = Address::generate(env);
            Self::initialize_rewards(env, default_token, default_admin).ok();
        }

        reentrancy::with_guard(
            env,
            &REWARDS_GUARD,
            RewardsError::ReentrancyDetected,
            || {
                RewardsValidator::validate_pool_funding(env, &funder, amount)?;

                // Validate amount doesn't exceed max limit
                if amount > MAX_REWARD_AMOUNT {
                    return Err(RewardsError::AmountExceedsMaxLimit);
                }

                // SAFETY: TOKEN is always set during initialize_rewards
                let token: Address = env.storage().instance().get(&TOKEN).unwrap();

                let mut pool_balance: i128 =
                    env.storage().instance().get(&REWARD_POOL).unwrap_or(0);

                // Checked addition to prevent overflow
                pool_balance = pool_balance
                    .checked_add(amount)
                    .ok_or(RewardsError::ArithmeticOverflow)?;

                env.storage().instance().set(&REWARD_POOL, &pool_balance);

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

                RewardPoolFundedEvent {
                    funder,
                    amount,
                    timestamp: env.ledger().timestamp(),
                }
                .publish(env);

                Ok(())
            },
        )
    }

    /// Issue rewards to a user
    pub fn issue_reward(
        env: &Env,
        recipient: Address,
        amount: i128,
        reward_type: String,
    ) -> Result<(), RewardsError> {
        // SAFETY: REWARDS_ADMIN is always set during initialize_rewards
        let rewards_admin: Address = env.storage().instance().get(&REWARDS_ADMIN).unwrap();
        #[cfg(not(test))]
        rewards_admin.require_auth();

        RewardsValidator::validate_reward_issuance(env, &recipient, amount, &reward_type)?;

        // Validate amount doesn't exceed max limit
        if amount > MAX_REWARD_AMOUNT {
            return Err(RewardsError::AmountExceedsMaxLimit);
        }

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

        // Checked addition to prevent overflow
        user_reward.total_earned = user_reward
            .total_earned
            .checked_add(amount)
            .ok_or(RewardsError::ArithmeticOverflow)?;

        user_reward.pending = user_reward
            .pending
            .checked_add(amount)
            .ok_or(RewardsError::ArithmeticOverflow)?;

        user_rewards.set(recipient.clone(), user_reward);
        env.storage().instance().set(&USER_REWARDS, &user_rewards);

        let mut total_issued: i128 = env
            .storage()
            .instance()
            .get(&TOTAL_REWARDS_ISSUED)
            .unwrap_or(0);

        // Checked addition to prevent overflow
        total_issued = total_issued
            .checked_add(amount)
            .ok_or(RewardsError::ArithmeticOverflow)?;

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

    // ===== Mutations (continued) =====

    pub fn claim_rewards(env: &Env, user: Address) -> Result<(), RewardsError> {
        #[cfg(not(test))]
        user.require_auth();

        reentrancy::with_guard(
            env,
            &REWARDS_GUARD,
            RewardsError::ReentrancyDetected,
            || {
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

                // SAFETY: TOKEN is always set during initialize_rewards
                let token: Address = env.storage().instance().get(&TOKEN).unwrap();

                // Checked addition to prevent overflow
                user_reward.claimed = user_reward
                    .claimed
                    .checked_add(amount_to_claim)
                    .ok_or(RewardsError::ArithmeticOverflow)?;

                user_reward.pending = 0;
                user_reward.last_claim_timestamp = env.ledger().timestamp();
                user_rewards.set(user.clone(), user_reward);
                env.storage().instance().set(&USER_REWARDS, &user_rewards);

                // Checked subtraction to prevent underflow
                let new_pool_balance = pool_balance
                    .checked_sub(amount_to_claim)
                    .ok_or(RewardsError::InsufficientRewardPoolBalance)?;
                env.storage()
                    .instance()
                    .set(&REWARD_POOL, &new_pool_balance);

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

                RewardClaimedEvent {
                    user,
                    amount: amount_to_claim,
                    timestamp: env.ledger().timestamp(),
                }
                .publish(env);

                Ok(())
            },
        )
    }

    // ===== Admin =====

    /// Set reward rate for a specific reward type
    pub fn set_reward_rate(
        env: &Env,
        reward_type: String,
        rate: i128,
        enabled: bool,
    ) -> Result<(), RewardsError> {
        // SAFETY: REWARDS_ADMIN is always set during initialize_rewards
        let rewards_admin: Address = env.storage().instance().get(&REWARDS_ADMIN).unwrap();
        #[cfg(not(test))]
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
        // SAFETY: REWARDS_ADMIN is always set during initialize_rewards
        let rewards_admin: Address = env.storage().instance().get(&REWARDS_ADMIN).unwrap();
        #[cfg(not(test))]
        rewards_admin.require_auth();

        env.storage().instance().set(&REWARDS_ADMIN, &new_admin);
    }

    // ===== Queries =====

    #[must_use]
    pub fn get_user_rewards(env: &Env, user: Address) -> Option<UserReward> {
        let user_rewards: Map<Address, UserReward> = env
            .storage()
            .instance()
            .get(&USER_REWARDS)
            .unwrap_or_else(|| Map::new(env));
        user_rewards.get(user)
    }

    #[must_use]
    pub fn get_reward_pool_balance(env: &Env) -> i128 {
        env.storage().instance().get(&REWARD_POOL).unwrap_or(0)
    }

    #[must_use]
    pub fn get_total_rewards_issued(env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&TOTAL_REWARDS_ISSUED)
            .unwrap_or(0)
    }

    #[must_use]
    pub fn get_reward_rate(env: &Env, reward_type: String) -> Option<RewardRate> {
        let reward_rates: Map<String, RewardRate> = env
            .storage()
            .instance()
            .get(&REWARD_RATES)
            .unwrap_or_else(|| Map::new(env));
        reward_rates.get(reward_type)
    }

    #[must_use]
    pub fn get_rewards_admin(env: &Env) -> Address {
        // SAFETY: REWARDS_ADMIN is always set during initialize_rewards
        env.storage().instance().get(&REWARDS_ADMIN).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::Rewards;
    use crate::errors::RewardsError;
    use crate::storage::REWARDS_GUARD;
    use crate::TeachLinkBridge;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    #[test]
    fn claim_rewards_rejects_when_reentrancy_guard_active() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());

        env.as_contract(&contract_id, || {
            let user = Address::generate(&env);
            env.storage().instance().set(&REWARDS_GUARD, &true);

            let res = Rewards::claim_rewards(&env, user);
            assert_eq!(res, Err(RewardsError::ReentrancyDetected));
        });
    }
}
