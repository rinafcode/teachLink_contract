//! Basic Notification System Events
//!
//! This module defines only the most essential events for the notification system.

<<<<<<< HEAD
use crate::notification_types::NotificationChannel;
use soroban_sdk::{contractevent, Address, Bytes};
=======
use crate::notification_types::{NotificationChannel, NotificationDeliveryStatus};
use soroban_sdk::{contractevent, Address, Bytes, Vec};
>>>>>>> 883874788426ad4ca0e91de987a6ceeea1da5f0b

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
pub struct NotificationPrefUpdatedEvent {
    pub user: Address,
    pub updated_at: u64,
}
