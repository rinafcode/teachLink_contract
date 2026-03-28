//! Notification System Events (Simplified)
//!
//! This module defines essential events emitted by the notification system.

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

// Simplified preference event
#[contractevent]
#[derive(Clone, Debug)]
pub struct NotificationPrefUpdatedEvent {
    pub user: Address,
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
