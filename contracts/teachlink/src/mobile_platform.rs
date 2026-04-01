//! Mobile-First Learning Platform
//!
//! This module implements mobile-optimized features including offline capabilities,
//! push notifications, and mobile-specific engagement features.

use crate::errors::MobilePlatformError;
use crate::types::*;
use soroban_sdk::{
    contracttype, panic_with_error, symbol_short, Address, Bytes, Env, Map, Symbol, Vec,
};

const MOBILE_PROFILE: Symbol = symbol_short!("mob_prof");
const OFFLINE_CONTENT: Symbol = symbol_short!("off_cont");
const PUSH_NOTIFICATIONS: Symbol = symbol_short!("push_not");
const MOBILE_ANALYTICS: Symbol = symbol_short!("mob_anal");
const MOBILE_PAYMENTS: Symbol = symbol_short!("mob_pay");
const MOBILE_SECURITY: Symbol = symbol_short!("mob_sec");
const MOBILE_OPTIMIZATION: Symbol = symbol_short!("mob_opt");
const MOBILE_COMMUNITY: Symbol = symbol_short!("mob_comm");

const ONBOARDING_STATUS: Symbol = symbol_short!("onboard");
const USER_FEEDBACK: Symbol = symbol_short!("feedback");
const UX_EXPERIMENTS: Symbol = symbol_short!("ux_exp");
const COMPONENT_CONFIG: Symbol = symbol_short!("comp_cfg");

// ========== Mobile Profile Types ==========

// ========== Offline Capabilities ==========

// Types are now imported from crate::types

// ========== Advanced UI/UX Types ==========

// ========== Errors ==========

// ========== Main Implementation ==========

pub struct MobilePlatformManager;

impl MobilePlatformManager {
    /// Initialize mobile profile for user
    pub fn initialize_mobile_profile(
        env: &Env,
        user: Address,
        device_info: DeviceInfo,
        preferences: MobilePreferences,
    ) -> Result<(), MobilePlatformError> {
        user.require_auth();

        let mobile_profile = MobileProfile {
            user: user.clone(),
            device_info,
            preferences,
            offline_settings: OfflineSettings {
                auto_download_enabled: true,
                download_quality: OfflineQuality::StandardQuality,
                storage_limit: 1024 * 1024 * 1024, // 1GB
                sync_strategy: SyncStrategy::WiFiOnly,
                offline_duration: 24 * 7, // 1 week
                priority_content: Vec::new(env),
                compression_enabled: true,
            },
            notification_preferences: NotificationPreferences {
                learning_reminders: true,
                deadline_alerts: true,
                achievement_notifications: true,
                social_updates: false,
                content_updates: true,
                quiet_hours: TimeRange {
                    start_hour: 22,
                    end_hour: 8,
                    timezone: Bytes::from_slice(env, b"UTC"),
                },
                frequency_limit: 10,
                sound_enabled: true,
                vibration_enabled: true,
                led_enabled: true,
            },
            security_settings: MobileSecuritySettings {
                biometric_enabled: false,
                biometric_type: BiometricType::Fingerprint,
                pin_required: true,
                two_factor_enabled: false,
                session_timeout: 30,
                encryption_enabled: true,
                remote_wipe_enabled: false,
                trusted_devices: Vec::new(env),
                login_attempts: 0,
                max_login_attempts: 5,
            },
            payment_methods: Vec::new(env),
            accessibility_settings: MobileAccessibilitySettings {
                screen_reader_enabled: false,
                high_contrast_enabled: false,
                large_text_enabled: false,
                voice_control_enabled: false,
                gesture_navigation_enabled: false,
                haptic_feedback_enabled: true,
                color_blind_mode: ColorBlindMode::None,
                reduced_motion_enabled: false,
                focus_indicator_style: FocusStyle::Default,
            },
            last_sync: env.ledger().timestamp(),
            data_usage: DataUsageTracking {
                total_downloaded: 0,
                total_uploaded: 0,
                cached_data: 0,
                streaming_data: 0,
                last_reset: env.ledger().timestamp(),
                daily_limit: 100 * 1024 * 1024, // 100MB
                warning_threshold: 8000,        // 80% (basis points)
            },
        };

        Self::set_mobile_profile(env, &user, &mobile_profile);

        Ok(())
    }

