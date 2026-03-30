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

#[cfg(test)]
mod tests {
    use super::{
        Article, DocCategory, DocKey, DocumentationContract, DocumentationContractClient, FaqEntry,
        Visibility,
    };
    use soroban_sdk::{
        testutils::{Address as _, Ledger, LedgerInfo},
        vec, Address, Env, String, Vec,
    };

    fn setup() -> (Env, DocumentationContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set(LedgerInfo {
            timestamp: 1_000,
            protocol_version: 25,
            sequence_number: 10,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 10,
            min_persistent_entry_ttl: 10,
            max_entry_ttl: 2_000_000,
        });

        let contract_id = env.register(DocumentationContract, ());
        let client = DocumentationContractClient::new(&env, &contract_id);

        (env, client, contract_id)
    }

    fn article_input(
        env: &Env,
        id: &str,
        title: &str,
        content: &str,
    ) -> (String, String, String, String, Vec<String>, Address) {
        (
            String::from_str(env, id),
            String::from_str(env, title),
            String::from_str(env, content),
            String::from_str(env, "en"),
            vec![env],
            Address::generate(env),
        )
    }

    fn seed_article(env: &Env, client: &DocumentationContractClient<'_>, id: &str) -> Article {
        let (id, title, content, language, tags, author) =
            article_input(env, id, "Getting Started", "Initial content");

        client.create_article(
            &id,
            &title,
            &content,
            &DocCategory::Guide,
            &language,
            &tags,
            &Visibility::Public,
            &author,
        )
    }

    #[test]
    fn empty_state_defaults_are_stable() {
        let (env, client, _) = setup();

        assert_eq!(client.get_article_count(), 0);
        assert_eq!(client.get_faq_count(), 0);
        assert_eq!(client.get_version(), 1);
        assert_eq!(
            client
                .search_articles(&String::from_str(&env, "rust"))
                .len(),
            0
        );
    }

    #[test]
    fn create_article_supports_empty_fields_and_tags() {
        let (env, client, _) = setup();
        let author = Address::generate(&env);
        let empty_tags: Vec<String> = Vec::new(&env);

        let article = client.create_article(
            &String::from_str(&env, "empty-article"),
            &String::from_str(&env, ""),
            &String::from_str(&env, ""),
            &DocCategory::KnowledgeBase,
            &String::from_str(&env, ""),
            &empty_tags,
            &Visibility::Community,
            &author,
        );

        assert_eq!(article.version, 1);
        assert_eq!(article.view_count, 0);
        assert_eq!(article.helpful_count, 0);
        assert_eq!(article.tags.len(), 0);
        assert_eq!(client.get_article_count(), 1);
    }

    #[test]
    fn duplicate_article_id_does_not_inflate_count() {
        let (env, client, _) = setup();
        seed_article(&env, &client, "guide-1");
        let original_timestamp = client
            .get_article(&String::from_str(&env, "guide-1"))
            .created_at;

        env.ledger().with_mut(|ledger| {
            ledger.timestamp += 5;
        });

        let (_, title, content, language, tags, author) =
            article_input(&env, "guide-1", "Updated Title", "Updated content");
        let replacement = client.create_article(
            &String::from_str(&env, "guide-1"),
            &title,
            &content,
            &DocCategory::Tutorial,
            &language,
            &tags,
            &Visibility::Private,
            &author,
        );

        assert_eq!(client.get_article_count(), 1);
        assert_eq!(replacement.title, String::from_str(&env, "Updated Title"));
        assert_eq!(replacement.created_at, original_timestamp + 5);
    }

    #[test]
    fn duplicate_faq_id_does_not_inflate_count() {
        let (env, client, _) = setup();
        let author = Address::generate(&env);

        client.create_faq(
            &String::from_str(&env, "faq-1"),
            &String::from_str(&env, "What is TeachLink?"),
            &String::from_str(&env, "A platform."),
            &String::from_str(&env, "general"),
            &String::from_str(&env, "en"),
            &author,
        );

        let replacement: FaqEntry = client.create_faq(
            &String::from_str(&env, "faq-1"),
            &String::from_str(&env, "What changed?"),
            &String::from_str(&env, "The answer."),
            &String::from_str(&env, "general"),
            &String::from_str(&env, "en"),
            &author,
        );

        assert_eq!(client.get_faq_count(), 1);
        assert_eq!(
            replacement.question,
            String::from_str(&env, "What changed?")
        );
    }

