# Security-Focused Test Cases

> Closes #337

## Overview

This document describes the security test cases required for the TeachLink smart contract. Each test targets a specific attack vector or security property that the contract must uphold.

---

## 1. Access Control

Verify that privileged functions reject unauthorized callers.

```rust
#[test]
#[should_panic(expected = "Unauthorized")]
fn test_mint_rejects_non_admin() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);
    let learner = Address::generate(&env);

    let contract_id = env.register_contract(None, TeachLinkContract);
    let client = TeachLinkContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    // Attacker attempts to mint — must panic
    client.mint(&attacker, &learner, &1000_i128);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_non_owner_cannot_transfer_on_behalf() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let recipient = Address::generate(&env);

    let contract_id = env.register_contract(None, TeachLinkContract);
    let client = TeachLinkContractClient::new(&env, &contract_id);
    client.initialize(&admin);
    client.mint(&admin, &owner, &500_i128);

    // Attacker tries to transfer owner's tokens — must panic
    client.transfer(&attacker, &owner, &recipient, &500_i128);
}
```

---

## 2. Input Sanitization

Verify that invalid inputs are rejected before any state change.

```rust
#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_mint_zero_amount_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let learner = Address::generate(&env);
    let contract_id = env.register_contract(None, TeachLinkContract);
    let client = TeachLinkContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    client.mint(&admin, &learner, &0_i128); // must panic
}

#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_transfer_negative_amount_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let contract_id = env.register_contract(None, TeachLinkContract);
    let client = TeachLinkContractClient::new(&env, &contract_id);
    client.initialize(&admin);
    client.mint(&admin, &from, &100_i128);

    client.transfer(&from, &from, &to, &-1_i128); // must panic
}
```

---

## 3. Reentrancy Protection

Soroban's execution model prevents reentrancy at the platform level, but the contract must not hold intermediate state that could be exploited across sequential calls.

```rust
#[test]
fn test_balance_consistent_after_sequential_transfers() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let contract_id = env.register_contract(None, TeachLinkContract);
    let client = TeachLinkContractClient::new(&env, &contract_id);
    client.initialize(&admin);
    client.mint(&admin, &alice, &1000_i128);

    // Simulate rapid sequential transfers
    client.transfer(&alice, &alice, &bob, &300_i128);
    client.transfer(&alice, &alice, &bob, &300_i128);
    client.transfer(&alice, &alice, &bob, &300_i128);

    assert_eq!(client.balance(&alice), 100);
    assert_eq!(client.balance(&bob), 900);
    assert_eq!(client.total_supply(), 1000); // supply must be conserved
}
```

---

## 4. Overflow Protection

Verify that arithmetic operations cannot overflow or underflow.

```rust
#[test]
#[should_panic(expected = "OverflowError")]
fn test_mint_overflow_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let learner = Address::generate(&env);
    let contract_id = env.register_contract(None, TeachLinkContract);
    let client = TeachLinkContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    // Mint near max, then mint again to trigger overflow
    client.mint(&admin, &learner, &i128::MAX);
    client.mint(&admin, &learner, &1_i128); // must panic
}

#[test]
#[should_panic(expected = "InsufficientBalance")]
fn test_transfer_exceeds_balance_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let contract_id = env.register_contract(None, TeachLinkContract);
    let client = TeachLinkContractClient::new(&env, &contract_id);
    client.initialize(&admin);
    client.mint(&admin, &from, &100_i128);

    client.transfer(&from, &from, &to, &101_i128); // must panic
}
```

---

## Running Security Tests

```bash
cargo test security
# or run all tests
cargo test
```
