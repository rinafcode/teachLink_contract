# Pre-commit Hooks Quick Reference

## Installation

```bash
# One command to install everything
./scripts/setup-hooks.sh
```

That's it! The hooks are now installed and will run automatically before each commit.

## What Happens on `git commit`

1. **Pre-commit hook runs** (`.git/hooks/pre-commit`):
   - ✓ Code formatting check with rustfmt
   - ✓ Clippy linter runs on all code
   - ✓ Merge conflict detection
   - ✓ Debug statement warnings
   - ✓ Script permission checking
   - ✓ Cargo validation

2. **If all checks pass**: Commit proceeds
3. **If any check fails**: Commit is blocked, error details shown

## Commit Message Format

```bash
git commit -m "type(scope): description"
```

### Examples

✅ Good commit messages:
```
feat(insurance): add policy renewal calculation
fix(market): resolve order matching bug
docs: update API reference
refactor(tokenization): simplify token minting
perf: optimize escrow lookups
test(bridge): add cross-chain integration tests
chore: update dependencies
```

❌ Bad commit messages:
```
fixed stuff
update code
WIP
changed things
```

## Bypass Hooks (Not Recommended)

```bash
git commit --no-verify -m "Your message"
```

## Fix Common Issues

### Formatting fails
```bash
cargo fmt --all
git add .
git commit -m "type: message"
```

### Clippy warnings
```bash
cargo clippy --workspace --all-features
# Fix the issues shown
git commit -m "type: message"
```

### Commit message rejected
Reword to follow: `type(scope): description`

Example: `feat(insurance): add renewal logic`

## Files Included

| File | Purpose |
|------|---------|
| `.pre-commit-config.yaml` | Optional framework config (run `pre-commit install`) |
| `.git/hooks/pre-commit` | Main validation hook |
| `.git/hooks/commit-msg` | Commit message validator |
| `scripts/pre-commit` | Source script for pre-commit hook |
| `scripts/commit-msg` | Source script for commit-msg hook |
| `scripts/setup-hooks.sh` | Installation script |
| `docs/PRE_COMMIT_HOOKS_GUIDE.md` | Detailed documentation |

## Useful Commands

```bash
# Run pre-commit manually
.git/hooks/pre-commit

# Format all code
cargo fmt --all

# Run clippy
cargo clippy --workspace --all-features -- -D warnings

# Run tests
cargo test --workspace

# View hook files
cat .git/hooks/pre-commit
cat .git/hooks/commit-msg
```

## Need Help?

See [docs/PRE_COMMIT_HOOKS_GUIDE.md](../../docs/PRE_COMMIT_HOOKS_GUIDE.md) for detailed documentation and troubleshooting.
