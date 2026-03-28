//! TeachLink Documentation Contract
//!
//! A smart contract for managing documentation, knowledge base articles,
//! FAQs, tutorials, and community-contributed content.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Vec};

/// Documentation category types
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

/// Content visibility levels
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Visibility {
    Public,
    Community,
    Private,
}

/// A documentation article
#[contracttype]
#[derive(Clone)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: DocCategory,
    pub language: String,
    pub version: u32,
    pub author: Address,
    pub visibility: Visibility,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub view_count: u64,
    pub helpful_count: u64,
}

/// A FAQ entry
#[contracttype]
#[derive(Clone)]
pub struct FaqEntry {
    pub id: String,
    pub question: String,
    pub answer: String,
    pub category: String,
    pub language: String,
    pub author: Address,
    pub created_at: u64,
    pub updated_at: u64,
    pub helpful_count: u64,
}

/// Documentation contract storage keys
#[contracttype]
enum DocKey {
    ArticleCount,
    FaqCount,
    Version,
}

/// Main documentation contract
#[contract]
pub struct DocumentationContract;

#[contractimpl]
impl DocumentationContract {
    /// Create a new documentation article
    ///
    /// Creates and stores a new article with the given parameters, incrementing
    /// the article count tracker.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `id` - Unique identifier for the article.
    /// * `title` - Title of the article.
    /// * `content` - Main content of the article.
    /// * `category` - The `DocCategory` enum variant.
    /// * `language` - Language code (e.g. "en").
    /// * `tags` - List of string tags for searching.
    /// * `visibility` - Public, Community, or Private visibility.
    /// * `author` - The address of the content creator.
    ///
    /// # Returns
    ///
    /// * `Article` - The newly created article.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let article = DocumentationContract::create_article(env, id, title, content, category, lang, tags, vis, author);
    /// ```
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

        // Increment article count
        let count_key = DocKey::ArticleCount;
        let current_count: u64 = env.storage().instance().get(&count_key).unwrap_or(0);
        env.storage()
            .instance()
            .set(&count_key, &(current_count + 1));

        article
    }

    /// Get an article by ID
    ///
    /// Retrieves a documentation article using its unique ID.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `id` - The unique identifier of the article.
    ///
    /// # Returns
    ///
    /// * `Article` - The requested article.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let article = DocumentationContract::get_article(env, id);
    /// ```
    pub fn get_article(env: Env, id: String) -> Article {
        env.storage().instance().get(&id).unwrap()
    }

    /// Update an existing article
    ///
    /// Modifies an article's title, content, and tags while incrementing
    /// its version number and updating the modification timestamp.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `id` - The ID of the article to update.
    /// * `title` - New title.
    /// * `content` - New content.
    /// * `tags` - New list of tags.
    ///
    /// # Returns
    ///
    /// * `Article` - The updated article.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let updated = DocumentationContract::update_article(env, id, new_title, new_content, new_tags);
    /// ```
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
        article.version += 1;
        article.updated_at = env.ledger().timestamp();

        env.storage().instance().set(&id, &article);

        article
    }

    /// Record a view for analytics
    ///
    /// Increments the view count of a specific article.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `article_id` - The ID of the article viewed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// DocumentationContract::record_view(env, article_id);
    /// ```
    pub fn record_view(env: Env, article_id: String) {
        let mut article: Article = env.storage().instance().get(&article_id).unwrap();
        article.view_count += 1;

        env.storage().instance().set(&article_id, &article);
    }

    /// Record that a user found an article helpful
    ///
    /// Increments the helpful count of a specific article.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `article_id` - The ID of the article found helpful.
    ///
    /// # Examples
    ///
    /// ```rust
    /// DocumentationContract::mark_helpful(env, article_id);
    /// ```
    pub fn mark_helpful(env: Env, article_id: String) {
        let mut article: Article = env.storage().instance().get(&article_id).unwrap();
        article.helpful_count += 1;

        env.storage().instance().set(&article_id, &article);
    }

    /// Create a new FAQ entry
    ///
    /// Stores a new question and answer pair under a specific category.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `id` - Unique identifier for the FAQ.
    /// * `question` - The FAQ question.
    /// * `answer` - The FAQ answer.
    /// * `category` - The category grouping for the FAQ.
    /// * `language` - Language code.
    /// * `author` - The address of the FAQ author.
    ///
    /// # Returns
    ///
    /// * `FaqEntry` - The newly created FAQ entry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let faq = DocumentationContract::create_faq(env, id, q, a, cat, lang, author);
    /// ```
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

        // Increment FAQ count
        let count_key = DocKey::FaqCount;
        let current_count: u64 = env.storage().instance().get(&count_key).unwrap_or(0);
        env.storage()
            .instance()
            .set(&count_key, &(current_count + 1));

        faq
    }

    /// Get FAQ by ID
    ///
    /// Retrieves a specific FAQ entry by its ID.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `id` - The unique identifier of the FAQ.
    ///
    /// # Returns
    ///
    /// * `FaqEntry` - The requested FAQ entry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let faq = DocumentationContract::get_faq(env, id);
    /// ```
    pub fn get_faq(env: Env, id: String) -> FaqEntry {
        env.storage().instance().get(&id).unwrap()
    }

    /// Search articles by keyword (simplified implementation)
    ///
    /// Returns a list of articles matching the search query.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `_query` - The string query to search for.
    ///
    /// # Returns
    ///
    /// * `Vec<Article>` - List of matching articles.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let results = DocumentationContract::search_articles(env, query);
    /// ```
    pub fn search_articles(env: Env, _query: String) -> Vec<Article> {
        // In a full implementation, this would search through articles
        // For now, return empty vector as placeholder
        Vec::new(&env)
    }

    /// Get total article count
    ///
    /// Returns the total number of articles stored in the contract.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    ///
    /// # Returns
    ///
    /// * `u64` - Number of articles.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let count = DocumentationContract::get_article_count(env);
    /// ```
    pub fn get_article_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DocKey::ArticleCount)
            .unwrap_or(0)
    }

    /// Get total FAQ count
    ///
    /// Returns the total number of FAQ entries stored in the contract.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    ///
    /// # Returns
    ///
    /// * `u64` - Number of FAQ entries.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let count = DocumentationContract::get_faq_count(env);
    /// ```
    pub fn get_faq_count(env: Env) -> u64 {
        env.storage().instance().get(&DocKey::FaqCount).unwrap_or(0)
    }

    /// Get current documentation version
    ///
    /// Retrieves the overall version number of the knowledge base.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    ///
    /// # Returns
    ///
    /// * `u32` - The current global documentation version.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let version = DocumentationContract::get_version(env);
    /// ```
    pub fn get_version(env: Env) -> u32 {
        env.storage().instance().get(&DocKey::Version).unwrap_or(1)
    }

    /// Update documentation version
    ///
    /// Sets a new overall version number for the entire knowledge base.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `version` - The new version number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// DocumentationContract::update_version(env, 2);
    /// ```
    pub fn update_version(env: Env, version: u32) {
        env.storage().instance().set(&DocKey::Version, &version);
    }
}
