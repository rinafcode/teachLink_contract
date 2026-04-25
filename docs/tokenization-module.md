# Tokenization Module Documentation

> Closes #348

## Overview

The tokenization module manages the lifecycle of TeachLink tokens (TLT): minting rewards, transferring balances, and querying supply. It is implemented as part of the Soroban smart contract and follows the Stellar token interface conventions.

---

## Functions

### `mint`

Mints new tokens and credits them to a recipient address. Only callable by the contract admin.

```rust
pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), Error>
```

| Parameter | Type      | Description                        |
|-----------|-----------|------------------------------------|
| `env`     | `Env`     | Soroban environment handle         |
| `to`      | `Address` | Recipient address                  |
| `amount`  | `i128`    | Number of tokens to mint (> 0)     |

**Errors**: `Unauthorized`, `InvalidAmount`, `OverflowError`

**Example**:
```rust
client.mint(&admin, &learner_address, &500_i128);
```

---

### `transfer`

Transfers tokens from one address to another. The sender must authorize the call.

```rust
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), Error>
```

| Parameter | Type      | Description                        |
|-----------|-----------|------------------------------------|
| `env`     | `Env`     | Soroban environment handle         |
| `from`    | `Address` | Sender address (must authorize)    |
| `to`      | `Address` | Recipient address                  |
| `amount`  | `i128`    | Number of tokens to transfer (> 0) |

**Errors**: `Unauthorized`, `InvalidAmount`, `InsufficientBalance`

**Example**:
```rust
client.transfer(&sender, &sender, &recipient, &100_i128);
```

---

### `balance`

Returns the token balance of a given address.

```rust
pub fn balance(env: Env, account: Address) -> i128
```

| Parameter | Type      | Description              |
|-----------|-----------|--------------------------|
| `env`     | `Env`     | Soroban environment handle |
| `account` | `Address` | Address to query         |

**Returns**: Token balance as `i128` (0 if account has no balance).

**Example**:
```rust
let bal = client.balance(&learner_address);
assert!(bal >= 0);
```

---

### `total_supply`

Returns the total number of tokens currently in circulation.

```rust
pub fn total_supply(env: Env) -> i128
```

**Returns**: Total minted supply as `i128`.

---

## Usage Example

```rust
#[test]
fn test_tokenization_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let learner = Address::generate(&env);
    let contract_id = env.register_contract(None, TeachLinkContract);
    let client = TeachLinkContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    // Mint tokens to learner
    client.mint(&admin, &learner, &1000_i128);
    assert_eq!(client.balance(&learner), 1000);

    // Transfer tokens
    client.transfer(&learner, &learner, &admin, &200_i128);
    assert_eq!(client.balance(&learner), 800);
    assert_eq!(client.balance(&admin), 200);

    // Check total supply
    assert_eq!(client.total_supply(), 1000);
}
```

---

## Notes

- All amounts are represented as `i128` to support Stellar's token precision.
- The module stores balances in Soroban's persistent ledger storage under each address key.
- Token decimals follow the Stellar standard of 7 decimal places.
