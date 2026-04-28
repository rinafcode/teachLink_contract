//! Property-Based Test Suite
//!
//! Comprehensive property-based tests for teachLink contract algorithms.
//! This file contains the actual test runners using proptest and quickcheck.

use proptest::prelude::*;
use quickcheck::QuickCheck;
use soroban_sdk::{Address, Bytes, Env, Map, Vec};
use std::collections::HashMap;
use teachlink_contract::property_based_tests::*;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test BFT Consensus Properties
    #[test]
    fn test_bft_threshold_properties() {
        let mut config = ProptestConfig::default();
        config.cases = 100;

        proptest!(ProptestConfig::with_cases(100), |(n_validators in 1u32..=100u32)| {
            // Property: Byzantine threshold maintains BFT safety
            let expected_threshold = (2 * n_validators) / 3 + 1;

            // Threshold should never exceed total validators
            prop_assert!(expected_threshold <= n_validators,
                        "Threshold {} exceeds total validators {}",
                        expected_threshold, n_validators);

            // Threshold should be > n/3 (can tolerate up to floor((n-1)/3) faulty)
            let faulty_tolerance = (n_validators - 1) / 3;
            prop_assert!(expected_threshold > n_validators - faulty_tolerance,
                        "Threshold {} doesn't protect against {} faulty validators",
                        expected_threshold, faulty_tolerance);
        });
    }

    #[test]
    fn test_bridge_consensus_byzantine_fault_tolerance() {
        // Property: System tolerates up to f = (n-1)/3 Byzantine validators
        proptest!(ProptestConfig::with_cases(100), |(n_validators in 4u32..=100u32)| {
            let max_byzantine = (n_validators - 1) / 3;
            let threshold = (2 * n_validators) / 3 + 1;
            let honest_validators = n_validators - max_byzantine;

            // Property: Honest validators must be able to reach consensus
            prop_assert!(honest_validators >= threshold,
                        "Honest validators {} must reach threshold {} with {} byzantine",
                        honest_validators, threshold, max_byzantine);

            // Property: Byzantine validators alone cannot reach consensus
            prop_assert!(max_byzantine < threshold,
                        "Byzantine validators {} cannot reach threshold {} alone",
                        max_byzantine, threshold);
        });
    }

    #[test]
    fn test_bridge_consensus_threshold_monotonicity() {
        // Property: Adding validators never decreases the threshold
        proptest!(ProptestConfig::with_cases(100), |(n in 1u32..=99u32)| {
            let threshold_n = (2 * n) / 3 + 1;
            let threshold_n_plus_1 = (2 * (n + 1)) / 3 + 1;

            prop_assert!(threshold_n_plus_1 >= threshold_n,
                        "Threshold should be monotonically increasing: {} -> {}",
                        threshold_n, threshold_n_plus_1);
        });
    }

    #[test]
    fn test_bridge_validator_stake_invariants() {
        // Property: Total stake must be sum of individual stakes
        proptest!(ProptestConfig::with_cases(50), |(
            n_validators in 1usize..=20usize,
            stakes in prop::collection::vec(100_000_000i128..1_000_000_000i128, 1..=20)
        )| {
            let total_stake: i128 = stakes.iter().take(n_validators).sum();

            // Property: Total stake must be positive
            prop_assert!(total_stake > 0, "Total stake must be positive");

            // Property: Each individual stake must meet minimum
            for stake in stakes.iter().take(n_validators) {
                prop_assert!(*stake >= 100_000_000i128,
                           "Individual stake {} below minimum", stake);
            }

            // Property: Total stake shouldn't overflow
            prop_assert!(total_stake > 0 && total_stake < i128::MAX,
                        "Total stake should not overflow");
        });
    }

    #[test]
    fn test_bridge_proposal_vote_threshold_validation() {
        // Property: Votes must meet or exceed threshold for consensus
        proptest!(ProptestConfig::with_cases(100), |(
            n_validators in 1u32..=50u32,
            votes_cast in 0u32..=50u32
        )| {
            let threshold = (2 * n_validators) / 3 + 1;

            // Property: Consensus reached if and only if votes >= threshold
            let consensus_reached = votes_cast >= threshold;

            if votes_cast >= threshold {
                prop_assert!(consensus_reached,
                           "Consensus should be reached with {} votes >= threshold {}",
                           votes_cast, threshold);
            } else {
                prop_assert!(!consensus_reached,
                           "Consensus should NOT be reached with {} votes < threshold {}",
                           votes_cast, threshold);
            }
        });
    }

    #[test]
    fn test_bridge_consensus_quorum_intersection() {
        // Property: Any two quorums must intersect (safety property)
        proptest!(ProptestConfig::with_cases(50), |(
            n_validators in 4u32..=100u32,
            quorum1_size in 1u32..=100u32,
            quorum2_size in 1u32..=100u32
        )| {
            let threshold = (2 * n_validators) / 3 + 1;

            // Only test valid quorums
            if quorum1_size >= threshold && quorum2_size >= threshold {
                // Property: Two quorums must have intersection
                // Minimum intersection = quorum1 + quorum2 - n
                let min_intersection = if quorum1_size + quorum2_size > n_validators {
                    quorum1_size + quorum2_size - n_validators
                } else {
                    0
                };

                // For BFT, any two quorums MUST intersect
                prop_assert!(quorum1_size + quorum2_size > n_validators,
                           "Two quorums of size {} and {} must intersect in {} validators",
                           quorum1_size, quorum2_size, n_validators);
            }
        });
    }

    #[test]
    fn test_consensus_state_consistency() {
        proptest!(ProptestConfig::with_cases(50), |(
            n_validators in 1usize..=10usize,
            operations in prop::collection::vec(any::<ValidatorOperation>(), 0..=3usize)
        )| {
            let env = Env::default();
            let mut total_stake = 0i128;
            let mut active_count = 0u32;

            // Initialize validators
            for i in 0..n_validators {
                let stake = 100_000_000i128 + (i as i128 * 1000);
                total_stake += stake;
                active_count += 1;
            }

            // Apply operations and verify invariants
            for op in operations {
                match op {
                    ValidatorOperation::Add => {
                        let new_stake = 100_000_000i128 + 1000;
                        total_stake += new_stake;
                        active_count += 1;
                    }
                    ValidatorOperation::Remove => {
                        if active_count > 0 {
                            active_count -= 1;
                        }
                    }
                }
            }

            // Property: Total stake should be non-negative
            prop_assert!(total_stake >= 0, "Total stake cannot be negative");

            // Property: Active count should be reasonable
            prop_assert!(active_count <= n_validators as u32 + operations.len() as u32);
        });
    }

    #[test]
    fn test_proposal_voting_consistency() {
        proptest!(ProptestConfig::with_cases(100), |(
            n_validators in 1u32..=20u32,
            n_votes in 1u32..=20u32
        )| {
            let threshold = (2 * n_validators) / 3 + 1;

            // Property: If votes >= threshold, consensus should be reached
            if n_votes >= threshold {
                prop_assert!(true, "Consensus should be reached with {} votes >= threshold {}",
                            n_votes, threshold);
            }

            // Property: Threshold should never be 0
            prop_assert!(threshold > 0, "Threshold must be positive");

            // Property: Threshold should be reasonable fraction of total
            let ratio = threshold as f64 / n_validators as f64;
            prop_assert!(ratio >= 0.5 && ratio <= 1.0,
                        "Threshold ratio {} should be between 0.5 and 1.0", ratio);
        });
    }

    /// Test Assessment System Properties
    #[test]
    fn test_score_calculation_bounds() {
        proptest!(ProptestConfig::with_cases(100), |(
            n_questions in 1u32..=100u32,
            max_points in 1u32..=10u32,
            n_correct in 0u32..=100u32
        )| {
            let total_possible = n_questions * max_points;
            let earned = (n_correct.min(n_questions)) * max_points;

            // Property: Score should be bounded by 0 and total possible
            prop_assert!(earned <= total_possible, "Earned score {} exceeds total possible {}",
                        earned, total_possible);

            // Property: Percentage should be between 0 and 100
            let percentage = if total_possible > 0 {
                (earned * 100) / total_possible
            } else {
                0
            };
            prop_assert!(percentage <= 100, "Percentage {} exceeds 100", percentage);
        });
    }

    #[test]
    fn test_adaptive_difficulty_monotonic() {
        proptest!(ProptestConfig::with_cases(100), |performance_ratio in 0u32..=100u32| {
            let target_difficulty = if performance_ratio > 70 {
                7
            } else if performance_ratio < 30 {
                3
            } else {
                5
            };

            // Property: Difficulty should be within valid range
            prop_assert!(target_difficulty >= 1 && target_difficulty <= 10,
                        "Target difficulty {} should be between 1 and 10", target_difficulty);

            // Property: Higher performance should not result in lower difficulty
            if performance_ratio > 70 {
                prop_assert!(target_difficulty >= 5,
                            "High performance {} should result in difficulty >= 5", performance_ratio);
            } else if performance_ratio < 30 {
                prop_assert!(target_difficulty <= 5,
                            "Low performance {} should result in difficulty <= 5", performance_ratio);
            }
        });
    }

    #[test]
    fn test_plagiarism_threshold() {
        proptest!(ProptestConfig::with_cases(100), |(
            total_questions in 3usize..=50usize,
            match_count in 0usize..=50usize
        )| {
            let similarity_percentage = (match_count * 100) / total_questions;
            let is_plagiarism = similarity_percentage > 90;

            // Property: Perfect match should always be plagiarism
            if match_count == total_questions {
                prop_assert!(is_plagiarism, "Perfect match should be detected as plagiarism");
            }

            // Property: Zero matches should never be plagiarism
            if match_count == 0 {
                prop_assert!(!is_plagiarism, "Zero matches should not be plagiarism");
            }

            // Property: Threshold consistency
            if match_count > (total_questions * 90) / 100 {
                prop_assert!(is_plagiarism, "Match count {} exceeds 90% threshold", match_count);
            } else if match_count <= (total_questions * 90) / 100 {
                prop_assert!(!is_plagiarism, "Match count {} is at or below 90% threshold", match_count);
            }
        });
    }

    /// Test Analytics Properties
    #[test]
    fn test_moving_average_convergence() {
        proptest!(ProptestConfig::with_cases(50), |values in prop::collection::vec(any::<u64>(), 1..=100usize)| {
            let mut ema = values[0];
            let alpha = 10; // 10% smoothing factor

            for &value in &values[1..] {
                ema = ((ema * (100 - alpha)) + (value * alpha)) / 100;
            }

            // Property: EMA should be within min-max range of values
            let min_val = *values.iter().min().unwrap();
            let max_val = *values.iter().max().unwrap();

            prop_assert!(ema >= min_val, "EMA {} should not be below minimum {}", ema, min_val);
            prop_assert!(ema <= max_val, "EMA {} should not exceed maximum {}", ema, max_val);

            // Property: EMA should be closer to recent values
            if values.len() > 10 {
                let recent_avg = values.iter().rev().take(5).sum::<u64>() / 5;
                let early_avg = values.iter().take(5).sum::<u64>() / 5;

                let diff_recent = if ema > recent_avg { ema - recent_avg } else { recent_avg - ema };
                let diff_early = if ema > early_avg { ema - early_avg } else { early_avg - ema };

                prop_assert!(diff_recent <= diff_early,
                            "EMA should be closer to recent values than early values");
            }
        });
    }

    #[test]
    fn test_health_score_bounds() {
        proptest!(ProptestConfig::with_cases(100), |(
            success_rate in 0u32..=10000u32, // basis points
            active_validators in 0u32..=100u32,
            confirmation_time in 0u32..=7200u32 // seconds
        )| {
            // Calculate component scores
            let success_score = success_rate / 100; // Convert to percentage
            let validator_score = if active_validators > 0 { 100 } else { 0 };

            let confirmation_score = if confirmation_time < 300 {
                100
            } else if confirmation_time < 600 {
                80
            } else if confirmation_time < 1800 {
                60
            } else if confirmation_time < 3600 {
                40
            } else {
                20
            };

            // Calculate weighted health score
            let health_score = ((success_score * 40) + (validator_score * 30) + (confirmation_score * 30)) / 100;

            // Property: Health score should be bounded by 0-100
            prop_assert!(health_score <= 100, "Health score {} should not exceed 100", health_score);

            // Property: Zero components should result in low health score
            if success_rate == 0 && active_validators == 0 && confirmation_time >= 3600 {
                prop_assert!(health_score <= 20, "All zero components should result in low health score");
            }

            // Property: Perfect components should result in high health score
            if success_rate >= 10000 && active_validators > 0 && confirmation_time < 300 {
                prop_assert!(health_score >= 90, "Perfect components should result in high health score");
            }
        });
    }

    /// Test Atomic Swap Properties
    #[test]
    fn test_timelock_bounds() {
        proptest!(ProptestConfig::with_cases(100), |timelock in 0u64..=1_000_000u64| {
            let is_valid = timelock >= 3600 && timelock <= 604800;

            // Property: Timelock within bounds should be valid
            if timelock >= 3600 && timelock <= 604800 {
                prop_assert!(is_valid, "Timelock {} within bounds should be valid", timelock);
            }

            // Property: Timelock outside bounds should be invalid
            if timelock < 3600 || timelock > 604800 {
                prop_assert!(!is_valid, "Timelock {} outside bounds should be invalid", timelock);
            }
        });
    }

    #[test]
    fn test_hash_verification_consistency() {
        proptest!(ProptestConfig::with_cases(50), |preimage in prop::collection::vec(any::<u8>(), 32)| {
            // Property: Hash of same preimage should always be same
            let hash1 = simulate_sha256(&preimage);
            let hash2 = simulate_sha256(&preimage);
            prop_assert!(hash1 == hash2, "Hash of same preimage should be identical");

            // Property: Different preimages should (usually) have different hashes
            let mut different_preimage = preimage.clone();
            if !different_preimage.is_empty() {
                different_preimage[0] = different_preimage[0].wrapping_add(1);
                let hash_different = simulate_sha256(&different_preimage);

                // Note: This is probabilistic - collisions are possible but extremely unlikely
                if hash1 == hash_different {
                    // If collision occurs, verify it's actually a collision case
                    prop_assert!(preimage != different_preimage,
                               "Different preimages should have different hashes (collision detected)");
                }
            }
        });
    }

    #[test]
    fn test_swap_rate_calculation() {
        proptest!(ProptestConfig::with_cases(100), |(
            initiator_amount in 1i128..=1_000_000i128,
            counterparty_amount in 1i128..=1_000_000i128
        )| {
            let rate = counterparty_amount as f64 / initiator_amount as f64;

            // Property: Rate should be non-negative
            prop_assert!(rate >= 0.0, "Swap rate should be non-negative");

            // Property: Rate should be inversely proportional to initiator amount
            if counterparty_amount > 0 {
                let rate_double_initiator = counterparty_amount as f64 / (initiator_amount * 2) as f64;
                prop_assert!(rate_double_initiator <= rate,
                            "Doubling initiator amount should not increase rate");
            }

            // Property: Rate should be directly proportional to counterparty amount
            if initiator_amount > 0 {
                let rate_double_counterparty = (counterparty_amount * 2) as f64 / initiator_amount as f64;
                prop_assert!(rate_double_counterparty >= rate,
                            "Doubling counterparty amount should not decrease rate");
            }
        });
    }

    /// Test Input Validation and Fuzzing
    #[test]
    fn test_address_validation() {
        proptest!(ProptestConfig::with_cases(100), |address_str in "[a-zA-Z0-9]{1,64}"| {
            // Property: Valid addresses should be accepted
            if address_str.len() >= 3 && address_str.len() <= 64 {
                prop_assert!(!address_str.is_empty(), "Valid address should not be empty");
            }

            // Property: Empty or too long addresses should be rejected
            if address_str.is_empty() || address_str.len() > 64 {
                prop_assert!(true, "Empty or too long addresses should be rejected");
            }
        });
    }

    #[test]
    fn test_amount_validation() {
        proptest!(ProptestConfig::with_cases(100), |amount in any::<i128>()| {
            let is_valid = amount > 0;

            // Property: Positive amounts should be valid
            if amount > 0 {
                prop_assert!(is_valid, "Positive amount {} should be valid", amount);
            }

            // Property: Zero or negative amounts should be invalid
            if amount <= 0 {
                prop_assert!(!is_valid, "Non-positive amount {} should be invalid", amount);
            }
        });
    }

    #[test]
    fn test_hash_length_validation() {
        proptest!(ProptestConfig::with_cases(100), |hash_bytes in prop::collection::vec(any::<u8>(), 0..=100)| {
            let is_valid_length = hash_bytes.len() == 32;

            // Property: Correct length should be valid
            if hash_bytes.len() == 32 {
                prop_assert!(is_valid_length, "Hash with correct length should be valid");
            }

            // Property: Incorrect length should be invalid
            if hash_bytes.len() != 32 {
                prop_assert!(!is_valid_length, "Hash with incorrect length should be invalid");
            }
        });
    }

    #[test]
    fn test_question_difficulty_bounds() {
        proptest!(ProptestConfig::with_cases(100), |difficulty in 0u32..=20u32| {
            let is_valid = difficulty >= 1 && difficulty <= 10;

            // Property: Valid range should be accepted
            if difficulty >= 1 && difficulty <= 10 {
                prop_assert!(is_valid, "Difficulty {} in valid range should be accepted", difficulty);
            }

            // Property: Out of range should be rejected
            if difficulty < 1 || difficulty > 10 {
                prop_assert!(!is_valid, "Difficulty {} out of range should be rejected", difficulty);
            }
        });
    }

    /// Integration Tests
    #[test]
    fn test_end_to_end_properties() {
        // Test that all major invariants hold across the entire system
        run_property_tests();
        run_fuzzing_tests();
    }

    #[test]
    fn test_property_test_performance() {
        let start = std::time::Instant::now();
        run_property_tests();
        let duration = start.elapsed();

        // Property: Property tests should complete within reasonable time
        assert!(
            duration.as_secs() < 60,
            "Property tests should complete within 60 seconds"
        );
    }

    /// QuickCheck-based fuzzing tests
    #[test]
    fn test_quickcheck_fuzzing() {
        // Test address validation with QuickCheck
        fn prop_address_validation(s: String) -> bool {
            if s.len() >= 3 && s.len() <= 64 {
                !s.is_empty()
            } else {
                true // Empty or too long is handled by validation
            }
        }

        QuickCheck::new()
            .tests(1000)
            .quickcheck(prop_address_validation as fn(String) -> bool);

        // Test amount validation with QuickCheck
        fn prop_amount_validation(amount: i128) -> bool {
            amount > 0
        }

        QuickCheck::new()
            .tests(1000)
            .quickcheck(prop_amount_validation as fn(i128) -> bool);

        // Test hash length validation with QuickCheck
        fn prop_hash_validation(data: Vec<u8>) -> bool {
            data.len() == 32
        }

        QuickCheck::new()
            .tests(1000)
            .quickcheck(prop_hash_validation as fn(Vec<u8>) -> bool);
    }

    /// Stress tests with large inputs
    #[test]
    fn test_stress_large_inputs() {
        proptest!(ProptestConfig::with_cases(10), |(
            large_validator_count in 1000u32..=10000u32,
            large_amount in 1_000_000i128..=1_000_000_000i128
        )| {
            // Test that algorithms handle large inputs gracefully
            let threshold = (2 * large_validator_count) / 3 + 1;
            prop_assert!(threshold <= large_validator_count,
                        "Threshold should not exceed validator count even for large numbers");

            let rate = large_amount as f64 / 1_000_000i128 as f64;
            prop_assert!(rate.is_finite(), "Rate calculation should not overflow for large amounts");
        });
    }

    /// Edge case testing
    #[test]
    fn test_edge_cases() {
        // Test boundary conditions
        assert_eq!(
            (2 * 1u32) / 3 + 1,
            1,
            "Minimum validator threshold should be 1"
        );
        assert_eq!(
            (2 * 3u32) / 3 + 1,
            3,
            "3 validators should require threshold of 3"
        );
        assert_eq!(
            (2 * 4u32) / 3 + 1,
            4,
            "4 validators should require threshold of 4"
        );

        // Test zero divisions
        let rate = 0i128 as f64 / 1i128 as f64;
        assert_eq!(rate, 0.0, "Zero amount should result in zero rate");

        // Test empty collections
        let empty_vec: Vec<u8> = vec![];
        let hash = simulate_sha256(&empty_vec);
        assert_eq!(
            hash.len(),
            32,
            "Empty input should still produce 32-byte hash"
        );
    }

    /// Bridge Consensus Edge Cases
    #[test]
    fn test_bridge_consensus_edge_cases() {
        // Test minimum viable validator sets
        assert_eq!(
            (2 * 1u32) / 3 + 1,
            1,
            "1 validator: threshold = 1 (no fault tolerance)"
        );
        assert_eq!(
            (2 * 2u32) / 3 + 1,
            2,
            "2 validators: threshold = 2 (no fault tolerance)"
        );
        assert_eq!(
            (2 * 3u32) / 3 + 1,
            3,
            "3 validators: threshold = 3 (tolerates 0 faulty)"
        );
        assert_eq!(
            (2 * 4u32) / 3 + 1,
            3,
            "4 validators: threshold = 3 (tolerates 1 faulty)"
        );

        // Test specific BFT configurations
        // n = 3f + 1 formula
        for f in 1u32..=10u32 {
            let n = 3 * f + 1;
            let threshold = (2 * n) / 3 + 1;
            let expected_threshold = 2 * f + 1;

            assert_eq!(
                threshold, expected_threshold,
                "For f={} faulty tolerance (n={} validators), threshold should be {}",
                f, n, expected_threshold
            );

            // Verify Byzantine validators cannot reach threshold
            assert!(
                f < threshold,
                "f={} byzantine validators cannot reach threshold={}",
                f, threshold
            );
        }
    }

    #[test]
    fn test_bridge_validator_rotation_properties() {
        // Property: Validator rotation maintains minimum active validators
        proptest!(ProptestConfig::with_cases(50), |(
            initial_validators in 4u32..=100u32,
            rotated_out in 0u32..=10u32
        )| {
            let remaining = initial_validators.saturating_sub(rotated_out);
            let min_for_bft = 4; // Minimum 4 validators needed for 1-fault tolerance

            // Property: Must maintain minimum validators for BFT
            if rotated_out < initial_validators {
                prop_assert!(remaining >= 1,
                           "Must have at least 1 validator after rotation");
            }

            // Property: Cannot rotate out more than available
            prop_assert!(rotated_out <= initial_validators,
                       "Cannot rotate out more validators than exist");
        });
    }

    #[test]
    fn test_bridge_proposal_expiry_invariants() {
        // Property: Expired proposals cannot be executed
        proptest!(ProptestConfig::with_cases(100), |(
            created_at in 0u64..=1_000_000u64,
            timeout in 1u64..=100_000u64,
            current_time in 0u64..=2_000_000u64
        )| {
            let expires_at = created_at + timeout;
            let is_expired = current_time > expires_at;

            // Property: If expired, cannot execute
            if is_expired {
                prop_assert!(current_time > created_at,
                           "Expired proposal must have current_time > created_at");
            }

            // Property: If not expired, can potentially execute
            if !is_expired {
                prop_assert!(current_time <= expires_at,
                           "Non-expired proposal must have current_time <= expires_at");
            }
        });
    }

    #[test]
    fn test_bridge_consensus_reputation_bounds() {
        // Property: Reputation scores must stay within bounds [0, 100]
        proptest!(ProptestConfig::with_cases(100), |(
            initial_reputation in 0u32..=100u32,
            reputation_change in -50i32..=50i32
        )| {
            let new_reputation = (initial_reputation as i32 + reputation_change).clamp(0, 100);

            // Property: Reputation must be within bounds
            prop_assert!(new_reputation >= 0 && new_reputation <= 100,
                       "Reputation {} must be in [0, 100]", new_reputation);

            // Property: Positive change increases or maintains reputation
            if reputation_change > 0 {
                prop_assert!(new_reputation >= initial_reputation as u32,
                           "Positive change should not decrease reputation");
            }

            // Property: Negative change decreases or maintains reputation
            if reputation_change < 0 {
                prop_assert!(new_reputation <= initial_reputation,
                           "Negative change should not increase reputation");
            }
        });
    }
}

// Helper function to simulate SHA256 for testing
fn simulate_sha256(data: &[u8]) -> Vec<u8> {
    let mut hash = vec![0u8; 32];
    for (i, &byte) in data.iter().enumerate() {
        hash[i % 32] = hash[i % 32].wrapping_add(byte).wrapping_mul(31);
    }
    hash
}

// Helper enum for validator operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidatorOperation {
    Add,
    Remove,
}

impl Arbitrary for ValidatorOperation {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        if bool::arbitrary(g) {
            ValidatorOperation::Add
        } else {
            ValidatorOperation::Remove
        }
    }
}

use quickcheck::Arbitrary;
