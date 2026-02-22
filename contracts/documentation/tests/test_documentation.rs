//! Documentation Contract Test Scenarios
//!
//! Test scenarios covering various documentation types and user needs

#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};
use teachlink_documentation::{
    DocCategory, DocumentationContract, DocumentationContractClient, Visibility,
};

/// Test scenario: Create a new guide article
#[test]
fn test_create_guide_article() {
    let env = Env::default();
    let contract_id = env.register(DocumentationContract, ());
    env.mock_all_auths();

    let client = DocumentationContractClient::new(&env, &contract_id);
    let author = Address::generate(&env);

    // Create a guide article
    let article = client.create_article(
        &String::from_str(&env, "guide-001"),
        &String::from_str(&env, "Getting Started with TeachLink"),
        &String::from_str(&env, "# Getting Started\n\nThis guide helps you..."),
        &DocCategory::Guide,
        &String::from_str(&env, "en"),
        &Vec::from_slice(
            &env,
            &[
                String::from_str(&env, "getting-started"),
                String::from_str(&env, "beginner"),
            ],
        ),
        &Visibility::Public,
        &author,
    );

    assert_eq!(article.version, 1);
    assert_eq!(article.view_count, 0);
}

/// Test scenario: Create API reference documentation
#[test]
fn test_create_api_reference() {
    let env = Env::default();
    let contract_id = env.register(DocumentationContract, ());
    env.mock_all_auths();

    let client = DocumentationContractClient::new(&env, &contract_id);
    let author = Address::generate(&env);

    let article = client.create_article(
        &String::from_str(&env, "api-001"),
        &String::from_str(&env, "Bridge API Reference"),
        &String::from_str(&env, "## bridge_out\n\nLock tokens for bridging..."),
        &DocCategory::ApiReference,
        &String::from_str(&env, "en"),
        &Vec::from_slice(
            &env,
            &[
                String::from_str(&env, "bridge"),
                String::from_str(&env, "api"),
            ],
        ),
        &Visibility::Public,
        &author,
    );

    assert_eq!(article.category, DocCategory::ApiReference);
}

/// Test scenario: Create tutorial content
#[test]
fn test_create_tutorial() {
    let env = Env::default();
    let contract_id = env.register(DocumentationContract, ());
    env.mock_all_auths();

    let client = DocumentationContractClient::new(&env, &contract_id);
    let author = Address::generate(&env);

    let article = client.create_article(
        &String::from_str(&env, "tutorial-001"),
        &String::from_str(&env, "Your First Course"),
        &String::from_str(&env, "Step 1: Initialize your project..."),
        &DocCategory::Tutorial,
        &String::from_str(&env, "en"),
        &Vec::from_slice(
            &env,
            &[
                String::from_str(&env, "tutorial"),
                String::from_str(&env, "beginner"),
            ],
        ),
        &Visibility::Public,
        &author,
    );

    assert_eq!(article.category, DocCategory::Tutorial);
}

/// Test scenario: Create FAQ entry
#[test]
fn test_create_faq() {
    let env = Env::default();
    let contract_id = env.register(DocumentationContract, ());
    env.mock_all_auths();

    let client = DocumentationContractClient::new(&env, &contract_id);
    let author = Address::generate(&env);

    let faq = client.create_faq(
        &String::from_str(&env, "faq-001"),
        &String::from_str(&env, "What is TeachLink?"),
        &String::from_str(
            &env,
            "TeachLink is a decentralized knowledge-sharing platform...",
        ),
        &String::from_str(&env, "general"),
        &String::from_str(&env, "en"),
        &author,
    );

    assert_eq!(faq.helpful_count, 0);
}

/// Test scenario: Record article view for analytics
#[test]
fn test_record_view() {
    let env = Env::default();
    let contract_id = env.register(DocumentationContract, ());
    env.mock_all_auths();

    let client = DocumentationContractClient::new(&env, &contract_id);
    let author = Address::generate(&env);

    // Create article first
    client.create_article(
        &String::from_str(&env, "test-001"),
        &String::from_str(&env, "Test Article"),
        &String::from_str(&env, "Test content"),
        &DocCategory::Guide,
        &String::from_str(&env, "en"),
        &Vec::from_slice(&env, &[]),
        &Visibility::Public,
        &author,
    );

    // Record view
    client.record_view(&String::from_str(&env, "test-001"));

    // Get article and verify view count
    let article = client.get_article(&String::from_str(&env, "test-001"));
    assert_eq!(article.view_count, 1);
}

