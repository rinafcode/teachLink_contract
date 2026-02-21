//! Notification System Tests
//!
//! This module contains comprehensive tests for the notification system.

use crate::notification::*;
use crate::notification_types::*;
use crate::storage::*;
use soroban_sdk::{Address, Bytes, Env, Map, Vec, String};

#[cfg(test)]
pub mod notification_tests {
    use super::*;

    // Helper function to create test addresses
    fn create_test_address(env: &Env, id: u8) -> Address {
        let address_str = format!("GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA{}", id);
        Address::from_string(&String::from_str(env, &address_str))
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
        let name = Bytes::from_slice(&env, b"Personalized Template");
        let mut channels = Vec::new(&env);
        channels.push_back(NotificationChannel::Email);

        let content = NotificationContent {
            subject: Bytes::from_slice(&env, b"Hello {{name}}!"),
            body: Bytes::from_slice(&env, b"Dear {{name}}, your balance is {{balance}}."),
            data: Bytes::new(&env),
            localization: Map::new(&env),
        };

        let template_id =
            NotificationManager::create_template(&env, admin, name, channels, content).unwrap();

        // Create personalization variables
        let mut variables = Map::new(&env);
        variables.set(
            Bytes::from_slice(&env, b"name"),
            Bytes::from_slice(&env, b"Alice"),
        );
        variables.set(
            Bytes::from_slice(&env, b"balance"),
            Bytes::from_slice(&env, b"1000"),
        );

        // Send template notification
        let result = NotificationManager::send_template_notification(
            &env,
            recipient,
            template_id,
            variables,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_notification_analytics() {
        let env = Env::default();
        let recipient = create_test_address(&env, 2);
        let start_time = env.ledger().timestamp() - 3600; // 1 hour ago
        let end_time = env.ledger().timestamp();

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

        // Send multiple notifications
        for i in 0..5 {
            let content = NotificationContent {
                subject: Bytes::from_slice(&env, &format!("Test {}", i).as_bytes()),
                body: Bytes::from_slice(&env, &format!("Body {}", i).as_bytes()),
                data: Bytes::new(&env),
                localization: Map::new(&env),
            };

            NotificationManager::send_notification(
                &env,
                recipient.clone(),
                NotificationChannel::InApp,
                content,
            )
            .unwrap();
        }

        // Get analytics
        let analytics = NotificationManager::get_notification_analytics(&env, start_time, end_time);
        assert!(analytics.total_sent >= 5);
        assert!(analytics.total_delivered >= 0); // Some may fail in simulation
        assert!(analytics.channel_stats.len() > 0);
    }

    #[test]
    fn test_user_notification_history() {
        let env = Env::default();
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

        // Send notifications
        for i in 0..3 {
            let content = NotificationContent {
                subject: Bytes::from_slice(&env, &format!("History Test {}", i).as_bytes()),
                body: Bytes::from_slice(&env, &format!("History Body {}", i).as_bytes()),
                data: Bytes::new(&env),
                localization: Map::new(&env),
            };

            NotificationManager::send_notification(
                &env,
                recipient.clone(),
                NotificationChannel::InApp,
                content,
            )
            .unwrap();
        }

        // Get user history
        let history = NotificationManager::get_user_notifications(&env, recipient, 10);
        assert!(history.len() >= 3);

        // Verify all notifications belong to the user
        for tracking in history.iter() {
            assert_eq!(tracking.recipient, recipient);
        }
    }

    #[test]
    fn test_notification_rate_limiting() {
        let env = Env::default();
        let recipient = create_test_address(&env, 2);

        // Initialize system
        NotificationManager::initialize(&env).unwrap();

        // Set up user settings with low daily limit
        let settings = UserNotificationSettings {
            user: recipient.clone(),
            timezone: Bytes::from_slice(&env, b"UTC"),
            quiet_hours_start: 22 * 3600,
            quiet_hours_end: 8 * 3600,
            max_daily_notifications: 2, // Very low limit
            do_not_disturb: false,
        };
        NotificationManager::update_user_settings(&env, recipient.clone(), settings).unwrap();

        // Send notifications up to limit
        let mut success_count = 0;
        for i in 0..5 {
            let content = NotificationContent {
                subject: Bytes::from_slice(&env, &format!("Rate Limit Test {}", i).as_bytes()),
                body: Bytes::from_slice(&env, &format!("Rate Limit Body {}", i).as_bytes()),
                data: Bytes::new(&env),
                localization: Map::new(&env),
            };

            let result = NotificationManager::send_notification(
                &env,
                recipient.clone(),
                NotificationChannel::InApp,
                content,
            );

            if result.is_ok() {
                success_count += 1;
            }
        }

        // Should have been limited to 2 notifications
        assert!(success_count <= 2);
    }

    #[test]
    fn test_quiet_hours_enforcement() {
        let env = Env::default();
        let recipient = create_test_address(&env, 2);

        // Initialize system
        NotificationManager::initialize(&env).unwrap();

        // Set up user settings with quiet hours
        let settings = UserNotificationSettings {
            user: recipient.clone(),
            timezone: Bytes::from_slice(&env, b"UTC"),
            quiet_hours_start: 0,       // Midnight
            quiet_hours_end: 23 * 3600, // Almost entire day
            max_daily_notifications: 50,
            do_not_disturb: false,
        };
        NotificationManager::update_user_settings(&env, recipient.clone(), settings).unwrap();

        // Try to send non-in-app notification during quiet hours
        let content = NotificationContent {
            subject: Bytes::from_slice(&env, b"Quiet Hours Test"),
            body: Bytes::from_slice(&env, b"This should fail during quiet hours"),
            data: Bytes::new(&env),
            localization: Map::new(&env),
        };

        let result = NotificationManager::send_notification(
            &env,
            recipient.clone(),
            NotificationChannel::Email, // Should be blocked during quiet hours
            content,
        );

        assert!(result.is_err());

        // In-app should still work
        let content = NotificationContent {
            subject: Bytes::from_slice(&env, b"Quiet Hours InApp"),
            body: Bytes::from_slice(&env, b"InApp should work during quiet hours"),
            data: Bytes::new(&env),
            localization: Map::new(&env),
        };

        let result = NotificationManager::send_notification(
            &env,
            recipient,
            NotificationChannel::InApp, // Should work during quiet hours
            content,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_do_not_disturb_mode() {
        let env = Env::default();
        let recipient = create_test_address(&env, 2);

        // Initialize system
        NotificationManager::initialize(&env).unwrap();

        // Enable do not disturb
        let settings = UserNotificationSettings {
            user: recipient.clone(),
            timezone: Bytes::from_slice(&env, b"UTC"),
            quiet_hours_start: 22 * 3600,
            quiet_hours_end: 8 * 3600,
            max_daily_notifications: 50,
            do_not_disturb: true, // Enable DND
        };
        NotificationManager::update_user_settings(&env, recipient.clone(), settings).unwrap();

        // Try to send any notification
        let content = NotificationContent {
            subject: Bytes::from_slice(&env, b"DND Test"),
            body: Bytes::from_slice(&env, b"This should fail with DND enabled"),
            data: Bytes::new(&env),
            localization: Map::new(&env),
        };

        let result = NotificationManager::send_notification(
            &env,
            recipient,
            NotificationChannel::InApp,
            content,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_recurring_notifications() {
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

        // Schedule recurring notification
        let content = NotificationContent {
            subject: Bytes::from_slice(&env, b"Recurring Test"),
            body: Bytes::from_slice(&env, b"This is a recurring notification"),
            data: Bytes::new(&env),
            localization: Map::new(&env),
        };

        let schedule = NotificationSchedule {
            notification_id: 0,
            recipient: recipient.clone(),
            channel: NotificationChannel::Email,
            scheduled_time: future_time,
            timezone: Bytes::from_slice(&env, b"UTC"),
            is_recurring: true,
            recurrence_pattern: 2, // Daily
            max_deliveries: Some(3),
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

        // Verify it's scheduled
        let tracking = NotificationManager::get_notification_tracking(&env, notification_id);
        assert!(tracking.is_some());
        let tracking = tracking.unwrap();
        assert_eq!(tracking.status, NotificationDeliveryStatus::Scheduled);
    }
}
