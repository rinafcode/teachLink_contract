#![no_std]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::panic_in_result_fn)]

//! Insurance Pool Contract
//!
//! This contract implements a decentralized insurance pool that protects learners
//! against course completion failures on the TeachLink platform.
//!
//! # Overview
//!
//! The insurance pool operates as follows:
//! 1. Users pay a premium to become insured
//! 2. If a course completion fails, the user can file a claim
//! 3. An oracle verifies the claim validity
//! 4. Verified claims are paid out from the pool
//!
//! # Roles
//!
//! - **Admin**: Can withdraw funds from the pool
//! - **Oracle**: Authorized to verify and process claims
//! - **Users**: Can pay premiums, file claims, and receive payouts
//!
//! # Example Workflow
//!
//! ```ignore
//! // 1. Admin initializes the pool
//! InsurancePool::initialize(env, admin, token, oracle, premium, payout);
//!
//! // 2. User pays premium to get insured
//! InsurancePool::pay_premium(env, user);
//!
//! // 3. User files a claim if course fails
//! let claim_id = InsurancePool::file_claim(env, user, course_id);
//!
//! // 4. Oracle verifies the claim
//! InsurancePool::process_claim(env, claim_id, true);
//!
//! // 5. User receives payout
//! InsurancePool::payout(env, claim_id);
//! ```

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

// ========== Error Types ==========

/// Error types for insurance pool contract operations
#[contracttype]
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum InsuranceError {
    /// Contract not initialized
    NotInitialized = 1,
    /// Contract already initialized
    AlreadyInitialized = 2,
    /// User is not insured
    NotInsured = 3,
    /// Claim not found
    ClaimNotFound = 4,
    /// Claim already processed
    ClaimAlreadyProcessed = 5,
    /// Claim status does not allow this operation
    InvalidClaimStatus = 6,
    /// Only oracle can process claims
    NotOracle = 7,
    /// Unauthorized caller
    UnauthorizedCaller = 8,
    /// Invalid premium amount
    InvalidPremiumAmount = 9,
    /// Invalid payout amount
    InvalidPayoutAmount = 10,
}

impl InsuranceError {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}

/// Storage keys for the insurance pool contract.
///
/// These keys are used to store and retrieve data from contract storage.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// The admin address with withdrawal privileges
    Admin,
    /// The token used for premiums and payouts
    Token,
    /// The oracle address authorized to verify claims
    Oracle,
    /// The premium amount required for insurance coverage
    PremiumAmount,
    /// The payout amount for verified claims
    PayoutAmount,
    /// Individual claim records, indexed by claim ID
    Claim(u64),
    /// Counter for generating unique claim IDs
    ClaimCount,
    /// Tracks whether a user currently has insurance coverage
    IsInsured(Address),
}

/// Status of an insurance claim throughout its lifecycle.
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClaimStatus {
    /// Claim filed, awaiting oracle verification
    Pending,
    /// Claim verified by oracle, eligible for payout
    Verified,
    /// Claim rejected by oracle
    Rejected,
    /// Claim has been paid out
    Paid,
}

/// Insurance claim record.
///
/// Represents a user's claim for insurance payout due to course failure.
///
/// # Fields
/// * `user` - Address of the claimant
/// * `course_id` - ID of the failed course
/// * `status` - Current claim status
#[contracttype]
#[derive(Clone)]
pub struct Claim {
    pub user: Address,
    pub course_id: u64,
    pub status: ClaimStatus,
}

/// Insurance Pool smart contract.
///
/// Manages insurance coverage for course completion protection on TeachLink.
#[contract]
pub struct InsurancePool;

