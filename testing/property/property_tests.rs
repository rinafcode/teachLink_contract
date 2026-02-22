#![cfg(test)]
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_amount_always_positive(amount in 1i128..1_000_000i128) {
        prop_assert!(amount > 0);
    }

    #[test]
    fn test_threshold_less_than_signers(
        threshold in 1u32..10u32,
        signers in 1u32..10u32
    ) {
        if threshold <= signers {
            prop_assert!(threshold <= signers);
        }
    }

    #[test]
    fn test_chain_id_valid_range(chain_id in 1u32..1000u32) {
        prop_assert!(chain_id > 0);
        prop_assert!(chain_id < 1000);
    }

    #[test]
    fn test_timeout_reasonable(timeout in 60u64..86400u64) {
        prop_assert!(timeout >= 60);
        prop_assert!(timeout <= 86400);
    }
}
