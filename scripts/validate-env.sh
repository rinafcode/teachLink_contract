#!/usr/bin/env bash
# Enhanced environment validation script with version checks and system requirements
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
ENV_EXAMPLE="$ROOT_DIR/.env.example"
ENV_FILE="$ROOT_DIR/.env"

# Minimum version requirements
MIN_RUST_VERSION="1.70.0"
MIN_CARGO_VERSION="1.70.0"
RECOMMENDED_DISK_SPACE_MB=1000

color() {
  if [[ -t 1 ]]; then
    case "$1" in
      green) printf "\033[32m" ;;
      yellow) printf "\033[33m" ;;
      red) printf "\033[31m" ;;
      blue) printf "\033[34m" ;;
      reset) printf "\033[0m" ;;
    esac
  fi
}

info() { color green; printf "[✓]"; color reset; printf " %s\n" "$*"; }
warn() { color yellow; printf "[⚠]"; color reset; printf " %s\n" "$*"; }
err() { color red; printf "[✗]"; color reset; printf " %s\n" "$*"; }
section() { color blue; printf "\n▸ %s\n" "$*"; color reset; }

missing=0
warnings=0

# Compare semantic versions
version_ge() {
  local ver1="$1"
  local ver2="$2"

  # Simple version comparison (major.minor.patch)
  IFS='.' read -ra V1 <<< "$ver1"
  IFS='.' read -ra V2 <<< "$ver2"

  for i in 0 1 2; do
    local v1="${V1[$i]:-0}"
    local v2="${V2[$i]:-0}"

    if [[ $v1 -gt $v2 ]]; then
      return 0
    elif [[ $v1 -lt $v2 ]]; then
      return 1
    fi
  done

  return 0
}

extract_version() {
  echo "$1" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -n1
}

require_cmd() {
  local name="$1"
  local hint="$2"
  local min_version="${3:-}"

  if ! command -v "$name" >/dev/null 2>&1; then
    err "Missing command: $name"
    [[ -n "$hint" ]] && printf "  → %s\n" "$hint"
    missing=$((missing + 1))
    return 1
  fi

  local version_output
  version_output="$($name --version 2>/dev/null | head -n1)"

  if [[ -n "$min_version" ]]; then
    local current_version
    current_version=$(extract_version "$version_output")

    if [[ -n "$current_version" ]]; then
      if version_ge "$current_version" "$min_version"; then
        info "$name $current_version (>= $min_version required)"
      else
        err "$name $current_version (< $min_version required)"
        [[ -n "$hint" ]] && printf "  → %s\n" "$hint"
        missing=$((missing + 1))
        return 1
      fi
    else
      info "$name found: $version_output"
    fi
  else
    info "$name found: $version_output"
  fi

  return 0
}

check_target() {
  local target="wasm32-unknown-unknown"
  if ! command -v rustup >/dev/null 2>&1; then
    warn "rustup not found; cannot verify $target target"
    warnings=$((warnings + 1))
    return 0
  fi
  if ! rustup target list --installed 2>/dev/null | grep -qx "$target"; then
    err "Rust target not installed: $target"
    printf "  → Run: rustup target add %s\n" "$target"
    missing=$((missing + 1))
  else
    info "Rust target installed: $target"
  fi
}

check_cli() {
  if command -v stellar >/dev/null 2>&1; then
    info "Stellar CLI found: $(stellar --version 2>/dev/null | head -n1)"
    return 0
  fi
  if command -v soroban >/dev/null 2>&1; then
    info "Soroban CLI found: $(soroban --version 2>/dev/null | head -n1)"
    return 0
  fi

  err "Missing CLI: stellar or soroban"
  printf "  → Install: cargo install --locked stellar-cli --features opt\n"
  missing=$((missing + 1))
}

check_disk_space() {
  if command -v df >/dev/null 2>&1; then
    local available_mb
    if [[ "$OSTYPE" == "darwin"* ]]; then
      # macOS
      available_mb=$(df -m "$ROOT_DIR" | tail -1 | awk '{print $4}')
    else
      # Linux
      available_mb=$(df -BM "$ROOT_DIR" | tail -1 | awk '{print $4}' | sed 's/M//')
    fi

    if [[ "$available_mb" -ge "$RECOMMENDED_DISK_SPACE_MB" ]]; then
      info "Disk space: ${available_mb}MB available"
    else
      warn "Low disk space: ${available_mb}MB available (${RECOMMENDED_DISK_SPACE_MB}MB recommended)"
      warnings=$((warnings + 1))
    fi
  fi
}

check_optional_tools() {
  section "Optional Tools"

  if command -v docker >/dev/null 2>&1; then
    info "Docker found: $(docker --version 2>/dev/null)"
  else
    warn "Docker not found (optional for containerized development)"
    printf "  → Install: https://docs.docker.com/get-docker/\n"
    warnings=$((warnings + 1))
  fi

  if command -v git >/dev/null 2>&1; then
    info "Git found: $(git --version 2>/dev/null)"
  else
    warn "Git not found (recommended for version control)"
    warnings=$((warnings + 1))
  fi
}

check_env_file() {
  section "Environment Configuration"

  if [[ -f "$ENV_EXAMPLE" && ! -f "$ENV_FILE" ]]; then
    cp "$ENV_EXAMPLE" "$ENV_FILE"
    warn "Created .env from .env.example. Set DEPLOYER_SECRET_KEY before deploying."
    warnings=$((warnings + 1))
  elif [[ -f "$ENV_FILE" ]]; then
    info ".env file exists"

    if grep -q '^DEPLOYER_SECRET_KEY=$' "$ENV_FILE" 2>/dev/null; then
      warn "DEPLOYER_SECRET_KEY is empty in .env"
      printf "  → Generate key: stellar keys generate --global teachlink-deployer\n"
      warnings=$((warnings + 1))
    else
      info "DEPLOYER_SECRET_KEY is configured"
    fi
  else
    warn ".env.example not found"
    warnings=$((warnings + 1))
  fi
}

# Main validation
printf "╔══════════════════════════════════════════════════════════╗\n"
printf "║       TeachLink Environment Validation                  ║\n"
printf "╚══════════════════════════════════════════════════════════╝\n"

section "Core Dependencies"
require_cmd rustc "Install Rust: https://www.rust-lang.org/tools/install" "$MIN_RUST_VERSION"
require_cmd cargo "Install Rust: https://www.rust-lang.org/tools/install" "$MIN_CARGO_VERSION"
require_cmd rustup "Install Rustup: https://rust-lang.github.io/rustup/"
check_target
check_cli

section "System Resources"
check_disk_space

check_optional_tools
check_env_file

# Summary
printf "\n"
printf "╔══════════════════════════════════════════════════════════╗\n"
printf "║       Validation Summary                                 ║\n"
printf "╚══════════════════════════════════════════════════════════╝\n"

if [[ $missing -eq 0 && $warnings -eq 0 ]]; then
  color green
  printf "✓ All checks passed! Your environment is ready.\n"
  color reset
  exit 0
elif [[ $missing -eq 0 ]]; then
  color yellow
  printf "⚠ Validation completed with %s warning(s).\n" "$warnings"
  printf "Your environment is functional but some optional features may be unavailable.\n"
  color reset
  exit 0
else
  color red
  printf "✗ Validation failed with %s error(s) and %s warning(s).\n" "$missing" "$warnings"
  printf "Please resolve the errors above before continuing.\n"
  color reset
  exit 1
fi