#[contractimpl]
impl InsurancePool {
    /// Initialize the insurance pool contract.
    ///
    /// Sets up the insurance pool with the required configuration parameters.
    /// This function can only be called once.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `admin` - Address with admin privileges (can withdraw funds)
    /// * `token` - Token address used for premiums and payouts
    /// * `oracle` - Address authorized to verify claims
    /// * `premium_amount` - Amount users must pay for coverage
    /// * `payout_amount` - Amount paid out for verified claims
    ///
    /// # Returns
    /// Ok(()) on success, or InsuranceError if already initialized.
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        oracle: Address,
        premium_amount: i128,
        payout_amount: i128,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("ERR_ALREADY_INITIALIZED: Contract already initialized");
        }

        // Validate amounts
        if premium_amount <= 0 {
            panic!("ERR_INVALID_PREMIUM_AMOUNT: Premium amount must be positive");
        }

        if payout_amount <= 0 {
            panic!("ERR_INVALID_PAYOUT_AMOUNT: Payout amount must be positive");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::Oracle, &oracle);
        env.storage()
            .instance()
            .set(&DataKey::PremiumAmount, &premium_amount);
        env.storage()
            .instance()
            .set(&DataKey::PayoutAmount, &payout_amount);
        env.storage().instance().set(&DataKey::ClaimCount, &0u64);
    }

    /// Pay the insurance premium to become insured.
    ///
    /// Transfers the premium amount from the user to the insurance pool
    /// and marks the user as insured. The user must have sufficient
    /// token balance and have approved the transfer.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `user` - Address paying the premium (must authorize)
    ///
    /// # Returns
    /// Ok(()) on success, or InsuranceError if contract not initialized.
    ///
    /// # Authorization
    /// Requires authorization from `user`.
    pub fn pay_premium(env: Env, user: Address) {
        user.require_auth();

        let token_addr = env
            .storage()
            .instance()
            .get::<_, Address>(&DataKey::Token)
            .unwrap_or_else(|| panic!("ERR_NOT_INITIALIZED: Contract not initialized"));
        let premium_amount = env
            .storage()
            .instance()
            .get::<_, i128>(&DataKey::PremiumAmount)
            .unwrap_or_else(|| panic!("ERR_NOT_INITIALIZED: Contract not initialized"));
        let client = token::Client::new(&env, &token_addr);

        client.transfer(&user, env.current_contract_address(), &premium_amount);

        env.storage()
            .instance()
            .set(&DataKey::IsInsured(user.clone()), &true);
    }

    /// File an insurance claim for a failed course.
    ///
    /// Creates a new claim record for the specified course. The claim
    /// starts in `Pending` status and must be verified by the oracle
    /// before payout.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `user` - Address filing the claim (must authorize)
    /// * `course_id` - ID of the course that failed
    ///
    /// # Returns
    /// The unique claim ID for tracking the claim status, or InsuranceError if validation fails.
    ///
    /// # Authorization
    /// Requires authorization from `user`.
    pub fn file_claim(env: Env, user: Address, course_id: u64) -> u64 {
        user.require_auth();

        if !env
            .storage()
            .instance()
            .get::<_, bool>(&DataKey::IsInsured(user.clone()))
            .unwrap_or(false)
        {
            panic!("ERR_NOT_INSURED: User is not insured");
        }

        let mut claim_count = env
            .storage()
            .instance()
            .get::<_, u64>(&DataKey::ClaimCount)
            .unwrap_or(0u64);
        claim_count += 1;

        let claim = Claim {
            user: user.clone(),
            course_id,
            status: ClaimStatus::Pending,
        };

        env.storage()
            .instance()
            .set(&DataKey::Claim(claim_count), &claim);
        env.storage()
            .instance()
            .set(&DataKey::ClaimCount, &claim_count);

        claim_count
    }

    /// Process and verify an insurance claim.
    ///
    /// Called by the oracle to verify or reject a pending claim.
    /// Once processed, the claim status is updated and cannot be
    /// changed again.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `claim_id` - ID of the claim to process
    /// * `result` - `true` to verify (approve), `false` to reject
    ///
    /// # Returns
    /// Ok(()) on success, or InsuranceError if validation fails.
    ///
    /// # Authorization
    /// Requires authorization from the oracle address.
    pub fn process_claim(env: Env, claim_id: u64, result: bool) {
        let oracle = env
            .storage()
            .instance()
            .get::<_, Address>(&DataKey::Oracle)
            .unwrap_or_else(|| panic!("ERR_NOT_INITIALIZED: Contract not initialized"));
        oracle.require_auth();

        let mut claim = env
            .storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id))
            .unwrap_or_else(|| panic!("ERR_CLAIM_NOT_FOUND: Claim not found"));

        if claim.status != ClaimStatus::Pending {
            panic!("ERR_CLAIM_ALREADY_PROCESSED: Claim already processed");
        }

        if result {
            claim.status = ClaimStatus::Verified;
        } else {
            claim.status = ClaimStatus::Rejected;
        }

        env.storage()
            .instance()
            .set(&DataKey::Claim(claim_id), &claim);
    }

    /// Pay out a verified insurance claim.
    ///
    /// Transfers the payout amount to the claimant for a verified claim.
    /// After payout, the user's insurance coverage is removed (one-time use).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `claim_id` - ID of the verified claim to pay out
    ///
    /// # Returns
    /// Ok(()) on success, or InsuranceError if validation fails.
    ///
    /// # Note
    /// Insurance coverage is consumed after payout. The user must pay
    /// another premium to be covered again.
    pub fn payout(env: Env, claim_id: u64) {
        let mut claim = env
            .storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id))
            .unwrap_or_else(|| panic!("ERR_CLAIM_NOT_FOUND: Claim not found"));

        if claim.status != ClaimStatus::Verified {
            panic!("ERR_INVALID_CLAIM_STATUS: Claim status does not allow this operation");
        }

        let token_addr = env
            .storage()
            .instance()
            .get::<_, Address>(&DataKey::Token)
            .unwrap_or_else(|| panic!("ERR_NOT_INITIALIZED: Contract not initialized"));
        let payout_amount = env
            .storage()
            .instance()
            .get::<_, i128>(&DataKey::PayoutAmount)
            .unwrap_or_else(|| panic!("ERR_NOT_INITIALIZED: Contract not initialized"));
        let client = token::Client::new(&env, &token_addr);

        client.transfer(&env.current_contract_address(), &claim.user, &payout_amount);

        claim.status = ClaimStatus::Paid;
        env.storage()
            .instance()
            .set(&DataKey::Claim(claim_id), &claim);

        // Remove insurance coverage after payout (one premium = one claim)
        env.storage()
            .instance()
            .remove(&DataKey::IsInsured(claim.user));
    }

    /// Withdraw tokens from the insurance pool.
    ///
    /// Allows the admin to withdraw excess funds from the pool.
    /// This is typically used for pool rebalancing or profit extraction.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `amount` - Amount of tokens to withdraw
    ///
    /// # Returns
    /// Ok(()) on success, or InsuranceError if validation fails.
    ///
    /// # Authorization
    /// Requires authorization from the admin address.
    pub fn withdraw(env: Env, amount: i128) {
        let admin = env
            .storage()
            .instance()
            .get::<_, Address>(&DataKey::Admin)
            .unwrap_or_else(|| panic!("ERR_NOT_INITIALIZED: Contract not initialized"));
        admin.require_auth();

        if amount <= 0 {
            panic!("ERR_INVALID_PREMIUM_AMOUNT: Amount must be positive");
        }

        let token_addr = env
            .storage()
            .instance()
            .get::<_, Address>(&DataKey::Token)
            .unwrap_or_else(|| panic!("ERR_NOT_INITIALIZED: Contract not initialized"));
        let client = token::Client::new(&env, &token_addr);

        client.transfer(&env.current_contract_address(), &admin, &amount);
    }

    // ========== View Functions ==========

    /// Get a claim by its ID.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `claim_id` - ID of the claim to retrieve
    ///
    /// # Returns
    /// The claim if it exists, `None` otherwise.
    pub fn get_claim(env: Env, claim_id: u64) -> Option<Claim> {
        env.storage().instance().get(&DataKey::Claim(claim_id))
    }

    /// Check if a user is currently insured.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `user` - Address to check
    ///
    /// # Returns
    /// `true` if the user has active insurance coverage, `false` otherwise.
    pub fn is_insured(env: Env, user: Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::IsInsured(user))
            .unwrap_or(false)
    }
}

mod test;
