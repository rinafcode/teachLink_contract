//! TeachLink Documentation Contract
//!
//! Manages documentation articles, FAQs, and versioning for the TeachLink platform.
//! Functionality is split across focused modules:
//!
//! - [`types`]      — shared data structures
//! - [`storage`]    — storage key definitions
//! - [`articles`]   — article CRUD and analytics
//! - [`faq`]        — FAQ CRUD
//! - [`versioning`] — global documentation versioning

#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

mod articles;
mod faq;
mod storage;
mod types;
mod versioning;

pub use types::*;

/// Main documentation contract — delegates to focused sub-modules.
#[contract]
pub struct DocumentationContract;

#[contractimpl]
impl DocumentationContract {
    // ── Articles ──────────────────────────────────────────────────────────────

    /// Create a new documentation article.
    pub fn create_article(
        env: Env,
        id: String,
        title: String,
        content: String,
        category: DocCategory,
        language: String,
        tags: Vec<String>,
        visibility: Visibility,
        author: Address,
    ) -> Article {
        articles::ArticleManager::create(
            &env, id, title, content, category, language, tags, visibility, author,
        )
    }

    /// Get an article by ID.
    pub fn get_article(env: Env, id: String) -> Article {
        articles::ArticleManager::get(&env, id)
    }

    /// Update title, content, and tags of an existing article.
    pub fn update_article(
        env: Env,
        id: String,
        title: String,
        content: String,
        tags: Vec<String>,
    ) -> Article {
        articles::ArticleManager::update(&env, id, title, content, tags)
    }

    /// Record a view for analytics.
    pub fn record_view(env: Env, article_id: String) {
        articles::ArticleManager::record_view(&env, article_id);
    }

    /// Mark an article as helpful.
    pub fn mark_helpful(env: Env, article_id: String) {
        articles::ArticleManager::mark_helpful(&env, article_id);
    }

    /// Return total article count.
    pub fn get_article_count(env: Env) -> u64 {
        articles::ArticleManager::count(&env)
    }

    // ── FAQ ───────────────────────────────────────────────────────────────────

    /// Create a new FAQ entry.
    pub fn create_faq(
        env: Env,
        id: String,
        question: String,
        answer: String,
        category: String,
        language: String,
        author: Address,
    ) -> FaqEntry {
        faq::FaqManager::create(&env, id, question, answer, category, language, author)
    }

    /// Get a FAQ entry by ID.
    pub fn get_faq(env: Env, id: String) -> FaqEntry {
        faq::FaqManager::get(&env, id)
    }

    /// Return total FAQ count.
    pub fn get_faq_count(env: Env) -> u64 {
        faq::FaqManager::count(&env)
    }

    // ── Search (placeholder) ──────────────────────────────────────────────────

    /// Search articles by keyword (placeholder — returns empty vec).
    pub fn search_articles(env: Env, _query: String) -> Vec<Article> {
        Vec::new(&env)
    }

    // ── Versioning ────────────────────────────────────────────────────────────

    /// Get current documentation version.
    pub fn get_version(env: Env) -> u32 {
        versioning::Versioning::get(&env)
    }

    /// Update documentation version.
    pub fn update_version(env: Env, version: u32) {
        versioning::Versioning::update(&env, version);
    }
}
