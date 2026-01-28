<div align="center">

```
                TTTTT eeeee aaaaa ccccc h   h L     i     n   n k   k
                  T   e     a   a c     h   h L           nn  n k  k
                  T   eeee  aaaaa c     hhhhh L     i     n n n kkk
                  T   e     a   a c     h   h L     i     n  nn k  k
                  T   eeeee a   a ccccc h   h LLLLL i     n   n k   k
```

</div>

# TeachLink: Decentralized Knowledge-Sharing on Stellar

TeachLink is a Soroban smart contract that powers tokenized learning rewards on the Stellar network. This repository contains the Rust smart contract and developer tooling for building, testing, and deploying the contract to Stellar testnet or mainnet.

## Table of Contents

- Overview
- Interactive Documentation
- Onboarding
- Developer Experience Toolkit
- Architecture
- Development Workflow
- Contribution Guidelines
- Troubleshooting
- License

## Overview

TeachLink enables tokenized learning rewards, proof-of-participation, and educator incentives. The contract is written in Rust for Soroban, Stellar's smart contract platform.

## Interactive Documentation

Explore the TeachLink contract interactively with live code execution, API exploration, and guided tutorials. The interactive documentation provides an engaging way to understand the contract's architecture and implementation.

To run the interactive docs:

```bash
cd docs/interactive
cargo run
```

Then open http://localhost:3000 in your browser.

## Onboarding

The onboarding flow is designed to take you from clone to first deployment with minimal guesswork.

### 1) Clone the repository

```bash
git clone https://github.com/rinafcode/teachLink_contract.git
cd teachLink_contract
```

### 2) Automated environment setup (dependency validation)

Run the setup script to validate required dependencies and create a local `.env` file if needed:

```bash
./scripts/setup-env.sh
```

What it checks:

- `rustc`, `cargo`, and `rustup`
- `wasm32-unknown-unknown` target
- `stellar` or `soroban` CLI
- local `.env` bootstrap from `.env.example`

### 3) Configure environment variables

Update `.env` with your deployment settings:

```bash
STELLAR_NETWORK=testnet
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
STELLAR_SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
DEPLOYER_SECRET_KEY=SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

If you do not have a key, generate one with the Stellar CLI:

```bash
stellar keys generate --global teachlink-deployer
```

### 4) Build and test the contract

```bash
cargo build --release --target wasm32-unknown-unknown -p teachlink-contract
cargo test
```

### 5) Interactive first deployment tutorial

The tutorial script walks you through building, funding, and deploying to testnet:

```bash
./scripts/first-deploy.sh
```

Common options:

```bash
./scripts/first-deploy.sh --network testnet --identity teachlink-deployer
./scripts/first-deploy.sh --skip-build
./scripts/first-deploy.sh --dry-run
```

### 6) Network-specific deployment scripts

Use the network-aware deployment script with managed config files:

```bash
./scripts/deploy.sh --network testnet
./scripts/deploy.sh --network mainnet
./scripts/deploy.sh --network local
```

Convenience wrappers:

```bash
./scripts/deploy-testnet.sh
./scripts/deploy-mainnet.sh
./scripts/deploy-local.sh
```

Configuration files live under `config/networks/` and can be customized per environment:

```bash
config/networks/testnet.env
config/networks/mainnet.env
config/networks/local.env
```

## Developer Experience Toolkit

TeachLink provides a comprehensive set of tools to streamline your development workflow, from environment setup to deployment.

### Environment Validation

Validate your development environment with version checks and system requirements:

```bash
./scripts/validate-env.sh
```

This enhanced validation script checks:
- Core dependencies (Rust, Cargo, Rustup) with minimum version requirements
- WASM target installation
- Stellar/Soroban CLI availability
- System resources (disk space)
- Optional tools (Docker, Git)
- Environment configuration (.env file)

### Automated Dependency Installation

Install all required dependencies automatically:

```bash
./scripts/install-deps.sh
```

This interactive script will:
- Install Rust toolchain via rustup (if missing)
- Add wasm32-unknown-unknown target
- Install Stellar CLI
- Update Rust components (rustfmt, clippy)
- Provide Docker installation instructions
- Check for additional development tools

### Quick-Start Development Scripts

#### Build Contracts

Build all contracts or a specific contract:

```bash
./scripts/build.sh                    # Build all contracts (debug mode)
./scripts/build.sh --release          # Build with optimizations
./scripts/build.sh --contract teachlink  # Build specific contract
./scripts/build.sh --verbose          # Verbose output
```

#### Run Tests

Execute unit tests with various options:

```bash
./scripts/test.sh                     # Run all tests
./scripts/test.sh --contract teachlink   # Test specific contract
./scripts/test.sh --verbose           # Verbose test output
./scripts/test.sh --nocapture         # Show println! output
```

#### Lint and Format

Check and fix code style issues:

```bash
./scripts/lint.sh                     # Format code and run clippy
./scripts/lint.sh --check             # Check formatting only
./scripts/lint.sh --fix               # Auto-fix clippy suggestions
```

#### Clean Build Artifacts

Remove build artifacts to free disk space:

```bash
./scripts/clean.sh                    # Standard clean (target dir)
./scripts/clean.sh --deep             # Deep clean (includes cargo cache)
```

#### Complete Development Cycle

Run a full development workflow (validate, build, test, lint):

```bash
./scripts/dev.sh                      # Full development cycle
./scripts/dev.sh --release            # Full cycle with release build
./scripts/dev.sh --skip-test          # Skip tests
./scripts/dev.sh --watch              # Watch mode (requires cargo-watch)
```

### Docker Development Environment

Work in a fully containerized environment with all dependencies pre-installed:

#### Using Docker Compose (Recommended)

```bash
# Start development environment
docker-compose up dev
docker-compose exec dev bash