    /// Download content for offline access
    pub fn download_offline_content(
        env: &Env,
        user: Address,
        content_id: u64,
        quality: OfflineQuality,
    ) -> Result<(), MobilePlatformError> {
        user.require_auth();

        let mut profile = Self::get_mobile_profile(env, &user);

        // Check storage availability
        if profile.data_usage.total_downloaded > profile.offline_settings.storage_limit {
            return Err(MobilePlatformError::InsufficientStorage);
        }

        let offline_content = OfflineContent {
            content_id,
            content_type: OfflineContentType::VideoLesson, // Would determine from content
            local_path: Bytes::from_slice(env, b"/offline/content/"),
            file_size: Self::estimate_content_size(content_id, quality.clone()),
            compressed_size: Self::estimate_compressed_size(content_id, quality),
            download_date: env.ledger().timestamp(),
            expiry_date: env.ledger().timestamp()
                + profile.offline_settings.offline_duration * 3600,
            is_available: true,
            version: 1,
            dependencies: Vec::new(env),
        };

        Self::add_offline_content(env, &user, &offline_content);

        // Update data usage
        profile.data_usage.total_downloaded += offline_content.file_size;
        profile.last_sync = env.ledger().timestamp();
        Self::set_mobile_profile(env, &user, &profile);

        Ok(())
    }

    /// Send push notification
    pub fn send_push_notification(
        env: &Env,
        user: Address,
        notification_type: NotificationType,
        title: Bytes,
        message: Bytes,
        priority: NotificationPriority,
    ) -> Result<u64, MobilePlatformError> {
        let notification_id = env.ledger().sequence() as u64;
        let notification = PushNotification {
            id: notification_id,
            user: user.clone(),
            notification_type,
            title,
            message,
            data: Map::new(env),
            priority,
            scheduled_time: env.ledger().timestamp(),
            expiry_time: env.ledger().timestamp() + 24 * 3600, // 24 hours
            is_read: false,
            action_buttons: Vec::new(env),
        };

        Self::add_push_notification(env, &user, &notification);

        Ok(notification_id)
    }

    /// Process mobile payment
    pub fn process_mobile_payment(
        env: &Env,
        user: Address,
        payment_method_id: u64,
        amount: u64,
        description: Bytes,
    ) -> Result<u64, MobilePlatformError> {
        user.require_auth();

        let profile = Self::get_mobile_profile(env, &user);
        let payment_method = Self::get_payment_method(&profile, payment_method_id);

        if payment_method.is_none() {
            return Err(MobilePlatformError::PaymentFailed);
        }

        let transaction_id = env.ledger().sequence() as u64;
        let transaction = MobileTransaction {
            id: transaction_id,
            user: user.clone(),
            payment_method_id,
            amount,
            currency: Bytes::from_slice(env, b"USD"),
            description,
            merchant: Bytes::from_slice(env, b"TeachLink"),
            status: TransactionStatus::Pending,
            timestamp: env.ledger().timestamp(),
            confirmation_code: Self::generate_confirmation_code(env),
            fraud_score: Self::calculate_fraud_score(&user, amount),
        };

        Self::add_mobile_transaction(env, &transaction);

        Ok(transaction_id)
    }

