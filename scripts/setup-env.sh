#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
ENV_EXAMPLE="$ROOT_DIR/.env.example"
ENV_FILE="$ROOT_DIR/.env"

color() {
  if [[ -t 1 ]]; then
    case "$1" in
      green) printf "\033[32m" ;;
      yellow) printf "\033[33m" ;;
      red) printf "\033[31m" ;;
      reset) printf "\033[0m" ;;
    esac
  fi
}

info() { color green; printf "[ok]"; color reset; printf " %s\n" "$*"; }
warn() { color yellow; printf "[warn]"; color reset; printf " %s\n" "$*"; }
err() { color red; printf "[err]"; color reset; printf " %s\n" "$*"; }

missing=0

require_cmd() {
  local name="$1"
  local hint="$2"
  if ! command -v "$name" >/dev/null 2>&1; then
    err "Missing command: $name"
    [[ -n "$hint" ]] && printf "  - %s\n" "$hint"
    missing=$((missing + 1))
  else
    info "$name found: $($name --version 2>/dev/null | head -n1)"
  fi
}

check_target() {
  local target="wasm32-unknown-unknown"
  if ! command -v rustup >/dev/null 2>&1; then
    warn "rustup not found; cannot verify $target target"
    return 0
  fi
  if ! rustup target list --installed 2>/dev/null | grep -qx "$target"; then
    err "Rust target not installed: $target"
    printf "  - Run: rustup target add %s\n" "$target"
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
  printf "  - Install: cargo install --locked stellar-cli --features opt\n"
  missing=$((missing + 1))
}

printf "TeachLink environment check\n\n"

require_cmd rustc "Install Rust: https://www.rust-lang.org/tools/install"
require_cmd cargo "Install Rust: https://www.rust-lang.org/tools/install"
require_cmd rustup "Install Rustup: https://rust-lang.github.io/rustup/"
check_target
check_cli

if [[ -f "$ENV_EXAMPLE" && ! -f "$ENV_FILE" ]]; then
  cp "$ENV_EXAMPLE" "$ENV_FILE"
  warn "Created .env from .env.example. Set DEPLOYER_SECRET_KEY before deploying."
elif [[ -f "$ENV_FILE" ]]; then
  if grep -q '^DEPLOYER_SECRET_KEY=$' "$ENV_FILE"; then
    warn "DEPLOYER_SECRET_KEY is empty in .env"
  fi
fi

if [[ $missing -gt 0 ]]; then
  printf "\nEnvironment check failed with %s issue(s).\n" "$missing"
  exit 1
fi

printf "\nAll required dependencies are installed.\n"
