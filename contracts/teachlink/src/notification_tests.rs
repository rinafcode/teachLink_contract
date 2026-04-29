//! Notification System Tests
//!
//! This module contains comprehensive tests for the notification system.

#[cfg(test)]
pub mod notification_tests {
        #[test]
        fn test_send_notification_rate_limit() {
            let env = Env::default();
            let recipient = create_test_address(&env, 2);
            let admin = create_test_address(&env, 1);

            NotificationManager::initialize(&env).unwrap();
            let settings = UserNotificationSettings {
                user: recipient.clone(),
                timezone: Bytes::from_slice(&env, b"UTC"),
                quiet_hours_start: 22 * 3600,
                quiet_hours_end: 8 * 3600,
                max_daily_notifications: 50,
                do_not_disturb: false,
            };
            NotificationManager::update_user_settings(&env, recipient.clone(), settings).unwrap();

            // Set rate limit: 2 notifications per 100 ledgers
            NotificationManager::set_notification_rate_limit(
                &env,
                admin.clone(),
                2,
                100,
                soroban_sdk::Symbol::short("notif_send"),
            ).unwrap();

            let content = NotificationContent {
                subject: Bytes::from_slice(&env, b"RL Test"),
                body: Bytes::from_slice(&env, b"Body"),
                data: Bytes::new(&env),
                localization: Map::new(&env),
            };

            // First two should succeed
            assert!(NotificationManager::send_notification(&env, recipient.clone(), NotificationChannel::InApp, content.clone()).is_ok());
            assert!(NotificationManager::send_notification(&env, recipient.clone(), NotificationChannel::InApp, content.clone()).is_ok());
            // Third should fail with rate limit error
            let result = NotificationManager::send_notification(&env, recipient.clone(), NotificationChannel::InApp, content.clone());
            assert!(result.is_err());
        }

