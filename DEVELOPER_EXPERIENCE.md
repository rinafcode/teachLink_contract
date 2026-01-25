# Developer Experience Toolkit

Complete guide to the TeachLink developer experience toolkit, designed to streamline your development workflow from environment setup to deployment.

## Table of Contents

- [Overview](#overview)
- [Environment Management](#environment-management)
- [Development Scripts](#development-scripts)
- [Docker Containerization](#docker-containerization)
- [CI/CD Integration](#cicd-integration)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Overview

The TeachLink developer experience toolkit provides:

1. **Environment Validation** - Comprehensive checks for prerequisites and system requirements
2. **Automated Installation** - Interactive dependency installation with version management
3. **Quick-Start Scripts** - Streamlined workflows for building, testing, and linting
4. **Docker Support** - Containerized development environment with Docker and Docker Compose
5. **Version Locking** - Deterministic builds with Cargo.lock and rust-toolchain.toml

## Environment Management

### Validation Script

**Location**: `./scripts/validate-env.sh`

**Purpose**: Validates your development environment with version checks and system requirements.

**Features**:
- Checks core dependencies (Rust, Cargo, Rustup) with minimum version requirements
- Verifies WASM target installation
- Validates Stellar/Soroban CLI availability
- Checks system resources (disk space)
- Validates optional tools (Docker, Git)
- Verifies environment configuration (.env file)
- Color-coded output with clear error messages
- Exit codes: 0 (success), 1 (critical failure)

**Usage**:
```bash
./scripts/validate-env.sh
```

**Output Example**:
```
╔══════════════════════════════════════════════════════════╗
║       TeachLink Environment Validation                  ║
╚══════════════════════════════════════════════════════════╝

▸ Core Dependencies
[✓] rustc 1.77.0 (>= 1.70.0 required)
[✓] cargo 1.77.0 (>= 1.70.0 required)
[✓] rustup found: rustup 1.26.0
[✓] Rust target installed: wasm32-unknown-unknown
[✓] Stellar CLI found: stellar 21.0.0

▸ System Resources
[✓] Disk space: 15000MB available

▸ Optional Tools
[✓] Docker found: Docker version 24.0.0
[✓] Git found: git version 2.40.0

▸ Environment Configuration
[✓] .env file exists
[✓] DEPLOYER_SECRET_KEY is configured

╔══════════════════════════════════════════════════════════╗
║       Validation Summary                                 ║
╚══════════════════════════════════════════════════════════╝
✓ All checks passed! Your environment is ready.
```

### Installation Script

**Location**: `./scripts/install-deps.sh`

**Purpose**: Automates dependency installation with interactive prompts.

**Features**:
- Detects operating system (macOS, Linux)
- Installs Rust toolchain via rustup
- Adds wasm32-unknown-unknown target
- Installs Stellar CLI via cargo
- Updates Rust components (rustfmt, clippy)
- Provides Docker installation instructions
- Interactive prompts for user consent
- Idempotent (safe to run multiple times)

**Usage**:
```bash
./scripts/install-deps.sh
```

**Installation Flow**:
1. Detects OS and checks existing installations
2. Prompts to install Rust if missing
3. Adds WASM target if not installed
4. Prompts to install Stellar CLI if missing
5. Updates Rust toolchain and components
6. Provides Docker installation instructions
7. Shows next steps and validation commands

### Legacy Setup Script

**Location**: `./scripts/setup-env.sh`

**Purpose**: Minimal environment validation (legacy, use validate-env.sh instead).

**Usage**:
```bash
./scripts/setup-env.sh
```

## Development Scripts

### Build Script

**Location**: `./scripts/build.sh`

**Purpose**: Compiles Soroban contracts to WASM with various options.

**Options**:
```bash
--release         Build in release mode (optimized)
--verbose, -v     Show verbose output
--contract, -c    Build specific contract (teachlink, insurance)
--help, -h        Show help message
```

**Examples**:
```bash
# Build all contracts in debug mode
./scripts/build.sh

# Build in release mode with optimizations
./scripts/build.sh --release

# Build only teachlink contract
./scripts/build.sh --contract teachlink

# Build with verbose output
./scripts/build.sh --release --verbose
```

**Output Location**:
- Debug: `target/wasm32-unknown-unknown/debug/*.wasm`
- Release: `target/wasm32-unknown-unknown/release/*.wasm`

### Test Script

**Location**: `./scripts/test.sh`

**Purpose**: Runs unit tests with various options.

**Options**:
```bash
--verbose, -v     Show verbose test output
--contract, -c    Test specific contract
--nocapture       Show println! output from tests
--help, -h        Show help message
```

**Examples**:
```bash
# Run all tests
./scripts/test.sh

# Test specific contract
./scripts/test.sh --contract teachlink

# Verbose output with println!
./scripts/test.sh --verbose --nocapture
```

### Lint Script

**Location**: `./scripts/lint.sh`

**Purpose**: Formats code and runs clippy linter.

**Options**:
```bash
--fix      Automatically fix formatting and apply suggestions
--check    Check formatting without making changes
--help, -h Show help message
```

**Examples**:
```bash
# Format code and run clippy
./scripts/lint.sh

# Check formatting only (CI mode)
./scripts/lint.sh --check

# Auto-fix clippy suggestions
./scripts/lint.sh --fix
```

**What it does**:
1. Runs `cargo fmt` to format code
2. Runs `cargo clippy` to detect code issues
3. Reports formatting and linting errors
4. Optionally applies automatic fixes

### Clean Script

**Location**: `./scripts/clean.sh`

**Purpose**: Removes build artifacts to free disk space.

**Options**:
```bash
--deep     Deep clean (includes cargo cache and registry)
--help, -h Show help message
```

**Examples**:
```bash
# Standard clean (removes target directory)
./scripts/clean.sh

# Deep clean (includes cargo cache)
./scripts/clean.sh --deep
```

**What gets cleaned**:
- Standard: `target/` directory
- Deep: `target/`, cargo cache, `Cargo.lock`

### Development Workflow Script

**Location**: `./scripts/dev.sh`

**Purpose**: Runs complete development cycle (validate → build → test → lint).

**Options**:
```bash
--skip-validate   Skip environment validation
--skip-build      Skip building contracts
--skip-test       Skip running tests
--skip-lint       Skip linting
--release         Build in release mode
--watch           Watch mode (requires cargo-watch)
--help, -h        Show help message
```

**Examples**:
```bash
# Full development cycle
./scripts/dev.sh

# Full cycle with release build
./scripts/dev.sh --release

# Build and lint without testing
./scripts/dev.sh --skip-test

# Watch mode for continuous development
./scripts/dev.sh --watch
```

**Workflow Stages**:
1. **Validation**: Checks environment prerequisites
2. **Build**: Compiles contracts to WASM
3. **Test**: Runs unit tests
4. **Lint**: Formats code and runs clippy

**Exit Behavior**:
- Stops at first failure (except when using --skip-* flags)
- Returns exit code 0 on success, 1 on failure
- Shows comprehensive summary at the end

## Docker Containerization

### Dockerfile

**Location**: `./Dockerfile`

**Purpose**: Multi-stage Dockerfile for development and production builds.

**Stages**:
1. **base**: Base image with Rust toolchain and Stellar CLI
2. **development**: Development environment with all tools
3. **builder**: Production WASM builder
4. **artifacts**: Minimal image with only built WASM files

**Usage**:
```bash
# Build development image
docker build --target development -t teachlink-dev .

# Build production artifacts
docker build --target builder -t teachlink-builder .

# Extract WASM files
docker build --target artifacts --output target/wasm .
```

### Docker Compose

**Location**: `./docker-compose.yml`

**Purpose**: Orchestrates multiple Docker services for development.

**Services**:
- **dev**: Interactive development environment
- **builder**: WASM builder service
- **test**: Test runner service
- **lint**: Linter/formatter service

**Usage**:
```bash
# Start development environment
docker-compose up dev
docker-compose exec dev bash

# Build WASM
docker-compose run --rm builder

# Run tests
docker-compose run --rm test

# Run linter
docker-compose run --rm lint

# Clean up
docker-compose down -v
```

**Volumes**:
- `cargo-cache`: Caches cargo registry for faster builds
- `target-cache`: Caches build artifacts

### Docker Best Practices

1. **Use Docker Compose**: Recommended for most workflows
2. **Cache Management**: Volumes persist between runs for faster rebuilds
3. **Clean Regularly**: Run `docker-compose down -v` to clean volumes
4. **Multi-Stage Builds**: Use appropriate stages for different tasks
5. **Environment Variables**: Mount `.env` file or use `env_file` in compose

## CI/CD Integration

### GitHub Actions Example

```yaml
name: CI

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Validate environment
        run: ./scripts/validate-env.sh

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run tests
        run: docker-compose run --rm test

  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run linter
        run: docker-compose run --rm lint

  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build WASM
        run: docker-compose run --rm builder
      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: wasm-contracts
          path: target/wasm32-unknown-unknown/release/*.wasm
```

### GitLab CI Example

```yaml
stages:
  - validate
  - test
  - lint
  - build

validate:
  stage: validate
  script:
    - ./scripts/validate-env.sh

test:
  stage: test
  script:
    - docker-compose run --rm test

lint:
  stage: lint
  script:
    - docker-compose run --rm lint

build:
  stage: build
  script:
    - docker-compose run --rm builder
  artifacts:
    paths:
      - target/wasm32-unknown-unknown/release/*.wasm
```

## Best Practices

### For New Contributors

1. **Initial Setup**:
   ```bash
   git clone https://github.com/rinafcode/teachLink_contract.git
   cd teachLink_contract
   ./scripts/install-deps.sh
   ./scripts/validate-env.sh
   ```

2. **Daily Workflow**:
   ```bash
   git pull
   ./scripts/dev.sh
   # Make changes
   ./scripts/dev.sh --release
   git commit -m "Your changes"
   ```

### For Experienced Developers

1. **Fast Iteration**:
   ```bash
   ./scripts/dev.sh --watch
   ```

2. **Pre-Commit Hook**:
   ```bash
   ./scripts/lint.sh --check && ./scripts/test.sh
   ```

3. **Release Build**:
   ```bash
   ./scripts/clean.sh
   ./scripts/build.sh --release
   ./scripts/test.sh
   ```

### For CI/CD Pipelines

1. **Use Docker for Consistency**:
   ```bash
   docker-compose run --rm test
   docker-compose run --rm lint
   docker-compose run --rm builder
   ```

2. **Cache Dependencies**:
   - Use volume mounts for cargo cache
   - Cache Docker layers
   - Persist `target/` between builds

3. **Parallel Execution**:
   - Run tests and lint in parallel
   - Build multiple contracts concurrently

## Troubleshooting

### Environment Issues

**Problem**: `rustc not found`
```bash
./scripts/install-deps.sh
# Follow prompts to install Rust
```

**Problem**: `wasm32-unknown-unknown target not installed`
```bash
rustup target add wasm32-unknown-unknown
```

**Problem**: `stellar or soroban CLI not found`
```bash
cargo install --locked stellar-cli --features opt
```

### Build Issues

**Problem**: Build fails with dependency errors
```bash
./scripts/clean.sh --deep
./scripts/build.sh
```

**Problem**: Out of disk space
```bash
./scripts/clean.sh --deep
docker system prune -a
```

### Docker Issues

**Problem**: Docker image build fails
```bash
docker-compose build --no-cache
```

**Problem**: Container can't access files
```bash
# Check volume mounts in docker-compose.yml
# Ensure files have correct permissions
```

**Problem**: Old cache causing issues
```bash
docker-compose down -v
docker system prune -a
docker-compose build
```

### Test Issues

**Problem**: Tests fail unexpectedly
```bash
./scripts/clean.sh
./scripts/build.sh
./scripts/test.sh --verbose --nocapture
```

**Problem**: Snapshot tests fail
```bash
# Update test snapshots if changes are intentional
# Review test_snapshots/ directory
```

### Script Issues

**Problem**: Permission denied
```bash
chmod +x scripts/*.sh
```

**Problem**: Script not found
```bash
# Ensure you're in repository root
cd /path/to/teachLink_contract
./scripts/validate-env.sh
```

## Additional Resources

- [README.md](README.md) - Main project documentation
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines
- [Stellar Documentation](https://developers.stellar.org/) - Stellar platform docs
- [Soroban Documentation](https://soroban.stellar.org/docs) - Soroban smart contracts
- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust programming

## Feedback and Contributions

We welcome feedback and contributions to improve the developer experience:

1. **Report Issues**: Open an issue on GitHub
2. **Suggest Improvements**: Submit a pull request
3. **Share Your Workflow**: Contribute to documentation
4. **Help Others**: Answer questions in discussions

---

Built with ❤️ for the Stellar community
