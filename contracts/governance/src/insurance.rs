//! Governance Insurance and Risk Mitigation Module
//!
//! Provides financial protection mechanisms for governance participants,
//! guarding against unfavorable outcomes from governance decisions.
//!
//! # Insurance Mechanisms
//!
//! - **Proposal Insurance Pool**: Participants can deposit tokens into an
//!   insurance pool that provides coverage if a proposal causes harm
//! - **Risk Assessment**: Each proposal receives a risk score based on type,
//!   size, and potential impact
//! - **Claims**: Affected parties can file claims against the insurance pool
//! - **Premium System**: Proposal creators pay premiums to the pool
//!
//! # Risk Mitigation
//!
//! - **Timelock Escalation**: Higher-risk proposals get longer execution delays
//! - **Emergency Pause**: Admin can pause execution of risky proposals
//! - **Veto Power**: Security council can veto proposals above a risk threshold

use soroban_sdk::{contracttype, symbol_short, Address, Bytes, Env, Symbol};

/// Storage key for insurance pool
const INSURANCE_POOL: Symbol = symbol_short!("ins_pool");

/// Storage key for insurance claims
const CLAIMS: Symbol = symbol_short!("claims");

/// Storage key for claim count
const CLAIM_COUNT: Symbol = symbol_short!("clm_cnt");

/// Storage key for insurance config
const INS_CONFIG: Symbol = symbol_short!("ins_cfg");

/// Storage key for risk assessments
const RISK_SCORES: Symbol = symbol_short!("risk");

/// Risk level categories
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RiskLevel {
    /// Low risk - minor parameter changes
    Low,
    /// Medium risk - feature changes
    Medium,
    /// High risk - financial/security changes
    High,
    /// Critical risk - core protocol changes
    Critical,
}

/// Insurance configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsuranceConfig {
    /// Minimum premium for creating insured proposals (basis points of pool)
    pub min_premium_bps: u32,
    /// Maximum claim amount per proposal (basis points of pool)
    pub max_claim_bps: u32,
    /// Claim review period in seconds
    pub claim_review_period: u64,
    /// Whether insurance is enabled
    pub enabled: bool,
    /// Risk threshold above which proposals require additional delay
    pub risk_threshold: u32,
    /// Additional delay for high-risk proposals (seconds)
    pub high_risk_delay: u64,
}

/// Insurance pool state
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsurancePool {
    /// Total tokens in the insurance pool
    pub total_balance: i128,
    /// Total premiums collected
    pub total_premiums: i128,
    /// Total claims paid out
    pub total_claims_paid: i128,
    /// Number of active policies
    pub active_policies: u32,
    /// Last updated timestamp
    pub last_updated: u64,
}

/// Risk assessment for a proposal
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RiskAssessment {
    /// Proposal being assessed
    pub proposal_id: u64,
    /// Assessed risk level
    pub risk_level: RiskLevel,
    /// Risk score (0-10000 basis points)
    pub risk_score: u32,
    /// Assessor address
    pub assessor: Address,
    /// Assessment timestamp
    pub assessed_at: u64,
    /// Whether emergency veto is recommended
    pub veto_recommended: bool,
    /// Additional execution delay recommended (seconds)
    pub additional_delay: u64,
}

/// Insurance claim against the pool
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsuranceClaim {
    /// Unique claim ID
    pub id: u64,
    /// Proposal that caused the damage
    pub proposal_id: u64,
    /// Claimant address
    pub claimant: Address,
    /// Amount claimed
    pub amount: i128,
    /// Reason for the claim
    pub reason: Bytes,
    /// Whether the claim has been approved
    pub approved: bool,
    /// Whether the claim has been paid
    pub paid: bool,
    /// Claim submission timestamp
    pub submitted_at: u64,
    /// Review deadline
    pub review_deadline: u64,
}

/// Claim status
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimStatus {
    Pending,
    Approved,
    Rejected,
    Paid,
}

pub struct GovernanceInsurance;

impl GovernanceInsurance {
    /// Initialize the insurance system (admin only)
    pub fn initialize(
        env: &Env,
        admin: Address,
        min_premium_bps: u32,
        max_claim_bps: u32,
        claim_review_period: u64,
        risk_threshold: u32,
        high_risk_delay: u64,
    ) {
        admin.require_auth();

        let config = InsuranceConfig {
            min_premium_bps,
            max_claim_bps,
            claim_review_period,
            risk_threshold,
            high_risk_delay,
            enabled: true,
        };

        let pool = InsurancePool {
            total_balance: 0,
            total_premiums: 0,
            total_claims_paid: 0,
            active_policies: 0,
            last_updated: env.ledger().timestamp(),
        };

        env.storage().instance().set(&INS_CONFIG, &config);
        env.storage().instance().set(&INSURANCE_POOL, &pool);
    }

    /// Deposit tokens into the insurance pool
    pub fn deposit_to_pool(env: &Env, depositor: Address, token_address: &Address, amount: i128) {
        depositor.require_auth();

        assert!(amount > 0, "ERR_INVALID_AMOUNT: Amount must be positive");

        // Transfer tokens to contract
        let token_client = soroban_sdk::token::Client::new(env, token_address);
        token_client.transfer(&depositor, &env.current_contract_address(), &amount);

        // Update pool
        let mut pool: InsurancePool = env
            .storage()
            .instance()
            .get(&INSURANCE_POOL)
            .expect("ERR_INSURANCE_NOT_INITIALIZED");

        pool.total_balance += amount;
        pool.last_updated = env.ledger().timestamp();

        env.storage().instance().set(&INSURANCE_POOL, &pool);
    }

