#[cfg(test)]
mod tests {
    use crate::validation::{TimeValidator, ValidationError, config};
    use soroban_sdk::{Env, testutils::Ledger};

    fn set_ledger_time(env: &Env, timestamp: u64) {
        env.ledger().with_mut(|li| {
            li.timestamp = timestamp;
        });
    }

    #[test]
    fn test_global_bounds_pass() {
        let env = Env::default();
        let now = 1_000_000;
        set_ledger_time(&env, now);
        
        assert!(TimeValidator::validate_global_bounds(&env, now).is_ok());
        assert!(TimeValidator::validate_global_bounds(&env, now + 3600).is_ok());
        assert!(TimeValidator::validate_global_bounds(&env, now - 3600).is_ok());
    }

    #[test]
    fn test_global_bounds_fail_future() {
        let env = Env::default();
        let now = 1_000_000;
        set_ledger_time(&env, now);
        
        let way_future = now + config::MAX_TIMEOUT_SECONDS + 1;
        assert_eq!(
            TimeValidator::validate_global_bounds(&env, way_future),
            Err(ValidationError::InvalidTimestamp)
        );
    }

    #[test]
    fn test_global_bounds_fail_past() {
        let env = Env::default();
        let now = config::MAX_TIMEOUT_SECONDS + 1_000_000;
        set_ledger_time(&env, now);
        
        let way_past = 0;
        assert_eq!(
            TimeValidator::validate_global_bounds(&env, way_past),
            Err(ValidationError::InvalidTimestamp)
        );
    }

    #[test]
    fn test_operational_bounds() {
        let env = Env::default();
        let now = 10_000_000;
        set_ledger_time(&env, now);
        
        // 90 days = 7,776,000 seconds
        assert!(TimeValidator::validate_operational_bounds(&env, now + 7_000_000).is_ok());
        
        assert_eq!(
            TimeValidator::validate_operational_bounds(&env, now + 8_000_000),
            Err(ValidationError::InvalidTimestamp)
        );
    }

    #[test]
    fn test_monotonicity() {
        assert!(TimeValidator::check_monotonic(100, 101).is_ok());
        assert!(TimeValidator::check_monotonic(100, 100).is_ok());
        assert_eq!(
            TimeValidator::check_monotonic(101, 100),
            Err(ValidationError::TimestampNotMonotonic)
        );
    }

    #[test]
    fn test_skew_tolerance() {
        let env = Env::default();
        let now = 1_000_000;
        set_ledger_time(&env, now);
        
        // 15 minutes = 900 seconds
        assert!(TimeValidator::validate_skew(&env, now + 800).is_ok());
        assert!(TimeValidator::validate_skew(&env, now - 800).is_ok());
        
        assert_eq!(
            TimeValidator::validate_skew(&env, now + 1000),
            Err(ValidationError::TimestampSkewExceeded)
        );
    }
}
