//! User Reputation Module
//!
//! Tracks and updates the on-chain reputation of TeachLink users based on
//! their learning activity and content contributions.
//!
//! # Reputation Components
//!
//! | Field                  | Description                                      |
//! |------------------------|--------------------------------------------------|
//! | `participation_score`  | Cumulative points from platform interactions     |
//! | `completion_rate`      | Course completion ratio in basis points (0–10000)|
//! | `contribution_quality` | Running average rating of submitted content (0–5)|
//! | `total_courses_started`| Total courses the user has enrolled in           |
//! | `total_courses_completed`| Total courses the user has finished            |
//! | `total_contributions`  | Total content items rated                        |
//!
//! # Completion Rate Algorithm
//!
//! Stored in basis points (10 000 = 100 %) to avoid floating-point arithmetic:
//!
//! ```text
//! completion_rate = (total_courses_completed * 10_000) / total_courses_started
//! ```
//!
//! Recalculated on every `update_course_progress` call.  A guard ensures
//! `total_courses_started >= total_courses_completed` to prevent impossible
//! states when a completion event arrives before the corresponding start event.
//!
//! # Contribution Quality Algorithm
//!
//! Uses a cumulative running average (Welford-style without variance):
//!
//! ```text
//! new_quality = (old_quality * old_count + new_rating) / new_count
//! ```
//!
//! This avoids storing the full rating history while keeping the average
//! accurate.  Ratings are on a 0–5 integer scale.
//!
//! # Storage
//!
//! Reputation data is stored in **persistent** storage (survives ledger
//! expiry) keyed by `(REPUTATION, user_address)`.  This is intentional:
//! reputation must not be lost due to TTL expiry.
//!
//! # TODO
//! - Add a `credit_score` field derived from all three components with
//!   configurable weights (see `score.rs`).
//! - Consider capping `participation_score` to prevent overflow on very
//!   active users (currently unbounded `u32`).

use crate::events::{
    ContributionRatedEvent, CourseProgressUpdatedEvent, ParticipationUpdatedEvent,
};
use crate::types::UserReputation;
use soroban_sdk::{symbol_short, Address, Env, Symbol};

/// Basis points divisor used for completion rate calculation (10 000 = 100 %).
const BASIS_POINTS: u32 = 10000;

/// Persistent storage key for user reputation records.
const REPUTATION: Symbol = symbol_short!("reptn");

/// Adds participation points to a user's reputation score.
///
/// Called by platform modules (e.g., forum posts, event attendance) to reward
/// active engagement.  Points are additive and never decay automatically.
///
/// # TODO
/// - Implement a time-decay factor so stale participation scores don't
///   permanently dominate the credit score calculation.
pub fn update_participation(env: &Env, user: Address, points: u32) {
    user.require_auth();
    let mut reputation = get_reputation(env, &user);
    reputation.participation_score += points;
    reputation.last_update = env.ledger().timestamp();
    set_reputation(env, &user, &reputation);

    // Emit event
    ParticipationUpdatedEvent {
        user: user.clone(),
        points_added: points,
        new_participation_score: reputation.participation_score,
        updated_at: env.ledger().timestamp(),
    }
    .publish(env);
}

/// Updates a user's course progress and recalculates their completion rate.
///
/// # Algorithm
///
/// - `is_completion = false`: increments `total_courses_started`.
/// - `is_completion = true`: increments `total_courses_completed`.
///   A guard ensures `total_courses_started >= total_courses_completed`
///   to handle out-of-order events (e.g., completion recorded before start).
///
/// Completion rate is recalculated after every update:
///
/// ```text
/// completion_rate = (total_courses_completed * 10_000) / total_courses_started
/// ```
///
/// Stored in basis points to avoid floating-point arithmetic on-chain.
///
/// # TODO
/// - Distinguish between "dropped" and "in-progress" courses so the
///   completion rate reflects genuine finishes, not just enrollments.
pub fn update_course_progress(env: &Env, user: Address, is_completion: bool) {
    user.require_auth();
    let mut reputation = get_reputation(env, &user);

    if is_completion {
        reputation.total_courses_completed += 1;
        // Guard: completion cannot exceed starts (handles out-of-order events).
        // The course-started logic is assumed to be handled elsewhere or previously.
        if reputation.total_courses_started < reputation.total_courses_completed {
            reputation.total_courses_started = reputation.total_courses_completed;
        }
    } else {
        reputation.total_courses_started += 1;
    }

    // Recalculate completion rate in basis points.
    if reputation.total_courses_started > 0 {
        reputation.completion_rate =
            (reputation.total_courses_completed * BASIS_POINTS) / reputation.total_courses_started;
    }

    reputation.last_update = env.ledger().timestamp();
    set_reputation(env, &user, &reputation);

    // Emit event
    CourseProgressUpdatedEvent {
        user: user.clone(),
        total_courses_started: reputation.total_courses_started,
        total_courses_completed: reputation.total_courses_completed,
        completion_rate: reputation.completion_rate,
        updated_at: env.ledger().timestamp(),
    }
    .publish(env);
}

/// Records a quality rating for a user's contribution and updates their
/// running average contribution quality score.
///
/// # Algorithm
///
/// Uses a cumulative running average to avoid storing the full rating history:
///
/// ```text
/// new_quality = (old_quality * old_count + new_rating) / new_count
/// ```
///
/// This is equivalent to the arithmetic mean of all ratings but requires
/// only O(1) storage.
///
/// # Parameters
/// - `rating` – Integer rating on a 0–5 scale.  Panics if > 5.
///
/// # Note
/// The caller is the rater, not the rated user.  The `user` parameter is the
/// content author whose reputation is being updated.  There is currently no
/// check preventing self-rating — this should be enforced at the call site.
///
/// # TODO
/// - Add a `rater: Address` parameter and enforce `rater != user` to prevent
///   self-rating abuse.
/// - Consider a weighted average where ratings from high-reputation users
///   carry more weight.
pub fn rate_contribution(env: &Env, user: Address, rating: u32) {
    // Rating must be on the 0–5 integer scale.
    assert!(rating <= 5, "Rating must be between 0 and 5");

    let mut reputation = get_reputation(env, &user);

    // Compute the new running average: (old_total + new_rating) / new_count.
    let current_total_quality = reputation.contribution_quality * reputation.total_contributions;
    reputation.total_contributions += 1;
    reputation.contribution_quality =
        (current_total_quality + rating) / reputation.total_contributions;
    reputation.last_update = env.ledger().timestamp();

    set_reputation(env, &user, &reputation);

    // Emit event
    ContributionRatedEvent {
        user: user.clone(),
        rating,
        new_contribution_quality: reputation.contribution_quality,
        total_contributions: reputation.total_contributions,
        rated_at: env.ledger().timestamp(),
    }
    .publish(env);
}

/// Retrieves a user's reputation record from persistent storage.
///
/// Returns a zeroed `UserReputation` for users who have no record yet,
/// so callers can safely read-modify-write without an existence check.
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

/// Persists a user's reputation record to storage.
///
/// Uses persistent storage so reputation survives ledger TTL expiry.
/// The composite key `(REPUTATION, user)` namespaces records per user.
fn set_reputation(env: &Env, user: &Address, reputation: &UserReputation) {
    env.storage()
        .persistent()
        .set(&(REPUTATION, user.clone()), reputation);
}