    #[test]
    fn repeated_view_and_helpful_updates_remain_consistent() {
        let (env, client, _) = setup();
        seed_article(&env, &client, "guide-2");
        let article_id = String::from_str(&env, "guide-2");

        for _ in 0..25 {
            client.record_view(&article_id);
        }

        for _ in 0..10 {
            client.mark_helpful(&article_id);
        }

        let article = client.get_article(&article_id);
        assert_eq!(article.view_count, 25);
        assert_eq!(article.helpful_count, 10);
        assert_eq!(article.version, 1);
    }

    #[test]
    fn update_article_increments_version_and_preserves_created_at() {
        let (env, client, _) = setup();
        let original = seed_article(&env, &client, "guide-3");

        env.ledger().with_mut(|ledger| {
            ledger.timestamp += 20;
        });

        let updated = client.update_article(
            &String::from_str(&env, "guide-3"),
            &String::from_str(&env, "Guide v2"),
            &String::from_str(&env, "New content"),
            &vec![&env, String::from_str(&env, "advanced")],
        );

        assert_eq!(updated.version, 2);
        assert_eq!(updated.created_at, original.created_at);
        assert_eq!(updated.updated_at, original.updated_at + 20);
    }

    #[test]
    #[should_panic(expected = "documentation article count overflow")]
    fn create_article_panics_when_article_count_overflows() {
        let (env, client, contract_id) = setup();

        env.as_contract(&contract_id, || {
            env.storage()
                .instance()
                .set(&DocKey::ArticleCount, &u64::MAX);
        });

        let (id, title, content, language, tags, author) =
            article_input(&env, "overflow-article", "A", "B");
        client.create_article(
            &id,
            &title,
            &content,
            &DocCategory::Guide,
            &language,
            &tags,
            &Visibility::Public,
            &author,
        );
    }

    #[test]
    #[should_panic(expected = "documentation faq count overflow")]
    fn create_faq_panics_when_faq_count_overflows() {
        let (env, client, contract_id) = setup();

        env.as_contract(&contract_id, || {
            env.storage().instance().set(&DocKey::FaqCount, &u64::MAX);
        });

        client.create_faq(
            &String::from_str(&env, "overflow-faq"),
            &String::from_str(&env, "Q"),
            &String::from_str(&env, "A"),
            &String::from_str(&env, "general"),
            &String::from_str(&env, "en"),
            &Address::generate(&env),
        );
    }

    #[test]
    #[should_panic(expected = "documentation article version overflow")]
    fn update_article_panics_when_version_overflows() {
        let (env, client, contract_id) = setup();
        seed_article(&env, &client, "overflow-version");

        env.as_contract(&contract_id, || {
            let id = String::from_str(&env, "overflow-version");
            let mut article: Article = env.storage().instance().get(&id).unwrap();
            article.version = u32::MAX;
            env.storage().instance().set(&id, &article);
        });

        client.update_article(
            &String::from_str(&env, "overflow-version"),
            &String::from_str(&env, "Version overflow"),
            &String::from_str(&env, "Still content"),
            &vec![&env],
        );
    }

    #[test]
    #[should_panic(expected = "documentation article view count overflow")]
    fn record_view_panics_when_view_count_overflows() {
        let (env, client, contract_id) = setup();
        seed_article(&env, &client, "overflow-view");

        env.as_contract(&contract_id, || {
            let id = String::from_str(&env, "overflow-view");
            let mut article: Article = env.storage().instance().get(&id).unwrap();
            article.view_count = u64::MAX;
            env.storage().instance().set(&id, &article);
        });

        client.record_view(&String::from_str(&env, "overflow-view"));
    }

    #[test]
    #[should_panic(expected = "documentation article helpful count overflow")]
    fn mark_helpful_panics_when_helpful_count_overflows() {
        let (env, client, contract_id) = setup();
        seed_article(&env, &client, "overflow-helpful");

        env.as_contract(&contract_id, || {
            let id = String::from_str(&env, "overflow-helpful");
            let mut article: Article = env.storage().instance().get(&id).unwrap();
            article.helpful_count = u64::MAX;
            env.storage().instance().set(&id, &article);
        });

        client.mark_helpful(&String::from_str(&env, "overflow-helpful"));
    }
}