/// Test scenario: Mark article as helpful
#[test]
fn test_mark_helpful() {
    let env = Env::default();
    let contract_id = env.register(DocumentationContract, ());
    env.mock_all_auths();

    let client = DocumentationContractClient::new(&env, &contract_id);
    let author = Address::generate(&env);

    // Create article
    client.create_article(
        &String::from_str(&env, "helpful-test"),
        &String::from_str(&env, "Helpful Article"),
        &String::from_str(&env, "Very helpful content"),
        &DocCategory::Guide,
        &String::from_str(&env, "en"),
        &Vec::from_slice(&env, &[]),
        &Visibility::Public,
        &author,
    );

    // Mark as helpful
    client.mark_helpful(&String::from_str(&env, "helpful-test"));
    client.mark_helpful(&String::from_str(&env, "helpful-test"));

    let article = client.get_article(&String::from_str(&env, "helpful-test"));
    assert_eq!(article.helpful_count, 2);
}

/// Test scenario: Update article content
#[test]
fn test_update_article() {
    let env = Env::default();
    let contract_id = env.register(DocumentationContract, ());
    env.mock_all_auths();

    let client = DocumentationContractClient::new(&env, &contract_id);
    let author = Address::generate(&env);

    // Create article
    client.create_article(
        &String::from_str(&env, "update-test"),
        &String::from_str(&env, "Original Title"),
        &String::from_str(&env, "Original content"),
        &DocCategory::Guide,
        &String::from_str(&env, "en"),
        &Vec::from_slice(&env, &[]),
        &Visibility::Public,
        &author,
    );

    // Update article
    let updated = client.update_article(
        &String::from_str(&env, "update-test"),
        &String::from_str(&env, "Updated Title"),
        &String::from_str(&env, "Updated content"),
        &Vec::from_slice(&env, &[String::from_str(&env, "updated")]),
    );

    assert_eq!(updated.title, String::from_str(&env, "Updated Title"));
    assert_eq!(updated.version, 2);
}

/// Test scenario: Multilingual content
#[test]
fn test_multilingual_content() {
    let env = Env::default();
    let contract_id = env.register(DocumentationContract, ());
    env.mock_all_auths();

    let client = DocumentationContractClient::new(&env, &contract_id);
    let author = Address::generate(&env);

    // Create English version
    let _en_article = client.create_article(
        &String::from_str(&env, "multi-en"),
        &String::from_str(&env, "Getting Started"),
        &String::from_str(&env, "This is the English version..."),
        &DocCategory::Guide,
        &String::from_str(&env, "en"),
        &Vec::from_slice(&env, &[]),
        &Visibility::Public,
        &author,
    );

    // Create Spanish version
    let _es_article = client.create_article(
        &String::from_str(&env, "multi-es"),
        &String::from_str(&env, "Primeros Pasos"),
        &String::from_str(&env, "Esta es la versión en español..."),
        &DocCategory::Guide,
        &String::from_str(&env, "es"),
        &Vec::from_slice(&env, &[]),
        &Visibility::Public,
        &author,
    );

    // Get versions
    let en_article = client.get_article(&String::from_str(&env, "multi-en"));
    let es_article = client.get_article(&String::from_str(&env, "multi-es"));

    assert_eq!(en_article.language, String::from_str(&env, "en"));
    assert_eq!(es_article.language, String::from_str(&env, "es"));
}

/// Test scenario: Documentation versioning
#[test]
fn test_documentation_versioning() {
    let env = Env::default();
    let contract_id = env.register(DocumentationContract, ());
    env.mock_all_auths();

    let client = DocumentationContractClient::new(&env, &contract_id);

    // Set version
    client.update_version(&2);

    // Get version
    let version = client.get_version();
    assert_eq!(version, 2);
}

/// Test scenario: Article counts
#[test]
fn test_article_counts() {
    let env = Env::default();
    let contract_id = env.register(DocumentationContract, ());
    env.mock_all_auths();

    let client = DocumentationContractClient::new(&env, &contract_id);
    let author = Address::generate(&env);

    // Create multiple articles
    client.create_article(
        &String::from_str(&env, "count-001"),
        &String::from_str(&env, "Article 1"),
        &String::from_str(&env, "Content 1"),
        &DocCategory::Guide,
        &String::from_str(&env, "en"),
        &Vec::from_slice(&env, &[]),
        &Visibility::Public,
        &author,
    );

    client.create_article(
        &String::from_str(&env, "count-002"),
        &String::from_str(&env, "Article 2"),
        &String::from_str(&env, "Content 2"),
        &DocCategory::Guide,
        &String::from_str(&env, "en"),
        &Vec::from_slice(&env, &[]),
        &Visibility::Public,
        &author,
    );

    let count = client.get_article_count();
    assert_eq!(count, 2);
}
