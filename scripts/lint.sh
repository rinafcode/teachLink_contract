#!/usr/bin/env bash
# Lint and format script for TeachLink contracts
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

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
section() { color blue; printf "\n▸ %s\n" "$*"; color reset; }

# Parse arguments
FIX=false
CHECK_ONLY=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --fix)
      FIX=true
      shift
      ;;
    --check)
      CHECK_ONLY=true
      shift
      ;;
    --help|-h)
      cat <<EOF
TeachLink Lint Script

Usage: $0 [OPTIONS]

Options:
  --fix      Automatically fix formatting and apply suggestions
  --check    Check formatting without making changes
  --help, -h Show this help message

Examples:
  $0              # Format code and run clippy
  $0 --check      # Check formatting only
  $0 --fix        # Format and apply clippy fixes
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

ERRORS=0

# Formatting
section "Checking Code Format"

if [[ "$CHECK_ONLY" == "true" ]]; then
  if cargo fmt --all -- --check; then
    info "Code formatting is correct"
  else
    warn "Code formatting issues found. Run: ./scripts/lint.sh --fix"
    ERRORS=$((ERRORS + 1))
  fi
elif [[ "$FIX" == "true" ]]; then
  cargo fmt --all
  info "Code formatted"
else
  cargo fmt --all
  info "Code formatted"
fi

# Clippy
section "Running Clippy"

CLIPPY_CMD="cargo clippy --all-targets --all-features"

if [[ "$FIX" == "true" ]]; then
  CLIPPY_CMD="$CLIPPY_CMD --fix --allow-dirty --allow-staged"
  if $CLIPPY_CMD; then
    info "Clippy fixes applied"
  else
    warn "Some clippy issues could not be fixed automatically"
    ERRORS=$((ERRORS + 1))
  fi
else
  CLIPPY_CMD="$CLIPPY_CMD -- -D warnings"
  if $CLIPPY_CMD; then
    info "No clippy warnings"
  else
    warn "Clippy found issues. Run: ./scripts/lint.sh --fix"
    ERRORS=$((ERRORS + 1))
  fi
fi

# Summary
echo ""
if [[ $ERRORS -eq 0 ]]; then
  info "Lint checks passed!"
  exit 0
else
  color red
  echo "[✗] Lint checks failed with $ERRORS issue(s)"
  color reset
  exit 1
fi
