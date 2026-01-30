use crate::events::{ContributionRecordedEvent, CourseCompletedEvent, CreditScoreUpdatedEvent};
use crate::storage::{CONTRIBUTIONS, COURSE_COMPLETIONS, CREDIT_SCORE};
use crate::types::{Contribution, ContributionType};
use soroban_sdk::{Address, Bytes, Env, Vec};

pub struct ScoreManager;

impl ScoreManager {
    /// Update the user's score by adding points
    pub fn update_score(env: &Env, user: Address, points: u64) {
        // Use a tuple key (CREDIT_SCORE, user) for mapping user to score
        let key = (CREDIT_SCORE, user.clone());
        let current_score: u64 = env.storage().persistent().get(&key).unwrap_or(0);
        let new_score = current_score + points;
        env.storage().persistent().set(&key, &new_score);

        CreditScoreUpdatedEvent { user, new_score }.publish(env);
    }

    /// Record a course completion and award points
    pub fn record_course_completion(env: &Env, user: Address, course_id: u64, points: u64) {
        let key = (COURSE_COMPLETIONS, user.clone());
        let mut completed_courses: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));

        // Avoid duplicate points for the same course
        if completed_courses.contains(course_id) {
            return; // Already completed
        }

        completed_courses.push_back(course_id);
        env.storage().persistent().set(&key, &completed_courses);

        // Update score internally
        Self::update_score(env, user.clone(), points);

        CourseCompletedEvent {
            user,
            course_id,
            points,
        }
        .publish(env);
    }

    /// Record a contribution and award points
    pub fn record_contribution(
        env: &Env,
        user: Address,
        c_type: ContributionType,
        description: Bytes,
        points: u64,
    ) {
        let key = (CONTRIBUTIONS, user.clone());
        let mut contributions: Vec<Contribution> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));

        let contribution = Contribution {
            contributor: user.clone(),
            c_type: c_type.clone(),
            description,
            timestamp: env.ledger().timestamp(),
            points,
        };

        contributions.push_back(contribution);
        env.storage().persistent().set(&key, &contributions);

        // Update score internally
        Self::update_score(env, user.clone(), points);

        ContributionRecordedEvent {
            user,
            c_type,
            points,
        }
        .publish(env);
    }

    /// Get the user's current credit score
    pub fn get_score(env: &Env, user: Address) -> u64 {
        env.storage()
            .persistent()
            .get(&(CREDIT_SCORE, user))
            .unwrap_or(0)
    }

    /// Get valid course completions
    pub fn get_courses(env: &Env, user: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&(COURSE_COMPLETIONS, user))
            .unwrap_or(Vec::new(env))
    }

    /// Get user contributions
    pub fn get_contributions(env: &Env, user: Address) -> Vec<Contribution> {
        env.storage()
            .persistent()
            .get(&(CONTRIBUTIONS, user))
            .unwrap_or(Vec::new(env))
    }
}
