# TeachLink Coding Standards - Quick Reference

**Version:** 1.0.0  
**Last Updated:** March 29, 2026

## Quick Start Guide

### For New Developers

1. **Read the Standards**: Start with `CODING_STANDARDS.md`
2. **Setup Environment**: Follow `docs/DEVELOPER_TRAINING.md`
3. **Run Checks**: Use `./scripts/lint.sh` before committing
4. **Ask Questions**: Join Discord for support

### For Existing Developers

1. **Review Changes**: Check `.rustfmt.toml` for new formatting rules
2. **Update Workflow**: Integrate new linting into your process
3. **Train Team**: Use training materials for team education
4. **Provide Feedback**: Help improve the standards

## Essential Commands

### Code Formatting

```bash
# Format all code
cargo fmt --all

# Check formatting (CI)
cargo fmt --all -- --check
```

### Linting and Analysis

```bash
# Run clippy with warnings as errors
cargo clippy --all-targets --all-features -- -D warnings

# Run all linting (formatting + clippy)
./scripts/lint.sh

# Fix automatically fixable issues
./scripts/lint.sh --fix
```

### Testing

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html

# Run specific contract tests
cargo test -p teachlink-contract
```

### Security Scanning

```bash
# Check for known vulnerabilities
cargo audit

# Check for security issues
cargo deny check
```

## Code Style Quick Reference

### Naming Conventions

| Element   | Convention             | Example              |
| --------- | ---------------------- | -------------------- |
| Functions | `snake_case`           | `bridge_out()`       |
| Variables | `snake_case`           | `bridge_transaction` |
| Modules   | `snake_case`           | `bridge`             |
| Structs   | `PascalCase`           | `BridgeTransaction`  |
| Enums     | `PascalCase`           | `BridgeError`        |
| Traits    | `PascalCase`           | `BridgeInterface`    |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_VALIDATORS`     |

### Function Documentation Template

````rust
/// Brief description of what the function does
///
/// More detailed explanation if needed.
///
/// # Arguments
///
/// * `param1` - Description of first parameter
/// * `param2` - Description of second parameter
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// Description of possible errors
///
/// # Examples
///
/// ```
/// let result = function_name(arg1, arg2);
/// assert_eq!(result, expected_value);
/// ```
pub fn function_name(param1: Type1, param2: Type2) -> ReturnType {
    // Implementation
}
````

### Error Handling Pattern

```rust
// Define custom errors
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BridgeError {
    InvalidInput = 1,
    InsufficientBalance = 2,
    ChainNotSupported = 3,
}

impl Into<Error> for BridgeError {
    fn into(self) -> Error {
        Error::from_contract_error(self as u32)
    }
}

// Use in functions
pub fn example_function(input: i128) -> Result<(), BridgeError> {
    // Input validation
    if input <= 0 {
        return Err(BridgeError::InvalidInput);
    }

    // Process with error propagation
    let result = process_input(input)?;

    Ok(())
}
```

### Security Patterns

#### Input Validation

```rust
pub fn secure_function(
    env: Env,
    user: Address,
    amount: i128,
    destination: Bytes,
) -> Result<u64, BridgeError> {
    // Validate inputs
    if amount <= 0 {
        return Err(BridgeError::InvalidInput);
    }

    if destination.len() == 0 {
        return Err(BridgeError::InvalidInput);
    }

    // Authorization
    user.require_auth();

    // Process
    // ...
}
```

#### Reentrancy Protection

```rust
pub fn withdraw_rewards(env: Env, user: Address) -> Result<(), BridgeError> {
    // Check lock
    let in_withdrawal = env.storage().instance()
        .get::<Address, bool>(&Symbol::new(&env, "WITHDRAWAL_LOCK"))
        .unwrap_or(false);

    if in_withdrawal {
        return Err(BridgeError::ReentrancyDetected);
    }

    // Set lock
    env.storage().instance().set(&Symbol::new(&env, "WITHDRAWAL_LOCK"), &true);

    // Perform operation
    // ...

    // Release lock
    env.storage().instance().remove::<Address, bool>(&Symbol::new(&env, "WITHDRAWAL_LOCK"));

    Ok(())
}
```

#### Arithmetic Safety

```rust
pub fn safe_arithmetic(a: i128, b: i128) -> Result<i128, BridgeError> {
    a.checked_add(b)
        .ok_or(BridgeError::ArithmeticOverflow)
}
```

### Testing Patterns

#### Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::{Address as _, MockAuth, MockAuthInvoke}, vec, Address, Bytes, Env};

    #[test]
    fn test_function_name() {
        // Setup
        let env = Env::default();
        let contract_id = env.register_contract(None, TeachLinkBridge);
        let client = TeachLinkBridgeClient::new(&env, &contract_id);

        // Test data
        let user = Address::generate(&env);
        let test_value = 1000;

        // Mock authorizations
        env.mock_all_auths();

        // Execute
        let result = client.function_name(&user, &test_value);

        // Verify
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_value);
    }
}
```

#### Mocking Examples

```rust
// Mock token transfer
env.mock_auths(&[
    MockAuth {
        address: &user,
        invoke: &MockAuthInvoke {
            contract: &token,
            fn_name: "transfer",
            args: (&user, &contract_id, &amount).into_val(&env),
            sub_invokes: &[],
        },
    }
]);

