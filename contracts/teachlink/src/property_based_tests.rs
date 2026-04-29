//! Property-based tests for core invariants.
//!
//! This module is intentionally test-only so it never affects contract builds.

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    // For n validators, BFT threshold is floor(2n/3) + 1.
    proptest! {
        #[test]
        fn bft_threshold_is_bounded(n in 1u32..=10_000) {
            let threshold = (2 * n) / 3 + 1;
            prop_assert!(threshold >= 1);
            prop_assert!(threshold <= n);
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