# Build WASM in container
docker-compose run --rm builder

# Run tests in container
docker-compose run --rm test

# Run linter in container
docker-compose run --rm lint

# Clean up
docker-compose down -v
```

#### Using Docker Directly

```bash
# Build development image
docker build --target development -t teachlink-dev .

# Run interactive container
docker run -it --rm -v $(pwd):/workspace teachlink-dev

# Build contracts in container
docker run --rm -v $(pwd):/workspace teachlink-dev cargo build --release --target wasm32-unknown-unknown
```

### Developer Workflow Best Practices

1. **Initial Setup**: Run `./scripts/install-deps.sh` and `./scripts/validate-env.sh`
2. **Before Coding**: Pull latest changes and run `./scripts/dev.sh` to ensure environment works
3. **During Development**: Use `./scripts/dev.sh --watch` for continuous feedback
4. **Before Committing**: Run `./scripts/dev.sh --release` to catch all issues
5. **CI/CD Integration**: Use Docker containers for consistent builds across environments

## Architecture

```
Client Apps
    |
    v
Indexer / API Layer (optional)
    |
    v
Soroban Smart Contract (Rust)
    |
    v
Stellar Network
```

Key project paths:

- `contracts/teachlink`: Soroban smart contract source
- `scripts/`: onboarding and deployment script

## Development Workflow

### Using Quick-Start Scripts (Recommended)

```bash
# Complete development cycle
./scripts/dev.sh

# Individual steps
./scripts/build.sh --release    # Build WASM
./scripts/test.sh               # Run tests
./scripts/lint.sh               # Format and lint
```

### Using Cargo Directly

Build the WASM:

```bash
cargo build --release --target wasm32-unknown-unknown -p teachlink-contract
```

Run unit tests:

```bash
cargo test
```

Lint and format:

```bash
cargo fmt
cargo clippy --all-targets --all-features
```

### Using Docker

```bash
docker-compose run --rm builder  # Build WASM
docker-compose run --rm test     # Run tests
docker-compose run --rm lint     # Lint code
```

## Contribution Guidelines

We welcome contributions that improve contract quality, developer experience, and documentation.

### How to contribute

1. Fork the repo and create a feature branch.
2. Make focused changes with tests.
3. Run the full test and lint suite.
4. Open a PR with a clear summary and testing notes.

### Code example (contract + test)

When adding contract entrypoints, include unit tests in the same module or under `#[cfg(test)]`:

```rust
#[contractimpl]
impl TeachLinkContract {
    #[must_use]
    pub fn hello(_env: Env, to: Symbol) -> Symbol {
        to
    }
}

#[test]
fn hello_returns_input() {
    let env = Env::default();
    let input = Symbol::new(&env, "teachlink");
    let out = TeachLinkContract::hello(env.clone(), input);
    assert_eq!(out, Symbol::new(&env, "teachlink"));
}
```

### Testing requirements

- All new contract logic must include unit tests.
- `cargo test` must pass.
- `cargo fmt` and `cargo clippy --all-targets --all-features` must pass with no new warnings.

## Troubleshooting

### First Steps

1. Run enhanced environment validation:
   ```bash
   ./scripts/validate-env.sh
   ```

2. If validation fails, try automated installation:
   ```bash
   ./scripts/install-deps.sh
   ```

3. For legacy validation (minimal checks):
   ```bash
   ./scripts/setup-env.sh
   ```

### Common Issues

- `Missing command: stellar or soroban`
  - Install the CLI: `cargo install --locked stellar-cli --features opt`
- `Rust target not installed: wasm32-unknown-unknown`
  - Run: `rustup target add wasm32-unknown-unknown`
- `WASM not found` during deployment
  - Rebuild: `cargo build --release --target wasm32-unknown-unknown -p teachlink-contract`
  - Verify the path: `target/wasm32-unknown-unknown/release/teachlink_contract.wasm`
- `DEPLOYER_SECRET_KEY` is empty
  - Generate a key: `stellar keys generate --global teachlink-deployer`
  - Update `.env` with the secret key
- `Account not funded` or `transaction failed` on testnet
  - Re-run the tutorial without `--skip-fund`
  - Or fund manually: `https://friendbot.stellar.org?addr=<PUBLIC_KEY>`
- `curl not found` while funding
  - Install curl or fund the account manually using the friendbot URL

## License

This project is licensed under the MIT License. See `LICENSE` for details.