    /// Record security event
    pub fn record_security_event(
        env: &Env,
        user: Address,
        event_type: SecurityEventType,
        device_id: Bytes,
        location: Option<MobileLocationData>,
        severity: SecuritySeverity,
    ) -> Result<(), MobilePlatformError> {
        let (has_location, lat, lon, accuracy, loc_ts) = match location {
            Some(loc) => (
                true,
                loc.latitude,
                loc.longitude,
                loc.accuracy,
                loc.timestamp,
            ),
            None => (false, 0i64, 0i64, 0u64, 0u64),
        };
        let security_event = SecurityEvent {
            id: env.ledger().sequence() as u64,
            user: user.clone(),
            event_type,
            device_id,
            has_location,
            location_lat: lat,
            location_lon: lon,
            location_accuracy: accuracy,
            location_ts: loc_ts,
            timestamp: env.ledger().timestamp(),
            severity,
            resolved: false,
        };

        Self::add_security_event(env, &user, &security_event);

        Ok(())
    }

    /// Update accessibility settings
    pub fn update_accessibility_settings(
        env: &Env,
        user: Address,
        settings: MobileAccessibilitySettings,
    ) -> Result<(), MobilePlatformError> {
        user.require_auth();
        let mut profile = Self::get_mobile_profile(env, &user);
        profile.accessibility_settings = settings;
        Self::set_mobile_profile(env, &user, &profile);
        Ok(())
    }

    /// Update personalization settings
    pub fn update_personalization(
        env: &Env,
        user: Address,
        preferences: MobilePreferences,
    ) -> Result<(), MobilePlatformError> {
        user.require_auth();
        let mut profile = Self::get_mobile_profile(env, &user);
        profile.preferences = preferences;
        Self::set_mobile_profile(env, &user, &profile);
        Ok(())
    }

    /// Record onboarding progress
    pub fn record_onboarding_progress(
        env: &Env,
        user: Address,
        stage: OnboardingStage,
    ) -> Result<(), MobilePlatformError> {
        user.require_auth();
        let mut status = Self::get_onboarding_status(env, &user).unwrap_or(OnboardingStatus {
            user: user.clone(),
            completed_stages: Vec::new(env),
            current_stage: OnboardingStage::ProfileSetup,
            last_updated: env.ledger().timestamp(),
            skipped: false,
        });

        if !status.completed_stages.contains(&stage) {
            status.completed_stages.push_back(stage.clone());
        }
        status.current_stage = stage;
        status.last_updated = env.ledger().timestamp();

        Self::set_onboarding_status(env, &user, &status);
        Ok(())
    }

    /// Submit user feedback
    pub fn submit_user_feedback(
        env: &Env,
        user: Address,
        rating: u32,
        comment: Bytes,
        category: FeedbackCategory,
    ) -> Result<u64, MobilePlatformError> {
        user.require_auth();
        let feedback_id = env.ledger().sequence() as u64;
        let feedback = UserFeedback {
            id: feedback_id,
            user,
            rating,
            comment,
            timestamp: env.ledger().timestamp(),
            category,
        };

        Self::add_user_feedback(env, &feedback);
        Ok(feedback_id)
    }

    /// Get userAllocated experiment variants
    pub fn get_user_experiment_variants(env: &Env, user: Address) -> Map<u64, Symbol> {
        let mut results = Map::new(env);
        let experiments = Self::get_active_experiments(env);

        for exp in experiments.iter() {
            if let Some(variant) = exp.variant_allocations.get(user.clone()) {
                results.set(exp.experiment_id, variant);
            }
        }
        results
    }

    /// Get design system configuration
    pub fn get_design_system_config(env: &Env) -> ComponentConfig {
        env.storage()
            .persistent()
            .get(&COMPONENT_CONFIG)
            .unwrap_or(ComponentConfig {
                spacing_unit: 8,
                border_radius_base: 4,
                transition_duration_base: 200,
                elevation_steps: Vec::new(env),
                typography_scale: Map::new(env),
            })
    }

    /// Set design system configuration (admin only)
    pub fn set_design_system_config(env: &Env, config: ComponentConfig) {
        env.storage().persistent().set(&COMPONENT_CONFIG, &config);
    }

    // ========== Helper Functions ==========