// Mock all authorizations
env.mock_all_auths();
```

## Git Workflow

### Branch Naming

```
feature/bridge-optimization
fix/validator-slashing-bug
docs/coding-standards
```

### Commit Message Format

```
<type>(<scope>): <description>

<body>

<footer>
```

**Types:**

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**

```
feat(bridge): add dynamic fee calculation

Implements dynamic fee calculation based on network congestion
and transaction volume. Fees adjust automatically to maintain
optimal bridge performance.

Closes #123

fix(validator): resolve consensus timeout issue

Fixes issue where validators could timeout during consensus
reaching due to incorrect timestamp handling.

fixes #789
```

### Pull Request Checklist

Before submitting a PR:

- [ ] Code follows style guidelines
- [ ] All tests pass
- [ ] Linting passes (`./scripts/lint.sh`)
- [ ] Documentation updated if needed
- [ ] Commit messages follow conventions
- [ ] PR description explains changes clearly
- [ ] No sensitive information in code or comments

## Common Pitfalls to Avoid

### ❌ Bad Practices

```rust
// No documentation
pub fn bridge(env: Env, a: Address, b: i128, c: u32, d: Bytes) -> u64 { ... }

// Poor naming
struct BT { n: u64, t: Address, a: i128, ... }

// No error handling
pub fn process_bridge(env: Env, tx: BT) {
    env.storage().set(&tx.n, &tx);
    // No validation, no authorization, no error returns
}

// Inconsistent formatting
pub fn calculate_fee(amount:i128,rate:u32)->i128{
    amount*rate/10000
}

// Magic numbers
const FEE_RATE:u32=25;
const MIN_VAL:u32=3;
```

### ✅ Good Practices

```rust
// Well-documented
/// Bridge tokens out to another chain (lock/burn tokens on Stellar)
///
/// This function locks the specified amount of tokens in the bridge contract
/// and creates a cross-chain message for processing by validators.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `from` - The address sending the tokens
/// * `amount` - The amount of tokens to bridge
/// * `destination_chain` - The target chain ID
/// * `destination_address` - The recipient address on the destination chain
///
/// # Returns
///
/// Returns the bridge transaction nonce on success, or an error on failure.
pub fn bridge_out(
    env: Env,
    from: Address,
    amount: i128,
    destination_chain: u32,
    destination_address: Bytes,
) -> Result<u64, BridgeError> {
    // Implementation with proper validation and error handling
}

// Clear naming
struct BridgeTransaction {
    pub nonce: u64,
    pub token: Address,
    pub amount: i128,
    pub recipient: Address,
    pub destination_chain: u32,
    pub destination_address: Bytes,
    pub timestamp: u64,
}

// Proper error handling
pub fn process_bridge(env: Env, tx: BridgeTransaction) -> Result<(), BridgeError> {
    // Input validation
    if tx.amount <= 0 {
        return Err(BridgeError::InvalidInput);
    }

    // Authorization check
    tx.recipient.require_auth();

    // Process with error propagation
    // ...

    Ok(())
}

// Consistent formatting
pub fn calculate_fee(amount: i128, fee_rate: u32) -> Result<i128, BridgeError> {
    if fee_rate > 10000 {
        return Err(BridgeError::InvalidInput);
    }

    amount
        .checked_mul(fee_rate as i128)
        .ok_or(BridgeError::ArithmeticOverflow)?
        .checked_div(10000)
        .ok_or(BridgeError::ArithmeticOverflow)
}

// Named constants
const MAX_FEE_RATE: u32 = 10000; // 100% in basis points
const MIN_VALIDATORS: u32 = 3;
```

## Resources

### Documentation

- **Main Standards**: `CODING_STANDARDS.md`
- **Training Guide**: `docs/DEVELOPER_TRAINING.md`
- **Implementation Checklist**: `docs/CODING_STANDARDS_IMPLEMENTATION.md`

### Tools and Scripts

- **Linting**: `./scripts/lint.sh`
- **Testing**: `./scripts/test.sh`
- **Building**: `./scripts/build.sh`
- **Environment Setup**: `./scripts/setup-env.sh`

### Configuration Files

- **Rust Formatting**: `.rustfmt.toml`
- **Workspace Settings**: `Cargo.toml`
- **IDE Settings**: `.vscode/settings.json`

### External Resources

- [Soroban Documentation](https://soroban.stellar.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

## Getting Help

### Team Support

- **Discord**: [Join our community](https://discord.gg/teachlink)
- **Email**: contributors@teachlink.io
- **Issues**: [GitHub Issues](https://github.com/rinafcode/teachLink_contract/issues)

### Mentorship

New developers are paired with experienced mentors:

- Weekly check-ins
- Code review guidance
- Architecture discussions
- Best practices explanation

### Continuous Learning

- Monthly tech talks
- Code review sessions
- Security workshops
- Performance reviews

---

**Remember**: These standards are here to help you write better, more secure, and more maintainable code. When in doubt, ask for help or review the detailed documentation.

**Last Updated**: March 29, 2026  
**Next Review**: September 2026
