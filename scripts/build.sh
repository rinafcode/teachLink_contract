#!/usr/bin/env bash
# Build script for TeachLink contracts
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

color() {
  if [[ -t 1 ]]; then
    case "$1" in
      green) printf "\033[32m" ;;
      blue) printf "\033[34m" ;;
      reset) printf "\033[0m" ;;
    esac
  fi
}

info() { color green; printf "[✓]"; color reset; printf " %s\n" "$*"; }
section() { color blue; printf "\n▸ %s\n" "$*"; color reset; }

# Parse arguments
RELEASE=false
VERBOSE=false
CONTRACT=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --release)
      RELEASE=true
      shift
      ;;
    --verbose|-v)
      VERBOSE=true
      shift
      ;;
    --contract|-c)
      CONTRACT="$2"
      shift 2
      ;;
    --help|-h)
      cat <<EOF
TeachLink Build Script

Usage: $0 [OPTIONS]

Options:
  --release         Build in release mode (optimized)
  --verbose, -v     Show verbose output
  --contract, -c    Build specific contract (teachlink, insurance)
  --help, -h        Show this help message

Examples:
  $0                          # Build all contracts in debug mode
  $0 --release                # Build all contracts in release mode
  $0 --contract teachlink     # Build only teachlink contract
  $0 --release --verbose      # Build with verbose output
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

section "Building TeachLink Contracts"

# Build command
BUILD_CMD="cargo build --target wasm32-unknown-unknown"

if [[ "$RELEASE" == "true" ]]; then
  BUILD_CMD="$BUILD_CMD --release"
  info "Building in RELEASE mode"
else
  info "Building in DEBUG mode"
fi

if [[ -n "$CONTRACT" ]]; then
  BUILD_CMD="$BUILD_CMD -p ${CONTRACT}-contract"
  info "Building contract: $CONTRACT"
else
  info "Building all contracts"
fi

if [[ "$VERBOSE" == "true" ]]; then
  BUILD_CMD="$BUILD_CMD --verbose"
fi

# Execute build
echo ""
if $BUILD_CMD; then
  echo ""
  info "Build completed successfully"

  # Show output location
  if [[ "$RELEASE" == "true" ]]; then
    OUTPUT_DIR="$ROOT_DIR/target/wasm32-unknown-unknown/release"
  else
    OUTPUT_DIR="$ROOT_DIR/target/wasm32-unknown-unknown/debug"
  fi

  if [[ -d "$OUTPUT_DIR" ]]; then
    echo ""
    info "WASM files located at:"
    ls -lh "$OUTPUT_DIR"/*.wasm 2>/dev/null || true
  fi
else
  echo ""
  color red
  echo "[✗] Build failed"
  color reset
  exit 1
fi
