//! Social Learning Events
//!
//! This module defines all events emitted by the social learning platform.

use soroban_sdk::{contractevent, Address, Bytes, Symbol, Vec};

#[contractevent]
pub struct StudyGroupCreatedEvent {
    pub group_id: u64,
    pub creator: Address,
    pub name: Bytes,
    pub subject: Bytes,
}

#[contractevent]
pub struct StudyGroupJoinedEvent {
    pub group_id: u64,
    pub user: Address,
    pub joined_at: u64,
}

#[contractevent]
pub struct StudyGroupLeftEvent {
    pub group_id: u64,
    pub user: Address,
    pub left_at: u64,
}

#[contractevent]
pub struct ForumCreatedEvent {
    pub forum_id: u64,
    pub creator: Address,
    pub title: Bytes,
    pub category: Bytes,
}

#[contractevent]
pub struct ForumPostCreatedEvent {
    pub post_id: u64,
    pub forum_id: u64,
    pub author: Address,
    pub title: Bytes,
}

#[contractevent]
pub struct WorkspaceCreatedEvent {
    pub workspace_id: u64,
    pub creator: Address,
    pub name: Bytes,
    pub project_type: Symbol,
}

#[contractevent]
pub struct PeerReviewCreatedEvent {
    pub review_id: u64,
    pub reviewer: Address,
    pub reviewee: Address,
    pub rating: u32,
    pub content_type: Symbol,
}

#[contractevent]
pub struct MentorshipProfileCreatedEvent {
    pub mentor: Address,
    pub expertise_areas: Vec<Bytes>,
}

#[contractevent]
pub struct MentorshipSessionScheduledEvent {
    pub session_id: u64,
    pub mentor: Address,
    pub mentee: Address,
    pub topic: Bytes,
    pub scheduled_time: u64,
}

#[contractevent]
pub struct SocialBadgeEarnedEvent {
    pub user: Address,
    pub badge_id: u64,
    pub badge_name: Bytes,
}

#[contractevent]
pub struct SocialPointsEarnedEvent {
    pub user: Address,
    pub points: u64,
    pub activity_type: Symbol,
}

#[contractevent]
pub struct CollaborationStartedEvent {
    pub workspace_id: u64,
    pub collaborator: Address,
    pub role: Symbol,
}
