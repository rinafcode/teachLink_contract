use crate::types::UserReputation;
use soroban_sdk::{symbol_short, Address, Env, Symbol};

const BASIS_POINTS: u32 = 10000;
const REPUTATION: Symbol = symbol_short!("reptn");

pub fn update_participation(env: &Env, user: Address, points: u32) {
    user.require_auth();
    let mut reputation = get_reputation(env, &user);
    reputation.participation_score += points;
    reputation.last_update = env.ledger().timestamp();
    set_reputation(env, &user, &reputation);
}

pub fn update_course_progress(env: &Env, user: Address, is_completion: bool) {
    user.require_auth();
    let mut reputation = get_reputation(env, &user);

    if is_completion {
        reputation.total_courses_completed += 1;
        // Logic: You can't complete a course without starting it,
        // but simple increment here assumes course started logic handled elsewhere or previously
        if reputation.total_courses_started < reputation.total_courses_completed {
            reputation.total_courses_started = reputation.total_courses_completed;
        }
    } else {
        reputation.total_courses_started += 1;
    }

    if reputation.total_courses_started > 0 {
        reputation.completion_rate =
            (reputation.total_courses_completed * BASIS_POINTS) / reputation.total_courses_started;
    }

    reputation.last_update = env.ledger().timestamp();
    set_reputation(env, &user, &reputation);
}

pub fn rate_contribution(env: &Env, user: Address, rating: u32) {
    // Rating should be 0-5 scaled (e.g. 0-100 or 0-500)
    // Here assuming 0-5
    if rating > 5 {
        panic!("Rating must be between 0 and 5");
    }

    let mut reputation = get_reputation(env, &user);

    let current_total_quality = reputation.contribution_quality * reputation.total_contributions;
    reputation.total_contributions += 1;

    // Weighted Average
    reputation.contribution_quality =
        (current_total_quality + rating) / reputation.total_contributions;
    reputation.last_update = env.ledger().timestamp();

    set_reputation(env, &user, &reputation);
}

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

fn set_reputation(env: &Env, user: &Address, reputation: &UserReputation) {
    env.storage()
        .persistent()
        .set(&(REPUTATION, user.clone()), reputation);
}
