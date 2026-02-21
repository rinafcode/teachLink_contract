//! Notification System Types
//!
//! This module defines all types used by the comprehensive notification system.

use soroban_sdk::{contracttype, Address, Bytes, Env, Map, String, Vec};

/// Notification delivery channels
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NotificationChannel {
    Email = 0,
    SMS = 1,
    Push = 2,
    InApp = 3,
}

/// Notification delivery status
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NotificationDeliveryStatus {
    Pending = 0,
    Scheduled = 1,
    Processing = 2,
    Delivered = 3,
    Failed = 4,
    Retrying = 5,
}

/// Notification content with localization support
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationContent {
    pub subject: Bytes,
    pub body: Bytes,
    pub data: Bytes,                                   // Additional structured data
    pub localization: Map<Bytes, NotificationContent>, // language_code -> content
}

/// Notification scheduling configuration
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationSchedule {
    pub notification_id: u64,
    pub recipient: Address,
    pub channel: NotificationChannel,
    pub scheduled_time: u64,
    pub timezone: Bytes,
    pub is_recurring: bool,
    pub recurrence_pattern: u32, // 1=hourly, 2=daily, 3=weekly, 4=monthly
    pub max_deliveries: Option<u32>,
    pub delivery_count: u32,
}

/// Notification tracking information
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationTracking {
    pub notification_id: u64,
    pub recipient: Address,
    pub channel: NotificationChannel,
    pub status: NotificationDeliveryStatus,
    pub sent_at: u64,
    pub delivered_at: u64,
    pub error_message: Bytes,
    pub retry_count: u32,
}

/// User notification preferences
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationPreference {
    pub channel: NotificationChannel,
    pub enabled: bool,
    pub frequency_hours: u32,   // Minimum hours between notifications
    pub quiet_hours_only: bool, // Only send during quiet hours
    pub urgent_only: bool,      // Only urgent notifications
}

/// User notification settings
#[contracttype]
#[derive(Clone, Debug)]
pub struct UserNotificationSettings {
    pub user: Address,
    pub timezone: Bytes,
    pub quiet_hours_start: u32, // Seconds from midnight
    pub quiet_hours_end: u32,   // Seconds from midnight
    pub max_daily_notifications: u32,
    pub do_not_disturb: bool,
}

