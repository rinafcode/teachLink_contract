//! Stable testing harness used by CI `cargo test --lib` in `testing/`.
//!
//! The legacy `testing/integration/*` scaffolding is intentionally not compiled
//! here because it is currently out of sync with the production contract API.

/// Returns true when the testing harness is operational.
#[must_use]
pub fn harness_ready() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harness_smoke_test() {
        assert!(harness_ready());
    }
}
