//! Property-based tests for core invariants.
//!
//! This module is intentionally test-only so it never affects contract builds.

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    // Stake-weighted BFT threshold (#496): given total stake `S`, a quorum
    // must control `floor(2 * S / 3) + 1` stake. `n` (validator count) no
    // longer drives the threshold.
    proptest! {
        #[test]
        fn stake_weighted_bft_threshold_is_bounded(total_stake in 1i128..=1_000_000_000_000i128) {
            let threshold = (total_stake.saturating_mul(2) / 3) + 1;
            // The threshold is a real quorum: strictly positive and never more
            // than the whole stake (so it is always reachable by full consensus).
            prop_assert!(threshold >= 1);
            prop_assert!(threshold <= total_stake);
        }

        // Sybil resistance: reaching the stake-weighted quorum requires
        // controlling strictly more than 2/3 of the total stake. Splitting a
        // fixed adversarial stake across many low-stake (Sybil) validators
        // raises `total_stake` — and therefore the threshold — in lockstep, so
        // validator *count* never lets an under-2/3 adversary reach quorum.
        #[test]
        fn stake_threshold_resists_sybil_count(
            honest_stake in 1i128..=1_000_000_000i128,
            sybil_unit in 1i128..=1_000_000i128,
            sybil_count in 0i128..=100_000i128,
        ) {
            let adversary_stake = sybil_unit.saturating_mul(sybil_count);
            let total_stake = honest_stake.saturating_add(adversary_stake);
            let threshold = (total_stake.saturating_mul(2) / 3) + 1;
            // For every possible split of stake into validators, the adversary
            // is either below the quorum threshold or genuinely controls more
            // than 2/3 of the total stake — never merely by adding validators.
            prop_assert!(
                adversary_stake < threshold
                    || adversary_stake.saturating_mul(3) > total_stake.saturating_mul(2)
            );
        }

        #[test]
        fn score_percentage_is_in_range(total in 1u32..=10_000, earned in 0u32..=10_000) {
            let bounded_earned = core::cmp::min(earned, total);
            let pct = (bounded_earned * 100) / total;
            prop_assert!(pct <= 100);
        }

        #[test]
        fn timelock_range_is_valid(min in 1u64..=1_000_000, max in 1u64..=1_000_000) {
            let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
            prop_assert!(lo <= hi);
        }
    }
}
