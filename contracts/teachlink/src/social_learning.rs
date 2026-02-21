//! Social Learning Platform Module
//!
//! This module implements comprehensive social learning features including:
//! - Real-time collaboration tools and workspaces
//! - Study group formation and management
//! - Discussion forums and knowledge sharing
//! - Peer review and feedback systems
//! - Social learning analytics and engagement tracking
//! - Mentorship and tutoring matching
//! - Collaborative project management
//! - Social gamification and recognition systems

use soroban_sdk::{Address, Bytes, Env, Map, Vec, Symbol, String, contracttype, contracterror, panic_with_error, TryFromVal, IntoVal, Val, symbol_short};

use crate::storage::*;
use crate::types::*;

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub struct StudyGroup {
    pub id: u64,
    pub name: Bytes,
    pub description: Bytes,
    pub creator: Address,
    pub members: Vec<Address>,
    pub admins: Vec<Address>,
    pub subject: Bytes,
    pub max_members: u32,
    pub is_private: bool,
    pub created_at: u64,
    pub last_activity: u64,
    pub tags: Vec<Bytes>,
    pub settings: StudyGroupSettings,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub struct StudyGroupSettings {
    pub allow_member_invites: bool,
    pub require_admin_approval: bool,
    pub enable_chat: bool,
    pub enable_file_sharing: bool,
    pub enable_video_calls: bool,
    pub auto_approve_members: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub struct DiscussionForum {
    pub id: u64,
    pub title: Bytes,
    pub description: Bytes,
    pub creator: Address,
    pub category: Bytes,
    pub tags: Vec<Bytes>,
    pub is_pinned: bool,
    pub is_locked: bool,
    pub created_at: u64,
    pub last_post_at: u64,
    pub post_count: u32,
    pub view_count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub struct ForumPost {
    pub id: u64,
    pub forum_id: u64,
    pub title: Bytes,
    pub content: Bytes,
    pub author: Address,
    pub created_at: u64,
    pub updated_at: u64,
    pub reply_count: u32,
    pub like_count: u32,
    pub is_pinned: bool,
    pub is_edited: bool,
    pub attachments: Vec<Bytes>,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub struct CollaborationWorkspace {
    pub id: u64,
    pub name: Bytes,
    pub description: Bytes,
    pub creator: Address,
    pub collaborators: Vec<Address>,
    pub project_type: ProjectType,
    pub status: WorkspaceStatus,
    pub created_at: u64,
    pub last_activity: u64,
    pub files: Vec<WorkspaceFile>,
    pub tasks: Vec<WorkspaceTask>,
    pub settings: WorkspaceSettings,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub enum ProjectType {
    Study,
    Research,
    Assignment,
    Tutorial,
    Discussion,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub enum WorkspaceStatus {
    Active,
    Completed,
    Archived,
    Suspended,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub struct WorkspaceFile {
    pub id: u64,
    pub name: Bytes,
    pub content_hash: Bytes,
    pub uploader: Address,
    pub uploaded_at: u64,
    pub file_type: Bytes,
    pub size: u64,
    pub version: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub struct WorkspaceTask {
    pub id: u64,
    pub title: Bytes,
    pub description: Bytes,
    pub assignee: Address,
    pub creator: Address,
    pub due_date: u64,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub created_at: u64,
    pub completed_at: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Review,
    Completed,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub struct WorkspaceSettings {
    pub allow_public_view: bool,
    pub require_approval_to_join: bool,
    pub enable_chat: bool,
    pub enable_video_calls: bool,
    pub auto_save_interval: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub struct PeerReview {
    pub id: u64,
    pub reviewer: Address,
    pub reviewee: Address,
    pub content_type: ReviewContentType,
    pub content_id: u64,
    pub rating: u32, // 1-5 stars
    pub feedback: Bytes,
    pub criteria: Map<Bytes, u32>,
    pub created_at: u64,
    pub is_helpful: bool,
    pub helpful_votes: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub enum ReviewContentType {
    Submission,
    Comment,
    Project,
    Tutorial,
    Resource,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub struct MentorshipProfile {
    pub mentor: Address,
    pub expertise_areas: Vec<Bytes>,
    pub experience_level: ExperienceLevel,
    pub availability: AvailabilityStatus,
    pub hourly_rate: Option<u64>,
    pub bio: Bytes,
    pub rating: u64,
    pub review_count: u32,
    pub mentee_count: u32,
    pub success_rate: u64, // represented as basis points (10000 = 100%)
    pub languages: Vec<Bytes>,
    pub timezone: Bytes,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub enum ExperienceLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub enum AvailabilityStatus {
    Available,
    Busy,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub struct MentorshipSession {
    pub id: u64,
    pub mentor: Address,
    pub mentee: Address,
    pub topic: Bytes,
    pub scheduled_time: u64,
    pub duration: u32, // minutes
    pub status: SessionStatus,
    pub notes: Bytes,
    pub rating: Option<u32>,
    pub feedback: Bytes,
    pub created_at: u64,
    pub completed_at: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub enum SessionStatus {
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
    NoShow,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub struct SocialAnalytics {
    pub user: Address,
    pub study_groups_joined: u32,
    pub discussions_participated: u32,
    pub posts_created: u32,
    pub reviews_given: u32,
    pub mentorship_hours: u64,
    pub collaboration_projects: u32,
    pub social_score: u64,
    pub engagement_level: EngagementLevel,
    pub badges: Vec<Bytes>,
    pub last_updated: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub enum EngagementLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub struct SocialBadge {
    pub id: u64,
    pub name: Bytes,
    pub description: Bytes,
    pub icon: Bytes,
    pub category: BadgeCategory,
    pub requirements: BadgeRequirements,
    pub rarity: BadgeRarity,
    pub created_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub enum BadgeCategory {
    Collaboration,
    Mentorship,
    Contribution,
    Leadership,
    Learning,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub struct BadgeRequirements {
    pub study_groups_joined: Option<u32>,
    pub discussions_participated: Option<u32>,
    pub mentorship_hours: Option<u64>,
    pub reviews_given: Option<u32>,
    pub projects_completed: Option<u32>,
    pub social_score: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub enum BadgeRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub struct GamificationSystem {
    pub points: Map<Address, u64>,
    pub levels: Map<Address, u32>,
    pub streaks: Map<Address, u32>,
    pub achievements: Map<Address, Vec<u64>>,
    pub leaderboards: Map<Bytes, Vec<Address>>,
    pub rewards: Vec<SocialReward>,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub struct SocialReward {
    pub id: u64,
    pub name: Bytes,
    pub description: Bytes,
    pub cost: u64,
    pub category: RewardCategory,
    pub is_available: bool,
    pub created_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub enum RewardCategory {
    Digital,
    Physical,
    Access,
    Recognition,
}

// Storage keys
const STUDY_GROUP_COUNTER: Symbol = Symbol::short("SGC");
const STUDY_GROUPS: Symbol = Symbol::short("SGS");
const USER_STUDY_GROUPS: Symbol = Symbol::short("USG");

const FORUM_COUNTER: Symbol = Symbol::short("FC");
const FORUMS: Symbol = Symbol::short("FRS");
const FORUM_POSTS: Symbol = Symbol::short("FPS");
const USER_POSTS: Symbol = Symbol::short("UPS");

const WORKSPACE_COUNTER: Symbol = Symbol::short("WC");
const WORKSPACES: Symbol = Symbol::short("WKS");
const USER_WORKSPACES: Symbol = Symbol::short("UWS");

const REVIEW_COUNTER: Symbol = Symbol::short("RC");
const REVIEWS: Symbol = Symbol::short("RVS");
const USER_REVIEWS: Symbol = Symbol::short("URS");

const MENTORSHIP_PROFILES: Symbol = Symbol::short("MPS");
const MENTORSHIP_SESSIONS: Symbol = Symbol::short("MSS");
const MENTORSHIP_COUNTER: Symbol = Symbol::short("MSC");

const SOCIAL_ANALYTICS: Symbol = Symbol::short("SAS");
const SOCIAL_BADGES: Symbol = Symbol::short("SBS");
const GAMIFICATION_SYSTEM: Symbol = Symbol::short("GMS");

// Errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SocialLearningError {
    Unauthorized = 1,
    StudyGroupNotFound = 2,
    AlreadyMember = 3,
    MaxMembersReached = 4,
    ForumNotFound = 5,
    PostNotFound = 6,
    WorkspaceNotFound = 7,
    ReviewNotFound = 8,
    MentorshipProfileNotFound = 9,
    SessionNotFound = 10,
    InvalidRating = 11,
    InsufficientPermissions = 12,
    DuplicateEntry = 13,
    InvalidInput = 14,
    ResourceNotFound = 15,
}

pub struct SocialLearningManager;

impl SocialLearningManager {
    // Study Group Management
    pub fn create_study_group(
        env: &Env,
        creator: Address,
        name: Bytes,
        description: Bytes,
        subject: Bytes,
        max_members: u32,
        is_private: bool,
        tags: Vec<Bytes>,
        settings: StudyGroupSettings,
    ) -> Result<u64, SocialLearningError> {
        let counter: u64 = env.storage().instance().get(&STUDY_GROUP_COUNTER).unwrap_or(0);
        let group_id = counter + 1;

        let mut members = Vec::new(&env);
        members.push_back(creator.clone());

        let mut admins = Vec::new(&env);
        admins.push_back(creator.clone());

        let study_group = StudyGroup {
            id: group_id,
            name,
            description,
            creator: creator.clone(),
            members,
            admins,
            subject,
            max_members,
            is_private,
            created_at: env.ledger().timestamp(),
            last_activity: env.ledger().timestamp(),
            tags,
            settings,
        };

        // Store study group
        let mut groups: Map<u64, StudyGroup> = env.storage().instance().get(&STUDY_GROUPS).unwrap_or(Map::new(&env));
        groups.set(group_id, study_group);
        env.storage().instance().set(&STUDY_GROUPS, &groups);

        // Update user's study groups
        let mut user_groups: Vec<u64> = env.storage().instance().get(&USER_STUDY_GROUPS).unwrap_or(Vec::new(&env));
        user_groups.push_back(group_id);
        env.storage().instance().set(&USER_STUDY_GROUPS, &user_groups);

        // Update counter
        env.storage().instance().set(&STUDY_GROUP_COUNTER, &group_id);

        Ok(group_id)
    }

    pub fn join_study_group(
        env: &Env,
        user: Address,
        group_id: u64,
    ) -> Result<(), SocialLearningError> {
        let mut groups: Map<u64, StudyGroup> = env.storage().instance().get(&STUDY_GROUPS)
            .ok_or(SocialLearningError::StudyGroupNotFound)?;
        
        let mut group = groups.get(group_id).ok_or(SocialLearningError::StudyGroupNotFound)?;

        // Check if user is already a member
        if group.members.contains(&user) {
            return Err(SocialLearningError::AlreadyMember);
        }

        // Check if max members reached
        if group.members.len() >= group.max_members as usize {
            return Err(SocialLearningError::MaxMembersReached);
        }

        // Add user to members
        group.members.push_back(user.clone());
        group.last_activity = env.ledger().timestamp();

        groups.set(group_id, group);
        env.storage().instance().set(&STUDY_GROUPS, &groups);

        // Update user's study groups
        let mut user_groups: Vec<u64> = env.storage().instance().get(&USER_STUDY_GROUPS).unwrap_or(Vec::new(&env));
        user_groups.push_back(group_id);
        env.storage().instance().set(&USER_STUDY_GROUPS, &user_groups);

        Ok(())
    }

    pub fn leave_study_group(
        env: &Env,
        user: Address,
        group_id: u64,
    ) -> Result<(), SocialLearningError> {
        let mut groups: Map<u64, StudyGroup> = env.storage().instance().get(&STUDY_GROUPS)
            .ok_or(SocialLearningError::StudyGroupNotFound)?;
        
        let mut group = groups.get(group_id).ok_or(SocialLearningError::StudyGroupNotFound)?;

        // Check if user is a member
        if !group.members.contains(&user) {
            return Err(SocialLearningError::Unauthorized);
        }

        // Remove user from members
        let new_members = group.members.iter().filter(|&member| member != user).collect::<Vec<_>>();
        group.members = new_members;

        // Remove from admins if applicable
        let new_admins = group.admins.iter().filter(|&admin| admin != user).collect::<Vec<_>>();
        group.admins = new_admins;

        group.last_activity = env.ledger().timestamp();

        groups.set(group_id, group);
        env.storage().instance().set(&STUDY_GROUPS, &groups);

        // Update user's study groups
        let mut user_groups: Vec<u64> = env.storage().instance().get(&USER_STUDY_GROUPS).unwrap_or(Vec::new(&env));
        let new_user_groups = user_groups.iter().filter(|&id| id != group_id).collect::<Vec<_>>();
        env.storage().instance().set(&USER_STUDY_GROUPS, &new_user_groups);

        Ok(())
    }

    pub fn get_study_group(env: &Env, group_id: u64) -> Result<StudyGroup, SocialLearningError> {
        let groups: Map<u64, StudyGroup> = env.storage().instance().get(&STUDY_GROUPS)
            .ok_or(SocialLearningError::StudyGroupNotFound)?;
        
        groups.get(group_id).ok_or(SocialLearningError::StudyGroupNotFound)
    }

    pub fn get_user_study_groups(env: &Env, user: Address) -> Vec<u64> {
        env.storage().instance().get(&USER_STUDY_GROUPS).unwrap_or(Vec::new(&env))
    }

    // Discussion Forum Management
    pub fn create_forum(
        env: &Env,
        creator: Address,
        title: Bytes,
        description: Bytes,
        category: Bytes,
        tags: Vec<Bytes>,
    ) -> Result<u64, SocialLearningError> {
        let counter: u64 = env.storage().instance().get(&FORUM_COUNTER).unwrap_or(0);
        let forum_id = counter + 1;

        let forum = DiscussionForum {
            id: forum_id,
            title,
            description,
            creator: creator.clone(),
            category,
            tags,
            is_pinned: false,
            is_locked: false,
            created_at: env.ledger().timestamp(),
            last_post_at: env.ledger().timestamp(),
            post_count: 0,
            view_count: 0,
        };

        // Store forum
        let mut forums: Map<u64, DiscussionForum> = env.storage().instance().get(&FORUMS).unwrap_or(Map::new(&env));
        forums.set(forum_id, forum);
        env.storage().instance().set(&FORUMS, &forums);

        // Update counter
        env.storage().instance().set(&FORUM_COUNTER, &forum_id);

        Ok(forum_id)
    }

    pub fn create_forum_post(
        env: &Env,
        forum_id: u64,
        author: Address,
        title: Bytes,
        content: Bytes,
        attachments: Vec<Bytes>,
    ) -> Result<u64, SocialLearningError> {
        // Check if forum exists and is not locked
        let mut forums: Map<u64, DiscussionForum> = env.storage().instance().get(&FORUMS)
            .ok_or(SocialLearningError::ForumNotFound)?;
        
        let mut forum = forums.get(forum_id).ok_or(SocialLearningError::ForumNotFound)?;
        
        if forum.is_locked {
            return Err(SocialLearningError::InsufficientPermissions);
        }

        let counter: u64 = env.storage().instance().get(&FORUM_COUNTER).unwrap_or(0);
        let post_id = counter + 1;

        let post = ForumPost {
            id: post_id,
            forum_id,
            title,
            content,
            author: author.clone(),
            created_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
            reply_count: 0,
            like_count: 0,
            is_pinned: false,
            is_edited: false,
            attachments,
        };

        // Store post
        let mut posts: Map<u64, ForumPost> = env.storage().instance().get(&FORUM_POSTS).unwrap_or(Map::new(&env));
        posts.set(post_id, post);
        env.storage().instance().set(&FORUM_POSTS, &posts);

        // Update forum
        forum.post_count += 1;
        forum.last_post_at = env.ledger().timestamp();
        forums.set(forum_id, forum);
        env.storage().instance().set(&FORUMS, &forums);

        // Update user's posts
        let mut user_posts: Vec<u64> = env.storage().instance().get(&USER_POSTS).unwrap_or(Vec::new(&env));
        user_posts.push_back(post_id);
        env.storage().instance().set(&USER_POSTS, &user_posts);

        // Update counter
        env.storage().instance().set(&FORUM_COUNTER, &post_id);

        Ok(post_id)
    }

    pub fn get_forum(env: &Env, forum_id: u64) -> Result<DiscussionForum, SocialLearningError> {
        let forums: Map<u64, DiscussionForum> = env.storage().instance().get(&FORUMS)
            .ok_or(SocialLearningError::ForumNotFound)?;
        
        forums.get(forum_id).ok_or(SocialLearningError::ForumNotFound)
    }

    pub fn get_forum_post(env: &Env, post_id: u64) -> Result<ForumPost, SocialLearningError> {
        let posts: Map<u64, ForumPost> = env.storage().instance().get(&FORUM_POSTS)
            .ok_or(SocialLearningError::PostNotFound)?;
        
        posts.get(post_id).ok_or(SocialLearningError::PostNotFound)
    }

    // Collaboration Workspace Management
    pub fn create_workspace(
        env: &Env,
        creator: Address,
        name: Bytes,
        description: Bytes,
        project_type: ProjectType,
        settings: WorkspaceSettings,
    ) -> Result<u64, SocialLearningError> {
        let counter: u64 = env.storage().instance().get(&WORKSPACE_COUNTER).unwrap_or(0);
        let workspace_id = counter + 1;

        let mut collaborators = Vec::new(&env);
        collaborators.push_back(creator.clone());

        let workspace = CollaborationWorkspace {
            id: workspace_id,
            name,
            description,
            creator: creator.clone(),
            collaborators,
            project_type,
            status: WorkspaceStatus::Active,
            created_at: env.ledger().timestamp(),
            last_activity: env.ledger().timestamp(),
            files: Vec::new(&env),
            tasks: Vec::new(&env),
            settings,
        };

        // Store workspace
        let mut workspaces: Map<u64, CollaborationWorkspace> = env.storage().instance().get(&WORKSPACES).unwrap_or(Map::new(&env));
        workspaces.set(workspace_id, workspace);
        env.storage().instance().set(&WORKSPACES, &workspaces);

        // Update user's workspaces
        let mut user_workspaces: Vec<u64> = env.storage().instance().get(&USER_WORKSPACES).unwrap_or(Vec::new(&env));
        user_workspaces.push_back(workspace_id);
        env.storage().instance().set(&USER_WORKSPACES, &user_workspaces);

        // Update counter
        env.storage().instance().set(&WORKSPACE_COUNTER, &workspace_id);

        Ok(workspace_id)
    }

    pub fn get_workspace(env: &Env, workspace_id: u64) -> Result<CollaborationWorkspace, SocialLearningError> {
        let workspaces: Map<u64, CollaborationWorkspace> = env.storage().instance().get(&WORKSPACES)
            .ok_or(SocialLearningError::WorkspaceNotFound)?;
        
        workspaces.get(workspace_id).ok_or(SocialLearningError::WorkspaceNotFound)
    }

    pub fn get_user_workspaces(env: &Env, user: Address) -> Vec<u64> {
        env.storage().instance().get(&USER_WORKSPACES).unwrap_or(Vec::new(&env))
    }

    // Peer Review System
    pub fn create_review(
        env: &Env,
        reviewer: Address,
        reviewee: Address,
        content_type: ReviewContentType,
        content_id: u64,
        rating: u32,
        feedback: Bytes,
        criteria: Map<Bytes, u32>,
    ) -> Result<u64, SocialLearningError> {
        if rating < 1 || rating > 5 {
            return Err(SocialLearningError::InvalidRating);
        }

        let counter: u64 = env.storage().instance().get(&REVIEW_COUNTER).unwrap_or(0);
        let review_id = counter + 1;

        let review = PeerReview {
            id: review_id,
            reviewer: reviewer.clone(),
            reviewee: reviewee.clone(),
            content_type,
            content_id,
            rating,
            feedback,
            criteria,
            created_at: env.ledger().timestamp(),
            is_helpful: false,
            helpful_votes: 0,
        };

        // Store review
        let mut reviews: Map<u64, PeerReview> = env.storage().instance().get(&REVIEWS).unwrap_or(Map::new(&env));
        reviews.set(review_id, review);
        env.storage().instance().set(&REVIEWS, &reviews);

        // Update user's reviews
        let mut user_reviews: Vec<u64> = env.storage().instance().get(&USER_REVIEWS).unwrap_or(Vec::new(&env));
        user_reviews.push_back(review_id);
        env.storage().instance().set(&USER_REVIEWS, &user_reviews);

        // Update counter
        env.storage().instance().set(&REVIEW_COUNTER, &review_id);

        Ok(review_id)
    }

    pub fn get_review(env: &Env, review_id: u64) -> Result<PeerReview, SocialLearningError> {
        let reviews: Map<u64, PeerReview> = env.storage().instance().get(&REVIEWS)
            .ok_or(SocialLearningError::ReviewNotFound)?;
        
        reviews.get(review_id).ok_or(SocialLearningError::ReviewNotFound)
    }

    // Mentorship System
    pub fn create_mentorship_profile(
        env: &Env,
        mentor: Address,
        expertise_areas: Vec<Bytes>,
        experience_level: ExperienceLevel,
        availability: AvailabilityStatus,
        hourly_rate: Option<u64>,
        bio: Bytes,
        languages: Vec<Bytes>,
        timezone: Bytes,
    ) -> Result<(), SocialLearningError> {
        let profile = MentorshipProfile {
            mentor: mentor.clone(),
            expertise_areas,
            experience_level,
            availability,
            hourly_rate,
            bio,
            rating: 0.0,
            review_count: 0,
            mentee_count: 0,
            success_rate: 0.0,
            languages,
            timezone,
        };

        // Store profile
        let mut profiles: Map<Address, MentorshipProfile> = env.storage().instance().get(&MENTORSHIP_PROFILES).unwrap_or(Map::new(&env));
        profiles.set(mentor, profile);
        env.storage().instance().set(&MENTORSHIP_PROFILES, &profiles);

        Ok(())
    }

    pub fn get_mentorship_profile(env: &Env, mentor: Address) -> Result<MentorshipProfile, SocialLearningError> {
        let profiles: Map<Address, MentorshipProfile> = env.storage().instance().get(&MENTORSHIP_PROFILES)
            .ok_or(SocialLearningError::MentorshipProfileNotFound)?;
        
        profiles.get(mentor).ok_or(SocialLearningError::MentorshipProfileNotFound)
    }

    // Social Analytics
    pub fn get_user_analytics(env: &Env, user: Address) -> SocialAnalytics {
        env.storage().instance().get(&SOCIAL_ANALYTICS).unwrap_or(SocialAnalytics {
            user: user.clone(),
            study_groups_joined: 0,
            discussions_participated: 0,
            posts_created: 0,
            reviews_given: 0,
            mentorship_hours: 0,
            collaboration_projects: 0,
            social_score: 0,
            engagement_level: EngagementLevel::Low,
            badges: Vec::new(&env),
            last_updated: env.ledger().timestamp(),
        })
    }

    pub fn update_user_analytics(env: &Env, user: Address, analytics: SocialAnalytics) {
        env.storage().instance().set(&SOCIAL_ANALYTICS, &analytics);
    }
}

// Soroban trait implementations for contract types
impl TryFromVal<Env, Val> for AvailabilityStatus {
    type Error = soroban_sdk::ConversionError;

    fn try_from_val(env: &Env, val: &Val) -> Result<Self, Self::Error> {
        let symbol = Symbol::try_from_val(env, val)?;
        let symbol_str = symbol_short!(symbol);
        match symbol_str {
            "Available" => Ok(AvailabilityStatus::Available),
            "Busy" => Ok(AvailabilityStatus::Busy),
            "Unavailable" => Ok(AvailabilityStatus::Unavailable),
            _ => Err(soroban_sdk::ConversionError {}),
        }
    }
}

impl IntoVal<Env, Val> for AvailabilityStatus {
    fn into_val(&self, env: &Env) -> Val {
        match self {
            AvailabilityStatus::Available => Symbol::new(env, "Available").into_val(env),
            AvailabilityStatus::Busy => Symbol::new(env, "Busy").into_val(env),
            AvailabilityStatus::Unavailable => Symbol::new(env, "Unavailable").into_val(env),
        }
    }
}

impl TryFromVal<Env, Val> for MentorshipProfile {
    type Error = soroban_sdk::ConversionError;

    fn try_from_val(env: &Env, val: &Val) -> Result<Self, Self::Error> {
        let map = Map::<Val, Val>::try_from_val(env, val)?;
        Ok(MentorshipProfile {
            mentor: Address::try_from_val(env, &map.get(Symbol::new(env, "mentor").into_val(env)).unwrap_or_default())?,
            expertise_areas: Vec::<Bytes>::try_from_val(env, &map.get(Symbol::new(env, "expertise_areas").into_val(env)).unwrap_or_default())?,
            experience_level: ExperienceLevel::try_from_val(env, &map.get(Symbol::new(env, "experience_level").into_val(env)).unwrap_or_default())?,
            availability: AvailabilityStatus::try_from_val(env, &map.get(Symbol::new(env, "availability").into_val(env)).unwrap_or_default())?,
            hourly_rate: Option::<u64>::try_from_val(env, &map.get(Symbol::new(env, "hourly_rate").into_val(env)).unwrap_or_default())?,
            bio: Bytes::try_from_val(env, &map.get(Symbol::new(env, "bio").into_val(env)).unwrap_or_default())?,
            rating: u64::try_from_val(env, &map.get(Symbol::new(env, "rating").into_val(env)).unwrap_or_default())?,
            review_count: u32::try_from_val(env, &map.get(Symbol::new(env, "review_count").into_val(env)).unwrap_or_default())?,
            mentee_count: u32::try_from_val(env, &map.get(Symbol::new(env, "mentee_count").into_val(env)).unwrap_or_default())?,
            success_rate: u64::try_from_val(env, &map.get(Symbol::new(env, "success_rate").into_val(env)).unwrap_or_default())?,
            languages: Vec::<Bytes>::try_from_val(env, &map.get(Symbol::new(env, "languages").into_val(env)).unwrap_or_default())?,
            timezone: Bytes::try_from_val(env, &map.get(Symbol::new(env, "timezone").into_val(env)).unwrap_or_default())?,
        })
    }
}

impl IntoVal<Env, Val> for MentorshipProfile {
    fn into_val(&self, env: &Env) -> Val {
        let mut map = Map::<Val, Val>::new(env);
        map.set(Symbol::new(env, "mentor").into_val(env), self.mentor.into_val(env));
        map.set(Symbol::new(env, "expertise_areas").into_val(env), self.expertise_areas.into_val(env));
        map.set(Symbol::new(env, "experience_level").into_val(env), self.experience_level.into_val(env));
        map.set(Symbol::new(env, "availability").into_val(env), self.availability.into_val(env));
        map.set(Symbol::new(env, "hourly_rate").into_val(env), self.hourly_rate.into_val(env));
        map.set(Symbol::new(env, "bio").into_val(env), self.bio.into_val(env));
        map.set(Symbol::new(env, "rating").into_val(env), self.rating.into_val(env));
        map.set(Symbol::new(env, "review_count").into_val(env), self.review_count.into_val(env));
        map.set(Symbol::new(env, "mentee_count").into_val(env), self.mentee_count.into_val(env));
        map.set(Symbol::new(env, "success_rate").into_val(env), self.success_rate.into_val(env));
        map.set(Symbol::new(env, "languages").into_val(env), self.languages.into_val(env));
        map.set(Symbol::new(env, "timezone").into_val(env), self.timezone.into_val(env));
        map.into_val(env)
    }
}

impl TryFromVal<Env, Val> for SocialAnalytics {
    type Error = soroban_sdk::ConversionError;

    fn try_from_val(env: &Env, val: &Val) -> Result<Self, Self::Error> {
        let map = Map::<Val, Val>::try_from_val(env, val)?;
        Ok(SocialAnalytics {
            user: Address::try_from_val(env, &map.get(Symbol::new(env, "user").into_val(env)).unwrap_or_default())?,
            study_groups_joined: u32::try_from_val(env, &map.get(Symbol::new(env, "study_groups_joined").into_val(env)).unwrap_or_default())?,
            discussions_participated: u32::try_from_val(env, &map.get(Symbol::new(env, "discussions_participated").into_val(env)).unwrap_or_default())?,
            posts_created: u32::try_from_val(env, &map.get(Symbol::new(env, "posts_created").into_val(env)).unwrap_or_default())?,
            reviews_given: u32::try_from_val(env, &map.get(Symbol::new(env, "reviews_given").into_val(env)).unwrap_or_default())?,
            mentorship_hours: u64::try_from_val(env, &map.get(Symbol::new(env, "mentorship_hours").into_val(env)).unwrap_or_default())?,
            collaboration_projects: u32::try_from_val(env, &map.get(Symbol::new(env, "collaboration_projects").into_val(env)).unwrap_or_default())?,
            social_score: u64::try_from_val(env, &map.get(Symbol::new(env, "social_score").into_val(env)).unwrap_or_default())?,
            engagement_level: EngagementLevel::try_from_val(env, &map.get(Symbol::new(env, "engagement_level").into_val(env)).unwrap_or_default())?,
            badges: Vec::<Bytes>::try_from_val(env, &map.get(Symbol::new(env, "badges").into_val(env)).unwrap_or_default())?,
            last_updated: u64::try_from_val(env, &map.get(Symbol::new(env, "last_updated").into_val(env)).unwrap_or_default())?,
        })
    }
}

impl IntoVal<Env, Val> for SocialAnalytics {
    fn into_val(&self, env: &Env) -> Val {
        let mut map = Map::<Val, Val>::new(env);
        map.set(Symbol::new(env, "user").into_val(env), self.user.into_val(env));
        map.set(Symbol::new(env, "study_groups_joined").into_val(env), self.study_groups_joined.into_val(env));
        map.set(Symbol::new(env, "discussions_participated").into_val(env), self.discussions_participated.into_val(env));
        map.set(Symbol::new(env, "posts_created").into_val(env), self.posts_created.into_val(env));
        map.set(Symbol::new(env, "reviews_given").into_val(env), self.reviews_given.into_val(env));
        map.set(Symbol::new(env, "mentorship_hours").into_val(env), self.mentorship_hours.into_val(env));
        map.set(Symbol::new(env, "collaboration_projects").into_val(env), self.collaboration_projects.into_val(env));
        map.set(Symbol::new(env, "social_score").into_val(env), self.social_score.into_val(env));
        map.set(Symbol::new(env, "engagement_level").into_val(env), self.engagement_level.into_val(env));
        map.set(Symbol::new(env, "badges").into_val(env), self.badges.into_val(env));
        map.set(Symbol::new(env, "last_updated").into_val(env), self.last_updated.into_val(env));
        map.into_val(env)
    }
}

// Additional enum implementations
impl TryFromVal<Env, Val> for ExperienceLevel {
    type Error = soroban_sdk::ConversionError;

    fn try_from_val(env: &Env, val: &Val) -> Result<Self, Self::Error> {
        let symbol = Symbol::try_from_val(env, val)?;
        match symbol.to_string().as_str() {
            "Beginner" => Ok(ExperienceLevel::Beginner),
            "Intermediate" => Ok(ExperienceLevel::Intermediate),
            "Advanced" => Ok(ExperienceLevel::Advanced),
            "Expert" => Ok(ExperienceLevel::Expert),
            _ => Err(soroban_sdk::ConversionError {}),
        }
    }
}

impl IntoVal<Env, Val> for ExperienceLevel {
    fn into_val(&self, env: &Env) -> Val {
        match self {
            ExperienceLevel::Beginner => Symbol::new(env, "Beginner").into_val(env),
            ExperienceLevel::Intermediate => Symbol::new(env, "Intermediate").into_val(env),
            ExperienceLevel::Advanced => Symbol::new(env, "Advanced").into_val(env),
            ExperienceLevel::Expert => Symbol::new(env, "Expert").into_val(env),
        }
    }
}

impl TryFromVal<Env, Val> for SessionStatus {
    type Error = soroban_sdk::ConversionError;

    fn try_from_val(env: &Env, val: &Val) -> Result<Self, Self::Error> {
        let symbol = Symbol::try_from_val(env, val)?;
        match symbol.to_string().as_str() {
            "Scheduled" => Ok(SessionStatus::Scheduled),
            "InProgress" => Ok(SessionStatus::InProgress),
            "Completed" => Ok(SessionStatus::Completed),
            "Cancelled" => Ok(SessionStatus::Cancelled),
            "NoShow" => Ok(SessionStatus::NoShow),
            _ => Err(soroban_sdk::ConversionError {}),
        }
    }
}

impl IntoVal<Env, Val> for SessionStatus {
    fn into_val(&self, env: &Env) -> Val {
        match self {
            SessionStatus::Scheduled => Symbol::new(env, "Scheduled").into_val(env),
            SessionStatus::InProgress => Symbol::new(env, "InProgress").into_val(env),
            SessionStatus::Completed => Symbol::new(env, "Completed").into_val(env),
            SessionStatus::Cancelled => Symbol::new(env, "Cancelled").into_val(env),
            SessionStatus::NoShow => Symbol::new(env, "NoShow").into_val(env),
        }
    }
}

impl TryFromVal<Env, Val> for EngagementLevel {
    type Error = soroban_sdk::ConversionError;

    fn try_from_val(env: &Env, val: &Val) -> Result<Self, Self::Error> {
        let symbol = Symbol::try_from_val(env, val)?;
        match symbol.to_string().as_str() {
            "Low" => Ok(EngagementLevel::Low),
            "Medium" => Ok(EngagementLevel::Medium),
            "High" => Ok(EngagementLevel::High),
            "VeryHigh" => Ok(EngagementLevel::VeryHigh),
            _ => Err(soroban_sdk::ConversionError {}),
        }
    }
}

impl IntoVal<Env, Val> for EngagementLevel {
    fn into_val(&self, env: &Env) -> Val {
        match self {
            EngagementLevel::Low => Symbol::new(env, "Low").into_val(env),
            EngagementLevel::Medium => Symbol::new(env, "Medium").into_val(env),
            EngagementLevel::High => Symbol::new(env, "High").into_val(env),
            EngagementLevel::VeryHigh => Symbol::new(env, "VeryHigh").into_val(env),
        }
    }
}

// PeerReview implementations
impl TryFromVal<Env, Val> for PeerReview {
    type Error = soroban_sdk::ConversionError;

    fn try_from_val(env: &Env, val: &Val) -> Result<Self, Self::Error> {
        let map = Map::<Val, Val>::try_from_val(env, val)?;
        Ok(PeerReview {
            id: u64::try_from_val(env, &map.get(Symbol::new(env, "id").into_val(env)).unwrap_or_default())?,
            reviewer: Address::try_from_val(env, &map.get(Symbol::new(env, "reviewer").into_val(env)).unwrap_or_default())?,
            reviewee: Address::try_from_val(env, &map.get(Symbol::new(env, "reviewee").into_val(env)).unwrap_or_default())?,
            content_type: ReviewContentType::try_from_val(env, &map.get(Symbol::new(env, "content_type").into_val(env)).unwrap_or_default())?,
            content_id: u64::try_from_val(env, &map.get(Symbol::new(env, "content_id").into_val(env)).unwrap_or_default())?,
            rating: u32::try_from_val(env, &map.get(Symbol::new(env, "rating").into_val(env)).unwrap_or_default())?,
            feedback: Bytes::try_from_val(env, &map.get(Symbol::new(env, "feedback").into_val(env)).unwrap_or_default())?,
            criteria: Map::<Bytes, u32>::try_from_val(env, &map.get(Symbol::new(env, "criteria").into_val(env)).unwrap_or_default())?,
            created_at: u64::try_from_val(env, &map.get(Symbol::new(env, "created_at").into_val(env)).unwrap_or_default())?,
            is_helpful: bool::try_from_val(env, &map.get(Symbol::new(env, "is_helpful").into_val(env)).unwrap_or_default())?,
            helpful_votes: u32::try_from_val(env, &map.get(Symbol::new(env, "helpful_votes").into_val(env)).unwrap_or_default())?,
        })
    }
}

impl IntoVal<Env, Val> for PeerReview {
    fn into_val(&self, env: &Env) -> Val {
        let mut map = Map::<Val, Val>::new(env);
        map.set(Symbol::new(env, "id").into_val(env), self.id.into_val(env));
        map.set(Symbol::new(env, "reviewer").into_val(env), self.reviewer.into_val(env));
        map.set(Symbol::new(env, "reviewee").into_val(env), self.reviewee.into_val(env));
        map.set(Symbol::new(env, "content_type").into_val(env), self.content_type.into_val(env));
        map.set(Symbol::new(env, "content_id").into_val(env), self.content_id.into_val(env));
        map.set(Symbol::new(env, "rating").into_val(env), self.rating.into_val(env));
        map.set(Symbol::new(env, "feedback").into_val(env), self.feedback.into_val(env));
        map.set(Symbol::new(env, "criteria").into_val(env), self.criteria.into_val(env));
        map.set(Symbol::new(env, "created_at").into_val(env), self.created_at.into_val(env));
        map.set(Symbol::new(env, "is_helpful").into_val(env), self.is_helpful.into_val(env));
        map.set(Symbol::new(env, "helpful_votes").into_val(env), self.helpful_votes.into_val(env));
        map.into_val(env)
    }
}

// ReviewContentType implementations
impl TryFromVal<Env, Val> for ReviewContentType {
    type Error = soroban_sdk::ConversionError;

    fn try_from_val(env: &Env, val: &Val) -> Result<Self, Self::Error> {
        let symbol = Symbol::try_from_val(env, val)?;
        match symbol.to_string().as_str() {
            "Submission" => Ok(ReviewContentType::Submission),
            "Comment" => Ok(ReviewContentType::Comment),
            "Project" => Ok(ReviewContentType::Project),
            "Tutorial" => Ok(ReviewContentType::Tutorial),
            "Resource" => Ok(ReviewContentType::Resource),
            _ => Err(soroban_sdk::ConversionError {}),
        }
    }
}

impl IntoVal<Env, Val> for ReviewContentType {
    fn into_val(&self, env: &Env) -> Val {
        match self {
            ReviewContentType::Submission => Symbol::new(env, "Submission").into_val(env),
            ReviewContentType::Comment => Symbol::new(env, "Comment").into_val(env),
            ReviewContentType::Project => Symbol::new(env, "Project").into_val(env),
            ReviewContentType::Tutorial => Symbol::new(env, "Tutorial").into_val(env),
            ReviewContentType::Resource => Symbol::new(env, "Resource").into_val(env),
        }
    }
}

// WorkspaceSettings implementations
impl TryFromVal<Env, Val> for WorkspaceSettings {
    type Error = soroban_sdk::ConversionError;

    fn try_from_val(env: &Env, val: &Val) -> Result<Self, Self::Error> {
        let map = Map::<Val, Val>::try_from_val(env, val)?;
        Ok(WorkspaceSettings {
            allow_public_view: bool::try_from_val(env, &map.get(Symbol::new(env, "allow_public_view").into_val(env)).unwrap_or_default())?,
            require_approval_to_join: bool::try_from_val(env, &map.get(Symbol::new(env, "require_approval_to_join").into_val(env)).unwrap_or_default())?,
            enable_chat: bool::try_from_val(env, &map.get(Symbol::new(env, "enable_chat").into_val(env)).unwrap_or_default())?,
            enable_video_calls: bool::try_from_val(env, &map.get(Symbol::new(env, "enable_video_calls").into_val(env)).unwrap_or_default())?,
            auto_save_interval: u64::try_from_val(env, &map.get(Symbol::new(env, "auto_save_interval").into_val(env)).unwrap_or_default())?,
        })
    }
}

impl IntoVal<Env, Val> for WorkspaceSettings {
    fn into_val(&self, env: &Env) -> Val {
        let mut map = Map::<Val, Val>::new(env);
        map.set(Symbol::new(env, "allow_public_view").into_val(env), self.allow_public_view.into_val(env));
        map.set(Symbol::new(env, "require_approval_to_join").into_val(env), self.require_approval_to_join.into_val(env));
        map.set(Symbol::new(env, "enable_chat").into_val(env), self.enable_chat.into_val(env));
        map.set(Symbol::new(env, "enable_video_calls").into_val(env), self.enable_video_calls.into_val(env));
        map.set(Symbol::new(env, "auto_save_interval").into_val(env), self.auto_save_interval.into_val(env));
        map.into_val(env)
    }
}

// CollaborationWorkspace implementations
impl TryFromVal<Env, Val> for CollaborationWorkspace {
    type Error = soroban_sdk::ConversionError;

    fn try_from_val(env: &Env, val: &Val) -> Result<Self, Self::Error> {
        let map = Map::<Val, Val>::try_from_val(env, val)?;
        Ok(CollaborationWorkspace {
            id: u64::try_from_val(env, &map.get(Symbol::new(env, "id").into_val(env)).unwrap_or_default())?,
            name: Bytes::try_from_val(env, &map.get(Symbol::new(env, "name").into_val(env)).unwrap_or_default())?,
            description: Bytes::try_from_val(env, &map.get(Symbol::new(env, "description").into_val(env)).unwrap_or_default())?,
            creator: Address::try_from_val(env, &map.get(Symbol::new(env, "creator").into_val(env)).unwrap_or_default())?,
            collaborators: Vec::<Address>::try_from_val(env, &map.get(Symbol::new(env, "collaborators").into_val(env)).unwrap_or_default())?,
            project_type: ProjectType::try_from_val(env, &map.get(Symbol::new(env, "project_type").into_val(env)).unwrap_or_default())?,
            status: WorkspaceStatus::try_from_val(env, &map.get(Symbol::new(env, "status").into_val(env)).unwrap_or_default())?,
            created_at: u64::try_from_val(env, &map.get(Symbol::new(env, "created_at").into_val(env)).unwrap_or_default())?,
            last_activity: u64::try_from_val(env, &map.get(Symbol::new(env, "last_activity").into_val(env)).unwrap_or_default())?,
            files: Vec::<WorkspaceFile>::try_from_val(env, &map.get(Symbol::new(env, "files").into_val(env)).unwrap_or_default())?,
            tasks: Vec::<WorkspaceTask>::try_from_val(env, &map.get(Symbol::new(env, "tasks").into_val(env)).unwrap_or_default())?,
            settings: WorkspaceSettings::try_from_val(env, &map.get(Symbol::new(env, "settings").into_val(env)).unwrap_or_default())?,
        })
    }
}

impl IntoVal<Env, Val> for CollaborationWorkspace {
    fn into_val(&self, env: &Env) -> Val {
        let mut map = Map::<Val, Val>::new(env);
        map.set(Symbol::new(env, "id").into_val(env), self.id.into_val(env));
        map.set(Symbol::new(env, "name").into_val(env), self.name.into_val(env));
        map.set(Symbol::new(env, "description").into_val(env), self.description.into_val(env));
        map.set(Symbol::new(env, "creator").into_val(env), self.creator.into_val(env));
        map.set(Symbol::new(env, "collaborators").into_val(env), self.collaborators.into_val(env));
        map.set(Symbol::new(env, "project_type").into_val(env), self.project_type.into_val(env));
        map.set(Symbol::new(env, "status").into_val(env), self.status.into_val(env));
        map.set(Symbol::new(env, "created_at").into_val(env), self.created_at.into_val(env));
        map.set(Symbol::new(env, "last_activity").into_val(env), self.last_activity.into_val(env));
        map.set(Symbol::new(env, "files").into_val(env), self.files.into_val(env));
        map.set(Symbol::new(env, "tasks").into_val(env), self.tasks.into_val(env));
        map.set(Symbol::new(env, "settings").into_val(env), self.settings.into_val(env));
        map.into_val(env)
    }
}

// ProjectType implementations
impl TryFromVal<Env, Val> for ProjectType {
    type Error = soroban_sdk::ConversionError;

    fn try_from_val(env: &Env, val: &Val) -> Result<Self, Self::Error> {
        let symbol = Symbol::try_from_val(env, val)?;
        match symbol.to_string().as_str() {
            "Study" => Ok(ProjectType::Study),
            "Research" => Ok(ProjectType::Research),
            "Assignment" => Ok(ProjectType::Assignment),
            "Tutorial" => Ok(ProjectType::Tutorial),
            "Discussion" => Ok(ProjectType::Discussion),
            _ => Err(soroban_sdk::ConversionError {}),
        }
    }
}

impl IntoVal<Env, Val> for ProjectType {
    fn into_val(&self, env: &Env) -> Val {
        match self {
            ProjectType::Study => Symbol::new(env, "Study").into_val(env),
            ProjectType::Research => Symbol::new(env, "Research").into_val(env),
            ProjectType::Assignment => Symbol::new(env, "Assignment").into_val(env),
            ProjectType::Tutorial => Symbol::new(env, "Tutorial").into_val(env),
            ProjectType::Discussion => Symbol::new(env, "Discussion").into_val(env),
        }
    }
}

// WorkspaceStatus implementations
impl TryFromVal<Env, Val> for WorkspaceStatus {
    type Error = soroban_sdk::ConversionError;

    fn try_from_val(env: &Env, val: &Val) -> Result<Self, Self::Error> {
        let symbol = Symbol::try_from_val(env, val)?;
        match symbol.to_string().as_str() {
            "Active" => Ok(WorkspaceStatus::Active),
            "Completed" => Ok(WorkspaceStatus::Completed),
            "Archived" => Ok(WorkspaceStatus::Archived),
            "Suspended" => Ok(WorkspaceStatus::Suspended),
            _ => Err(soroban_sdk::ConversionError {}),
        }
    }
}

impl IntoVal<Env, Val> for WorkspaceStatus {
    fn into_val(&self, env: &Env) -> Val {
        match self {
            WorkspaceStatus::Active => Symbol::new(env, "Active").into_val(env),
            WorkspaceStatus::Completed => Symbol::new(env, "Completed").into_val(env),
            WorkspaceStatus::Archived => Symbol::new(env, "Archived").into_val(env),
            WorkspaceStatus::Suspended => Symbol::new(env, "Suspended").into_val(env),
        }
    }
}

// WorkspaceFile implementations
impl TryFromVal<Env, Val> for WorkspaceFile {
    type Error = soroban_sdk::ConversionError;

    fn try_from_val(env: &Env, val: &Val) -> Result<Self, Self::Error> {
        let map = Map::<Val, Val>::try_from_val(env, val)?;
        Ok(WorkspaceFile {
            id: u64::try_from_val(env, &map.get(Symbol::new(env, "id").into_val(env)).unwrap_or_default())?,
            name: Bytes::try_from_val(env, &map.get(Symbol::new(env, "name").into_val(env)).unwrap_or_default())?,
            content_hash: Bytes::try_from_val(env, &map.get(Symbol::new(env, "content_hash").into_val(env)).unwrap_or_default())?,
            uploader: Address::try_from_val(env, &map.get(Symbol::new(env, "uploader").into_val(env)).unwrap_or_default())?,
            uploaded_at: u64::try_from_val(env, &map.get(Symbol::new(env, "uploaded_at").into_val(env)).unwrap_or_default())?,
            file_type: Bytes::try_from_val(env, &map.get(Symbol::new(env, "file_type").into_val(env)).unwrap_or_default())?,
            size: u64::try_from_val(env, &map.get(Symbol::new(env, "size").into_val(env)).unwrap_or_default())?,
            version: u32::try_from_val(env, &map.get(Symbol::new(env, "version").into_val(env)).unwrap_or_default())?,
        })
    }
}

impl IntoVal<Env, Val> for WorkspaceFile {
    fn into_val(&self, env: &Env) -> Val {
        let mut map = Map::<Val, Val>::new(env);
        map.set(Symbol::new(env, "id").into_val(env), self.id.into_val(env));
        map.set(Symbol::new(env, "name").into_val(env), self.name.into_val(env));
        map.set(Symbol::new(env, "content_hash").into_val(env), self.content_hash.into_val(env));
        map.set(Symbol::new(env, "uploader").into_val(env), self.uploader.into_val(env));
        map.set(Symbol::new(env, "uploaded_at").into_val(env), self.uploaded_at.into_val(env));
        map.set(Symbol::new(env, "file_type").into_val(env), self.file_type.into_val(env));
        map.set(Symbol::new(env, "size").into_val(env), self.size.into_val(env));
        map.set(Symbol::new(env, "version").into_val(env), self.version.into_val(env));
        map.into_val(env)
    }
}

// WorkspaceTask implementations
impl TryFromVal<Env, Val> for WorkspaceTask {
    type Error = soroban_sdk::ConversionError;

    fn try_from_val(env: &Env, val: &Val) -> Result<Self, Self::Error> {
        let map = Map::<Val, Val>::try_from_val(env, val)?;
        Ok(WorkspaceTask {
            id: u64::try_from_val(env, &map.get(Symbol::new(env, "id").into_val(env)).unwrap_or_default())?,
            title: Bytes::try_from_val(env, &map.get(Symbol::new(env, "title").into_val(env)).unwrap_or_default())?,
            description: Bytes::try_from_val(env, &map.get(Symbol::new(env, "description").into_val(env)).unwrap_or_default())?,
            assignee: Address::try_from_val(env, &map.get(Symbol::new(env, "assignee").into_val(env)).unwrap_or_default())?,
            creator: Address::try_from_val(env, &map.get(Symbol::new(env, "creator").into_val(env)).unwrap_or_default())?,
            due_date: u64::try_from_val(env, &map.get(Symbol::new(env, "due_date").into_val(env)).unwrap_or_default())?,
            status: TaskStatus::try_from_val(env, &map.get(Symbol::new(env, "status").into_val(env)).unwrap_or_default())?,
            priority: TaskPriority::try_from_val(env, &map.get(Symbol::new(env, "priority").into_val(env)).unwrap_or_default())?,
            created_at: u64::try_from_val(env, &map.get(Symbol::new(env, "created_at").into_val(env)).unwrap_or_default())?,
            completed_at: Option::<u64>::try_from_val(env, &map.get(Symbol::new(env, "completed_at").into_val(env)).unwrap_or_default())?,
        })
    }
}

impl IntoVal<Env, Val> for WorkspaceTask {
    fn into_val(&self, env: &Env) -> Val {
        let mut map = Map::<Val, Val>::new(env);
        map.set(Symbol::new(env, "id").into_val(env), self.id.into_val(env));
        map.set(Symbol::new(env, "title").into_val(env), self.title.into_val(env));
        map.set(Symbol::new(env, "description").into_val(env), self.description.into_val(env));
        map.set(Symbol::new(env, "assignee").into_val(env), self.assignee.into_val(env));
        map.set(Symbol::new(env, "creator").into_val(env), self.creator.into_val(env));
        map.set(Symbol::new(env, "due_date").into_val(env), self.due_date.into_val(env));
        map.set(Symbol::new(env, "status").into_val(env), self.status.into_val(env));
        map.set(Symbol::new(env, "priority").into_val(env), self.priority.into_val(env));
        map.set(Symbol::new(env, "created_at").into_val(env), self.created_at.into_val(env));
        map.set(Symbol::new(env, "completed_at").into_val(env), self.completed_at.into_val(env));
        map.into_val(env)
    }
}

// TaskStatus implementations
impl TryFromVal<Env, Val> for TaskStatus {
    type Error = soroban_sdk::ConversionError;

    fn try_from_val(env: &Env, val: &Val) -> Result<Self, Self::Error> {
        let symbol = Symbol::try_from_val(env, val)?;
        match symbol.to_string().as_str() {
            "Todo" => Ok(TaskStatus::Todo),
            "InProgress" => Ok(TaskStatus::InProgress),
            "Review" => Ok(TaskStatus::Review),
            "Completed" => Ok(TaskStatus::Completed),
            "Cancelled" => Ok(TaskStatus::Cancelled),
            _ => Err(soroban_sdk::ConversionError {}),
        }
    }
}

impl IntoVal<Env, Val> for TaskStatus {
    fn into_val(&self, env: &Env) -> Val {
        match self {
            TaskStatus::Todo => Symbol::new(env, "Todo").into_val(env),
            TaskStatus::InProgress => Symbol::new(env, "InProgress").into_val(env),
            TaskStatus::Review => Symbol::new(env, "Review").into_val(env),
            TaskStatus::Completed => Symbol::new(env, "Completed").into_val(env),
            TaskStatus::Cancelled => Symbol::new(env, "Cancelled").into_val(env),
        }
    }
}

// TaskPriority implementations
impl TryFromVal<Env, Val> for TaskPriority {
    type Error = soroban_sdk::ConversionError;

    fn try_from_val(env: &Env, val: &Val) -> Result<Self, Self::Error> {
        let symbol = Symbol::try_from_val(env, val)?;
        match symbol.to_string().as_str() {
            "Low" => Ok(TaskPriority::Low),
            "Medium" => Ok(TaskPriority::Medium),
            "High" => Ok(TaskPriority::High),
            "Urgent" => Ok(TaskPriority::Urgent),
            _ => Err(soroban_sdk::ConversionError {}),
        }
    }
}

impl IntoVal<Env, Val> for TaskPriority {
    fn into_val(&self, env: &Env) -> Val {
        match self {
            TaskPriority::Low => Symbol::new(env, "Low").into_val(env),
            TaskPriority::Medium => Symbol::new(env, "Medium").into_val(env),
            TaskPriority::High => Symbol::new(env, "High").into_val(env),
            TaskPriority::Urgent => Symbol::new(env, "Urgent").into_val(env),
        }
    }
}
