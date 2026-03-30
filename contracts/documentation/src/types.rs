//! Documentation contract data types.

use soroban_sdk::{contracttype, Address, String, Vec};

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
