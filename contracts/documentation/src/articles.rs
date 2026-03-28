//! Article management: create, retrieve, update, and analytics for documentation articles.

use soroban_sdk::{Address, Env, String, Vec};

use crate::storage::DocKey;
use crate::types::{Article, DocCategory, Visibility};

pub struct ArticleManager;

impl ArticleManager {
    /// Create a new documentation article.
    pub fn create(
        env: &Env,
        id: String,
        title: String,
        content: String,
        category: DocCategory,
        language: String,
        tags: Vec<String>,
        visibility: Visibility,
        author: Address,
    ) -> Article {
        let timestamp = env.ledger().timestamp();

        let article = Article {
            id: id.clone(),
            title,
            content,
            category,
            language,
            version: 1,
            author,
            visibility,
            tags,
            created_at: timestamp,
            updated_at: timestamp,
            view_count: 0,
            helpful_count: 0,
        };

        env.storage().instance().set(&id, &article);

        let count: u64 = env
            .storage()
            .instance()
            .get(&DocKey::ArticleCount)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DocKey::ArticleCount, &(count + 1));

        article
    }

    /// Retrieve an article by ID.
    pub fn get(env: &Env, id: String) -> Article {
        env.storage().instance().get(&id).unwrap()
    }

    /// Update title, content, and tags of an existing article.
    pub fn update(
        env: &Env,
        id: String,
        title: String,
        content: String,
        tags: Vec<String>,
    ) -> Article {
        let mut article: Article = env.storage().instance().get(&id).unwrap();

        article.title = title;
        article.content = content;
        article.tags = tags;
        article.version += 1;
        article.updated_at = env.ledger().timestamp();

        env.storage().instance().set(&id, &article);
        article
    }

    /// Increment the view counter for an article.
    pub fn record_view(env: &Env, article_id: String) {
        let mut article: Article = env.storage().instance().get(&article_id).unwrap();
        article.view_count += 1;
        env.storage().instance().set(&article_id, &article);
    }

    /// Increment the helpful counter for an article.
    pub fn mark_helpful(env: &Env, article_id: String) {
        let mut article: Article = env.storage().instance().get(&article_id).unwrap();
        article.helpful_count += 1;
        env.storage().instance().set(&article_id, &article);
    }

    /// Return total number of articles stored.
    pub fn count(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DocKey::ArticleCount)
            .unwrap_or(0)
    }
}
