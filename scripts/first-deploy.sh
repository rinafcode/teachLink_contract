#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
WASM_DEFAULT="$ROOT_DIR/target/wasm32-unknown-unknown/release/teachlink_contract.wasm"

NETWORK="testnet"
IDENTITY="teachlink-deployer"
WASM_PATH="$WASM_DEFAULT"
SKIP_BUILD=0
SKIP_FUND=0
DRY_RUN=0
NON_INTERACTIVE=0

usage() {
  cat <<USAGE
Usage: $0 [options]

Options:
  --network <name>       Network to deploy to (default: testnet)
  --identity <name>      Local key name for the deployer (default: teachlink-deployer)
  --wasm <path>          Path to compiled WASM (default: $WASM_DEFAULT)
  --skip-build           Skip cargo build step
  --skip-fund            Skip friendbot funding (testnet only)
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

while [[ $# -gt 0 ]]; do
  case "$1" in
    --network) NETWORK="$2"; shift 2 ;;
    --identity) IDENTITY="$2"; shift 2 ;;
    --wasm) WASM_PATH="$2"; shift 2 ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    --skip-fund) SKIP_FUND=1; shift ;;
    --dry-run) DRY_RUN=1; shift ;;
    --non-interactive) NON_INTERACTIVE=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) printf "Unknown option: %s\n\n" "$1"; usage; exit 1 ;;
  esac
done

if command -v stellar >/dev/null 2>&1; then
  CLI=stellar
elif command -v soroban >/dev/null 2>&1; then
  CLI=soroban
else
  printf "Missing CLI: install stellar or soroban.\n"
  exit 1
fi

printf "TeachLink first deployment tutorial\n\n"
printf "Network : %s\n" "$NETWORK"
printf "Identity: %s\n" "$IDENTITY"
printf "WASM    : %s\n\n" "$WASM_PATH"

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

printf "\nStep 2: Ensure deployer identity exists\n"
if ! $CLI keys address "$IDENTITY" >/dev/null 2>&1; then
  run_cmd $CLI keys generate --global "$IDENTITY"
fi

printf "\nStep 3: Fund deployer (testnet only)\n"
if [[ "$NETWORK" == "testnet" && $SKIP_FUND -eq 0 ]]; then
  if command -v curl >/dev/null 2>&1; then
    PUBLIC_KEY=$($CLI keys address "$IDENTITY")
    run_cmd curl -s "https://friendbot.stellar.org?addr=$PUBLIC_KEY" >/dev/null
  else
    printf "curl not found. Fund the account manually or install curl.\n"
  fi
else
  printf "Funding skipped.\n"
fi

printf "\nStep 4: Deploy the contract\n"
if [[ ! -f "$WASM_PATH" ]]; then
  printf "WASM not found: %s\n" "$WASM_PATH"
  exit 1
fi

CONTRACT_ID=$(run_cmd $CLI contract deploy --wasm "$WASM_PATH" --source "$IDENTITY" --network "$NETWORK")

printf "\nDeployment complete.\n"
printf "Contract ID: %s\n" "$CONTRACT_ID"
printf "\nNext: record the contract ID in your .env file.\n"
