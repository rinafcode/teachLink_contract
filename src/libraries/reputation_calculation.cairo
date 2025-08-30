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


    fn calculate_reviewer_credibility(
        total_reviews: u32,
        accurate_reviews: u32,
        flagged_reviews: u32,
        account_age_days: u32
    ) -> u256 {
        if total_reviews == 0 {
            return 50; // Default credibility for new reviewers
        }
        
        // Base credibility from accuracy
        let accuracy_rate = (accurate_reviews * 100) / total_reviews;
        let mut credibility = accuracy_rate.into();
        
        // Penalty for flagged reviews
        let flagged_penalty = (flagged_reviews * 10).into();
        if credibility > flagged_penalty {
            credibility -= flagged_penalty;
        } else {
            credibility = 0;
        }
        
        // Bonus for account age (up to 20 points)
        let age_bonus = if account_age_days > 365 {
            20
        } else {
            (account_age_days * 20) / 365
        };
        
        credibility += age_bonus.into();
        
        // Bonus for review volume (up to 10 points)
        let volume_bonus = if total_reviews > 100 {
            10
        } else {
            (total_reviews * 10) / 100
        };
        
        credibility += volume_bonus.into();
        
        // Cap at 100
        if credibility > 100 {
            100
        } else {
            credibility
        }
    }
    
    fn apply_anti_gaming_penalties(
        base_score: u256,
        suspicious_patterns: u32,
        flagged_reviews: u32
    ) -> u256 {
        let mut final_score = base_score;
        
        // Penalty for suspicious patterns (5% per pattern)
        let pattern_penalty = (suspicious_patterns * 5).into();
        let pattern_reduction = (final_score * pattern_penalty) / 100;
        if final_score > pattern_reduction {
            final_score -= pattern_reduction;
        } else {
            final_score = 0;
        }
        
        // Penalty for flagged reviews (10% per flagged review)
        let flagged_penalty = (flagged_reviews * 10).into();
        let flagged_reduction = (final_score * flagged_penalty) / 100;
        if final_score > flagged_reduction {
            final_score -= flagged_reduction;
        } else {
            final_score = 0;
        }
        
        final_score
    }
    
    fn calculate_recency_factor(review_timestamp: u64, current_timestamp: u64) -> u256 {
        if current_timestamp <= review_timestamp {
            return 100; // Future timestamp, full weight
        }
        
        let age_seconds = current_timestamp - review_timestamp;
        let age_days = age_seconds / 86400; // Convert to days
        
        // Exponential decay: newer reviews have more weight
        if age_days <= 30 {
            100 // Full weight for reviews within 30 days
        } else if age_days <= 90 {
            80  // 80% weight for reviews within 90 days
        } else if age_days <= 180 {
            60  // 60% weight for reviews within 180 days
        } else if age_days <= 365 {
            40  // 40% weight for reviews within 1 year
        } else {
            20  // 20% weight for older reviews
        }
    }
    
    fn detect_review_patterns(reviews: Span<super::interfaces::IReputationSystem::Review>) -> u32 {
        let mut suspicious_count = 0;
        let mut reviewer_frequency: Felt252Dict<u32> = Default::default();
        let mut rating_clusters: Felt252Dict<u32> = Default::default();
    
    
        let mut i = 0;
        loop {
            if i >= reviews.len() {
                break;
            }
            
            let review = *reviews.at(i);
            
            // Check for reviewer frequency (same reviewer multiple times)
            let reviewer_key: felt252 = review.reviewer.into();
            let current_count = reviewer_frequency.get(reviewer_key);
            reviewer_frequency.insert(reviewer_key, current_count + 1);
            
            if current_count + 1 > 3 { // More than 3 reviews from same reviewer
                suspicious_count += 1;
            }
            
            // Check for rating clustering (too many similar ratings in short time)
            let rating_key: felt252 = review.rating.into();
            let rating_count = rating_clusters.get(rating_key);
            rating_clusters.insert(rating_key, rating_count + 1);
            
            i += 1;
        };
        
        // Check for rating manipulation (too many 5-star or 1-star reviews)
        let five_star_count = rating_clusters.get(5);
        let one_star_count = rating_clusters.get(1);
        let total_reviews = reviews.len();
        
        if total_reviews > 0 {
            let five_star_ratio = (five_star_count * 100) / total_reviews;
            let one_star_ratio = (one_star_count * 100) / total_reviews;
            
            if five_star_ratio > 80 || one_star_ratio > 80 {
                suspicious_count += 2; // Higher penalty for extreme rating bias
            }
        }
        
        suspicious_count
    }
    
    fn normalize_score(raw_score: u256, max_possible: u256) -> u256 {
        if max_possible == 0 {
            return 0;
        }
        
        (raw_score * 100) / max_possible
    }
    
    fn count_flagged_reviews(reviews: Span<super::interfaces::IReputationSystem::Review>) -> u32 {
        let mut flagged_count = 0;
        let mut i = 0;
        
        loop {
            if i >= reviews.len() {
                break;
            }
            
            if (*reviews.at(i)).is_flagged {
                flagged_count += 1;
            }
            
            i += 1;
        };
        
        flagged_count
    }
}

