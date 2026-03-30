# Documentation Standards - TeachLink Smart Contracts

This document outlines the standard format for documenting smart contract interfaces in the TeachLink project. Consistent documentation improves developer experience and allows for automated documentation generation.

## Doc Comment Standard

All public-facing contract functions must use Rust doc comments (`///`) following this structure:

1.  **Summary**: A one-line summary of what the function does.
2.  **Description**: (Optional) Detailed explanation of the function's behavior, side effects, and logic.
3.  **Arguments**: A `# Arguments` section listing each parameter with its name and description.
4.  **Returns**: A `# Returns` section describing the return value.
5.  **Errors**: An `# Errors` section listing potential error variants and the conditions under which they are returned.
6.  **Events**: (Optional) An `# Events` section describing the events emitted by the function.
7.  **Examples**: A `# Examples` section with one or more code snippets demonstrating how to use the function.

### Template

```rust
/// [Summary]
///
/// [Detailed description...]
///
/// # Arguments
///
/// * `[arg1]` - [Description]
/// * `[arg2]` - [Description]
///
/// # Returns
///
/// * `[ReturnType]` - [Description]
///
/// # Errors
///
/// * `[ErrorVariant]` - [Condition]
///
/// # Events
///
/// * `[EventSymbol]` - [Description]
///
/// # Examples
///
/// ```rust
/// let result = contract.my_function(env, arg1, arg2);
/// ```
```

## Automated Documentation Generation

We use `cargo doc` to generate HTML documentation from these comments.

### Commands

```bash
# Generate documentation
cargo doc --no-deps --document-private-items

# View documentation
# open target/doc/teachlink/index.html
```

## CI/CD Validation

The CI/CD pipeline ensures that:
1.  All public functions have doc comments.
2.  Documentation builds without errors.
3.  Examples for public functions are present.

To check for missing documentation locally:

```bash
cargo clippy -- -D missing-docs
```

(Note: You may need to enable `#![warn(missing_docs)]` in your crate root).
