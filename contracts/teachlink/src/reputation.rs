//! User reputation tracking.
//!
//! Responsibilities:
//! - Track participation score, course progress, and contribution quality
//! - Compute completion rate in basis points
//! - Emit events on every state change
//! - Expose read-only views for reputation data

use crate::events::{
    ContributionRatedEvent, CourseProgressUpdatedEvent, ParticipationUpdatedEvent,
};
use crate::types::UserReputation;
use soroban_sdk::{symbol_short, Address, Env, Symbol};

const BASIS_POINTS: u32 = 10000;
const REPUTATION: Symbol = symbol_short!("reptn");

/// Manages user reputation scores.
pub struct ReputationManager;

impl ReputationManager {
    // ===== Mutations =====

    /// Add participation points for a user.
    pub fn update_participation(env: &Env, user: Address, points: u32) {
        user.require_auth();
        let mut reputation = Self::get_reputation(env, &user);
        reputation.participation_score += points;
        reputation.last_update = env.ledger().timestamp();
        Self::set_reputation(env, &user, &reputation);

        ParticipationUpdatedEvent {
            user: user.clone(),
            points_added: points,
            new_participation_score: reputation.participation_score,
            updated_at: reputation.last_update,
        }
        .publish(env);
    }

    /// Record a course start or completion for a user.
    pub fn update_course_progress(env: &Env, user: Address, is_completion: bool) {
        user.require_auth();
        let mut reputation = Self::get_reputation(env, &user);

        if is_completion {
            reputation.total_courses_completed += 1;
            if reputation.total_courses_started < reputation.total_courses_completed {
                reputation.total_courses_started = reputation.total_courses_completed;
            }
        } else {
            reputation.total_courses_started += 1;
        }

        if reputation.total_courses_started > 0 {
            reputation.completion_rate = (reputation.total_courses_completed * BASIS_POINTS)
                / reputation.total_courses_started;
        }

        reputation.last_update = env.ledger().timestamp();
        Self::set_reputation(env, &user, &reputation);

        CourseProgressUpdatedEvent {
            user: user.clone(),
            total_courses_started: reputation.total_courses_started,
            total_courses_completed: reputation.total_courses_completed,
            completion_rate: reputation.completion_rate,
            updated_at: reputation.last_update,
        }
        .publish(env);
    }

    /// Record a contribution rating (0–5) for a user.
    pub fn rate_contribution(env: &Env, user: Address, rating: u32) {
        assert!(rating <= 5, "Rating must be between 0 and 5");

        let mut reputation = Self::get_reputation(env, &user);
        let current_total_quality = reputation.contribution_quality * reputation.total_contributions;
        reputation.total_contributions += 1;
        reputation.contribution_quality =
            (current_total_quality + rating) / reputation.total_contributions;
        reputation.last_update = env.ledger().timestamp();
        Self::set_reputation(env, &user, &reputation);

        ContributionRatedEvent {
            user: user.clone(),
            rating,
            new_contribution_quality: reputation.contribution_quality,
            total_contributions: reputation.total_contributions,
            rated_at: reputation.last_update,
        }
        .publish(env);
    }

    // ===== Queries =====

    /// Return the reputation record for a user, defaulting to zeroes.
    #[must_use]
    pub fn get_reputation(env: &Env, user: &Address) -> UserReputation {
        env.storage()
            .persistent()
            .get(&(REPUTATION, user.clone()))
            .unwrap_or(UserReputation {
                participation_score: 0,
                completion_rate: 0,
                contribution_quality: 0,
                total_courses_started: 0,
                total_courses_completed: 0,
                total_contributions: 0,
                last_update: 0,
            })
    }

    // ===== Internal =====

    fn set_reputation(env: &Env, user: &Address, reputation: &UserReputation) {
        env.storage()
            .persistent()
            .set(&(REPUTATION, user.clone()), reputation);
    }
}
