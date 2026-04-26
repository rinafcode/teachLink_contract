# TeachLink Contract Error Handling Guide

## Overview

This document outlines the consistent error handling patterns implemented across the TeachLink smart contract. All modules must follow these guidelines to ensure predictable, safe, and maintainable error propagation.

## Principles

1. **No Silent Failures**: Never use `.unwrap()` or `.expect()` in production code
2. **Result-Based**: All fallible operations return `Result<T, E>`
3. **Specific Errors**: Use specific error variants, not generic errors
4. **Proper Propagation**: Use `?` operator to propagate errors up the call stack
5. **Clear Authorization Failures**: Use `Unauthorized` errors instead of panics
6. **Storage Safety**: Handle missing storage gracefully with defaults or errors

## Error Hierarchy

### Core Error Enums

```
BridgeError (100-147)
├── AlreadyInitialized
├── AmountMustBePositive
├── DestinationChainNotSupported
├── InsufficientValidatorSignatures
├── ... other bridge-specific errors

EscrowError (200-227)
├── AmountMustBePositive
├── AtLeastOneSignerRequired
├── ... other escrow-specific errors

RewardsError (300-309)
├── AlreadyInitialized
├── AmountMustBePositive
├── StorageError
├── ... other rewards-specific errors

AccessControlError (NEW)
├── Unauthorized
├── InvalidRole
├── MissingRole

AnalyticsError (NEW)
├── InvalidIndex
├── InsufficientData
```

## Patterns

### Pattern 1: Result-Based Function (CORRECT)

```rust
pub fn update_reputation(env: &Env, user: Address, points: u32) -> Result<(), ReputationError> {
    user.require_auth();

    if points > MAX_POINTS {
        return Err(ReputationError::InvalidPoints);
    }

    let mut reputation = get_reputation(env, &user)?; // Propagate errors
    reputation.score += points;
    set_reputation(env, &user, &reputation)?;

    Ok(())
}
```

### Pattern 2: Authorization Errors

```rust
// WRONG - Direct panic
pub fn check_role(env: &Env, address: &Address, role: AccessRole) {
    if !Self::has_role(env, address, role) {
        panic!("Unauthorized: Missing required role");
    }
}

// CORRECT - Return error
pub fn check_role(env: &Env, address: &Address, role: AccessRole) -> Result<(), AccessControlError> {
    address.require_auth();
    if !Self::has_role(env, address, role) {
        return Err(AccessControlError::MissingRole);
    }
    Ok(())
}
```

### Pattern 3: Storage Access

```rust
// WRONG - Silent default or panic
let value = storage.get(&key).unwrap_or_default();

// CORRECT - Explicit handling
let value = storage
    .get(&key)
    .ok_or(StorageError::DataNotFound)?;
```

### Pattern 4: Vec Operations

```rust
// WRONG - Panics on index out of bounds
let item = vec.get(index).unwrap();

// CORRECT - Explicit handling
let item = vec.get(index)
    .ok_or(AnalyticsError::InvalidIndex)?;
```

### Pattern 5: Error Conversion

```rust
// For Result-to-Result conversions
some_operation().map_err(|_| ContractError::OperationFailed)?;

// For Option-to-Result conversions
value.ok_or(ContractError::NotFound)?;
```

## Implementation Checklist

- [ ] Replace all `.unwrap()` with `?` operator or `ok_or()`
- [ ] Replace all `.expect()` with explicit error returns
- [ ] Replace all `assert!()` with `if condition { return Err(...) }`
- [ ] Replace all `panic!()` with `Err()` returns
- [ ] Add specific error variants for each error condition
- [ ] Ensure all public functions return `Result<T, E>`
- [ ] Update callers to handle `Result` types
- [ ] Add tests for error paths

## Testing Error Paths

All error variants must have corresponding tests:

```rust
#[test]
fn test_unauthorized_access_returns_error() {
    let env = Env::default();
    let unauthorized_user = Address::generate(&env);

    let result = protected_function(&env, unauthorized_user);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ExpectedError::Unauthorized);
}

#[test]
fn test_invalid_input_returns_error() {
    let env = Env::default();
    let result = function_with_validation(&env, invalid_input);

    assert!(result.is_err());
    match result.unwrap_err() {
        ExpectedError::InvalidInput => {},
        _ => panic!("Expected InvalidInput error"),
    }
}
```

## Error Codes Reference

See [errors.rs](src/errors.rs) for complete error code reference. Error codes are grouped by module:

- 100-147: Bridge errors
- 200-227: Escrow errors
- 300-308: Rewards errors
- 400-407: Mobile Platform errors
- 500-599: New error codes

## Migration Guide

### Step 1: Add Error Enum

Define all error variants for the module:

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MyModuleError {
    NotInitialized = 500,
    InvalidInput = 501,
    Unauthorized = 502,
}

pub type MyModuleResult<T> = core::result::Result<T, MyModuleError>;
```

### Step 2: Replace Panics

Convert all panics to error returns:

```rust
// Before
pub fn validate(value: u32) {
    assert!(value > 0, "Value must be positive");
}

// After
pub fn validate(value: u32) -> Result<(), MyModuleError> {
    if value == 0 {
        return Err(MyModuleError::InvalidInput);
    }
    Ok(())
}
```

### Step 3: Update Callers

Ensure all callers handle the Result:

```rust
// Before
validate(value); // Could panic

// After
validate(value)?; // Properly propagates error
```

## Common Mistakes

| Mistake                | Fix                                        |
| ---------------------- | ------------------------------------------ |
| `obj.unwrap()`         | `obj.ok_or(Error::...)?)`                  |
| `vec.get(i).unwrap()`  | `vec.get(i).ok_or(Error::InvalidIndex)?`   |
| `panic!("msg")`        | `return Err(Error::...)`                   |
| `assert!(cond, "msg")` | `if !cond { return Err(...) }`             |
| `.unwrap_or_default()` | `.ok_or(Error::...)` if error is needed    |
| `.unwrap_or(fallback)` | `.ok_or(Error::...)` or explicit if needed |

## Questions?

Reference the specific module documentation or review error handling examples in:

- `src/rewards.rs` - Good error handling examples
- `src/bridge.rs` - Result-based design
- `src/validation.rs` - Validation patterns