        #[test]
        fn test_schedule_notification_rate_limit() {
            let env = Env::default();
            let recipient = create_test_address(&env, 2);
            let admin = create_test_address(&env, 1);
            let current_time = env.ledger().timestamp();
            let future_time = current_time + 3600;

            NotificationManager::initialize(&env).unwrap();
            let settings = UserNotificationSettings {
                user: recipient.clone(),
                timezone: Bytes::from_slice(&env, b"UTC"),
                quiet_hours_start: 22 * 3600,
                quiet_hours_end: 8 * 3600,
                max_daily_notifications: 50,
                do_not_disturb: false,
            };
            NotificationManager::update_user_settings(&env, recipient.clone(), settings).unwrap();

            // Set rate limit: 1 scheduled notification per 100 ledgers
            NotificationManager::set_notification_rate_limit(
                &env,
                admin.clone(),
                1,
                100,
                soroban_sdk::Symbol::short("notif_schedule"),
            ).unwrap();

            let content = NotificationContent {
                subject: Bytes::from_slice(&env, b"RL Sched Test"),
                body: Bytes::from_slice(&env, b"Body"),
                data: Bytes::new(&env),
                localization: Map::new(&env),
            };
            let schedule = NotificationSchedule {
                notification_id: 0,
                recipient: recipient.clone(),
                channel: NotificationChannel::Email,
                scheduled_time: future_time,
                timezone: Bytes::from_slice(&env, b"UTC"),
                is_recurring: false,
                recurrence_pattern: 0,
                max_deliveries: None,
                delivery_count: 0,
            };

            // First should succeed
            assert!(NotificationManager::schedule_notification(&env, recipient.clone(), NotificationChannel::Email, content.clone(), schedule.clone()).is_ok());
            // Second should fail
            let result = NotificationManager::schedule_notification(&env, recipient.clone(), NotificationChannel::Email, content.clone(), schedule.clone());
            assert!(result.is_err());
        }
    use crate::notification::*;
    use crate::notification_types::*;
    use crate::storage::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env, Map, String, Vec};

    // Helper function to create test addresses
    fn create_test_address(env: &Env, id: u8) -> Address {
        // Use Address::generate for test addresses
        Address::generate(&env)
    }

    #[test]
    fn test_notification_initialization() {
        let env = Env::default();
        let admin = create_test_address(&env, 1);

        // Test initialization
        let result = NotificationManager::initialize(&env);
        assert!(result.is_ok());

        // Verify counter is set
        let counter: u64 = env.storage().instance().get(&NOTIFICATION_COUNTER).unwrap();
        assert_eq!(counter, 0);

        // Verify default templates are created
        let templates: Map<u64, NotificationTemplate> = env
            .storage()
            .instance()
            .get(&NOTIFICATION_TEMPLATES)
            .unwrap();
        assert!(templates.len() >= 2); // Welcome and transaction templates
    }

    #[test]
    fn test_send_immediate_notification() {
        let env = Env::default();
        let recipient = create_test_address(&env, 2);
        let admin = create_test_address(&env, 1);

        // Initialize system
        NotificationManager::initialize(&env).unwrap();

        // Set up user settings
        let settings = UserNotificationSettings {
            user: recipient.clone(),
            timezone: Bytes::from_slice(&env, b"UTC"),
            quiet_hours_start: 22 * 3600,
            quiet_hours_end: 8 * 3600,
            max_daily_notifications: 50,
            do_not_disturb: false,
        };
        NotificationManager::update_user_settings(&env, recipient.clone(), settings).unwrap();

        // Send notification
        let content = NotificationContent {
            subject: Bytes::from_slice(&env, b"Test Subject"),
            body: Bytes::from_slice(&env, b"Test Body"),
            data: Bytes::new(&env),
            localization: Map::new(&env),
        };

        let result = NotificationManager::send_notification(
            &env,
            recipient.clone(),
            NotificationChannel::InApp,
            content.clone(),
        );

        assert!(result.is_ok());
        let notification_id = result.unwrap();

        // Verify tracking
        let tracking = NotificationManager::get_notification_tracking(&env, notification_id);
        assert!(tracking.is_some());
        let tracking = tracking.unwrap();
        assert_eq!(tracking.recipient, recipient);
        assert_eq!(tracking.channel, NotificationChannel::InApp);
        assert!(matches!(
            tracking.status,
            NotificationDeliveryStatus::Delivered | NotificationDeliveryStatus::Failed
        ));
    }

    #[test]
    fn test_schedule_notification() {
        let env = Env::default();
        let recipient = create_test_address(&env, 2);
        let current_time = env.ledger().timestamp();
        let future_time = current_time + 3600; // 1 hour from now

        // Initialize system
        NotificationManager::initialize(&env).unwrap();

        // Set up user settings
        let settings = UserNotificationSettings {
            user: recipient.clone(),
            timezone: Bytes::from_slice(&env, b"UTC"),
            quiet_hours_start: 22 * 3600,
            quiet_hours_end: 8 * 3600,
            max_daily_notifications: 50,
            do_not_disturb: false,
        };
        NotificationManager::update_user_settings(&env, recipient.clone(), settings).unwrap();

        // Schedule notification
        let content = NotificationContent {
            subject: Bytes::from_slice(&env, b"Scheduled Test"),
            body: Bytes::from_slice(&env, b"This is a scheduled notification"),
            data: Bytes::new(&env),
            localization: Map::new(&env),
        };

        let schedule = NotificationSchedule {
            notification_id: 0,
            recipient: recipient.clone(),
            channel: NotificationChannel::Email,
            scheduled_time: future_time,
            timezone: Bytes::from_slice(&env, b"UTC"),
            is_recurring: false,
            recurrence_pattern: 0,
            max_deliveries: None,
            delivery_count: 0,
        };

        let result = NotificationManager::schedule_notification(
            &env,
            recipient.clone(),
            NotificationChannel::Email,
            content,
            schedule,
        );

        assert!(result.is_ok());
        let notification_id = result.unwrap();

        // Verify tracking shows scheduled status
        let tracking = NotificationManager::get_notification_tracking(&env, notification_id);
        assert!(tracking.is_some());
        let tracking = tracking.unwrap();
        assert_eq!(tracking.status, NotificationDeliveryStatus::Scheduled);
    }

    #[test]
    fn test_process_scheduled_notifications() {
        let env = Env::default();
        let recipient = create_test_address(&env, 2);
        let current_time = env.ledger().timestamp();
        let past_time = current_time - 100; // Schedule in the past for immediate processing

        // Initialize system
        NotificationManager::initialize(&env).unwrap();

        // Set up user settings
        let settings = UserNotificationSettings {
            user: recipient.clone(),
            timezone: Bytes::from_slice(&env, b"UTC"),
            quiet_hours_start: 22 * 3600,
            quiet_hours_end: 8 * 3600,
            max_daily_notifications: 50,
            do_not_disturb: false,
        };
        NotificationManager::update_user_settings(&env, recipient.clone(), settings).unwrap();

        // Schedule notification in the past
        let content = NotificationContent {
            subject: Bytes::from_slice(&env, b"Past Scheduled"),
            body: Bytes::from_slice(&env, b"This should be processed immediately"),
            data: Bytes::new(&env),
            localization: Map::new(&env),
        };

        let schedule = NotificationSchedule {
            notification_id: 0,
            recipient: recipient.clone(),
            channel: NotificationChannel::InApp,
            scheduled_time: past_time,
            timezone: Bytes::from_slice(&env, b"UTC"),
            is_recurring: false,
            recurrence_pattern: 0,
            max_deliveries: None,
            delivery_count: 0,
        };

        NotificationManager::schedule_notification(
            &env,
            recipient.clone(),
            NotificationChannel::InApp,
            content,
            schedule,
        )
        .unwrap();

        // Process scheduled notifications
        let processed_count = NotificationManager::process_scheduled_notifications(&env).unwrap();
        assert!(processed_count > 0);
    }

    #[test]
    fn test_update_preferences() {
        let env = Env::default();
        let user = create_test_address(&env, 3);

        // Initialize system
        NotificationManager::initialize(&env).unwrap();

        // Create preferences
        let mut preferences = Vec::new(&env);
        preferences.push_back(NotificationPreference {
            channel: NotificationChannel::Email,
            enabled: true,
            frequency_hours: 24,
            quiet_hours_only: false,
            urgent_only: false,
        });
        preferences.push_back(NotificationPreference {
            channel: NotificationChannel::SMS,
            enabled: false,
            frequency_hours: 1,
            quiet_hours_only: true,
            urgent_only: true,
        });

        // Update preferences
        let result =
            NotificationManager::update_preferences(&env, user.clone(), preferences.clone());
        assert!(result.is_ok());

        // Verify preferences were stored (would need to add a getter method to fully test)
    }

    #[test]
    fn test_create_template() {
        let env = Env::default();
        let admin = create_test_address(&env, 1);

        // Initialize system
        NotificationManager::initialize(&env).unwrap();

        // Create template
        let name = Bytes::from_slice(&env, b"Test Template");
        let mut channels = Vec::new(&env);
        channels.push_back(NotificationChannel::Email);
        channels.push_back(NotificationChannel::InApp);

        let content = NotificationContent {
            subject: Bytes::from_slice(&env, b"Template Subject"),
            body: Bytes::from_slice(&env, b"Template body with {{variable}}"),
            data: Bytes::new(&env),
            localization: Map::new(&env),
        };

        let result = NotificationManager::create_template(&env, admin, name, channels, content);
        assert!(result.is_ok());
        let template_id = result.unwrap();
        assert!(template_id > 0);
    }

    #[test]
    fn test_send_template_notification() {
        let env = Env::default();
        let admin = create_test_address(&env, 1);
        let recipient = create_test_address(&env, 2);

        // Initialize system
        NotificationManager::initialize(&env).unwrap();

        // Set up user settings
        let settings = UserNotificationSettings {
            user: recipient.clone(),
            timezone: Bytes::from_slice(&env, b"UTC"),
            quiet_hours_start: 22 * 3600,
            quiet_hours_end: 8 * 3600,
            max_daily_notifications: 50,
            do_not_disturb: false,
        };
        NotificationManager::update_user_settings(&env, recipient.clone(), settings).unwrap();

        // Create template
        let name = Bytes::from_slice(&env, b"Test Template");
        let mut channels = Vec::new(&env);
        channels.push_back(NotificationChannel::Email);
        channels.push_back(NotificationChannel::InApp);

        let content = NotificationContent {
            subject: Bytes::from_slice(&env, b"Template Subject"),
            body: Bytes::from_slice(&env, b"Template body with {{variable}}"),
            data: Bytes::new(&env),
            localization: Map::new(&env),
        };

        let template_id = NotificationManager::create_template(&env, admin, name, channels, content).unwrap();

        // Send template notification
        let result = NotificationManager::send_template_notification(
            &env,
            recipient.clone(),
            NotificationChannel::Email,
            template_id,
            Map::new(&env),
        );
        assert!(result.is_ok());
    }
}
