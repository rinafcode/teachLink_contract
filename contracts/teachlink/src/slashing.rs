//! Validator Slashing and Rewards Module
//!
//! This module implements slashing mechanisms for malicious or negligent validators
//! and reward distribution for honest validators.
//!
//! # Slashing Algorithm
//!
//! When a validator is slashed, the penalty is computed as a percentage of their
//! current stake using basis-point arithmetic (10 000 bp = 100 %):
//!
//! ```text
//! slash_amount = (stake * slash_percentage_bp) / 10_000
//! ```
//!
//! Slashing percentages by offence type:
//! | Reason              | Basis Points | Effective % |
//! |---------------------|-------------|-------------|
//! | DoubleVote          | 5 000       | 50 %        |
//! | InvalidSignature    | 1 000       | 10 %        |
//! | Inactivity          |   500       |  5 %        |
//! | ByzantineBehavior   | 10 000      | 100 %       |
//! | MaliciousProposal   | 10 000      | 100 %       |
//!
//! The slashed tokens are redirected into the shared reward pool so that honest
//! validators benefit from penalising bad actors.
//!
//! # Reputation Decay
//!
//! Each slashing event also reduces the validator's `reputation_score`.  The
//! decay amount mirrors the severity of the offence.  Reputation is clamped at
//! zero to prevent underflow.
//!
//! # Inactivity Detection
//!
//! `check_inactivity` uses a dual-signal approach:
//! 1. **Timestamp** – primary signal; inactive if `now - last_activity > 7 days`.
//! 2. **Ledger sequence** – fallback for environments where `timestamp()` is
//!    unreliable; converts the 7-day threshold to an approximate ledger delta
//!    via `ledger_time::seconds_to_ledger_delta`.
//!
//! # Spec Reference
//! See `contracts/documentation/TOKENIZATION.md` §Validator Economics for the
//! economic rationale behind these percentages.

use crate::errors::BridgeError;
use crate::events::{
    StakeDepositedEvent, StakeWithdrawnEvent, ValidatorRewardedEvent, ValidatorSlashedEvent,
};
use crate::storage::{
    REWARD_POOL, SLASHING_COUNTER, SLASHING_RECORDS, VALIDATOR_ACTIVITY_SEQ, VALIDATOR_INFO,
    VALIDATOR_REWARDS, VALIDATOR_STAKES,
};
use crate::types::{RewardType, SlashingReason, SlashingRecord, ValidatorInfo, ValidatorReward};
use soroban_sdk::{Address, Env, Map, Vec};

/// Slashing percentages (in basis points, 10000 = 100%)
pub const SLASHING_PERCENTAGE_DOUBLE_VOTE: u32 = 5000; // 50%
pub const SLASHING_PERCENTAGE_INVALID_SIGNATURE: u32 = 1000; // 10%
pub const SLASHING_PERCENTAGE_INACTIVITY: u32 = 500; // 5%
pub const SLASHING_PERCENTAGE_BYZANTINE: u32 = 10000; // 100%
pub const SLASHING_PERCENTAGE_MALICIOUS: u32 = 10000; // 100%

/// Inactivity threshold (in seconds, 7 days)
pub const INACTIVITY_THRESHOLD: u64 = 604_800;

/// Minimum reputation score before slashing
pub const MIN_REPUTATION_SCORE: u32 = 50;

/// Slashing and Rewards Manager
pub struct SlashingManager;

impl SlashingManager {
    /// Deposit stake for a validator
    pub fn deposit_stake(env: &Env, validator: Address, amount: i128) -> Result<(), BridgeError> {
        validator.require_auth();

        if amount <= 0 {
            return Err(BridgeError::AmountMustBePositive);
        }

        // Get current stake
        let mut stakes: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&VALIDATOR_STAKES)
            .unwrap_or_else(|| Map::new(env));
        let current_stake = stakes.get(validator.clone()).unwrap_or(0);
        let new_stake = current_stake + amount;

        // Update stake
        stakes.set(validator.clone(), new_stake);
        env.storage().instance().set(&VALIDATOR_STAKES, &stakes);

        // Update validator info
        let mut validator_infos: Map<Address, ValidatorInfo> = env
            .storage()
            .instance()
            .get(&VALIDATOR_INFO)
            .unwrap_or_else(|| Map::new(env));
        if let Some(mut info) = validator_infos.get(validator.clone()) {
            info.stake = new_stake;
            validator_infos.set(validator.clone(), info);
            env.storage()
                .instance()
                .set(&VALIDATOR_INFO, &validator_infos);
        }

        // Emit event
        StakeDepositedEvent {
            validator: validator.clone(),
            amount,
            total_stake: new_stake,
        }
        .publish(env);