    fn estimate_content_size(content_id: u64, quality: OfflineQuality) -> u64 {
        // Simulated size estimation
        match quality {
            OfflineQuality::TextOnly => 1024 * 100,               // 100KB
            OfflineQuality::LowQuality => 1024 * 1024 * 50,       // 50MB
            OfflineQuality::StandardQuality => 1024 * 1024 * 200, // 200MB
            OfflineQuality::HighQuality => 1024 * 1024 * 500,     // 500MB
        }
    }

    fn estimate_compressed_size(content_id: u64, quality: OfflineQuality) -> u64 {
        let original_size = Self::estimate_content_size(content_id, quality);
        (original_size * 70) / 100 // 30% compression
    }

    fn get_payment_method(
        profile: &MobileProfile,
        payment_method_id: u64,
    ) -> Option<MobilePaymentMethod> {
        profile
            .payment_methods
            .iter()
            .find(|method| method.id == payment_method_id)
    }

    fn generate_confirmation_code(env: &Env) -> Bytes {
        let code = env.ledger().sequence() % 1000000;
        Bytes::from_slice(env, &code.to_be_bytes())
    }

    fn calculate_fraud_score(user: &Address, amount: u64) -> u32 {
        // Simple fraud detection - would be more sophisticated
        match amount {
            0..=100 => 5,
            101..=1000 => 15,
            1001..=5000 => 25,
            _ => 40,
        }
    }

    // ========== Storage Functions ==========

    fn get_mobile_profile(env: &Env, user: &Address) -> MobileProfile {
        env.storage()
            .persistent()
            .get(&(MOBILE_PROFILE, user.clone()))
            .unwrap_or_else(|| panic_with_error!(env, MobilePlatformError::DeviceNotSupported))
    }

    fn set_mobile_profile(env: &Env, user: &Address, profile: &MobileProfile) {
        env.storage()
            .persistent()
            .set(&(MOBILE_PROFILE, user.clone()), profile);
    }

    fn add_offline_content(env: &Env, user: &Address, content: &OfflineContent) {
        let key = (OFFLINE_CONTENT, user.clone());
        let mut contents: Vec<OfflineContent> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));

        contents.push_back(content.clone());
        env.storage().persistent().set(&key, &contents);
    }

    fn add_push_notification(env: &Env, user: &Address, notification: &PushNotification) {
        let key = (PUSH_NOTIFICATIONS, user.clone());
        let mut notifications: Vec<PushNotification> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));

        notifications.push_back(notification.clone());
        env.storage().persistent().set(&key, &notifications);
    }

    fn add_mobile_transaction(env: &Env, transaction: &MobileTransaction) {
        let key = (MOBILE_PAYMENTS, transaction.user.clone());
        let mut transactions: Vec<MobileTransaction> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));

        transactions.push_back(transaction.clone());
        env.storage().persistent().set(&key, &transactions);
    }

    fn add_security_event(env: &Env, user: &Address, event: &SecurityEvent) {
        let key = (MOBILE_SECURITY, user.clone());
        let mut events: Vec<SecurityEvent> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));

        events.push_back(event.clone());
        env.storage().persistent().set(&key, &events);
    }

    fn get_onboarding_status(env: &Env, user: &Address) -> Option<OnboardingStatus> {
        env.storage()
            .persistent()
            .get(&(ONBOARDING_STATUS, user.clone()))
    }

    fn set_onboarding_status(env: &Env, user: &Address, status: &OnboardingStatus) {
        env.storage()
            .persistent()
            .set(&(ONBOARDING_STATUS, user.clone()), status);
    }

    fn add_user_feedback(env: &Env, feedback: &UserFeedback) {
        let mut feedbacks: Vec<UserFeedback> = env
            .storage()
            .persistent()
            .get(&USER_FEEDBACK)
            .unwrap_or(Vec::new(env));

        feedbacks.push_back(feedback.clone());
        env.storage().persistent().set(&USER_FEEDBACK, &feedbacks);
    }

    fn get_active_experiments(env: &Env) -> Vec<UXExperiment> {
        env.storage()
            .persistent()
            .get(&UX_EXPERIMENTS)
            .unwrap_or(Vec::new(env))
    }
}
