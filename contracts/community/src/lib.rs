#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Map, String, Symbol, Vec};

mod types;
mod errors;
mod storage;
mod test;

use crate::types::*;
use crate::errors::CommunityError;
use crate::storage::*;

#[contract]
pub struct CommunityContract;

#[contractimpl]
impl CommunityContract {
    /// Initialize the community contract
    pub fn initialize(env: Env, admin: Address, token: Address, fee_recipient: Address) -> Result<(), CommunityError> {
        if env.storage().instance().has(&CONFIG) {
            return Err(CommunityError::AlreadExists);
        }
        
        let config = CommunityConfig {
            admin,
            token,
            fee_recipient,
            min_reputation_for_moderation: 1000,
        };
        
        env.storage().instance().set(&CONFIG, &config);
        Ok(())
    }

    // ========== Forum Functions (Criterion 1) ==========

    pub fn create_forum_category(env: Env, admin: Address, name: Bytes, description: Bytes, is_private: bool, allowed_roles: Vec<CommunityRole>) -> Result<u64, CommunityError> {
        admin.require_auth();
        let config: CommunityConfig = env.storage().instance().get(&CONFIG).ok_or(CommunityError::Unauthorized)?;
        if admin != config.admin {
            return Err(CommunityError::Unauthorized);
        }
        
        let mut categories: Map<u64, ForumCategory> = env.storage().instance().get(&FORUM_CATEGORIES).unwrap_or(Map::new(&env));
        let next_id: u64 = env.storage().instance().get(&NEXT_CAT_ID).unwrap_or(1);
        
        let category = ForumCategory {
            id: next_id,
            name,
            description,
            is_private,
            allowed_roles,
        };
        
        categories.set(next_id, category);
        env.storage().instance().set(&FORUM_CATEGORIES, &categories);
        env.storage().instance().set(&NEXT_CAT_ID, &(next_id + 1));
        
        Ok(next_id)
    }

    pub fn create_forum_post(env: Env, author: Address, category_id: u64, title: Bytes, content: Bytes, tags: Vec<Bytes>) -> Result<u64, CommunityError> {
        author.require_auth();
        
        let categories: Map<u64, ForumCategory> = env.storage().instance().get(&FORUM_CATEGORIES).ok_or(CommunityError::ForumCategoryNotFound)?;
        if !categories.has(category_id) {
            return Err(CommunityError::ForumCategoryNotFound);
        }
        
        let mut posts: Map<u64, ForumPost> = env.storage().instance().get(&FORUM_POSTS).unwrap_or(Map::new(&env));
        let next_id: u64 = env.storage().instance().get(&NEXT_POST_ID).unwrap_or(1);
        
        let post = ForumPost {
            id: next_id,
            category_id,
            author: author.clone(),
            title,
            content,
            created_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
            tags,
            likes: 0,
            is_resolved: false,
            solution_post_id: None,
        };
        
        posts.set(next_id, post);
        env.storage().instance().set(&FORUM_POSTS, &posts);
        env.storage().instance().set(&NEXT_POST_ID, &(next_id + 1));
        
        // Update reputation
        Self::update_user_points(&env, author, 5);
        
        Ok(next_id)
    }

    pub fn create_forum_comment(env: Env, author: Address, post_id: u64, content: Bytes) -> Result<u64, CommunityError> {
        author.require_auth();
        
        let posts: Map<u64, ForumPost> = env.storage().instance().get(&FORUM_POSTS).ok_or(CommunityError::PostNotFound)?;
        if !posts.has(post_id) {
            return Err(CommunityError::PostNotFound);
        }
        
        let mut comments: Map<u64, ForumComment> = env.storage().instance().get(&FORUM_COMMENTS).unwrap_or(Map::new(&env));
        let next_id: u64 = env.storage().instance().get(&NEXT_COMM_ID).unwrap_or(1);
        
        let comment = ForumComment {
            id: next_id,
            post_id,
            author: author.clone(),
            content,
            created_at: env.ledger().timestamp(),
            likes: 0,
            is_solution: false,
        };
        
        comments.set(next_id, comment);
        env.storage().instance().set(&FORUM_COMMENTS, &comments);
        env.storage().instance().set(&NEXT_COMM_ID, &(next_id + 1));
        
        Self::update_user_points(&env, author, 2);
        
        Ok(next_id)
    }

    pub fn resolve_forum_post(env: Env, author: Address, post_id: u64, solution_comment_id: u64) -> Result<(), CommunityError> {
        author.require_auth();
        let mut posts: Map<u64, ForumPost> = env.storage().instance().get(&FORUM_POSTS).ok_or(CommunityError::PostNotFound)?;
        let mut post = posts.get(post_id).ok_or(CommunityError::PostNotFound)?;
        
        if post.author != author {
            return Err(CommunityError::Unauthorized);
        }
        
        post.is_resolved = true;
        post.solution_post_id = Some(solution_comment_id);
        posts.set(post_id, post);
        env.storage().instance().set(&FORUM_POSTS, &posts);
        
        // Reward solver - finding solver author
        let comments: Map<u64, ForumComment> = env.storage().instance().get(&FORUM_COMMENTS).unwrap_or(Map::new(&env));
        if let Some(comment) = comments.get(solution_comment_id) {
             Self::update_user_points(&env, comment.author, 50);
             Self::increment_help_resolved(&env, comment.author);
        }
        
        Ok(())
    }

