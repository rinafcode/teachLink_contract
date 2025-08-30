use starknet::ContractAddress;
use alexandria_math::pow;

#[derive(Drop, Serde)]
struct WeightingFactors {
    credibility_weight: u256,
    recency_weight: u256,
    volume_weight: u256,
    consistency_weight: u256,
}

#[derive(Drop, Serde)]
struct ScoreComponents {
    base_score: u256,
    credibility_bonus: u256,
    recency_factor: u256,
    consistency_factor: u256,
}

trait ReputationCalculationTrait {
    fn calculate_weighted_score(
        reviews: Span<super::interfaces::IReputationSystem::Review>,
        current_timestamp: u64,
        weighting_factors: WeightingFactors
    ) -> u256;
    
    fn calculate_reviewer_credibility(
        total_reviews: u32,
        accurate_reviews: u32,
        flagged_reviews: u32,
        account_age_days: u32
    ) -> u256;
    
    fn apply_anti_gaming_penalties(
        base_score: u256,
        suspicious_patterns: u32,
        flagged_reviews: u32
    ) -> u256;
    
    fn calculate_recency_factor(review_timestamp: u64, current_timestamp: u64) -> u256;
    fn detect_review_patterns(reviews: Span<super::interfaces::IReputationSystem::Review>) -> u32;
    fn normalize_score(raw_score: u256, max_possible: u256) -> u256;
}

impl ReputationCalculationImpl of ReputationCalculationTrait {
    fn calculate_weighted_score(
        reviews: Span<super::interfaces::IReputationSystem::Review>,
        current_timestamp: u64,
        weighting_factors: WeightingFactors
    ) -> u256 {
        if reviews.len() == 0 {
            return 0;
        }
        
        let mut total_weighted_score: u256 = 0;
        let mut total_weight: u256 = 0;
        
        let mut i = 0;
        loop {
            if i >= reviews.len() {
                break;
            }
            
            let review = *reviews.at(i);
            if !review.is_flagged {
                // Calculate individual review weight
                let credibility_factor = review.credibility_score * weighting_factors.credibility_weight / 100;
                let recency_factor = Self::calculate_recency_factor(review.timestamp, current_timestamp);
                let recency_weighted = recency_factor * weighting_factors.recency_weight / 100;
                
                let review_weight = credibility_factor + recency_weighted;
                let weighted_rating = review.rating.into() * review_weight;
                
                total_weighted_score += weighted_rating;
                total_weight += review_weight;
            }
            
            i += 1;
        };
        
        if total_weight == 0 {
            return 0;
        }
        
        // Normalize to 0-100 scale
        let base_score = (total_weighted_score * 100) / (total_weight * 5); // 5 is max rating
        
        // Apply anti-gaming penalties
        let suspicious_patterns = Self::detect_review_patterns(reviews);
        let flagged_count = Self::count_flagged_reviews(reviews);
        
        Self::apply_anti_gaming_penalties(base_score, suspicious_patterns, flagged_count)
    }


    