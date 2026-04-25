//! TeachLink Documentation Contract
//!
//! A smart contract for managing documentation, knowledge base articles,
//! FAQs, tutorials, and community-contributed content.
//!
//! # Storage Model
//!
//! Articles and FAQs are stored in **instance storage** keyed directly by
//! their string `id`.  This means:
//! - IDs must be unique across both articles and FAQs (they share the same
//!   storage namespace).
//! - Calling `create_article` with an existing ID overwrites the record but
//!   does **not** increment the article count (idempotent upsert).
//!
//! Counters (`ArticleCount`, `FaqCount`) are stored under `DocKey` enum
//! variants to avoid collisions with content IDs.
//!
//! # Overflow Protection
//!
//! All counter increments and version bumps use `checked_add` with an
//! explicit `expect` message.  On Soroban, a panic aborts the transaction
//! and rolls back all state changes, so overflow is safe to panic on.
//!
//! # Versioning
//!
//! Each `update_article` call increments `article.version` starting from 1.
//! The `created_at` timestamp is preserved across updates; only `updated_at`
//! changes.  This allows clients to detect staleness without comparing content.
//!
//! # TODO
//! - Implement `search_articles` with an on-chain inverted index or off-chain
//!   indexer integration (current implementation returns an empty vector).
//! - Add author authorization check to `update_article` so only the original
//!   author or an admin can modify content.
//! - Separate article and FAQ storage namespaces to prevent ID collisions.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Vec};

/// Documentation category types.
///
/// Used to classify articles for filtering and navigation.
/// `Faq` overlaps with the dedicated `FaqEntry` type — prefer `FaqEntry`
/// for structured Q&A content and `DocCategory::Faq` only for article-style
/// FAQ pages.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocCategory {
    Guide,
    ApiReference,
    Tutorial,
    Faq,
    KnowledgeBase,
    Troubleshooting,
}

/// Content visibility levels.
///
/// Controls who can read an article:
/// - `Public`    – visible to all users, including unauthenticated.
/// - `Community` – visible to registered TeachLink users only.
/// - `Private`   – visible to the author and admins only.
///
/// # TODO
/// - Enforce visibility at the read path (`get_article`) once an
///   authentication context is available in the contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Visibility {
    Public,
    Community,
    Private,
}

/// A documentation article.
///
/// Articles are the primary content unit.  The `version` field starts at 1
/// and is incremented on every `update_article` call.  `created_at` is
/// immutable after creation; `updated_at` reflects the last modification.
///
/// `view_count` and `helpful_count` are analytics counters incremented by
/// `record_view` and `mark_helpful` respectively.  Both use `checked_add`
/// to panic on overflow rather than silently wrapping.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: DocCategory,
    /// BCP-47 language tag (e.g. "en", "es", "zh-CN").
    pub language: String,
    /// Monotonically increasing version counter, starting at 1.
    pub version: u32,
    pub author: Address,
    pub visibility: Visibility,
    pub tags: Vec<String>,
    /// Unix timestamp (seconds) when the article was first created.
    pub created_at: u64,
    /// Unix timestamp (seconds) of the most recent update.
    pub updated_at: u64,
    /// Total number of times this article has been viewed.
    pub view_count: u64,
    /// Total number of users who marked this article as helpful.
    pub helpful_count: u64,
}

/// A FAQ entry.
///
/// Structured Q&A content distinct from general articles.  FAQs share the
/// same instance storage namespace as articles, so their IDs must not
/// collide with article IDs.
///
/// # TODO
/// - Add a `related_article_ids: Vec<String>` field to link FAQs to
///   relevant documentation articles.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FaqEntry {
    pub id: String,
    pub question: String,
    pub answer: String,
    /// Free-form category string (e.g. "billing", "technical").
    pub category: String,
    /// BCP-47 language tag.
    pub language: String,
    pub author: Address,
    pub created_at: u64,
    pub updated_at: u64,
    pub helpful_count: u64,
}

/// Documentation contract storage keys.
///
/// Enum variants are used as storage keys to avoid string-based key
/// collisions with content IDs stored directly by their string value.
#[contracttype]
enum DocKey {
    /// Total number of unique article IDs ever created.
    ArticleCount,
    /// Total number of unique FAQ IDs ever created.
    FaqCount,
    /// Global documentation schema version.
    Version,
}

/// Main documentation contract
#[contract]
pub struct DocumentationContract;

#[contractimpl]
impl DocumentationContract {
    /// Create a new documentation article (upsert by ID).
    ///
    /// If an article with the same `id` already exists, it is overwritten
    /// but the `ArticleCount` is **not** incremented (idempotent upsert).
    /// The `created_at` timestamp reflects the time of this call, not the
    /// original creation time — callers should use `update_article` for
    /// in-place edits to preserve `created_at`.
    ///
    /// # Overflow Protection
    /// `ArticleCount` uses `checked_add` and panics on overflow, aborting
    /// the transaction before any state is committed.
    ///
    /// # TODO
    /// - Require `author.require_auth()` to prevent anyone from creating
    ///   articles on behalf of another address.
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
        let timestamp = env.ledger().timestamp();
        let exists = env.storage().instance().has(&id);

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

