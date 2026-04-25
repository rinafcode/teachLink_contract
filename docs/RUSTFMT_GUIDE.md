# Rustfmt Configuration Guide

## Overview
This document describes the rustfmt configuration for the TeachLink contract project.

## Configuration File
The project uses [rustfmt.toml](rustfmt.toml) located at the workspace root to define consistent code formatting standards.

## Key Configuration Settings

### Code Layout
- **Max Width**: 100 columns - balances readability with modern screen sizes
- **Edition**: 2021 - latest Rust edition
- **Tab Spaces**: 4 - standard indentation
- **Hard Tabs**: False - uses spaces for consistency

### Imports and Modules
- **Imports Granularity**: Crate level - groups imports logically
- **Reorder Imports**: Enabled - maintains alphabetical order
- **Reorder Modules**: Enabled - consistent module ordering

### Code Formatting Rules
- **Use Field Init Shorthand**: Enables shorthand syntax for struct initialization
- **Use Try Shorthand**: Enables `?` operator use
- **Trailing Comma**: Vertical - adds trailing commas in multiline collections
- **Match Arm Blocks**: Forces consistent block formatting in match arms
- **Force Explicit ABI**: Prevents implicit C ABI in extern blocks

### Comments
- **Normalize Doc Attributes**: Standardizes documentation comment formatting
- **Comment Width**: 80 columns
- **Wrap Comments**: Automatically wraps long comments
- **Format Code in Doc Comments**: Applies formatting to code examples in docs
- **Format Macro Bodies**: Formats code within macros

### Function and Type Formatting
- **Function Arguments Layout**: Tall - each argument on separate line when formatted
- **Where Single Line**: False - places where clauses on new line if needed
- **Chain Width**: 60 - method chains wrap after this width

## Running Rustfmt

### Check Formatting (Dry Run)
```bash
cargo fmt --all -- --check
```

### Apply Formatting
```bash
cargo fmt --all
```

### Format Specific File
```bash
cargo fmt --path path/to/file.rs
```

## CI/CD Integration

The project has automatic formatting checks in GitHub Actions:

- **CI Pipeline** ([.github/workflows/ci.yml](.github/workflows/ci.yml)): Runs `cargo fmt --all -- --check` to verify formatting
- **Branch Protection**: Requires formatting to pass before PR merge
- **Docker Compose**: Development container includes formatting checks

### PR Requirements
All pull requests must pass the formatting check:
```bash
cargo fmt --all -- --check
```

## Editor Integration

### VS Code Setup
1. Install Rust-analyzer extension
2. Configure auto-format on save in settings.json:
```json
{
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### Pre-commit Hook (Optional)
To run formatting before commits, add to `.git/hooks/pre-commit`:
```bash
#!/bin/bash
cargo fmt --all -- --check
if [ $? -ne 0 ]; then
  echo "Code formatting failed. Run: cargo fmt --all"
  exit 1
fi
```

## Guidelines for Developers

1. **Before Committing**: Run `cargo fmt --all` to ensure consistency
2. **Editor Setup**: Enable auto-formatting for immediate feedback
3. **Respect the Configuration**: Don't override rustfmt settings locally
4. **Code Reviews**: Ensure formatting passes before merging PRs

## Additional Resources

- [Rustfmt Documentation](https://rust-lang.github.io/rustfmt/)
- [Configurable Properties](https://rust-lang.github.io/rustfmt/?version=v1.7.0&search=max_width)
- [Style Guide](https://doc.rust-lang.org/1.0.0/style/)

## Configuration Status

✅ **Configuration**: Fully configured in rustfmt.toml
✅ **CI Integration**: Checks enabled in GitHub Actions
✅ **Current State**: Codebase fully formatted and passing checks
✅ **Toolchain**: Rust stable with rustfmt and clippy installed
