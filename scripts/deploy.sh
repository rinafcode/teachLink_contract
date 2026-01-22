#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
CONFIG_DIR="$ROOT_DIR/config/networks"
WASM_DEFAULT="$ROOT_DIR/target/wasm32-unknown-unknown/release/teachlink_contract.wasm"

NETWORK="testnet"
CONFIG_PATH=""
IDENTITY=""
WASM_PATH="$WASM_DEFAULT"
SKIP_BUILD=0
SKIP_FUND=0
DRY_RUN=0
NON_INTERACTIVE=0
PRINT_CONFIG=0

usage() {
  cat <<USAGE
Usage: $0 [options]

Options:
  --network <name>       Network to deploy to (testnet|mainnet|local)
  --config <path>        Override config file path
  --identity <name>      Local key name for the deployer (overrides config)
  --wasm <path>          Path to compiled WASM (default: $WASM_DEFAULT)
  --skip-build           Skip cargo build step
  --skip-fund            Skip friendbot funding (testnet only)
  --print-config         Print resolved config and exit
  --dry-run              Print commands without executing them
  --non-interactive      Use defaults and do not prompt
  -h, --help             Show this help
USAGE
}

run_cmd() {
  if [[ $DRY_RUN -eq 1 ]]; then
    printf "> %s\n" "$*"
  else
    "$@"
  fi
}

confirm() {
  local prompt="$1"
  if [[ $NON_INTERACTIVE -eq 1 ]]; then
    return 0
  fi
  read -r -p "$prompt [Y/n] " reply
  [[ -z "$reply" || "$reply" =~ ^[Yy]$ ]]
}

load_env_file() {
  local file="$1"
  if [[ ! -f "$file" ]]; then
    printf "Config file not found: %s\n" "$file"
    exit 1
  fi

  while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line%%#*}"
    line="${line%"${line##*[![:space:]]}"}"
    line="${line#"${line%%[![:space:]]*}"}"
    [[ -z "$line" ]] && continue
    if [[ "$line" =~ ^[A-Za-z_][A-Za-z0-9_]*= ]]; then
      local key="${line%%=*}"
      local value="${line#*=}"
      value="${value%\"}"
      value="${value#\"}"
      export "$key"="$value"
    fi
  done < "$file"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --network) NETWORK="$2"; shift 2 ;;
    --config) CONFIG_PATH="$2"; shift 2 ;;
    --identity) IDENTITY="$2"; shift 2 ;;
    --wasm) WASM_PATH="$2"; shift 2 ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    --skip-fund) SKIP_FUND=1; shift ;;
    --print-config) PRINT_CONFIG=1; shift ;;
    --dry-run) DRY_RUN=1; shift ;;
    --non-interactive) NON_INTERACTIVE=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) printf "Unknown option: %s\n\n" "$1"; usage; exit 1 ;;
  esac
done

if [[ -z "$CONFIG_PATH" ]]; then
  CONFIG_PATH="$CONFIG_DIR/${NETWORK}.env"
fi

if [[ -f "$ROOT_DIR/.env" ]]; then
  load_env_file "$ROOT_DIR/.env"
fi
load_env_file "$CONFIG_PATH"

NETWORK_NAME="${NETWORK_NAME:-$NETWORK}"
SOROBAN_RPC_URL="${SOROBAN_RPC_URL:-}"
NETWORK_PASSPHRASE="${NETWORK_PASSPHRASE:-}"
FRIENDBOT_URL="${FRIENDBOT_URL:-}"
HORIZON_URL="${HORIZON_URL:-}"
IDENTITY="${IDENTITY:-${IDENTITY_NAME:-teachlink-deployer}}"

if [[ $PRINT_CONFIG -eq 1 ]]; then
  printf "Config file    : %s\n" "$CONFIG_PATH"
  printf "Network        : %s\n" "$NETWORK_NAME"
  printf "RPC URL        : %s\n" "$SOROBAN_RPC_URL"
  printf "Horizon URL    : %s\n" "$HORIZON_URL"
  printf "Passphrase     : %s\n" "$NETWORK_PASSPHRASE"
  printf "Identity       : %s\n" "$IDENTITY"
  printf "Friendbot URL  : %s\n" "${FRIENDBOT_URL:-<none>}"
  exit 0
fi

if command -v stellar >/dev/null 2>&1; then
  CLI=stellar
elif command -v soroban >/dev/null 2>&1; then
  CLI=soroban
else
  printf "Missing CLI: install stellar or soroban.\n"
  exit 1
fi

printf "TeachLink deployment\n\n"
printf "Network : %s\n" "$NETWORK_NAME"
printf "Identity: %s\n" "$IDENTITY"
printf "WASM    : %s\n" "$WASM_PATH"
printf "Config  : %s\n\n" "$CONFIG_PATH"

if [[ -z "$SOROBAN_RPC_URL" || -z "$NETWORK_PASSPHRASE" ]]; then
  printf "Config is missing SOROBAN_RPC_URL or NETWORK_PASSPHRASE.\n"
  exit 1
fi

if ! confirm "Continue?"; then
  printf "Canceled.\n"
  exit 0
fi

if [[ $SKIP_BUILD -eq 0 ]]; then
  printf "\nStep 1: Build the contract\n"
  run_cmd cargo build --release --target wasm32-unknown-unknown -p teachlink-contract
else
  printf "\nStep 1: Build skipped\n"
fi

printf "\nStep 2: Ensure network config exists\n"
NETWORKS_OUTPUT="$($CLI network ls 2>/dev/null || true)"
if [[ "$NETWORKS_OUTPUT" != *"$NETWORK_NAME"* ]]; then
  run_cmd $CLI network add "$NETWORK_NAME" \
    --rpc-url "$SOROBAN_RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE"
else
  printf "Network '%s' already configured.\n" "$NETWORK_NAME"
fi

printf "\nStep 3: Ensure deployer identity exists\n"
if ! $CLI keys address "$IDENTITY" >/dev/null 2>&1; then
  if confirm "Key '$IDENTITY' not found. Generate a new key?"; then
    run_cmd $CLI keys generate --global "$IDENTITY"
  else
    printf "Key missing. Create one with '%s keys generate --global %s'.\n" "$CLI" "$IDENTITY"
    exit 1
  fi
fi

printf "\nStep 4: Fund deployer (testnet only)\n"
if [[ "$NETWORK_NAME" == "testnet" && -n "$FRIENDBOT_URL" && $SKIP_FUND -eq 0 ]]; then
  if command -v curl >/dev/null 2>&1; then
    PUBLIC_KEY=$($CLI keys address "$IDENTITY")
    run_cmd curl -s "${FRIENDBOT_URL}?addr=${PUBLIC_KEY}" >/dev/null
  else
    printf "curl not found. Fund the account manually.\n"
  fi
else
  printf "Funding skipped.\n"
fi

printf "\nStep 5: Deploy the contract\n"
if [[ ! -f "$WASM_PATH" ]]; then
  printf "WASM not found: %s\n" "$WASM_PATH"
  exit 1
fi

CONTRACT_ID=$(run_cmd $CLI contract deploy --wasm "$WASM_PATH" --source "$IDENTITY" --network "$NETWORK_NAME")

printf "\nDeployment complete.\n"
printf "Contract ID: %s\n" "$CONTRACT_ID"
printf "\nNext: record the contract ID in your .env file.\n"
