use crate::errors::EscrowError;
use crate::storage::INSURANCE_POOL;
use crate::types::InsurancePool;
#[cfg(test)]
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, vec, Address, Env, IntoVal};

pub struct InsuranceManager;

impl InsuranceManager {
    /// Initialize the insurance pool
    pub fn initialize_pool(
        env: &Env,
        token: Address,
        premium_rate: u32,
    ) -> Result<(), EscrowError> {
        let pool = InsurancePool {
            token,
            balance: 0,
            premium_rate,
            total_claims_paid: 0,
            max_payout_percentage: 8000, // 80%
        };
        env.storage().instance().set(&INSURANCE_POOL, &pool);
        Ok(())
    }

    /// Fund the insurance pool
    pub fn fund_pool(env: &Env, funder: Address, amount: i128) -> Result<(), EscrowError> {
        funder.require_auth();
        let mut pool: InsurancePool = env
            .storage()
            .instance()
            .get(&INSURANCE_POOL)
            .ok_or(EscrowError::AmountMustBePositive)?;

        env.invoke_contract::<()>(
            &pool.token,
            &symbol_short!("transfer"),
            vec![
                env,
                funder.into_val(env),
                env.current_contract_address().into_val(env),
                amount.into_val(env),
            ],
        );

        pool.balance += amount;
        env.storage().instance().set(&INSURANCE_POOL, &pool);
        Ok(())
    }

    /// Calculate insurance premium for an escrow amount
    pub fn calculate_premium(env: &Env, amount: i128) -> i128 {
        let pool: InsurancePool =
            env.storage()
                .instance()
                .get(&INSURANCE_POOL)
                .unwrap_or(InsurancePool {
                    token: env.current_contract_address(), // Use current contract as dummy if not initialized
                    balance: 0,
                    premium_rate: 100, // 1%
                    total_claims_paid: 0,
                    max_payout_percentage: 8000,
                });

        (amount * pool.premium_rate as i128) / 10000
    }

    /// Pay premium and add to pool
    pub fn pay_premium(env: &Env, user: Address, amount: i128) -> Result<(), EscrowError> {
        user.require_auth();
        Self::pay_premium_internal(env, user, amount)
    }

    /// Internal: Pay premium and add to pool (no auth check)
    pub fn pay_premium_internal(env: &Env, user: Address, amount: i128) -> Result<(), EscrowError> {
        let mut pool: InsurancePool = env
            .storage()
            .instance()
            .get(&INSURANCE_POOL)
            .ok_or(EscrowError::AmountMustBePositive)?;

        env.invoke_contract::<()>(
            &pool.token,
            &symbol_short!("transfer"),
            vec![
                env,
                user.into_val(env),
                env.current_contract_address().into_val(env),
                amount.into_val(env),
            ],
        );

        pool.balance += amount;
        env.storage().instance().set(&INSURANCE_POOL, &pool);
        Ok(())
    }

    /// Process an insurance claim
    pub fn process_claim(
        env: &Env,
        recipient: Address,
        requested_amount: i128,
    ) -> Result<(), EscrowError> {
        let mut pool: InsurancePool = env
            .storage()
            .instance()
            .get(&INSURANCE_POOL)
            .ok_or(EscrowError::AmountMustBePositive)?;

        let max_payout = (pool.balance * pool.max_payout_percentage as i128) / 10000;
        let final_payout = requested_amount.min(max_payout);

        if final_payout <= 0 {
            return Err(EscrowError::AmountMustBePositive);
        }

        env.invoke_contract::<()>(
            &pool.token,
            &symbol_short!("transfer"),
            vec![
                env,
                env.current_contract_address().into_val(env),
                recipient.into_val(env),
                final_payout.into_val(env),
            ],
        );

        pool.balance -= final_payout;
        pool.total_claims_paid += final_payout;
        env.storage().instance().set(&INSURANCE_POOL, &pool);

        Ok(())
    }

    /// Risk assessment for an escrow
    pub fn assess_risk(env: &Env, escrow_id: u64, amount: i128) -> u32 {
        // Simple risk assessment: score 0-100
        // Higher amount = higher risk
        if amount > 100_000_000 {
            return 80;
        } else if amount > 10_000_000 {
            return 40;
        }
        10
    }
}
