//! Advanced Reputation and Credit Scoring System
//!
//! This module implements sophisticated reputation algorithms with social proof,
//! cross-platform aggregation, and external credit scoring integration.

use crate::types::{UserReputation, Address, Bytes, Map, Vec, u64, u32};
use soroban_sdk::{contracttype, contracterror, Env, Symbol, symbol_short, panic_with_error};

const ADVANCED_REPUTATION: Symbol = symbol_short!("adv_rep");
const SOCIAL_PROOF: Symbol = symbol_short!("soc_proof");
const CROSS_PLATFORM_REP: Symbol = symbol_short!("cross_rep");
const CREDIT_SCORE_EXTERNAL: Symbol = symbol_short!("credit_ext");
const REPUTATION_INSURANCE: Symbol = symbol_short!("rep_ins");
const REPUTATION_MARKET: Symbol = symbol_short!("rep_market");

// ========== Advanced Reputation Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvancedReputation {
    pub user: Address,
    pub base_reputation: UserReputation,
    pub social_proof_score: u32,
    pub cross_platform_score: u32,
    pub external_credit_score: u32,
    pub reputation_tier: ReputationTier,
    pub insurance_coverage: ReputationInsurance,
    pub reputation_score: u64, // Composite score
    pub last_updated: u64,
    pub decay_factor: u32, // Reputation decay over time
    pub boost_factors: Vec<ReputationBoost>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReputationTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SocialProof {
    pub user: Address,
    pub endorsements: Vec<Endorsement>,
    pub verifications: Vec<Verification>,
    pub social_connections: u32,
    pub trust_score: u32,
    pub vouches_received: u32,
    pub vouches_given: u32,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Endorsement {
    pub endorser: Address,
    pub skill_area: Bytes,
    pub rating: u32, // 1-5
    pub comment: Bytes,
    pub weight: u32, // Based on endorser's reputation
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Verification {
    pub verifier: Address,
    pub verification_type: VerificationType,
    pub verified_attribute: Bytes,
    pub verification_data: Bytes,
    pub expires_at: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerificationType {
    Identity,
    Education,
    Professional,
    Skill,
    Achievement,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossPlatformReputation {
    pub user: Address,
    pub platform_scores: Map<Bytes, u32>, // Platform -> Score
    pub aggregated_score: u32,
    pub verification_count: u32,
    pub last_sync: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalCreditData {
    pub user: Address,
    pub credit_bureau_score: u32,
    pub alternative_data_score: u32,
    pub payment_history: PaymentHistory,
    pub debt_to_income_ratio: u32,
    pub credit_utilization: u32,
    pub account_age: u32,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentHistory {
    pub on_time_payments: u32,
    pub late_payments: u32,
    pub missed_payments: u32,
    pub total_payments: u32,
    pub payment_streak: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReputationInsurance {
    pub user: Address,
    pub coverage_amount: u64,
    pub premium_paid: u64,
    pub deductible: u64,
    pub coverage_type: InsuranceType,
    pub is_active: bool,
    pub expires_at: u64,
    pub claims_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InsuranceType {
    ReputationLoss,
    DisputeProtection,
    IncomeProtection,
    CompletionGuarantee,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReputationBoost {
    pub boost_type: BoostType,
    pub multiplier: u32,
    pub duration: u64,
    pub expires_at: u64,
    pub cost: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BoostType {
    CourseCompletion,
    SocialEngagement,
    ExpertEndorsement,
    VerificationBonus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReputationMarketplace {
    pub reputation_listings: Vec<ReputationListing>,
    pub active_bids: Map<u64, ReputationBid>,
    pub transaction_history: Vec<ReputationTransaction>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReputationListing {
    pub id: u64,
    pub seller: Address,
    pub reputation_amount: u32,
    pub price: u64,
    pub duration: u64,
    pub conditions: Vec<Bytes>,
    pub is_active: bool,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReputationBid {
    pub id: u64,
    pub bidder: Address,
    pub listing_id: u64,
    pub amount: u64,
    pub terms: Bytes,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReputationTransaction {
    pub id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub reputation_amount: u32,
    pub price: u64,
    pub timestamp: u64,
    pub status: TransactionStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Completed,
    Cancelled,
    Disputed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReputationAnalytics {
    pub user: Address,
    pub reputation_trend: Vec<u64>, // Historical scores
    pub growth_rate: u64, // Basis points
    pub volatility: u64, // Basis points
    pub prediction: u64, // Predicted future score
    pub recommendations: Vec<ReputationRecommendation>,
    pub last_analyzed: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReputationRecommendation {
    pub recommendation_type: RecommendationType,
    pub description: Bytes,
    pub potential_impact: u32,
    pub effort_required: u32,
    pub priority: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecommendationType {
    CompleteCourse,
    GetEndorsed,
    VerifySkills,
    EngageCommunity,
    ImprovePaymentHistory,
}

// ========== Errors ==========

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AdvancedReputationError {
    InvalidEndorsement = 1,
    InsufficientReputation = 2,
    InsuranceNotActive = 3,
    InvalidVerification = 4,
    MarketplaceError = 5,
    UnauthorizedAccess = 6,
    ReputationTooLow = 7,
}

// ========== Main Implementation ==========

pub struct AdvancedReputationManager;

impl AdvancedReputationManager {
    /// Initialize advanced reputation system for a user
    pub fn initialize_user(env: &Env, user: Address) -> Result<(), AdvancedReputationError> {
        user.require_auth();
        
        let base_rep = UserReputation {
            participation_score: 0,
            completion_rate: 0,
            contribution_quality: 0,
            total_courses_started: 0,
            total_courses_completed: 0,
            total_contributions: 0,
            last_update: env.ledger().timestamp(),
        };

        let advanced_rep = AdvancedReputation {
            user: user.clone(),
            base_reputation: base_rep,
            social_proof_score: 0,
            cross_platform_score: 0,
            external_credit_score: 0,
            reputation_tier: ReputationTier::Bronze,
            insurance_coverage: ReputationInsurance {
                user: user.clone(),
                coverage_amount: 0,
                premium_paid: 0,
                deductible: 1000,
                coverage_type: InsuranceType::ReputationLoss,
                is_active: false,
                expires_at: 0,
                claims_count: 0,
            },
            reputation_score: 0,
            last_updated: env.ledger().timestamp(),
            decay_factor: 100, // No decay initially
            boost_factors: Vec::new(env),
        };

        let social_proof = SocialProof {
            user: user.clone(),
            endorsements: Vec::new(env),
            verifications: Vec::new(env),
            social_connections: 0,
            trust_score: 0,
            vouches_received: 0,
            vouches_given: 0,
            last_updated: env.ledger().timestamp(),
        };

        Self::set_advanced_reputation(env, &user, &advanced_rep);
        Self::set_social_proof(env, &user, &social_proof);
        
        Ok(())
    }

    /// Add endorsement to user's social proof
    pub fn add_endorsement(
        env: &Env,
        endorser: Address,
        target_user: Address,
        skill_area: Bytes,
        rating: u32,
        comment: Bytes,
    ) -> Result<(), AdvancedReputationError> {
        endorser.require_auth();
        
        if rating < 1 || rating > 5 {
            return Err(AdvancedReputationError::InvalidEndorsement);
        }

        let mut social_proof = Self::get_social_proof(env, &target_user);
        let endorser_rep = Self::calculate_reputation_score(env, &endorser);
        
        let endorsement = Endorsement {
            endorser: endorser.clone(),
            skill_area: skill_area.clone(),
            rating,
            comment,
            weight: Self::calculate_endorsement_weight(endorser_rep),
            timestamp: env.ledger().timestamp(),
        };

        social_proof.endorsements.push_back(endorsement);
        social_proof.trust_score = Self::calculate_trust_score(&social_proof);
        social_proof.last_updated = env.ledger().timestamp();

        Self::set_social_proof(env, &target_user, &social_proof);
        Self::update_composite_score(env, &target_user);
        
        Ok(())
    }

    /// Add verification to user's profile
    pub fn add_verification(
        env: &Env,
        verifier: Address,
        target_user: Address,
        verification_type: VerificationType,
        verified_attribute: Bytes,
        verification_data: Bytes,
        expires_at: u64,
    ) -> Result<(), AdvancedReputationError> {
        verifier.require_auth();
        
        let mut social_proof = Self::get_social_proof(env, &target_user);
        
        let verification = Verification {
            verifier: verifier.clone(),
            verification_type: verification_type.clone(),
            verified_attribute: verified_attribute.clone(),
            verification_data,
            expires_at,
            timestamp: env.ledger().timestamp(),
        };

        social_proof.verifications.push_back(verification);
        social_proof.last_updated = env.ledger().timestamp();

        Self::set_social_proof(env, &target_user, &social_proof);
        Self::update_composite_score(env, &target_user);
        
        Ok(())
    }

    /// Update cross-platform reputation data
    pub fn update_cross_platform_reputation(
        env: &Env,
        user: Address,
        platform: Bytes,
        score: u32,
    ) -> Result<(), AdvancedReputationError> {
        // This would typically be called by an oracle or authorized updater
        let mut cross_platform = Self::get_cross_platform_reputation(env, &user);
        cross_platform.platform_scores.set(platform.clone(), score);
        cross_platform.aggregated_score = Self::aggregate_cross_platform_scores(&cross_platform);
        cross_platform.last_sync = env.ledger().timestamp();

        Self::set_cross_platform_reputation(env, &user, &cross_platform);
        Self::update_composite_score(env, &user);
        
        Ok(())
    }

    /// Update external credit data
    pub fn update_external_credit_data(
        env: &Env,
        user: Address,
        credit_bureau_score: u32,
        alternative_data_score: u32,
        payment_history: PaymentHistory,
        debt_to_income_ratio: u32,
        credit_utilization: u32,
        account_age: u32,
    ) -> Result<(), AdvancedReputationError> {
        // This would typically be called by an oracle or authorized updater
        let external_credit = ExternalCreditData {
            user: user.clone(),
            credit_bureau_score,
            alternative_data_score,
            payment_history,
            debt_to_income_ratio,
            credit_utilization,
            account_age,
            last_updated: env.ledger().timestamp(),
        };

        Self::set_external_credit_data(env, &user, &external_credit);
        Self::update_composite_score(env, &user);
        
        Ok(())
    }

    /// Purchase reputation insurance
    pub fn purchase_reputation_insurance(
        env: &Env,
        user: Address,
        coverage_amount: u64,
        coverage_type: InsuranceType,
        duration: u64,
    ) -> Result<(), AdvancedReputationError> {
        user.require_auth();
        
        let premium = Self::calculate_insurance_premium(coverage_amount, coverage_type);
        let mut advanced_rep = Self::get_advanced_reputation(env, &user);
        
        // Check if user has sufficient reputation for insurance
        if advanced_rep.reputation_score < coverage_amount / 1000 {
            return Err(AdvancedReputationError::ReputationTooLow);
        }

        let insurance = ReputationInsurance {
            user: user.clone(),
            coverage_amount,
            premium_paid: premium,
            deductible: coverage_amount / 10, // 10% deductible
            coverage_type,
            is_active: true,
            expires_at: env.ledger().timestamp() + duration,
            claims_count: 0,
        };

        advanced_rep.insurance_coverage = insurance;
        Self::set_advanced_reputation(env, &user, &advanced_rep);
        
        Ok(())
    }

    /// Create reputation marketplace listing
    pub fn create_reputation_listing(
        env: &Env,
        seller: Address,
        reputation_amount: u32,
        price: u64,
        duration: u64,
        conditions: Vec<Bytes>,
    ) -> Result<u64, AdvancedReputationError> {
        seller.require_auth();
        
        let mut advanced_rep = Self::get_advanced_reputation(env, &seller);
        if advanced_rep.reputation_score < reputation_amount as u64 {
            return Err(AdvancedReputationError::InsufficientReputation);
        }

        let listing_id = env.ledger().sequence();
        let listing = ReputationListing {
            id: listing_id,
            seller: seller.clone(),
            reputation_amount,
            price,
            duration,
            conditions,
            is_active: true,
            created_at: env.ledger().timestamp(),
        };

        let mut marketplace = Self::get_reputation_marketplace(env);
        marketplace.reputation_listings.push_back(listing);
        Self::set_reputation_marketplace(env, &marketplace);
        
        Ok(listing_id)
    }

    /// Calculate composite reputation score using multi-dimensional algorithm
    fn update_composite_score(env: &Env, user: &Address) {
        let mut advanced_rep = Self::get_advanced_reputation(env, user);
        let social_proof = Self::get_social_proof(env, user);
        let cross_platform = Self::get_cross_platform_reputation(env, user);
        let external_credit = Self::get_external_credit_data(env, user);

        // Multi-dimensional scoring algorithm
        let base_score = Self::calculate_base_score(&advanced_rep.base_reputation);
        let social_score = Self::calculate_social_score(&social_proof);
        let platform_score = cross_platform.aggregated_score;
        let credit_score = Self::calculate_credit_score(&external_credit);

        // Weighted composite score (weights can be adjusted)
        let composite_score = (
            base_score * 40 + // 40% weight
            social_score * 25 + // 25% weight
            platform_score * 20 + // 20% weight
            credit_score * 15 // 15% weight
        ) / 100;

        // Apply boost factors
        let boosted_score = Self::apply_boost_factors(&advanced_rep.boost_factors, composite_score);

        // Apply time decay
        let final_score = Self::apply_time_decay(boosted_score, advanced_rep.decay_factor, advanced_rep.last_updated);

        advanced_rep.reputation_score = final_score;
        advanced_rep.reputation_tier = Self::calculate_reputation_tier(final_score);
        advanced_rep.last_updated = env.ledger().timestamp();

        Self::set_advanced_reputation(env, user, &advanced_rep);
    }

    // ========== Helper Functions ==========

    fn calculate_reputation_score(env: &Env, user: &Address) -> u64 {
        let advanced_rep = Self::get_advanced_reputation(env, user);
        advanced_rep.reputation_score
    }

    fn calculate_endorsement_weight(endorser_reputation: u64) -> u32 {
        // Higher reputation endorsers have more weight
        match endorser_reputation {
            0..=1000 => 1,
            1001..=5000 => 2,
            5001..=10000 => 3,
            10001..=50000 => 4,
            _ => 5,
        }
    }

    fn calculate_trust_score(social_proof: &SocialProof) -> u32 {
        let endorsement_weight = social_proof.endorsements.len() as u32 * 10;
        let verification_weight = social_proof.verifications.len() as u32 * 20;
        let connection_weight = social_proof.social_connections * 5;
        let vouch_weight = social_proof.vouches_received * 15;

        (endorsement_weight + verification_weight + connection_weight + vouch_weight).min(1000)
    }

    fn aggregate_cross_platform_scores(cross_platform: &CrossPlatformReputation) -> u32 {
        let mut total_score = 0u32;
        let mut count = 0u32;

        for (_, score) in cross_platform.platform_scores.iter() {
            total_score += score;
            count += 1;
        }

        if count > 0 {
            total_score / count
        } else {
            0
        }
    }

    fn calculate_base_score(base_rep: &UserReputation) -> u32 {
        let participation_score = base_rep.participation_score;
        let completion_score = base_rep.completion_rate;
        let contribution_score = base_rep.contribution_quality;
        let course_score = (base_rep.total_courses_completed * 10).min(300);

        (participation_score + completion_score + contribution_score + course_score).min(1000)
    }

    fn calculate_social_score(social_proof: &SocialProof) -> u32 {
        social_proof.trust_score.min(1000)
    }

    fn calculate_credit_score(external_credit: &ExternalCreditData) -> u32 {
        let bureau_score = external_credit.credit_bureau_score;
        let alternative_score = external_credit.alternative_data_score;
        let payment_score = Self::calculate_payment_score(&external_credit.payment_history);

        (bureau_score + alternative_score + payment_score) / 3
    }

    fn calculate_payment_score(payment_history: &PaymentHistory) -> u32 {
        if payment_history.total_payments == 0 {
            return 0;
        }

        let on_time_ratio = (payment_history.on_time_payments * 1000) / payment_history.total_payments;
        let streak_bonus = payment_history.payment_streak * 10;

        (on_time_ratio + streak_bonus).min(1000)
    }

    fn apply_boost_factors(boost_factors: &Vec<ReputationBoost>, base_score: u64) -> u64 {
        let mut multiplier = 100u32; // Base multiplier (100%)

        for boost in boost_factors.iter() {
            if boost.expires_at > 0 { // Active boost
                multiplier += boost.multiplier;
            }
        }

        (base_score * multiplier as u64) / 100
    }

    fn apply_time_decay(score: u64, decay_factor: u32, last_updated: u64) -> u64 {
        // Simple time decay - can be made more sophisticated
        let current_time = 1234567890; // Would use env.ledger().timestamp()
        let time_diff = current_time - last_updated;
        
        if time_diff > 86400 * 30 { // 30 days
            (score * decay_factor as u64) / 100
        } else {
            score
        }
    }

    fn calculate_reputation_tier(score: u64) -> ReputationTier {
        match score {
            0..=1000 => ReputationTier::Bronze,
            1001..=5000 => ReputationTier::Silver,
            5001..=15000 => ReputationTier::Gold,
            15001..=50000 => ReputationTier::Platinum,
            _ => ReputationTier::Diamond,
        }
    }

    fn calculate_insurance_premium(coverage_amount: u64, coverage_type: InsuranceType) -> u64 {
        let base_rate = match coverage_type {
            InsuranceType::ReputationLoss => 50,
            InsuranceType::DisputeProtection => 75,
            InsuranceType::IncomeProtection => 100,
            InsuranceType::CompletionGuarantee => 60,
        };

        (coverage_amount * base_rate as u64) / 10000
    }

    // ========== Storage Functions ==========

    fn get_advanced_reputation(env: &Env, user: &Address) -> AdvancedReputation {
        env.storage()
            .persistent()
            .get(&(ADVANCED_REPUTATION, user.clone()))
            .unwrap_or_else(|| {
                panic_with_error!(env, AdvancedReputationError::UnauthorizedAccess)
            })
    }

    fn set_advanced_reputation(env: &Env, user: &Address, reputation: &AdvancedReputation) {
        env.storage()
            .persistent()
            .set(&(ADVANCED_REPUTATION, user.clone()), reputation);
    }

    fn get_social_proof(env: &Env, user: &Address) -> SocialProof {
        env.storage()
            .persistent()
            .get(&(SOCIAL_PROOF, user.clone()))
            .unwrap_or_else(|| SocialProof {
                user: user.clone(),
                endorsements: Vec::new(env),
                verifications: Vec::new(env),
                social_connections: 0,
                trust_score: 0,
                vouches_received: 0,
                vouches_given: 0,
                last_updated: 0,
            })
    }

    fn set_social_proof(env: &Env, user: &Address, social_proof: &SocialProof) {
        env.storage()
            .persistent()
            .set(&(SOCIAL_PROOF, user.clone()), social_proof);
    }

    fn get_cross_platform_reputation(env: &Env, user: &Address) -> CrossPlatformReputation {
        env.storage()
            .persistent()
            .get(&(CROSS_PLATFORM_REP, user.clone()))
            .unwrap_or_else(|| CrossPlatformReputation {
                user: user.clone(),
                platform_scores: Map::new(env),
                aggregated_score: 0,
                verification_count: 0,
                last_sync: 0,
            })
    }

    fn set_cross_platform_reputation(env: &Env, user: &Address, cross_platform: &CrossPlatformReputation) {
        env.storage()
            .persistent()
            .set(&(CROSS_PLATFORM_REP, user.clone()), cross_platform);
    }

    fn get_external_credit_data(env: &Env, user: &Address) -> ExternalCreditData {
        env.storage()
            .persistent()
            .get(&(CREDIT_SCORE_EXTERNAL, user.clone()))
            .unwrap_or_else(|| ExternalCreditData {
                user: user.clone(),
                credit_bureau_score: 0,
                alternative_data_score: 0,
                payment_history: PaymentHistory {
                    on_time_payments: 0,
                    late_payments: 0,
                    missed_payments: 0,
                    total_payments: 0,
                    payment_streak: 0,
                },
                debt_to_income_ratio: 0,
                credit_utilization: 0,
                account_age: 0,
                last_updated: 0,
            })
    }

    fn set_external_credit_data(env: &Env, user: &Address, credit_data: &ExternalCreditData) {
        env.storage()
            .persistent()
            .set(&(CREDIT_SCORE_EXTERNAL, user.clone()), credit_data);
    }

    fn get_reputation_marketplace(env: &Env) -> ReputationMarketplace {
        env.storage()
            .persistent()
            .get(&REPUTATION_MARKET)
            .unwrap_or_else(|| ReputationMarketplace {
                reputation_listings: Vec::new(env),
                active_bids: Map::new(env),
                transaction_history: Vec::new(env),
            })
    }

    fn set_reputation_marketplace(env: &Env, marketplace: &ReputationMarketplace) {
        env.storage()
            .persistent()
            .set(&REPUTATION_MARKET, marketplace);
    }
}
