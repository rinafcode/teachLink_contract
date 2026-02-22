//! Documentation Contract Test Scenarios
//!
//! Test scenarios covering various documentation types and user needs

#![cfg(test)]

extern crate teachlink_documentation;

use soroban_sdk::{Address, Env, String, Vec};
use teachlink_documentation::{
    DocCategory, DocumentationContract, DocumentationContractClient, Visibility,
};

/// Test scenario: Create a new guide article
#[test]
fn test_create_guide_article() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = [0u8; 32];
    let client = DocumentationContractClient::new(&env, &contract_id.into());

    // Create a guide article
    let article = client.create_article(
        &String::from_slice(&env, "guide-001"),
        &String::from_slice(&env, "Getting Started with TeachLink"),
        &String::from_slice(&env, "# Getting Started\n\nThis guide helps you..."),
        &DocCategory::Guide,
        &String::from_slice(&env, "en"),
        &Vec::from_slice(
            &env,
            &[
                String::from_slice(&env, "getting-started"),
                String::from_slice(&env, "beginner"),
            ],
        ),
        &Visibility::Public,
    );

    assert_eq!(article.version, 1);
    assert_eq!(article.view_count, 0);
}

/// Test scenario: Create API reference documentation
#[test]
fn test_create_api_reference() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = [0u8; 32];
    let client = DocumentationContractClient::new(&env, &contract_id.into());

    let article = client.create_article(
        &String::from_slice(&env, "api-001"),
        &String::from_slice(&env, "Bridge API Reference"),
        &String::from_slice(&env, "## bridge_out\n\nLock tokens for bridging..."),
        &DocCategory::ApiReference,
        &String::from_slice(&env, "en"),
        &Vec::from_slice(
            &env,
            &[
                String::from_slice(&env, "bridge"),
                String::from_slice(&env, "api"),
            ],
        ),
        &Visibility::Public,
    );

    assert_eq!(article.category, DocCategory::ApiReference);
}

/// Test scenario: Create tutorial content
#[test]
fn test_create_tutorial() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = [0u8; 32];
    let client = DocumentationContractClient::new(&env, &contract_id.into());

    let article = client.create_article(
        &String::from_slice(&env, "tutorial-001"),
        &String::from_slice(&env, "Your First Course"),
        &String::from_slice(&env, "Step 1: Initialize your project..."),
        &DocCategory::Tutorial,
        &String::from_slice(&env, "en"),
        &Vec::from_slice(
            &env,
            &[
                String::from_slice(&env, "tutorial"),
                String::from_slice(&env, "beginner"),
            ],
        ),
        &Visibility::Public,
    );

    assert_eq!(article.category, DocCategory::Tutorial);
}

/// Test scenario: Create FAQ entry
#[test]
fn test_create_faq() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = [0u8; 32];
    let client = DocumentationContractClient::new(&env, &contract_id.into());

    let faq = client.create_faq(
        &String::from_slice(&env, "faq-001"),
        &String::from_slice(&env, "What is TeachLink?"),
        &String::from_slice(
            &env,
            "TeachLink is a decentralized knowledge-sharing platform...",
        ),
        &String::from_slice(&env, "general"),
        &String::from_slice(&env, "en"),
    );

    assert_eq!(faq.helpful_count, 0);
}

/// Test scenario: Record article view for analytics
#[test]
fn test_record_view() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = [0u8; 32];
    let client = DocumentationContractClient::new(&env, &contract_id.into());

    // Create article first
    client.create_article(
        &String::from_slice(&env, "test-001"),
        &String::from_slice(&env, "Test Article"),
        &String::from_slice(&env, "Test content"),
        &DocCategory::Guide,
        &String::from_slice(&env, "en"),
        &Vec::from_slice(&env, &[]),
        &Visibility::Public,
    );

    // Record view
    client.record_view(&String::from_slice(&env, "test-001"));

    // Get article and verify view count
    let article = client.get_article(&String::from_slice(&env, "test-001"));
    assert_eq!(article.view_count, 1);
}

/// Test scenario: Mark article as helpful
#[test]
fn test_mark_helpful() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = [0u8; 32];
    let client = DocumentationContractClient::new(&env, &contract_id.into());

    // Create article
    client.create_article(
        &String::from_slice(&env, "helpful-test"),
        &String::from_slice(&env, "Helpful Article"),
        &String::from_slice(&env, "Very helpful content"),
        &DocCategory::Guide,
        &String::from_slice(&env, "en"),
        &Vec::from_slice(&env, &[]),
        &Visibility::Public,
    );

    // Mark as helpful
    client.mark_helpful(&String::from_slice(&env, "helpful-test"));
    client.mark_helpful(&String::from_slice(&env, "helpful-test"));

    let article = client.get_article(&String::from_slice(&env, "helpful-test"));
    assert_eq!(article.helpful_count, 2);
}

/// Test scenario: Update article content
#[test]
fn test_update_article() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = [0u8; 32];
    let client = DocumentationContractClient::new(&env, &contract_id.into());

    // Create article
    client.create_article(
        &String::from_slice(&env, "update-test"),
        &String::from_slice(&env, "Original Title"),
        &String::from_slice(&env, "Original content"),
        &DocCategory::Guide,
        &String::from_slice(&env, "en"),
        &Vec::from_slice(&env, &[]),
        &Visibility::Public,
    );

    // Update article
    let updated = client.update_article(
        &String::from_slice(&env, "update-test"),
        &String::from_slice(&env, "Updated Title"),
        &String::from_slice(&env, "Updated content"),
        &Vec::from_slice(&env, &[String::from_slice(&env, "updated")]),
    );

    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.version, 2);
}

/// Test scenario: Multilingual content
#[test]
fn test_multilingual_content() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = [0u8; 32];
    let client = DocumentationContractClient::new(&env, &contract_id.into());

    // Create English version
    let _en_article = client.create_article(
        &String::from_slice(&env, "multi-en"),
        &String::from_slice(&env, "Getting Started"),
        &String::from_slice(&env, "This is the English version..."),
        &DocCategory::Guide,
        &String::from_slice(&env, "en"),
        &Vec::from_slice(&env, &[]),
        &Visibility::Public,
    );

    // Create Spanish version
    let _es_article = client.create_article(
        &String::from_slice(&env, "multi-es"),
        &String::from_slice(&env, "Primeros Pasos"),
        &String::from_slice(&env, "Esta es la versión en español..."),
        &DocCategory::Guide,
        &String::from_slice(&env, "es"),
        &Vec::from_slice(&env, &[]),
        &Visibility::Public,
    );

    // Get versions
    let en_article = client.get_article(&String::from_slice(&env, "multi-en"));
    let es_article = client.get_article(&String::from_slice(&env, "multi-es"));

    assert_eq!(en_article.language, "en");
    assert_eq!(es_article.language, "es");
}

/// Test scenario: Documentation versioning
#[test]
fn test_documentation_versioning() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = [0u8; 32];
    let client = DocumentationContractClient::new(&env, &contract_id.into());

    // Set version
    client.update_version(&2);

    // Get version
    let version = client.get_version();
    assert_eq!(version, 2);
}

/// Test scenario: Article not found
#[test]
fn test_article_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = [0u8; 32];
    let client = DocumentationContractClient::new(&env, &contract_id.into());

    let result = client.try_get_article(&String::from_slice(&env, "nonexistent"));
    assert!(result.is_err());
}