        if !exists {
            // Increment article count for new IDs only.
            let count_key = DocKey::ArticleCount;
            let current_count: u64 = env.storage().instance().get(&count_key).unwrap_or(0);
            let next_count = current_count
                .checked_add(1)
                .expect("documentation article count overflow");
            env.storage().instance().set(&count_key, &next_count);
        }

        article
    }

    /// Get an article by ID
    pub fn get_article(env: Env, id: String) -> Article {
        env.storage().instance().get(&id).unwrap()
    }

    /// Update an existing article's content, title, and tags.
    ///
    /// Increments `version` and updates `updated_at`.  `created_at` and
    /// `author` are preserved.  Panics if the article does not exist
    /// (unwrap on missing storage entry).
    ///
    /// # Version Overflow
    /// `version` uses `checked_add` and panics at `u32::MAX`.  In practice
    /// this requires ~4 billion updates to a single article, which is
    /// unreachable in normal operation.
    ///
    /// # TODO
    /// - Add `caller: Address` parameter and verify `caller == article.author`
    ///   or caller has admin role before allowing the update.
    pub fn update_article(
        env: Env,
        id: String,
        title: String,
        content: String,
        tags: Vec<String>,
    ) -> Article {
        let mut article: Article = env.storage().instance().get(&id).unwrap();

        article.title = title;
        article.content = content;
        article.tags = tags;
        article.version = article
            .version
            .checked_add(1)
            .expect("documentation article version overflow");
        article.updated_at = env.ledger().timestamp();

        env.storage().instance().set(&id, &article);

        article
    }

    /// Record a view for analytics.
    ///
    /// Increments `view_count` using `checked_add` to panic on overflow
    /// rather than silently wrapping.  No authentication required — views
    /// are public read events.
    pub fn record_view(env: Env, article_id: String) {
        let mut article: Article = env.storage().instance().get(&article_id).unwrap();
        article.view_count = article
            .view_count
            .checked_add(1)
            .expect("documentation article view count overflow");

        env.storage().instance().set(&article_id, &article);
    }

    /// Record that a user found an article helpful.
    ///
    /// Increments `helpful_count` using `checked_add`.  No deduplication —
    /// the same user can mark an article helpful multiple times.
    ///
    /// # TODO
    /// - Track which addresses have already voted to prevent duplicate
    ///   helpful counts from inflating article quality signals.
    pub fn mark_helpful(env: Env, article_id: String) {
        let mut article: Article = env.storage().instance().get(&article_id).unwrap();
        article.helpful_count = article
            .helpful_count
            .checked_add(1)
            .expect("documentation article helpful count overflow");

        env.storage().instance().set(&article_id, &article);
    }

    /// Create a new FAQ entry (upsert by ID).
    ///
    /// Same upsert semantics as `create_article`: existing entries are
    /// overwritten without incrementing `FaqCount`.
    ///
    /// # TODO
    /// - Require `author.require_auth()` to prevent impersonation.
    pub fn create_faq(
        env: Env,
        id: String,
        question: String,
        answer: String,
        category: String,
        language: String,
        author: Address,
    ) -> FaqEntry {
        let timestamp = env.ledger().timestamp();
        let exists = env.storage().instance().has(&id);

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

        if !exists {
            // Increment FAQ count for new IDs only.
            let count_key = DocKey::FaqCount;
            let current_count: u64 = env.storage().instance().get(&count_key).unwrap_or(0);
            let next_count = current_count
                .checked_add(1)
                .expect("documentation faq count overflow");
            env.storage().instance().set(&count_key, &next_count);
        }

        faq
    }

    /// Get FAQ by ID
    pub fn get_faq(env: Env, id: String) -> FaqEntry {
        env.storage().instance().get(&id).unwrap()
    }

    /// Search articles by keyword (stub implementation).
    ///
    /// # Current Behaviour
    /// Always returns an empty vector.  Full-text search requires either:
    /// 1. An off-chain indexer that listens to article creation events and
    ///    exposes a query API.
    /// 2. An on-chain inverted index (expensive in terms of storage and gas).
    ///
    /// # TODO
    /// - Integrate with an off-chain indexer via oracle or implement a
    ///   tag-based search using a `Map<String, Vec<String>>` tag index.
    pub fn search_articles(env: Env, _query: String) -> Vec<Article> {
        // In a full implementation, this would search through articles
        // For now, return empty vector as placeholder
        Vec::new(&env)
    }

    /// Get total article count
    pub fn get_article_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DocKey::ArticleCount)
            .unwrap_or(0)
    }

    /// Get total FAQ count
    pub fn get_faq_count(env: Env) -> u64 {
        env.storage().instance().get(&DocKey::FaqCount).unwrap_or(0)
    }

    /// Get current documentation version
    pub fn get_version(env: Env) -> u32 {
        env.storage().instance().get(&DocKey::Version).unwrap_or(1)
    }

    /// Update documentation version
    pub fn update_version(env: Env, version: u32) {
        env.storage().instance().set(&DocKey::Version, &version);
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
