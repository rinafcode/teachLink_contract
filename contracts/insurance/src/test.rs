#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env};

#[test]
fn test_insurance_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let oracle = Address::generate(&env);
    let token_admin = Address::generate(&env);

    // Setup Token
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_address = token_contract.address();
    let token = token::Client::new(&env, &token_address);
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    // Setup Insurance Contract
    let contract_id = env.register(InsurancePool, ());
    let client = InsurancePoolClient::new(&env, &contract_id);

    let premium_amount = 100;
    let payout_amount = 500;

    // Initialize
    client.initialize(&admin, &token_address, &oracle, &premium_amount, &payout_amount);

    // Mint tokens to user and contract (for payout liquidity)
    token_admin_client.mint(&user, &1000);
    token_admin_client.mint(&contract_id, &1000);

    // 1. Pay Premium
    client.pay_premium(&user);
    assert!(client.is_insured(&user));
    assert_eq!(token.balance(&user), 900); // 1000 - 100
    assert_eq!(token.balance(&contract_id), 1100); // 1000 + 100

    // 2. File Claim
    let course_id = 101;
    let claim_id = client.file_claim(&user, &course_id);
    
    let claim = client.get_claim(&claim_id).unwrap();
    assert_eq!(claim.user, user);
    assert_eq!(claim.course_id, course_id);
    assert_eq!(claim.status, ClaimStatus::Pending);

    // 3. Process Claim (Verify)
    client.process_claim(&claim_id, &true);
    
    let claim = client.get_claim(&claim_id).unwrap();
    assert_eq!(claim.status, ClaimStatus::Verified);

    // 4. Payout
    client.payout(&claim_id);
    
    let claim = client.get_claim(&claim_id).unwrap();
    assert_eq!(claim.status, ClaimStatus::Paid);
    
    assert_eq!(token.balance(&user), 1400); // 900 + 500
    assert_eq!(token.balance(&contract_id), 600); // 1100 - 500
    
    // User should no longer be insured (consumed)
    assert!(!client.is_insured(&user));
}

#[test]
fn test_claim_rejection() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let oracle = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_address = token_contract.address();
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    let contract_id = env.register(InsurancePool, ());
    let client = InsurancePoolClient::new(&env, &contract_id);

    client.initialize(&admin, &token_address, &oracle, &100, &500);
    token_admin_client.mint(&user, &1000);

    client.pay_premium(&user);
    let claim_id = client.file_claim(&user, &101);

    // Reject Claim
    client.process_claim(&claim_id, &false);

    let claim = client.get_claim(&claim_id).unwrap();
    assert_eq!(claim.status, ClaimStatus::Rejected);
}

#[test]
#[should_panic(expected = "User is not insured")]
fn test_file_claim_not_insured() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let oracle = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_address = token_contract.address();
    let contract_id = env.register(InsurancePool, ());
    let client = InsurancePoolClient::new(&env, &contract_id);

    client.initialize(&admin, &token_address, &oracle, &100, &500);

    client.file_claim(&user, &101);
}
