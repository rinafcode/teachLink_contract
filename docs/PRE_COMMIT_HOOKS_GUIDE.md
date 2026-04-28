# Pre-commit Hooks Setup Guide

## Overview

Pre-commit hooks are automated checks that run before each commit, ensuring code quality and consistency. The TeachLink contract project includes both Git hooks and an optional pre-commit framework configuration.

## Installation

### Option 1: Using Git Hooks (Recommended for Rust projects)

Install the local Git hooks:

```bash
./scripts/setup-hooks.sh
```

Or manually:

```bash
chmod +x scripts/pre-commit scripts/commit-msg
cp scripts/pre-commit .git/hooks/pre-commit
cp scripts/commit-msg .git/hooks/commit-msg
```

### Option 2: Using Pre-commit Framework (Language-agnostic)

Install pre-commit:

```bash
pip install pre-commit
```

Install the hooks:

```bash
pre-commit install
```

Update hooks to latest versions:

```bash
pre-commit autoupdate
```

## What Gets Checked

### Pre-commit Hook Checks

1. **Rustfmt (Code Formatting)**
   - Ensures all Rust code follows the project's formatting standards
   - Automatically formatted with `cargo fmt --all`

2. **Clippy (Linting)**
   - Rust linter that catches common mistakes
   - All warnings treated as errors (`-D warnings`)

3. **Merge Conflict Detection**
   - Prevents committing files with merge conflict markers

4. **Debug Statement Detection**
   - Warns about `println!`, `dbg!`, `eprintln!` left in code
   - Non-blocking (allows commit with warning)

5. **Script Permissions**
   - Ensures shell scripts have execute permissions

6. **Cargo Validation**
   - Validates `Cargo.toml` files

### Commit-msg Hook Checks

Validates commit messages follow **Conventional Commits** format:

```
type(scope): description
```

**Valid types:**
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Formatting or code style changes
- `refactor`: Code restructuring without feature changes
- `perf`: Performance improvements
- `test`: Test additions or corrections
- `chore`: Build system, dependency updates, tooling
- `ci`: CI/CD configuration changes
- `build`: Build system changes

**Examples:**

```
feat(insurance): add policy renewal calculation
fix(market): resolve order matching bug
docs: update API reference
refactor(tokenization): simplify token minting logic
perf: optimize escrow lookups
```

### Optional: Pre-commit Framework

The `.pre-commit-config.yaml` includes additional checks:

- **File quality**: Trailing whitespace, EOF fixes, mixed line endings
- **YAML/TOML/JSON validation**: Syntax checking
- **Private key detection**: Prevents credential leaks
- **Markdown linting**: Documentation quality
- **Spell checking**: Catches typos in code and docs

## Running Checks Manually

### Git Hooks

```bash
# Run pre-commit checks
.git/hooks/pre-commit

# Test commit message validation
.git/hooks/commit-msg <(echo "feat: test commit")
```

### Cargo-based Checks

```bash
# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --workspace --all-features -- -D warnings

# Run tests
cargo test --workspace
```

### Pre-commit Framework

```bash
# Run all hooks on staged files
pre-commit run --hook-stage commit

# Run all hooks on all files
pre-commit run --all-files

# Run specific hook
pre-commit run rustfmt --all-files

# Bypass hooks temporarily (not recommended)
git commit --no-verify
```

## Fixing Common Issues

### Formatting Issues

```bash
# Auto-fix formatting
cargo fmt --all

# Verify fix
cargo fmt --all -- --check
```

### Clippy Warnings

```bash
# Run clippy to see all issues
cargo clippy --workspace --all-features

# Fix common issues
cargo fix --allow-dirty
```

### Commit Message Format

Ensure your commit message follows the pattern:

```
type(scope): description
```

For example:

```bash
git commit -m "feat(insurance): add policy renewal"
```

## Configuration

### Rustfmt Configuration

See [rustfmt.toml](../../rustfmt.toml) for formatting rules.

### Pre-commit Configuration

See [.pre-commit-config.yaml](../../.pre-commit-config.yaml) for framework hooks.

### Git Hooks Scripts

- [scripts/pre-commit](../pre-commit) - Main pre-commit validation
- [scripts/commit-msg](../commit-msg) - Commit message validation
- [scripts/setup-hooks.sh](../setup-hooks.sh) - Hook installation script

## CI/CD Integration

The project's GitHub Actions automatically run these checks:

- `.github/workflows/ci.yml`: Full CI pipeline with formatting and linting
- `.github/workflows/regression.yml`: Additional regression tests
- Branch protection rules enforce passing checks before merge

## Troubleshooting

### Hooks not running

```bash
# Verify hooks are executable
ls -la .git/hooks/pre-commit .git/hooks/commit-msg

# Re-install hooks
./scripts/setup-hooks.sh
```

### Bypass hooks (use sparingly)

```bash
# Skip pre-commit checks
git commit --no-verify -m "chore: emergency fix"

# Skip commit-msg validation
git commit --no-verify -m "WIP"
```

### Update hooks

```bash
# With pre-commit framework
pre-commit autoupdate

# With git hooks
git pull  # Get latest scripts
./scripts/setup-hooks.sh  # Reinstall
```

## Best Practices

1. **Always let hooks run** - They catch mistakes before they reach the repository
2. **Fix issues locally** - Don't bypass hooks with `--no-verify`
3. **Follow commit conventions** - Makes git history readable and searchable
4. **Keep hooks updated** - Run `pre-commit autoupdate` regularly
5. **Review hook output** - Understand what failed and why

## Related Documentation

- [Rustfmt Guide](../RUSTFMT_GUIDE.md)
- [Contributing Guidelines](../../CONTRIBUTING.md)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Git Hooks Documentation](https://git-scm.com/docs/githooks)