        Ok(())
    }

    /// Withdraw stake (with restrictions)
    pub fn withdraw_stake(env: &Env, validator: Address, amount: i128) -> Result<(), BridgeError> {
        validator.require_auth();

        if amount <= 0 {
            return Err(BridgeError::AmountMustBePositive);
        }

        // Get current stake
        let mut stakes: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&VALIDATOR_STAKES)
            .unwrap_or_else(|| Map::new(env));
        let current_stake = stakes.get(validator.clone()).unwrap_or(0);

        if amount > current_stake {
            return Err(BridgeError::InsufficientBalance);
        }

        let new_stake = current_stake - amount;

        // Update stake
        stakes.set(validator.clone(), new_stake);
        env.storage().instance().set(&VALIDATOR_STAKES, &stakes);

        // Update validator info
        let mut validator_infos: Map<Address, ValidatorInfo> = env
            .storage()
            .instance()
            .get(&VALIDATOR_INFO)
            .unwrap_or_else(|| Map::new(env));
        if let Some(mut info) = validator_infos.get(validator.clone()) {
            info.stake = new_stake;
            validator_infos.set(validator.clone(), info);
            env.storage()
                .instance()
                .set(&VALIDATOR_INFO, &validator_infos);
        }

        // Emit event
        StakeWithdrawnEvent {
            validator: validator.clone(),
            amount,
            remaining_stake: new_stake,
        }
        .publish(env);

        Ok(())
    }

    /// Slash a validator for malicious behavior
    pub fn slash_validator(
        env: &Env,
        validator: Address,
        reason: SlashingReason,
        evidence: soroban_sdk::Bytes,
        slasher: Address,
    ) -> Result<i128, BridgeError> {
        slasher.require_auth();

        // Cannot slash self
        if validator == slasher {
            return Err(BridgeError::CannotSlashSelf);
        }

        // Get validator info
        let mut validator_infos: Map<Address, ValidatorInfo> = env
            .storage()
            .instance()
            .get(&VALIDATOR_INFO)
            .unwrap_or_else(|| Map::new(env));
        let mut info = validator_infos
            .get(validator.clone())
            .ok_or(BridgeError::ValidatorNotActive)?;

        // Calculate slash amount
        let slash_percentage = match reason {
            SlashingReason::DoubleVote => SLASHING_PERCENTAGE_DOUBLE_VOTE,
            SlashingReason::InvalidSignature => SLASHING_PERCENTAGE_INVALID_SIGNATURE,
            SlashingReason::Inactivity => SLASHING_PERCENTAGE_INACTIVITY,
            SlashingReason::ByzantineBehavior => SLASHING_PERCENTAGE_BYZANTINE,
            SlashingReason::MaliciousProposal => SLASHING_PERCENTAGE_MALICIOUS,
        };

        let slash_amount = (info.stake * slash_percentage as i128) / 10000;

        if slash_amount <= 0 {
            return Err(BridgeError::InvalidSlashingEvidence);
        }

        // Update validator stake
        info.stake -= slash_amount;
        info.slashed_amount += slash_amount;
        info.reputation_score = Self::calculate_new_reputation(info.reputation_score, &reason);
        validator_infos.set(validator.clone(), info.clone());
        env.storage()
            .instance()
            .set(&VALIDATOR_INFO, &validator_infos);

        // Update stakes
        let mut stakes: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&VALIDATOR_STAKES)
            .unwrap_or_else(|| Map::new(env));
        stakes.set(validator.clone(), info.stake);
        env.storage().instance().set(&VALIDATOR_STAKES, &stakes);

        // Record slashing
        let mut slashing_counter: u64 = env
            .storage()
            .instance()
            .get(&SLASHING_COUNTER)
            .unwrap_or(0u64);
        slashing_counter += 1;

        let slashing_record = SlashingRecord {
            validator: validator.clone(),
            amount: slash_amount,
            reason: reason.clone(),
            timestamp: env.ledger().timestamp(),
            evidence,
        };

        let mut slashing_records: Map<u64, SlashingRecord> = env
            .storage()
            .instance()
            .get(&SLASHING_RECORDS)
            .unwrap_or_else(|| Map::new(env));
        slashing_records.set(slashing_counter, slashing_record);
        env.storage()
            .instance()
            .set(&SLASHING_RECORDS, &slashing_records);
        env.storage()
            .instance()
            .set(&SLASHING_COUNTER, &slashing_counter);

        // Add slashed amount to reward pool
        let reward_pool: i128 = env.storage().instance().get(&REWARD_POOL).unwrap_or(0i128);
        env.storage()
            .instance()
            .set(&REWARD_POOL, &(reward_pool + slash_amount));

        // Emit event
        ValidatorSlashedEvent {
            validator: validator.clone(),
            amount: slash_amount,
            reason,
            timestamp: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(slash_amount)
    }

    /// Reward a validator for honest behavior
    pub fn reward_validator(
        env: &Env,
        validator: Address,
        amount: i128,
        reward_type: RewardType,
    ) -> Result<(), BridgeError> {
        if amount <= 0 {
            return Err(BridgeError::AmountMustBePositive);
        }

        // Check reward pool
        let reward_pool: i128 = env.storage().instance().get(&REWARD_POOL).unwrap_or(0i128);
        if amount > reward_pool {
            return Err(BridgeError::InsufficientBalance);
        }

        // Deduct from reward pool
        env.storage()
            .instance()
            .set(&REWARD_POOL, &(reward_pool - amount));

        // Record reward
        let validator_reward = ValidatorReward {
            validator: validator.clone(),
            amount,
            reward_type: reward_type.clone(),
            timestamp: env.ledger().timestamp(),
        };

        let mut rewards: Map<Address, Vec<ValidatorReward>> = env
            .storage()
            .instance()
            .get(&VALIDATOR_REWARDS)
            .unwrap_or_else(|| Map::new(env));
        let mut validator_rewards = rewards
            .get(validator.clone())
            .unwrap_or_else(|| Vec::new(env));
        validator_rewards.push_back(validator_reward.clone());
        rewards.set(validator.clone(), validator_rewards);
        env.storage().instance().set(&VALIDATOR_REWARDS, &rewards);

        // Update validator info
        let mut validator_infos: Map<Address, ValidatorInfo> = env
            .storage()
            .instance()
            .get(&VALIDATOR_INFO)
            .unwrap_or_else(|| Map::new(env));
        if let Some(mut info) = validator_infos.get(validator.clone()) {
            info.stake += amount; // Add reward to stake
            validator_infos.set(validator.clone(), info);
            env.storage()
                .instance()
                .set(&VALIDATOR_INFO, &validator_infos);
        }

        // Emit event
        ValidatorRewardedEvent {
            validator: validator.clone(),
            amount,
            reward_type,
            timestamp: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    /// Check and slash inactive validators.
    ///
    /// # Inactivity Detection Algorithm
    ///
    /// Uses a dual-signal approach to handle environments where `ledger().timestamp()`
    /// may be unreliable (e.g., local test networks with frozen clocks):
    ///
    /// 1. **Primary – wall-clock timestamp**: inactive if
    ///    `now - last_activity > INACTIVITY_THRESHOLD` (7 days).
    /// 2. **Fallback – ledger sequence**: if the timestamp check passes, also
    ///    verify using ledger sequence numbers converted to an approximate delta
    ///    via `ledger_time::seconds_to_ledger_delta`.  This prevents false
    ///    negatives on networks where time is not advancing.
    ///
    /// If inactivity is confirmed, `slash_validator` is called with
    /// `SlashingReason::Inactivity` and the contract itself as the slasher
    /// (self-enforcing rule, no external reporter needed).
    ///
    /// # Returns
    /// `Ok(true)` if the validator was slashed, `Ok(false)` if still active.
    ///
    /// # TODO
    /// - Add a grace period counter so a validator gets one warning before
    ///   being slashed (reduces false positives from transient network issues).
    pub fn check_inactivity(env: &Env, validator: Address) -> Result<bool, BridgeError> {
        let validator_infos: Map<Address, ValidatorInfo> = env
            .storage()
            .instance()
            .get(&VALIDATOR_INFO)
            .unwrap_or_else(|| Map::new(env));

        if let Some(info) = validator_infos.get(validator.clone()) {
            let mut inactive =
                (env.ledger().timestamp() - info.last_activity) > INACTIVITY_THRESHOLD;

            // Sequence-based fallback for environments where timestamps are unreliable.
            if !inactive {
                let activity_seq: Map<Address, u32> = env
                    .storage()
                    .instance()
                    .get(&VALIDATOR_ACTIVITY_SEQ)
                    .unwrap_or_else(|| Map::new(env));
                if let Some(last_seq) = activity_seq.get(validator.clone()) {
                    let threshold_ledgers =
                        crate::ledger_time::seconds_to_ledger_delta(INACTIVITY_THRESHOLD);
                    inactive = env.ledger().sequence().saturating_sub(last_seq) > threshold_ledgers;
                }
            }

            if inactive {
                // Slash for inactivity
                Self::slash_validator(
                    env,
                    validator,
                    SlashingReason::Inactivity,
                    soroban_sdk::Bytes::from_slice(env, b"inactivity"),
                    env.current_contract_address(),
                )?;
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Fund the reward pool
    pub fn fund_reward_pool(env: &Env, funder: Address, amount: i128) -> Result<(), BridgeError> {
        funder.require_auth();

        if amount <= 0 {
            return Err(BridgeError::AmountMustBePositive);
        }

        let reward_pool: i128 = env.storage().instance().get(&REWARD_POOL).unwrap_or(0i128);
        let new_balance = reward_pool + amount;
        env.storage().instance().set(&REWARD_POOL, &new_balance);

        Ok(())
    }

    /// Calculate new reputation score after slashing.
    ///
    /// # Algorithm
    ///
    /// Applies a fixed reputation penalty that scales with offence severity.
    /// `saturating_sub` is used to clamp the result at zero, preventing
    /// underflow on validators that have already been penalised heavily.
    ///
    /// Penalty table (reputation points deducted):
    /// | Reason              | Penalty |
    /// |---------------------|---------|
    /// | DoubleVote          |  20     |
    /// | InvalidSignature    |  10     |
    /// | Inactivity          |   5     |
    /// | ByzantineBehavior   |  50     |
    /// | MaliciousProposal   | 100     |
    ///
    /// A validator starting at 100 reputation would be fully removed from the
    /// active set (threshold: `MIN_ACTIVE_REPUTATION = 40`) after two
    /// ByzantineBehavior slashes.
    ///
    /// # TODO
    /// - Consider a time-based reputation recovery mechanism so validators can
    ///   rehabilitate after a period of honest behaviour.
    fn calculate_new_reputation(current: u32, reason: &SlashingReason) -> u32 {
        let penalty = match reason {
            SlashingReason::DoubleVote => 20,
            SlashingReason::InvalidSignature => 10,
            SlashingReason::Inactivity => 5,
            SlashingReason::ByzantineBehavior => 50,
            SlashingReason::MaliciousProposal => 100,
        };

        current.saturating_sub(penalty)
    }

    /// Get validator stake
    pub fn get_stake(env: &Env, validator: Address) -> i128 {
        let stakes: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&VALIDATOR_STAKES)
            .unwrap_or_else(|| Map::new(env));
        stakes.get(validator).unwrap_or(0)
    }

    /// Get reward pool balance
    pub fn get_reward_pool(env: &Env) -> i128 {
        env.storage().instance().get(&REWARD_POOL).unwrap_or(0i128)
    }

    /// Get slashing record
    pub fn get_slashing_record(env: &Env, record_id: u64) -> Option<SlashingRecord> {
        let slashing_records: Map<u64, SlashingRecord> = env
            .storage()
            .instance()
            .get(&SLASHING_RECORDS)
            .unwrap_or_else(|| Map::new(env));
        slashing_records.get(record_id)
    }

    /// Get validator rewards history
    pub fn get_validator_rewards(env: &Env, validator: Address) -> Vec<ValidatorReward> {
        let rewards: Map<Address, Vec<ValidatorReward>> = env
            .storage()
            .instance()
            .get(&VALIDATOR_REWARDS)
            .unwrap_or_else(|| Map::new(env));
        rewards.get(validator).unwrap_or_else(|| Vec::new(env))
    }
}

#[cfg(test)]
mod tests {
    use super::SlashingManager;
    use crate::errors::BridgeError;
    use crate::storage::{VALIDATOR_ACTIVITY_SEQ, VALIDATOR_INFO};
    use crate::types::ValidatorInfo;
    use crate::TeachLinkBridge;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::{Address, Env, Map};

    fn set_ledger(env: &Env, timestamp: u64, sequence: u32) {
        env.ledger().with_mut(|li| {
            li.timestamp = timestamp;
            li.sequence_number = sequence;
        });
    }

    #[test]
    fn inactivity_check_uses_sequence_fallback() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());

        env.as_contract(&contract_id, || {
            let validator = Address::generate(&env);

            // Timestamp indicates activity is fresh.
            set_ledger(&env, 1_000, 1);
            let info = ValidatorInfo {
                address: validator.clone(),
                stake: 100,
                reputation_score: 100,
                is_active: true,
                joined_at: 1_000,
                last_activity: 1_000,
                total_validations: 0,
                missed_validations: 0,
                slashed_amount: 0,
            };
            let mut infos: Map<Address, ValidatorInfo> = Map::new(&env);
            infos.set(validator.clone(), info);
            env.storage().instance().set(&VALIDATOR_INFO, &infos);

            // Sequence says it's been a long time since last activity.
            let mut seqs: Map<Address, u32> = Map::new(&env);
            seqs.set(validator.clone(), 1u32);
            env.storage().instance().set(&VALIDATOR_ACTIVITY_SEQ, &seqs);

            // Advance sequence far beyond the fallback threshold.
            set_ledger(&env, 1_000, 1_000_000);
            let r = SlashingManager::check_inactivity(&env, validator);

            assert_eq!(r, Ok(true));
        });
    }
}
