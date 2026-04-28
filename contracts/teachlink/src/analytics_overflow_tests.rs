#[cfg(test)]
mod tests {
    use super::super::escrow_analytics::EscrowAnalyticsManager;
    use super::super::notification::NotificationManager;
    use super::super::notification_types::{NotificationChannel, NotificationContent};
    use super::super::storage::{ESCROW_ANALYTICS, NOTIFICATION_COUNTER, NOTIFICATION_LOGS, NOTIFICATION_TRACKING, SCHEDULED_NOTIFICATIONS};
    use super::super::types::EscrowMetrics;
    use soroban_sdk::testutils::Ledger;
    use soroban_sdk::{Address, Bytes, Env, Map};

    fn setup_env() -> Env {
        let env = Env::default();
        env.mock_all_auths();
        env
    }

    #[test]
    fn escrow_overflow_triggers_resets_and_can_be_cleared() {
        let env = setup_env();

        let metrics = EscrowMetrics {
            total_escrows: u64::MAX,
            total_volume: i128::MAX,
            total_disputes: u64::MAX,
            total_resolved: u64::MAX,
            average_resolution_time: 123,
            resets: 0,
        };

        env.storage().instance().set(&ESCROW_ANALYTICS, &metrics);

        // This should overflow both the u64 counter and the i128 volume
        EscrowAnalyticsManager::update_creation(&env, 1);

        let m = EscrowAnalyticsManager::get_metrics(&env);
        assert_eq!(m.total_escrows, 0);
        assert_eq!(m.total_volume, 0);
        assert!(m.resets >= 1);

        // Reset via admin (mocked auth)
        let admin = Address::random(&env);
        EscrowAnalyticsManager::reset_metrics(&env, admin.clone());

        let m2 = EscrowAnalyticsManager::get_metrics(&env);
        assert_eq!(m2.total_escrows, 0);
        assert_eq!(m2.total_volume, 0);
        assert_eq!(m2.resets, 0);
    }

    #[test]
    fn notification_id_overflow_and_reset_behaviour() {
        let env = setup_env();

        // set counter to max
        env.storage().instance().set(&NOTIFICATION_COUNTER, &u64::MAX);

        let recipient = Address::random(&env);
        let content = NotificationContent {
            subject: Bytes::from_slice(&env, b"t"),
            body: Bytes::from_slice(&env, b"b"),
            data: Bytes::from_slice(&env, b"{}"),
            localization: Map::new(&env),
        };

        let id = NotificationManager::send_notification(&env, recipient.clone(), NotificationChannel::InApp, content.clone()).unwrap();
        // after overflow, implementation returns id 1 and stores 1
        assert_eq!(id, 1u64);

        let stored_counter: u64 = env.storage().instance().get(&NOTIFICATION_COUNTER).unwrap_or(0u64);
        assert_eq!(stored_counter, 1u64);

        // Now test reset_counters admin API
        let admin = Address::random(&env);
        NotificationManager::reset_counters(&env, admin.clone()).unwrap();

        let c: u64 = env.storage().instance().get(&NOTIFICATION_COUNTER).unwrap_or(0u64);
        assert_eq!(c, 0u64);

        let logs: Map<u64, NotificationContent> = env.storage().instance().get(&NOTIFICATION_LOGS).unwrap_or_else(|| Map::new(&env));
        assert_eq!(logs.len(), 0);

        let tr: Map<u64, super::super::notification_types::NotificationTracking> = env.storage().instance().get(&NOTIFICATION_TRACKING).unwrap_or_else(|| Map::new(&env));
        assert_eq!(tr.len(), 0);

        let sch: Map<u64, super::super::notification_types::NotificationSchedule> = env.storage().instance().get(&SCHEDULED_NOTIFICATIONS).unwrap_or_else(|| Map::new(&env));
        assert_eq!(sch.len(), 0);
    }
}