    // ========== Mentorship Functions (Criterion 2) ==========

    pub fn create_mentorship_program(env: Env, mentor: Address, mentee: Address, topic: Bytes, duration_days: u32) -> Result<u64, CommunityError> {
        mentor.require_auth();
        
        let mut mentorships: Map<u64, MentorshipProgram> = env.storage().instance().get(&MENTORSHIPS).unwrap_or(Map::new(&env));
        let next_id: u64 = env.storage().instance().get(&NEXT_MENT_ID).unwrap_or(1);
        
        let mentorship = MentorshipProgram {
            id: next_id,
            mentor: mentor.clone(),
            mentee: mentee.clone(),
            topic,
            status: MentorshipStatus::Pending,
            created_at: env.ledger().timestamp(),
            duration_days,
        };
        
        mentorships.set(next_id, mentorship);
        env.storage().instance().set(&MENTORSHIPS, &mentorships);
        env.storage().instance().set(&NEXT_MENT_ID, &(next_id + 1));
        
        Ok(next_id)
    }

    pub fn update_mentorship_status(env: Env, mentor: Address, program_id: u64, status: MentorshipStatus) -> Result<(), CommunityError> {
        mentor.require_auth();
        let mut mentorships: Map<u64, MentorshipProgram> = env.storage().instance().get(&MENTORSHIPS).ok_or(CommunityError::MentorshipNotFound)?;
        let mut mentorship = mentorships.get(program_id).ok_or(CommunityError::MentorshipNotFound)?;
        
        if mentorship.mentor != mentor && mentorship.mentee != mentor {
            return Err(CommunityError::Unauthorized);
        }
        
        mentorship.status = status;
        mentorships.set(program_id, mentorship);
        env.storage().instance().set(&MENTORSHIPS, &mentorships);
        
        Ok(())
    }

    // ========== Knowledge Sharing Functions (Criterion 3) ==========

    pub fn share_knowledge(env: Env, author: Address, title: Bytes, content: Bytes, tags: Vec<Bytes>) -> Result<u64, CommunityError> {
        author.require_auth();
        
        let mut articles: Map<u64, KnowledgeArticle> = env.storage().instance().get(&ARTICLES).unwrap_or(Map::new(&env));
        let next_id: u64 = env.storage().instance().get(&NEXT_ART_ID).unwrap_or(1);
        
        let article = KnowledgeArticle {
            id: next_id,
            title,
            content,
            author: author.clone(),
            created_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
            tags,
            contribution_points: 20,
        };
        
        articles.set(next_id, article);
        env.storage().instance().set(&ARTICLES, &articles);
        env.storage().instance().set(&NEXT_ART_ID, &(next_id + 1));
        
        Self::update_user_points(&env, author, 20);
        
        Ok(next_id)
    }

    // ========== Gamification & Recognition (Criterion 4) ==========

    pub fn get_user_reputation(env: Env, user: Address) -> UserReputation {
        let reputations: Map<Address, UserReputation> = env.storage().instance().get(&REPUTATION).unwrap_or(Map::new(&env));
        reputations.get(user.clone()).unwrap_or(UserReputation {
            user,
            points: 0,
            level: 1,
            badges: Vec::new(&env),
            help_requests_resolved: 0,
            contributions_count: 0,
        })
    }

    fn update_user_points(env: &Env, user: Address, points_to_add: u64) {
        let mut reputations: Map<Address, UserReputation> = env.storage().instance().get(&REPUTATION).unwrap_or(Map::new(&env));
        let mut reputation = reputations.get(user.clone()).unwrap_or(UserReputation {
            user: user.clone(),
            points: 0,
            level: 1,
            badges: Vec::new(&env),
            help_requests_resolved: 0,
            contributions_count: 0,
        });
        
        reputation.points += points_to_add;
        reputation.level = (reputation.points / 100) as u32 + 1;
        
        reputations.set(user, reputation);
        env.storage().instance().set(&REPUTATION, &reputations);
    }

    fn increment_help_resolved(env: &Env, user: Address) {
        let mut reputations: Map<Address, UserReputation> = env.storage().instance().get(&REPUTATION).unwrap_or(Map::new(&env));
        let mut reputation = reputations.get(user.clone()).unwrap_or(UserReputation {
            user: user.clone(),
            points: 0,
            level: 1,
            badges: Vec::new(&env),
            help_requests_resolved: 0,
            contributions_count: 0,
        });
        
        reputation.help_requests_resolved += 1;
        
        reputations.set(user, reputation);
        env.storage().instance().set(&REPUTATION, &reputations);
    }