/// Notification template
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationTemplate {
    pub template_id: u64,
    pub name: Bytes,
    pub channels: Vec<NotificationChannel>,
    pub content: NotificationContent,
    pub is_active: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

/// A/B testing configuration
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationABTest {
    pub test_id: u64,
    pub name: Bytes,
    pub template_a_id: u64,
    pub template_b_id: u64,
    pub traffic_split: u32, // Percentage for template A (0-100)
    pub start_time: u64,
    pub end_time: u64,
    pub is_active: bool,
    pub metrics_a: ABTestMetrics,
    pub metrics_b: ABTestMetrics,
}

/// A/B test metrics
#[contracttype]
#[derive(Clone, Debug)]
pub struct ABTestMetrics {
    pub sent_count: u64,
    pub delivered_count: u64,
    pub open_count: u64,
    pub click_count: u64,
    pub conversion_count: u64,
}

/// Notification compliance settings
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationCompliance {
    pub region: Bytes,
    pub require_opt_in: bool,
    pub max_daily_per_user: u32,
    pub quiet_hours_required: bool,
    pub data_retention_days: u32,
    pub age_gating_required: bool,
    pub content_restrictions: Vec<Bytes>,
}

/// Notification engagement tracking
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationEngagement {
    pub notification_id: u64,
    pub user: Address,
    pub opened_at: u64,
    pub clicked_at: u64,
    pub converted_at: u64,
    pub device_type: Bytes,
    pub user_agent: Bytes,
}

/// Notification batch delivery
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationBatch {
    pub batch_id: u64,
    pub notifications: Vec<u64>,
    pub created_at: u64,
    pub processed_at: u64,
    pub status: NotificationDeliveryStatus,
    pub success_count: u32,
    pub failure_count: u32,
}

/// Notification rate limiting
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationRateLimit {
    pub user: Address,
    pub channel: NotificationChannel,
    pub window_start: u64,
    pub window_end: u64,
    pub count: u32,
    pub max_allowed: u32,
}

/// Notification delivery optimization
#[contracttype]
#[derive(Clone, Debug)]
pub struct DeliveryOptimization {
    pub user: Address,
    pub preferred_channels: Vec<NotificationChannel>,
    pub optimal_send_times: Vec<u32>, // Hours of day
    pub engagement_score: u32,        // 0-10000 basis points
    pub last_optimization: u64,
}

/// Notification content personalization
#[contracttype]
#[derive(Clone, Debug)]
pub struct PersonalizationRule {
    pub rule_id: u64,
    pub name: Bytes,
    pub conditions: Vec<PersonalizationCondition>,
    pub actions: Vec<PersonalizationAction>,
    pub priority: u32,
    pub is_active: bool,
}

/// Personalization condition
#[contracttype]
#[derive(Clone, Debug)]
pub struct PersonalizationCondition {
    pub field: Bytes,  // e.g., "user_age", "user_location"
    pub operator: u32, // 0=equals, 1=greater_than, 2=less_than, 3=contains
    pub value: Bytes,
}

/// Personalization action
#[contracttype]
#[derive(Clone, Debug)]
pub struct PersonalizationAction {
    pub action_type: u32, // 0=modify_content, 1=change_channel, 2=adjust_timing
    pub parameters: Map<Bytes, Bytes>,
}

/// Notification localization
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationLocalization {
    pub template_id: u64,
    pub language_code: Bytes,
    pub subject: Bytes,
    pub body: Bytes,
    pub is_active: bool,
}

/// Notification analytics aggregation
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationAnalyticsAggregation {
    pub period_start: u64,
    pub period_end: u64,
    pub total_notifications: u64,
    pub unique_users: u32,
    pub channel_breakdown: Map<Bytes, u64>,
    pub delivery_rate: u32,   // basis points
    pub open_rate: u32,       // basis points
    pub click_rate: u32,      // basis points
    pub conversion_rate: u32, // basis points
}

/// Channel statistics for analytics
#[contracttype]
#[derive(Clone, Debug)]
pub struct ChannelStats {
    pub sent: u64,
    pub delivered: u64,
    pub failed: u64,
}

/// Notification error tracking
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationError {
    pub error_id: u64,
    pub notification_id: u64,
    pub error_type: u32, // 0=delivery, 1=template, 2=personalization, 3=scheduling
    pub error_code: u32,
    pub error_message: Bytes,
    pub retry_count: u32,
    pub next_retry_at: u64,
    pub resolved: bool,
}

/// Notification webhook configuration
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationWebhook {
    pub webhook_id: u64,
    pub name: Bytes,
    pub url: Bytes,
    pub secret: Bytes,
    pub events: Vec<Bytes>, // e.g., ["delivered", "failed", "opened"]
    pub is_active: bool,
    pub last_triggered: u64,
    pub success_count: u64,
    pub failure_count: u64,
}

/// Notification content filtering
#[contracttype]
#[derive(Clone, Debug)]
pub struct ContentFilter {
    pub filter_id: u64,
    pub name: Bytes,
    pub patterns: Vec<Bytes>, // Regex patterns to block
    pub replacement: Option<Bytes>,
    pub is_active: bool,
    pub applied_count: u64,
}

/// Notification user segmentation
#[contracttype]
#[derive(Clone, Debug)]
pub struct UserSegment {
    pub segment_id: u64,
    pub name: Bytes,
    pub criteria: Vec<SegmentationCriteria>,
    pub user_count: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Segmentation criteria
#[contracttype]
#[derive(Clone, Debug)]
pub struct SegmentationCriteria {
    pub field: Bytes,
    pub operator: u32,
    pub value: Bytes,
    pub weight: u32,
}

/// Notification campaign
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationCampaign {
    pub campaign_id: u64,
    pub name: Bytes,
    pub template_id: u64,
    pub segments: Vec<u64>, // User segment IDs
    pub schedule: NotificationSchedule,
    pub budget: Option<i128>,
    pub status: CampaignStatus,
    pub metrics: CampaignMetrics,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Campaign status
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CampaignStatus {
    Draft = 0,
    Scheduled = 1,
    Running = 2,
    Paused = 3,
    Completed = 4,
    Cancelled = 5,
}

/// Campaign metrics
#[contracttype]
#[derive(Clone, Debug)]
pub struct CampaignMetrics {
    pub total_sent: u64,
    pub total_delivered: u64,
    pub total_opened: u64,
    pub total_clicked: u64,
    pub total_converted: u64,
    pub total_spent: i128,
    pub cost_per_conversion: i128,
}

/// Notification priority levels
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NotificationPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

/// Notification throttling configuration
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationThrottling {
    pub channel: NotificationChannel,
    pub max_per_second: u32,
    pub max_per_minute: u32,
    pub max_per_hour: u32,
    pub max_per_day: u32,
    pub current_window_start: u64,
    pub current_counts: Map<u32, u32>, // window_type -> count
}

/// Notification content validation
#[contracttype]
#[derive(Clone, Debug)]
pub struct ContentValidation {
    pub validation_id: u64,
    pub content: NotificationContent,
    pub validation_rules: Vec<ValidationRule>,
    pub results: Vec<ValidationResult>,
    pub is_valid: bool,
    pub validated_at: u64,
}

/// Validation rule
#[contracttype]
#[derive(Clone, Debug)]
pub struct ValidationRule {
    pub rule_type: u32, // 0=length, 1=content, 2=spam, 3=personal_info
    pub parameters: Map<Bytes, Bytes>,
    pub required: bool,
}

/// Validation result
#[contracttype]
#[derive(Clone, Debug)]
pub struct ValidationResult {
    pub rule_type: u32,
    pub passed: bool,
    pub message: Bytes,
    pub severity: u32, // 0=info, 1=warning, 2=error
}

impl Default for NotificationContent {
    fn default() -> Self {
        Self {
            subject: Bytes::new(&Env::default()),
            body: Bytes::new(&Env::default()),
            data: Bytes::new(&Env::default()),
            localization: Map::new(&Env::default()),
        }
    }
}

impl Default for UserNotificationSettings {
    fn default() -> Self {
        Self {
            user: Address::from_string(&String::from_str(
                &Env::default(),
                "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWH",
            )),
            timezone: Bytes::from_slice(&Env::default(), b"UTC"),
            quiet_hours_start: 22 * 3600, // 10 PM
            quiet_hours_end: 8 * 3600,    // 8 AM
            max_daily_notifications: 50,
            do_not_disturb: false,
        }
    }
}

impl Default for NotificationTemplate {
    fn default() -> Self {
        Self {
            template_id: 0,
            name: Bytes::new(&Env::default()),
            channels: Vec::new(&Env::default()),
            content: NotificationContent::default(),
            is_active: true,
            created_at: 0,
            updated_at: 0,
        }
    }
}

impl Default for NotificationAnalyticsAggregation {
    fn default() -> Self {
        Self {
            period_start: 0,
            period_end: 0,
            total_notifications: 0,
            unique_users: 0,
            channel_breakdown: Map::new(&Env::default()),
            delivery_rate: 0,
            open_rate: 0,
            click_rate: 0,
            conversion_rate: 0,
        }
    }
}
