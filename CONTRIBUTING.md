# Contributing to TeachLink

Thank you for your interest in contributing to TeachLink! This document provides comprehensive guidelines for contributing to our decentralized knowledge-sharing platform on Stellar.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [How to Contribute](#how-to-contribute)
- [Issue Guidelines](#issue-guidelines)
- [Pull Request Process](#pull-request-process)
- [Development Standards](#development-standards)
- [Review Process](#review-process)
- [Recognition & Rewards](#recognition--rewards)

---

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md). We are committed to providing a welcoming and inclusive environment for all contributors.

---

## Getting Started

### Prerequisites

Before contributing, ensure you have:

1. **Rust toolchain** installed with `wasm32-unknown-unknown` target
2. **Stellar CLI** or **Soroban CLI** installed
3. A GitHub account
4. Basic understanding of Soroban smart contracts

### Setup

```bash
# Clone the repository
git clone https://github.com/rinafcode/teachLink_contract.git
cd teachLink_contract

# Run environment setup
./scripts/setup-env.sh

# Build the project
cargo build --release --target wasm32-unknown-unknown

# Run tests
cargo test
```

---

## How to Contribute

### Types of Contributions

We welcome various types of contributions:

| Type                 | Description                         | Label           |
| -------------------- | ----------------------------------- | --------------- |
| 🐛 **Bug Fixes**     | Fix issues in existing code         | `bug`           |
| ✨ **Features**      | Add new functionality               | `enhancement`   |
| 📚 **Documentation** | Improve docs, comments, or examples | `documentation` |
| 🧪 **Tests**         | Add or improve test coverage        | `testing`       |
| 🔧 **Tooling**       | Improve scripts, CI/CD, or DX       | `tooling`       |
| 🔒 **Security**      | Security improvements or fixes      | `security`      |
| ♿ **Accessibility** | Improve accessibility               | `accessibility` |

### Contribution Workflow

```
1. Find or Create an Issue
         ↓
2. Fork & Create Branch
         ↓
3. Make Changes
         ↓
4. Write/Update Tests
         ↓
5. Run Lints & Tests
         ↓
6. Submit Pull Request
         ↓
7. Address Review Feedback
         ↓
8. Merge! 🎉
```

---

## Issue Guidelines

### Before Creating an Issue

1. **Search existing issues** to avoid duplicates
2. **Check the FAQ** and troubleshooting guide
3. **Gather relevant information** (logs, screenshots, reproduction steps)

### Issue Types

#### 🐛 Bug Reports

Use the bug report template and include:

- Clear, descriptive title
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, Rust version, CLI version)
- Relevant logs or error messages

#### ✨ Feature Requests

Use the feature request template and include:

- Problem statement
- Proposed solution
- Alternative solutions considered
- Additional context

#### 📋 Tasks

For internal project tasks:

- Clear description of the work
- Acceptance criteria
- Dependencies on other issues

### Issue Labels

| Label                | Description                           |
| -------------------- | ------------------------------------- |
| `priority: critical` | Must be addressed immediately         |
| `priority: high`     | Should be addressed in current sprint |
| `priority: medium`   | Should be addressed soon              |
| `priority: low`      | Nice to have, no urgency              |
| `good first issue`   | Good for newcomers                    |
| `help wanted`        | Extra attention needed                |
| `blocked`            | Waiting on external dependency        |
| `wontfix`            | Won't be worked on                    |

---

## Pull Request Process

### Before Submitting

1. **Create an issue first** for significant changes
2. **Fork the repository** and create a feature branch
3. **Follow naming conventions**: `feature/`, `fix/`, `docs/`, `refactor/`
4. **Keep PRs focused** - one logical change per PR

### PR Requirements

- [ ] Code follows project style guidelines
- [ ] All tests pass (`cargo test`)
- [ ] Lints pass (`cargo fmt && cargo clippy --all-targets --all-features`)
- [ ] Documentation updated if needed
- [ ] Commit messages follow conventions
- [ ] PR description explains changes clearly

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Adding/updating tests
- `chore`: Maintenance tasks

**Example:**

```
feat(contract): add learning reward distribution

Implements the reward distribution mechanism for completed lessons.
Tokens are distributed proportionally based on participation score.

Closes #42
```

### Review Process

1. **Automated checks** run first (CI/CD)
2. **Code review** by at least one maintainer
3. **Security review** for sensitive changes
4. **Final approval** and merge

---

## Development Standards

### Code Style

- Follow Rust idioms and best practices
- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Write self-documenting code with clear naming

### Linting Standards

All Rust code must pass `cargo clippy --all-targets --all-features -- -D warnings` with no warnings.

**Clippy configuration** is centralised in `[workspace.lints.clippy]` inside `Cargo.toml`.
The following lints are project-wide allows due to Soroban SDK constraints:

| Lint                                             | Reason                                                                         |
| ------------------------------------------------ | ------------------------------------------------------------------------------ |
| `needless_pass_by_value`                         | Soroban SDK contract functions require owned values in signatures              |
| `must_use_candidate`                             | Contract functions are invoked via WASM ABI, not from Rust                     |
| `missing_panics_doc` / `missing_errors_doc`      | Internal contracts don't need full rustdoc coverage                            |
| `doc_markdown`                                   | Contract docstrings don't need markdown link formatting                        |
| `panic_in_result_fn`                             | Soroban contracts may panic inside result-returning functions                  |
| `too_many_arguments`                             | Cross-chain APIs require many params; splitting would break the WASM ABI       |
| `trivially_copy_pass_by_ref` / `needless_borrow` | Soroban SDK semantics sometimes require owned copies                           |
| `unreadable_literal`                             | Addresses/hashes are byte-level values where underscores can obscure structure |
| `assertions_on_constants`                        | Soroban test macros emit constant-assertion patterns                           |
| `too_many_lines`                                 | Contract setup and test functions are inherently long                          |

**Rules for adding a new `#[allow(clippy::...)]`:**

1. **Do not use `#![allow(clippy::all)]`** — this is forbidden and will block CI.
2. Suppressions **must be as narrow as possible** — prefer an item-level `#[allow]` over a file-level `#![allow]`.
3. Every suppression **must include a comment** explaining why it is necessary.
4. If a lint applies broadly to Soroban patterns, add it to `[workspace.lints.clippy]` in `Cargo.toml` with a comment, rather than scattering `#![allow]` across files.
5. Never suppress safety-related lints (`cast_possible_truncation`, `integer_arithmetic`, `indexing_slicing`) without a code-level comment justifying the safety invariant.

**Running Clippy locally:**

```bash
# Check for warnings (same as CI)
cargo clippy --all-targets --all-features -- -D warnings

# Or use the convenience script
./scripts/lint.sh --check

# Auto-fix where possible
./scripts/lint.sh --fix
```

### Testing Requirements

- All new features must have unit tests
- Bug fixes should include regression tests
- Integration tests for contract interactions
- Minimum 80% code coverage for new code

### Documentation Requirements

- Document all public functions with doc comments
- Update README for user-facing changes
- Add inline comments for complex logic
- Include examples in documentation

### Security Guidelines

- Never commit secrets or private keys
- Follow Soroban security best practices
- Report security issues privately (see SECURITY.md)
- Review dependencies for vulnerabilities

---

## Review Process

### Review Timeline

| PR Size                  | Expected Review Time |
| ------------------------ | -------------------- |
| Small (< 50 lines)       | 1-2 days             |
| Medium (50-200 lines)    | 2-3 days             |
| Large (200-500 lines)    | 3-5 days             |
| Extra Large (500+ lines) | 5-7 days             |

### Review Criteria

- **Correctness**: Does the code work as intended?
- **Security**: Are there any security concerns?
- **Performance**: Are there performance implications?
- **Maintainability**: Is the code easy to understand and maintain?
- **Testing**: Is the code adequately tested?
- **Documentation**: Is the code properly documented?

### Reviewer Guidelines

- Be respectful and constructive
- Explain the reasoning behind suggestions
- Distinguish between required changes and suggestions
- Approve promptly when requirements are met

---

## Recognition & Rewards

We believe in recognizing and rewarding contributors for their valuable work.

### Contributor Tiers

| Tier                       | Requirements                           | Badge             |
| -------------------------- | -------------------------------------- | ----------------- |
| 🌱 **Newcomer**            | First contribution merged              | Newcomer Badge    |
| 🌿 **Contributor**         | 3+ contributions merged                | Contributor Badge |
| 🌳 **Regular Contributor** | 10+ contributions merged               | Regular Badge     |
| 🏆 **Core Contributor**    | 25+ contributions + consistent quality | Core Badge        |
| ⭐ **Maintainer**          | Invited by existing maintainers        | Maintainer Badge  |

### Recognition Programs

#### Monthly MVP

Each month, we recognize one contributor who has made exceptional contributions.

**Selection Criteria:**

- Quality of contributions
- Helpfulness in community
- Innovation and creativity
- Code review participation

#### Hall of Fame

Contributors with significant impact are featured in our [HALL_OF_FAME.md](docs/governance/HALL_OF_FAME.md).

#### Token Rewards

Active contributors may be eligible for TEACH token rewards:

| Achievement                | Reward          |
| -------------------------- | --------------- |
| First merged PR            | 50 TEACH        |
| Bug fix                    | 100-500 TEACH   |
| Feature implementation     | 500-2000 TEACH  |
| Security vulnerability fix | 1000-5000 TEACH |
| Documentation improvements | 50-200 TEACH    |
| Monthly MVP                | 1000 TEACH      |

_Token rewards are subject to availability and maintainer approval._

### Attribution

All contributors are listed in:

- [CONTRIBUTORS.md](CONTRIBUTORS.md) - Full contributor list
- Git commit history
- Release notes for significant contributions

---

## Getting Help

- 💬 **Discord**: [Join our community](https://discord.gg/teachlink)
- 📧 **Email**: contributors@teachlink.io
- 🐦 **Twitter**: [@TeachLinkDAO](https://twitter.com/teachlinkdao)
- 📖 **Documentation**: [docs.teachlink.io](https://docs.teachlink.io)

---

## License

By contributing to TeachLink, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to TeachLink! Together, we're building the future of decentralized education. 🎓
