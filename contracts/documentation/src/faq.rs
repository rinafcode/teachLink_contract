//! FAQ management: create and retrieve FAQ entries.

use soroban_sdk::{Address, Env, String};

use crate::storage::DocKey;
use crate::types::FaqEntry;

pub struct FaqManager;

impl FaqManager {
    /// Create a new FAQ entry.
    pub fn create(
        env: &Env,
        id: String,
        question: String,
        answer: String,
        category: String,
        language: String,
        author: Address,
    ) -> FaqEntry {
        let timestamp = env.ledger().timestamp();

        let faq = FaqEntry {
            id: id.clone(),
            question,
            answer,
            category,
            language,
            author,
            created_at: timestamp,
            updated_at: timestamp,
            helpful_count: 0,
        };

        env.storage().instance().set(&id, &faq);

        let count: u64 = env
            .storage()
            .instance()
            .get(&DocKey::FaqCount)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DocKey::FaqCount, &(count + 1));

        faq
    }

    /// Retrieve a FAQ entry by ID.
    pub fn get(env: &Env, id: String) -> FaqEntry {
        env.storage().instance().get(&id).unwrap()
    }

    /// Return total number of FAQ entries stored.
    pub fn count(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DocKey::FaqCount)
            .unwrap_or(0)
    }
}