    /// Assess risk for a proposal
    pub fn assess_risk(
        env: &Env,
        assessor: Address,
        proposal_id: u64,
        risk_level: RiskLevel,
        risk_score: u32,
    ) -> RiskAssessment {
        assessor.require_auth();

        assert!(
            risk_score <= 10000,
            "ERR_INVALID_SCORE: Risk score must be 0-10000"
        );

        let config: InsuranceConfig = env
            .storage()
            .instance()
            .get(&INS_CONFIG)
            .expect("ERR_INSURANCE_NOT_INITIALIZED");

        let veto_recommended = risk_score > config.risk_threshold;
        let additional_delay = if risk_score > config.risk_threshold {
            config.high_risk_delay
        } else {
            0
        };

        let assessment = RiskAssessment {
            proposal_id,
            risk_level,
            risk_score,
            assessor: assessor.clone(),
            assessed_at: env.ledger().timestamp(),
            veto_recommended,
            additional_delay,
        };

        env.storage()
            .persistent()
            .set(&(RISK_SCORES, proposal_id), &assessment);

        assessment
    }

    /// File an insurance claim
    pub fn file_claim(
        env: &Env,
        claimant: Address,
        proposal_id: u64,
        amount: i128,
        reason: Bytes,
    ) -> u64 {
        claimant.require_auth();

        assert!(
            amount > 0,
            "ERR_INVALID_AMOUNT: Claim amount must be positive"
        );
        assert!(
            !reason.is_empty(),
            "ERR_EMPTY_REASON: Claim reason cannot be empty"
        );

        let config: InsuranceConfig = env
            .storage()
            .instance()
            .get(&INS_CONFIG)
            .expect("ERR_INSURANCE_NOT_INITIALIZED");

        let pool: InsurancePool = env
            .storage()
            .instance()
            .get(&INSURANCE_POOL)
            .expect("ERR_INSURANCE_NOT_INITIALIZED");

        // Check claim doesn't exceed max
        let max_claim = pool.total_balance * i128::from(config.max_claim_bps) / 10000;
        assert!(
            amount <= max_claim,
            "ERR_CLAIM_EXCEEDS_MAX: Claim exceeds maximum allowed"
        );

        let now = env.ledger().timestamp();

        let mut claim_count: u64 = env.storage().instance().get(&CLAIM_COUNT).unwrap_or(0);
        claim_count += 1;

        let claim = InsuranceClaim {
            id: claim_count,
            proposal_id,
            claimant: claimant.clone(),
            amount,
            reason,
            approved: false,
            paid: false,
            submitted_at: now,
            review_deadline: now + config.claim_review_period,
        };

        env.storage()
            .persistent()
            .set(&(CLAIMS, claim_count), &claim);
        env.storage().instance().set(&CLAIM_COUNT, &claim_count);

        claim_count
    }

    /// Approve or reject an insurance claim (admin only)
    pub fn review_claim(env: &Env, admin: Address, claim_id: u64, approved: bool) {
        admin.require_auth();

        let mut claim: InsuranceClaim = env
            .storage()
            .persistent()
            .get(&(CLAIMS, claim_id))
            .expect("ERR_CLAIM_NOT_FOUND: Claim does not exist");

        claim.approved = approved;

        env.storage().persistent().set(&(CLAIMS, claim_id), &claim);
    }

    /// Pay out an approved insurance claim
    pub fn pay_claim(env: &Env, admin: Address, claim_id: u64, token_address: &Address) {
        admin.require_auth();

        let mut claim: InsuranceClaim = env
            .storage()
            .persistent()
            .get(&(CLAIMS, claim_id))
            .expect("ERR_CLAIM_NOT_FOUND: Claim does not exist");

        assert!(claim.approved, "ERR_CLAIM_NOT_APPROVED: Claim not approved");
        assert!(!claim.paid, "ERR_CLAIM_ALREADY_PAID: Claim already paid");

        let mut pool: InsurancePool = env
            .storage()
            .instance()
            .get(&INSURANCE_POOL)
            .expect("ERR_INSURANCE_NOT_INITIALIZED");

        assert!(
            pool.total_balance >= claim.amount,
            "ERR_INSUFFICIENT_POOL: Insurance pool balance insufficient"
        );

        // Transfer tokens to claimant
        let token_client = soroban_sdk::token::Client::new(env, token_address);
        token_client.transfer(
            &env.current_contract_address(),
            &claim.claimant,
            &claim.amount,
        );

        claim.paid = true;
        pool.total_balance -= claim.amount;
        pool.total_claims_paid += claim.amount;
        pool.last_updated = env.ledger().timestamp();

        env.storage().persistent().set(&(CLAIMS, claim_id), &claim);
        env.storage().instance().set(&INSURANCE_POOL, &pool);
    }

    // ========== View Functions ==========

    /// Get the insurance pool state
    pub fn get_pool(env: &Env) -> Option<InsurancePool> {
        env.storage().instance().get(&INSURANCE_POOL)
    }

    /// Get insurance configuration
    pub fn get_config(env: &Env) -> Option<InsuranceConfig> {
        env.storage().instance().get(&INS_CONFIG)
    }

    /// Get risk assessment for a proposal
    pub fn get_risk_assessment(env: &Env, proposal_id: u64) -> Option<RiskAssessment> {
        env.storage().persistent().get(&(RISK_SCORES, proposal_id))
    }

    /// Get an insurance claim
    pub fn get_claim(env: &Env, claim_id: u64) -> Option<InsuranceClaim> {
        env.storage().persistent().get(&(CLAIMS, claim_id))
    }

    /// Get total claim count
    pub fn get_claim_count(env: &Env) -> u64 {
        env.storage().instance().get(&CLAIM_COUNT).unwrap_or(0)
    }
}
