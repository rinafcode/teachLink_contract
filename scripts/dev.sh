#!/usr/bin/env bash
# Comprehensive development workflow script for TeachLink
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
SCRIPTS_DIR="$ROOT_DIR/scripts"

color() {
  if [[ -t 1 ]]; then
    case "$1" in
      green) printf "\033[32m" ;;
      blue) printf "\033[34m" ;;
      yellow) printf "\033[33m" ;;
      red) printf "\033[31m" ;;
      reset) printf "\033[0m" ;;
    esac
  fi
}

info() { color green; printf "[✓]"; color reset; printf " %s\n" "$*"; }
warn() { color yellow; printf "[⚠]"; color reset; printf " %s\n" "$*"; }
err() { color red; printf "[✗]"; color reset; printf " %s\n" "$*"; }
section() { color blue; printf "\n╔════════════════════════════════════════════════════════╗\n"; printf "║  %-52s║\n" "$*"; printf "╚════════════════════════════════════════════════════════╝\n"; color reset; }

# Parse arguments
SKIP_VALIDATE=false
SKIP_BUILD=false
SKIP_TEST=false
SKIP_LINT=false
RELEASE=false
WATCH=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --skip-validate)
      SKIP_VALIDATE=true
      shift
      ;;
    --skip-build)
      SKIP_BUILD=true
      shift
      ;;
    --skip-test)
      SKIP_TEST=true
      shift
      ;;
    --skip-lint)
      SKIP_LINT=true
      shift
      ;;
    --release)
      RELEASE=true
      shift
      ;;
    --watch)
      WATCH=true
      shift
      ;;
    --help|-h)
      cat <<EOF
TeachLink Development Workflow Script

This script runs a complete development cycle: validation, build, test, and lint.

Usage: $0 [OPTIONS]

Options:
  --skip-validate   Skip environment validation
  --skip-build      Skip building contracts
  --skip-test       Skip running tests
  --skip-lint       Skip linting
  --release         Build in release mode
  --watch           Watch mode (requires cargo-watch)
  --help, -h        Show this help message

Examples:
  $0                     # Full development cycle
  $0 --release           # Full cycle with release build
  $0 --skip-test         # Build and lint without testing
  $0 --watch             # Watch mode for continuous development
EOF
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      echo "Use --help for usage information"
      exit 1
      ;;
  esac
done

cd "$ROOT_DIR"

printf "\n"
color blue
cat <<'EOF'
╔══════════════════════════════════════════════════════════╗
║                                                          ║
║   TeachLink Development Workflow                         ║
║   Decentralized Knowledge-Sharing on Stellar             ║
║                                                          ║
╚══════════════════════════════════════════════════════════╝
EOF
color reset
printf "\n"

FAILED=0

# Watch mode
if [[ "$WATCH" == "true" ]]; then
  if ! command -v cargo-watch >/dev/null 2>&1; then
    err "cargo-watch not found"
    echo "Install with: cargo install cargo-watch"
    exit 1
  fi

  info "Starting watch mode..."
  exec cargo-watch -x "build --target wasm32-unknown-unknown" -x test -x "clippy --all-targets"
fi

# 1. Environment Validation
if [[ "$SKIP_VALIDATE" == "false" ]]; then
  section "Environment Validation"
  if [[ -x "$SCRIPTS_DIR/validate-env.sh" ]]; then
    if "$SCRIPTS_DIR/validate-env.sh"; then
      info "Environment validation passed"
    else
      err "Environment validation failed"
      FAILED=1
      exit 1
    fi
  else
    warn "Validation script not found or not executable"
  fi
else
  warn "Skipping environment validation"
fi

# 2. Build
if [[ "$SKIP_BUILD" == "false" ]]; then
  section "Building Contracts"
  if [[ -x "$SCRIPTS_DIR/build.sh" ]]; then
    BUILD_ARGS=""
    if [[ "$RELEASE" == "true" ]]; then
      BUILD_ARGS="--release"
    fi

    if "$SCRIPTS_DIR/build.sh" $BUILD_ARGS; then
      info "Build completed successfully"
    else
      err "Build failed"
      FAILED=1
    fi
  else
    warn "Build script not found, running cargo directly..."
    if cargo build --target wasm32-unknown-unknown; then
      info "Build completed"
    else
      err "Build failed"
      FAILED=1
    fi
  fi
else
  warn "Skipping build"
fi

# 3. Tests
if [[ "$SKIP_TEST" == "false" && $FAILED -eq 0 ]]; then
  section "Running Tests"
  if [[ -x "$SCRIPTS_DIR/test.sh" ]]; then
    if "$SCRIPTS_DIR/test.sh"; then
      info "All tests passed"
    else
      err "Tests failed"
      FAILED=1
    fi
  else
    warn "Test script not found, running cargo directly..."
    if cargo test; then
      info "Tests passed"
    else
      err "Tests failed"
      FAILED=1
    fi
  fi
else
  if [[ "$SKIP_TEST" == "true" ]]; then
    warn "Skipping tests"
  fi
fi

# 4. Lint
if [[ "$SKIP_LINT" == "false" && $FAILED -eq 0 ]]; then
  section "Linting Code"
  if [[ -x "$SCRIPTS_DIR/lint.sh" ]]; then
    if "$SCRIPTS_DIR/lint.sh"; then
      info "Lint checks passed"
    else
      err "Lint checks failed"
      FAILED=1
    fi
  else
    warn "Lint script not found, running cargo directly..."
    cargo fmt --all
    if cargo clippy --all-targets --all-features -- -D warnings; then
      info "Lint passed"
    else
      err "Lint failed"
      FAILED=1
    fi
  fi
else
  if [[ "$SKIP_LINT" == "true" ]]; then
    warn "Skipping lint"
  fi
fi

# Summary
printf "\n"
section "Development Workflow Summary"

if [[ $FAILED -eq 0 ]]; then
  color green
  cat <<'EOF'
    ✓ All checks passed!

    Your code is ready for deployment.
    Next steps:
      - Deploy to testnet: ./scripts/deploy-testnet.sh
      - Create a commit: git add . && git commit
      - Push to remote: git push
EOF
  color reset
  exit 0
else
  color red
  cat <<'EOF'
    ✗ Development workflow failed!

    Please fix the issues above and try again.

    Quick fixes:
      - Format code: ./scripts/lint.sh --fix
      - Run specific tests: ./scripts/test.sh --contract <name>
      - Clean build: ./scripts/clean.sh && ./scripts/build.sh
EOF
  color reset
  exit 1
fi
