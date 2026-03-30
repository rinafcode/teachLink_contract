use soroban_sdk::{contracttype, Address, Bytes, Map, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunityConfig {
    pub admin: Address,
    pub token: Address,
    pub fee_recipient: Address,
    pub min_reputation_for_moderation: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForumCategory {
    pub id: u64,
    pub name: Bytes,
    pub description: Bytes,
    pub is_private: bool,
    pub allowed_roles: Vec<CommunityRole>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunityRole {
    Member,
    Moderator,
    Mentor,
    Admin,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForumPost {
    pub id: u64,
    pub category_id: u64,
    pub author: Address,
    pub title: Bytes,
    pub content: Bytes,
    pub created_at: u64,
    pub updated_at: u64,
    pub tags: Vec<Bytes>,
    pub likes: u32,
    pub is_resolved: bool,
    pub solution_post_id: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForumComment {
    pub id: u64,
    pub post_id: u64,
    pub author: Address,
    pub content: Bytes,
    pub created_at: u64,
    pub likes: u32,
    pub is_solution: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MentorshipProgram {
    pub id: u64,
    pub mentor: Address,
    pub mentee: Address,
    pub topic: Bytes,
    pub status: MentorshipStatus,
    pub created_at: u64,
    pub duration_days: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MentorshipStatus {
    Pending,
    Active,
    Completed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnowledgeArticle {
    pub id: u64,
    pub title: Bytes,
    pub content: Bytes,
    pub author: Address,
    pub created_at: u64,
    pub updated_at: u64,
    pub tags: Vec<Bytes>,
    pub contribution_points: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserReputation {
    pub user: Address,
    pub points: u64,
    pub level: u32,
    pub badges: Vec<Bytes>,
    pub help_requests_resolved: u32,
    pub contributions_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunityEvent {
    pub id: u64,
    pub title: Bytes,
    pub description: Bytes,
    pub organizer: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub participants: Vec<Address>,
    pub capacity: u32,
    pub event_type: EventType,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventType {
    Workshop,
    Seminar,
    OfficeHours,
    CommunityMeeting,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunityFeedback {
    pub id: u64,
    pub user: Address,
    pub category: FeedbackCategory,
    pub content: Bytes,
    pub rating: u32, // 1-5
    pub status: FeedbackStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FeedbackCategory {
    Platform,
    Content,
    Support,
    General,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FeedbackStatus {
    New,
    Reviewing,
    Addressed,
    Closed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModerationAction {
    pub moderator: Address,
    pub target_user: Address,
    pub action_type: ModerationType,
    pub reason: Bytes,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModerationType {
    Warning,
    Mute,
    Ban,
    Clear,
}
