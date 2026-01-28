#!/usr/bin/env bash
# Clean script for TeachLink contracts
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

color() {
  if [[ -t 1 ]]; then
    case "$1" in
      green) printf "\033[32m" ;;
      blue) printf "\033[34m" ;;
      yellow) printf "\033[33m" ;;
      reset) printf "\033[0m" ;;
    esac
  fi
}

info() { color green; printf "[✓]"; color reset; printf " %s\n" "$*"; }
warn() { color yellow; printf "[⚠]"; color reset; printf " %s\n" "$*"; }
section() { color blue; printf "\n▸ %s\n" "$*"; color reset; }

# Parse arguments
DEEP=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --deep)
      DEEP=true
      shift
      ;;
    --help|-h)
      cat <<EOF
TeachLink Clean Script

Usage: $0 [OPTIONS]

Options:
  --deep     Deep clean (includes cargo cache and registry)
  --help, -h Show this help message

Examples:
  $0         # Standard clean (target directory)
  $0 --deep  # Deep clean (includes cargo cache)
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

section "Cleaning TeachLink Build Artifacts"

# Standard clean
if [[ -d "target" ]]; then
  info "Cleaning target directory..."
  cargo clean
  info "Target directory cleaned"
else
  warn "Target directory not found"
fi

# Deep clean
if [[ "$DEEP" == "true" ]]; then
  section "Performing Deep Clean"

  # Clean cargo cache
  if command -v cargo-cache >/dev/null 2>&1; then
    info "Cleaning cargo cache..."
    cargo cache --autoclean
    info "Cargo cache cleaned"
  else
    warn "cargo-cache not installed. Skipping cache cleanup."
    echo "  Install with: cargo install cargo-cache"
  fi

  # Remove Cargo.lock if it exists (will be regenerated)
  if [[ -f "Cargo.lock" ]]; then
    warn "Removing Cargo.lock (will be regenerated on next build)"
    rm -f Cargo.lock
  fi

  # Clean test snapshots if desired
  warn "Test snapshots preserved. Remove manually if needed."
fi

# Show disk space saved
echo ""
info "Clean completed!"

if command -v du >/dev/null 2>&1; then
  if [[ ! -d "target" ]]; then
    info "Disk space: target directory removed"
  else
    TARGET_SIZE=$(du -sh target 2>/dev/null | cut -f1)
    info "Remaining target size: $TARGET_SIZE"
  fi
fi
