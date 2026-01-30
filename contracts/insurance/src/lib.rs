#![no_std]

use crate::errors::InsuranceError;
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    Oracle,
    PremiumAmount,
    PayoutAmount,
    Claim(u64),
    ClaimCount,
    IsInsured(Address),
}

#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClaimStatus {
    Pending,
    Verified,
    Rejected,
    Paid,
}

#[contracttype]
#[derive(Clone)]
pub struct Claim {
    pub user: Address,
    pub course_id: u64,
    pub status: ClaimStatus,
}

#[contract]
pub struct InsurancePool;

#[contractimpl]
impl InsurancePool {
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        oracle: Address,
        premium_amount: i128,
        payout_amount: i128,
    ) -> Result<(), InsuranceError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(InsuranceError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::Oracle, &oracle);
        env.storage().instance().set(&DataKey::PremiumAmount, &premium_amount);
        env.storage().instance().set(&DataKey::PayoutAmount, &payout_amount);
        env.storage().instance().set(&DataKey::ClaimCount, &0u64);

        Ok(())
    }

    pub fn pay_premium(env: Env, user: Address) {
        user.require_auth();

        let token_addr = env.storage().instance().get::<_, Address>(&DataKey::Token).unwrap();
        let premium_amount = env.storage().instance().get::<_, i128>(&DataKey::PremiumAmount).unwrap();

        let client = token::Client::new(&env, &token_addr);
        client.transfer(&user, &env.current_contract_address(), &premium_amount);

        env.storage()
            .instance()
            .set(&DataKey::IsInsured(user), &true);
    }

    pub fn file_claim(
        env: Env,
        user: Address,
        course_id: u64,
    ) -> Result<u64, InsuranceError> {
        user.require_auth();

        let insured = env
            .storage()
            .instance()
            .get::<_, bool>(&DataKey::IsInsured(user.clone()))
            .unwrap_or(false);

        if !insured {
            return Err(InsuranceError::UserNotInsured);
        }

        let mut claim_count = env.storage().instance().get::<_, u64>(&DataKey::ClaimCount).unwrap();
        claim_count += 1;

        let claim = Claim {
            user: user.clone(),
            course_id,
            status: ClaimStatus::Pending,
        };

        env.storage().instance().set(&DataKey::Claim(claim_count), &claim);
        env.storage().instance().set(&DataKey::ClaimCount, &claim_count);

        Ok(claim_count)
    }

    pub fn process_claim(
        env: Env,
        claim_id: u64,
        result: bool,
    ) -> Result<(), InsuranceError> {
        let oracle = env.storage().instance().get::<_, Address>(&DataKey::Oracle).unwrap();
        oracle.require_auth();

        let mut claim = env
            .storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id))
            .ok_or(InsuranceError::ClaimNotFound)?;

        if claim.status != ClaimStatus::Pending {
            return Err(InsuranceError::ClaimAlreadyProcessed);
        }

        claim.status = if result {
            ClaimStatus::Verified
        } else {
            ClaimStatus::Rejected
        };

        env.storage().instance().set(&DataKey::Claim(claim_id), &claim);
        Ok(())
    }

    pub fn payout(env: Env, claim_id: u64) -> Result<(), InsuranceError> {
        let mut claim = env
            .storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id))
            .ok_or(InsuranceError::ClaimNotFound)?;

        if claim.status != ClaimStatus::Verified {
            return Err(InsuranceError::ClaimNotVerified);
        }

        let token_addr = env.storage().instance().get::<_, Address>(&DataKey::Token).unwrap();
        let payout_amount = env.storage().instance().get::<_, i128>(&DataKey::PayoutAmount).unwrap();

        let client = token::Client::new(&env, &token_addr);
        client.transfer(&env.current_contract_address(), &claim.user, &payout_amount);

        claim.status = ClaimStatus::Paid;
        env.storage().instance().set(&DataKey::Claim(claim_id), &claim);

        // One premium = one claim
        env.storage().instance().remove(&DataKey::IsInsured(claim.user));

        Ok(())
    }

    pub fn withdraw(env: Env, amount: i128) {
        let admin = env.storage().instance().get::<_, Address>(&DataKey::Admin).unwrap();
        admin.require_auth();

        let token_addr = env.storage().instance().get::<_, Address>(&DataKey::Token).unwrap();
        let client = token::Client::new(&env, &token_addr);

        client.transfer(&env.current_contract_address(), &admin, &amount);
    }

    // ===== View Functions =====

    pub fn get_claim(env: Env, claim_id: u64) -> Option<Claim> {
        env.storage().instance().get(&DataKey::Claim(claim_id))
    }

    pub fn is_insured(env: Env, user: Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::IsInsured(user))
            .unwrap_or(false)
    }
}

mod errors;
