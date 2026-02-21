//! Notification System Events
//!
//! This module defines all events emitted by the comprehensive notification system.

use soroban_sdk::{contractevent, Address, Bytes, Vec};
use crate::notification_types::{
    NotificationChannel, NotificationDeliveryStatus, NotificationPreference,
    NotificationTemplate,
};

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationSentEvent {
    pub notification_id: u64,
    pub recipient: Address,
    pub channel: NotificationChannel,
    pub sent_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationDeliveredEvent {
    pub notification_id: u64,
    pub recipient: Address,
    pub channel: NotificationChannel,
    pub delivered_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationFailedEvent {
    pub notification_id: u64,
    pub recipient: Address,
    pub channel: NotificationChannel,
    pub error: Bytes,
    pub retry_count: u32,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationScheduledEvent {
    pub notification_id: u64,
    pub recipient: Address,
    pub channel: NotificationChannel,
    pub scheduled_time: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationOpenedEvent {
    pub notification_id: u64,
    pub user: Address,
    pub opened_at: u64,
    pub device_type: Bytes,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationClickedEvent {
    pub notification_id: u64,
    pub user: Address,
    pub clicked_at: u64,
    pub click_target: Bytes,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationPreferenceUpdatedEvent {
    pub user: Address,
    pub preferences: Vec<NotificationPreference>,
    pub updated_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationTemplateCreatedEvent {
    pub template_id: u64,
    pub name: Bytes,
    pub channels: Vec<NotificationChannel>,
    pub created_by: Address,
    pub created_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationTemplateUpdatedEvent {
    pub template_id: u64,
    pub name: Bytes,
    pub updated_by: Address,
    pub updated_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationBatchProcessedEvent {
    pub batch_id: u64,
    pub notification_count: u32,
    pub success_count: u32,
    pub failure_count: u32,
    pub processed_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationEngagementTrackedEvent {
    pub notification_id: u64,
    pub user: Address,
    pub engagement_type: u32, // 0=open, 1=click, 2=convert
    pub timestamp: u64,
    pub metadata: Bytes,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationABTestStartedEvent {
    pub test_id: u64,
    pub name: Bytes,
    pub template_a_id: u64,
    pub template_b_id: u64,
    pub traffic_split: u32,
    pub started_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationABTestCompletedEvent {
    pub test_id: u64,
    pub winner: u32, // 0=A, 1=B, 2=tie
    pub confidence: u32, // basis points
    pub completed_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationComplianceCheckedEvent {
    pub notification_id: u64,
    pub user: Address,
    pub region: Bytes,
    pub passed: bool,
    pub checked_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationRateLimitedEvent {
    pub user: Address,
    pub channel: NotificationChannel,
    pub limit_type: u32, // 0=daily, 1=hourly, 2=per_minute
    pub current_count: u32,
    pub max_allowed: u32,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationPersonalizationAppliedEvent {
    pub notification_id: u64,
    pub user: Address,
    pub rules_applied: Vec<u64>,
    pub personalization_score: u32, // basis points
    pub applied_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationOptimizationPerformedEvent {
    pub user: Address,
    pub optimization_type: u32, // 0=timing, 1=channel, 2=content
    pub old_score: u32,
    pub new_score: u32,
    pub optimized_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationWebhookTriggeredEvent {
    pub webhook_id: u64,
    pub event_type: Bytes,
    pub notification_id: u64,
    pub payload: Bytes,
    pub triggered_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationContentFilteredEvent {
    pub notification_id: u64,
    pub filter_id: u64,
    pub content_modified: bool,
    pub filtered_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationCampaignStartedEvent {
    pub campaign_id: u64,
    pub name: Bytes,
    pub segment_count: u32,
    pub estimated_notifications: u64,
    pub started_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationCampaignCompletedEvent {
    pub campaign_id: u64,
    pub total_sent: u64,
    pub total_delivered: u64,
    pub total_converted: u64,
    pub roi: i128, // basis points
    pub completed_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationUserSegmentUpdatedEvent {
    pub segment_id: u64,
    pub user_count: u32,
    pub updated_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationThrottlingActivatedEvent {
    pub channel: NotificationChannel,
    pub current_rate: u32,
    pub max_rate: u32,
    pub activated_at: u64,
}

// Events exceeding 32 character limit commented out
/*
#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationValidationFailedEvent {
    pub notification_id: u64,
    pub validation_errors: Vec<Bytes>,
    pub failed_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationDeliveryOptimizedEvent {
    pub notification_id: u64,
    pub original_channel: NotificationChannel,
    pub optimized_channel: NotificationChannel,
    pub optimization_reason: Bytes,
    pub optimized_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationUserPreferencesMigratedEvent {
    pub user: Address,
    pub old_preferences: Vec<NotificationPreference>,
    pub new_preferences: Vec<NotificationPreference>,
    pub migrated_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationSystemInitializedEvent {
    pub initialized_by: Address,
    pub default_templates: Vec<u64>,
    pub initialized_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationConfigurationUpdatedEvent {
    pub config_type: Bytes, // "rate_limits", "compliance", "templates", etc.
    pub updated_by: Address,
    pub old_config: Bytes,
    pub new_config: Bytes,
    pub updated_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationEmergencyModeActivatedEvent {
    pub activated_by: Address,
    pub reason: Bytes,
    pub affected_channels: Vec<NotificationChannel>,
    pub activated_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationEmergencyModeDeactivatedEvent {
    pub deactivated_by: Address,
    pub deactivated_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationBatchCreatedEvent {
    pub batch_id: u64,
    pub notification_count: u32,
    pub channels: Vec<NotificationChannel>,
    pub created_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationDeliveryRetryEvent {
    pub notification_id: u64,
    pub retry_attempt: u32,
    pub max_retries: u32,
    pub retry_reason: Bytes,
    pub scheduled_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationUserActivityTrackedEvent {
    pub user: Address,
    pub activity_type: Bytes, // "login", "transaction", "profile_update", etc.
    pub timestamp: u64,
    pub metadata: Bytes,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationContentRenderedEvent {
    pub notification_id: u64,
    pub template_id: u64,
    pub user: Address,
    pub render_time_ms: u32,
    pub rendered_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationLocalizationAppliedEvent {
    pub notification_id: u64,
    pub language_code: Bytes,
    pub original_subject: Bytes,
    pub localized_subject: Bytes,
    pub applied_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationFeedbackReceivedEvent {
    pub notification_id: u64,
    pub user: Address,
    pub feedback_type: u32, // 0=positive, 1=negative, 2=neutral
    pub feedback_message: Bytes,
    pub received_at: u64,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationPerformanceReportedEvent {
    pub period_start: u64,
    pub period_end: u64,
    pub total_notifications: u64,
    pub delivery_rate: u32,
    pub engagement_rate: u32,
    pub reported_at: u64,
}
*/