    // ========== Analytics Functions (Criterion 5) ==========

    pub fn get_community_summary(env: Env) -> Map<Symbol, u64> {
        let mut summary = Map::new(&env);
        summary.set(Symbol::new(&env, "categories"), env.storage().instance().get(&NEXT_CAT_ID).unwrap_or(1) - 1);
        summary.set(Symbol::new(&env, "posts"), env.storage().instance().get(&NEXT_POST_ID).unwrap_or(1) - 1);
        summary.set(Symbol::new(&env, "articles"), env.storage().instance().get(&NEXT_ART_ID).unwrap_or(1) - 1);
        summary.set(Symbol::new(&env, "mentorships"), env.storage().instance().get(&NEXT_MENT_ID).unwrap_or(1) - 1);
        summary.set(Symbol::new(&env, "events"), env.storage().instance().get(&NEXT_EV_ID).unwrap_or(1) - 1);
        summary.set(Symbol::new(&env, "feedback"), env.storage().instance().get(&NEXT_FEED_ID).unwrap_or(1) - 1);
        summary
    }

    // ========== Moderation and Governance (Criterion 6) ==========

    pub fn take_moderation_action(env: Env, moderator: Address, target_user: Address, action: ModerationType, reason: Bytes) -> Result<(), CommunityError> {
        moderator.require_auth();
        let config: CommunityConfig = env.storage().instance().get(&CONFIG).ok_or(CommunityError::Unauthorized)?;
        let moderator_reputation = Self::get_user_reputation(env.clone(), moderator.clone());
        
        if moderator != config.admin && moderator_reputation.points < config.min_reputation_for_moderation as u64 {
            return Err(CommunityError::Unauthorized);
        }
        
        let mut moderation_actions: Vec<ModerationAction> = env.storage().instance().get(&MODERATION).unwrap_or(Vec::new(&env));
        moderation_actions.push_back(ModerationAction {
            moderator,
            target_user,
            action_type: action,
            reason,
            timestamp: env.ledger().timestamp(),
        });
        
        env.storage().instance().set(&MODERATION, &moderation_actions);
        Ok(())
    }

    // ========== Community Events (Criterion 7) ==========

    pub fn create_event(env: Env, organizer: Address, title: Bytes, description: Bytes, start_time: u64, end_time: u64, capacity: u32, event_type: EventType) -> Result<u64, CommunityError> {
        organizer.require_auth();
        
        let mut events: Map<u64, CommunityEvent> = env.storage().instance().get(&EVENTS).unwrap_or(Map::new(&env));
        let next_id: u64 = env.storage().instance().get(&NEXT_EV_ID).unwrap_or(1);
        
        let event = CommunityEvent {
            id: next_id,
            title,
            description,
            organizer: organizer.clone(),
            start_time,
            end_time,
            participants: Vec::new(&env),
            capacity,
            event_type,
        };
        
        events.set(next_id, event);
        env.storage().instance().set(&EVENTS, &events);
        env.storage().instance().set(&NEXT_EV_ID, &(next_id + 1));
        
        Ok(next_id)
    }

    pub fn join_event(env: Env, user: Address, event_id: u64) -> Result<(), CommunityError> {
        user.require_auth();
        let mut events: Map<u64, CommunityEvent> = env.storage().instance().get(&EVENTS).ok_or(CommunityError::EventNotFound)?;
        let mut event = events.get(event_id).ok_or(CommunityError::EventNotFound)?;
        
        if event.participants.len() >= event.capacity {
            return Err(CommunityError::EventFull);
        }
        
        event.participants.push_back(user);
        events.set(event_id, event);
        env.storage().instance().set(&EVENTS, &events);
        
        Ok(())
    }

    // ========== Feedback and Improvement (Criterion 8) ==========

    pub fn submit_feedback(env: Env, user: Address, category: FeedbackCategory, content: Bytes, rating: u32) -> Result<u64, CommunityError> {
        user.require_auth();
        if rating < 1 || rating > 5 {
            return Err(CommunityError::InvalidRating);
        }
        
        let mut feedbacks: Map<u64, CommunityFeedback> = env.storage().instance().get(&FEEDBACK).unwrap_or(Map::new(&env));
        let next_id: u64 = env.storage().instance().get(&NEXT_FEED_ID).unwrap_or(1);
        
        let feedback = CommunityFeedback {
            id: next_id,
            user,
            category,
            content,
            rating,
            status: FeedbackStatus::New,
        };
        
        feedbacks.set(next_id, feedback);
        env.storage().instance().set(&FEEDBACK, &feedbacks);
        env.storage().instance().set(&NEXT_FEED_ID, &(next_id + 1));
        
        Ok(next_id)
    }
}
