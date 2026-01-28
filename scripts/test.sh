#!/usr/bin/env bash
# Test script for TeachLink contracts
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

color() {
  if [[ -t 1 ]]; then
    case "$1" in
      green) printf "\033[32m" ;;
      blue) printf "\033[34m" ;;
      red) printf "\033[31m" ;;
      reset) printf "\033[0m" ;;
    esac
  fi
}

info() { color green; printf "[✓]"; color reset; printf " %s\n" "$*"; }
section() { color blue; printf "\n▸ %s\n" "$*"; color reset; }

# Parse arguments
VERBOSE=false
CONTRACT=""
NOCAPTURE=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --verbose|-v)
      VERBOSE=true
      shift
      ;;
    --contract|-c)
      CONTRACT="$2"
      shift 2
      ;;
    --nocapture)
      NOCAPTURE=true
      shift
      ;;
    --help|-h)
      cat <<EOF
TeachLink Test Script

Usage: $0 [OPTIONS]

Options:
  --verbose, -v       Show verbose test output
  --contract, -c      Test specific contract (teachlink, insurance)
  --nocapture         Show println! output from tests
  --help, -h          Show this help message

Examples:
  $0                        # Run all tests
  $0 --contract teachlink   # Test only teachlink contract
  $0 --verbose --nocapture  # Verbose output with println!
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

section "Running TeachLink Tests"

# Build test command
TEST_CMD="cargo test"

if [[ -n "$CONTRACT" ]]; then
  TEST_CMD="$TEST_CMD -p ${CONTRACT}-contract"
  info "Testing contract: $CONTRACT"
else
  info "Testing all contracts"
fi

if [[ "$VERBOSE" == "true" ]]; then
  TEST_CMD="$TEST_CMD --verbose"
fi

if [[ "$NOCAPTURE" == "true" ]]; then
  TEST_CMD="$TEST_CMD -- --nocapture"
fi

# Execute tests
echo ""
if $TEST_CMD; then
  echo ""
  info "All tests passed!"
else
  echo ""
  color red
  echo "[✗] Tests failed"
  color reset
  exit 1
fi
