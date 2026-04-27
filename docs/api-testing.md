# Comprehensive API Testing

> Closes #336

## Overview

This document covers the full API test suite for the TeachLink smart contract. Tests validate all public endpoints, expected responses, error cases, and rate-limit behaviour.

---

## 1. All Endpoints

### `initialize`

```rust
#[test]
fn test_initialize_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, TeachLinkContract);
    let client = TeachLinkContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    // No panic = success
}

#[test]
#[should_panic(expected = "AlreadyInitialized")]
fn test_initialize_twice_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, TeachLinkContract);
    let client = TeachLinkContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.initialize(&admin); // must panic
}
```

### `mint`

```rust
#[test]
fn test_mint_increases_balance_and_supply() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, learner, client) = setup(&env);

    client.mint(&admin, &learner, &500_i128);

    assert_eq!(client.balance(&learner), 500);
    assert_eq!(client.total_supply(), 500);
}
```

### `transfer`

```rust
#[test]
fn test_transfer_moves_balance() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, alice, client) = setup(&env);
    let bob = Address::generate(&env);

    client.mint(&admin, &alice, &300_i128);
    client.transfer(&alice, &alice, &bob, &100_i128);

    assert_eq!(client.balance(&alice), 200);
    assert_eq!(client.balance(&bob), 100);
}
```

### `balance`

```rust
#[test]
fn test_balance_returns_zero_for_unknown_address() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, client) = setup(&env);
    let stranger = Address::generate(&env);

    assert_eq!(client.balance(&stranger), 0);
}
```

### `total_supply`

```rust
#[test]
fn test_total_supply_reflects_all_mints() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, alice, client) = setup(&env);
    let bob = Address::generate(&env);

    client.mint(&admin, &alice, &400_i128);
    client.mint(&admin, &bob, &600_i128);

    assert_eq!(client.total_supply(), 1000);
}
```

---

## 2. Response Validation

Each function must return the correct type and value:

| Function        | Return type   | Expected on success          |
|-----------------|---------------|------------------------------|
| `initialize`    | `()`          | No error                     |
| `mint`          | `Result<(),Error>` | `Ok(())`                |
| `transfer`      | `Result<(),Error>` | `Ok(())`                |
| `balance`       | `i128`        | Current balance ≥ 0          |
| `total_supply`  | `i128`        | Sum of all minted amounts    |

```rust
#[test]
fn test_mint_returns_ok() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, learner, client) = setup(&env);

    let result = client.try_mint(&admin, &learner, &100_i128);
    assert!(result.is_ok());
}
```

---

## 3. Error Cases

```rust
#[test]
fn test_transfer_insufficient_balance_returns_error() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, alice, client) = setup(&env);
    let bob = Address::generate(&env);
    client.mint(&admin, &alice, &50_i128);

    let result = client.try_transfer(&alice, &alice, &bob, &100_i128);
    assert_eq!(result.unwrap_err().unwrap(), Error::InsufficientBalance);
}

#[test]
fn test_mint_invalid_amount_returns_error() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, learner, client) = setup(&env);

    let result = client.try_mint(&admin, &learner, &0_i128);
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidAmount);
}

#[test]
fn test_mint_unauthorized_returns_error() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (_, _, client) = setup(&env);
    let attacker = Address::generate(&env);
    let victim = Address::generate(&env);

    let result = client.try_mint(&attacker, &victim, &100_i128);
    assert_eq!(result.unwrap_err().unwrap(), Error::Unauthorized);
}
```

---

## 4. Rate Limits

Soroban enforces resource limits (CPU instructions, memory, ledger reads/writes) per transaction. The following test verifies the contract stays within those limits under bulk operations.

```rust
#[test]
fn test_bulk_mints_stay_within_resource_limits() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    // 10 sequential mints — each in its own simulated tx
    for i in 1..=10_i128 {
        let recipient = Address::generate(&env);
        client.mint(&admin, &recipient, &(i * 10));
    }
    // If any mint exceeds resource limits, Soroban will panic
}
```

---

## Test Helper

```rust
fn setup(env: &Env) -> (Address, Address, TeachLinkContractClient) {
    let admin = Address::generate(env);
    let learner = Address::generate(env);
    let contract_id = env.register_contract(None, TeachLinkContract);
    let client = TeachLinkContractClient::new(env, &contract_id);
    client.initialize(&admin);
    (admin, learner, client)
}
```

---

## Running API Tests

```bash
cargo test api
# or run all tests
cargo test
```
