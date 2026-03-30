#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, Bytes};

#[test]
fn test_initialize() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    
    let contract_id = env.register_contract(None, CommunityContract);
    let client = CommunityContractClient::new(&env, &contract_id);
    
    client.initialize(&admin, &token, &fee_recipient);
    
    // Check config (internal implementation detail, but we can check it via functions)
    let summary = client.get_community_summary();
    assert_eq!(summary.get(Symbol::new(&env, "categories")).unwrap(), 0);
}

#[test]
fn test_create_forum_category() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    
    let contract_id = env.register_contract(None, CommunityContract);
    let client = CommunityContractClient::new(&env, &contract_id);
    
    client.initialize(&admin, &token, &fee_recipient);
    
    let name = Bytes::from_slice(&env, b"General Support");
    let description = Bytes::from_slice(&env, b"General help and support");
    let allowed_roles = Vec::new(&env);
    
    let category_id = client.create_forum_category(&admin, &name, &description, &false, &allowed_roles);
    assert_eq!(category_id, 1);
}

#[test]
fn test_create_forum_post_and_resolve() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let author = Address::generate(&env);
    let solver = Address::generate(&env);
    let token = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    
    let contract_id = env.register_contract(None, CommunityContract);
    let client = CommunityContractClient::new(&env, &contract_id);
    
    client.initialize(&admin, &token, &fee_recipient);
    
    let name = Bytes::from_slice(&env, b"General Support");
    let description = Bytes::from_slice(&env, b"General help and support");
    client.create_forum_category(&admin, &name, &description, &false, &Vec::new(&env));
    
    let title = Bytes::from_slice(&env, b"How to use TeachLink?");
    let content = Bytes::from_slice(&env, b"I need help getting started.");
    let post_id = client.create_forum_post(&author, &1, &title, &content, &Vec::new(&env));
    
    assert_eq!(post_id, 1);
    
    let comment_content = Bytes::from_slice(&env, b"Check the documentation.");
    let comment_id = client.create_forum_comment(&solver, &post_id, &comment_content);
    assert_eq!(comment_id, 1);
    
    client.resolve_forum_post(&author, &post_id, &comment_id);
    
    // Let's verify reputation update for solver
    let solver_rep = client.get_user_reputation(&solver);
    assert_eq!(solver_rep.points, 52); // 2 for comment + 50 for resolution
    assert_eq!(solver_rep.help_requests_resolved, 1);
    
    // Let's verify reputation update for author
    let author_rep = client.get_user_reputation(&author);
    assert_eq!(author_rep.points, 5);
}

#[test]
fn test_community_events() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let p1 = Address::generate(&env);
    let token = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    
    let contract_id = env.register_contract(None, CommunityContract);
    let client = CommunityContractClient::new(&env, &contract_id);
    
    client.initialize(&admin, &token, &fee_recipient);
    
    let title = Bytes::from_slice(&env, b"TeachLink Workshop");
    let description = Bytes::from_slice(&env, b"Learn how to build on TeachLink");
    let event_id = client.create_event(&organizer, &title, &description, &1000, &2000, &10, &EventType::Workshop);
    
    assert_eq!(event_id, 1);
    
    client.join_event(&p1, &1);
    
    let summary = client.get_community_summary();
    assert_eq!(summary.get(Symbol::new(&env, "events")).unwrap(), 1);
}
