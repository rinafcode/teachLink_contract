#!/usr/bin/env bash
set -euo pipefail

# TeachLink Contract Migration Rollback Tools
# Automated rollback functionality for failed migrations

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
MIGRATION_DIR="$ROOT_DIR/migration"
CONFIG_DIR="$ROOT_DIR/config/networks"

NETWORK="testnet"
IDENTITY=""
CONTRACT_ID=""
ROLLBACK_TYPE="auto"  # auto|manual|emergency
DRY_RUN=0
NON_INTERACTIVE=0
FORCE=0

usage() {
  cat <<USAGE
Usage: $0 [options]

TeachLink Contract Migration Rollback

Options:
  --network <name>       Network to rollback on (testnet|mainnet|local)
  --identity <name>      Identity name for rollback
  --contract-id <id>     Contract ID to rollback
  --type <type>          Rollback type: auto|manual|emergency (default: auto)
  --dry-run              Print commands without executing them
  --non-interactive      Use defaults and do not prompt
  --force                Force rollback even if checks fail
  -h, --help             Show this help

Rollback Types:
  auto     - Automatic rollback using contract's rollback function
  manual   - Manual rollback with custom steps
  emergency- Emergency rollback for critical failures

Examples:
  $0 --network testnet --contract-id CB4HK... --identity deployer
  $0 --contract-id CB4HK... --type emergency --force

USAGE
}

log() {
  local level="$1"
  local message="$2"
  local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
  echo "[$timestamp] [$level] $message"
}

error_exit() {
  local message="$1"
  log "ERROR" "$message"
  exit 1
}

run_cmd() {
  local cmd="$*"
  log "CMD" "$cmd"
  if [[ $DRY_RUN -eq 1 ]]; then
    printf "> %s\n" "$cmd"
  else
    if ! eval "$cmd"; then
      error_exit "Command failed: $cmd"
    fi
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

validate_rollback_prerequisites() {
  log "INFO" "Validating rollback prerequisites..."

  # Check CLI availability
  if command -v stellar >/dev/null 2>&1; then
    CLI=stellar
  elif command -v soroban >/dev/null 2>&1; then
    CLI=soroban
  else
    error_exit "Missing CLI: install stellar or soroban"
  fi

  # Check network config
  local config_file="$CONFIG_DIR/${NETWORK}.env"
  if [[ ! -f "$config_file" ]]; then
    error_exit "Network config not found: $config_file"
  fi

  # Load network config
  set -a
  source "$config_file"
  set +a

  # Check identity
  if ! $CLI keys address "$IDENTITY" >/dev/null 2>&1; then
    error_exit "Identity not found: $IDENTITY"
  fi

  # Check contract connectivity
  if ! $CLI contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    -- \
    get_current_version >/dev/null 2>&1; then
    error_exit "Cannot connect to contract $CONTRACT_ID"
  fi

  log "INFO" "Rollback prerequisites validated"
}

check_rollback_availability() {
  log "INFO" "Checking rollback availability..."

  # Check if contract supports rollback
  local rollback_available
  if ! rollback_available=$($CLI contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    -- \
    is_rollback_available 2>/dev/null); then
    log "WARN" "Contract does not have rollback functionality"
    return 1
  fi

  if [[ "$rollback_available" != "true" ]]; then
    log "ERROR" "Rollback not available on contract"
    return 1
  fi

  log "INFO" "Rollback is available"
  return 0
}

get_rollback_info() {
  log "INFO" "Gathering rollback information..."

  # Get current version
  local current_version
  current_version=$($CLI contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    -- \
    get_current_version)

  log "INFO" "Current contract version: $current_version"

  # Get state backup info
  local backup_info
  if backup_info=$($CLI contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    -- \
    get_state_backup 2>/dev/null); then
    log "INFO" "State backup available: $backup_info"
  else
    log "WARN" "No state backup information available"
  fi

  # Get upgrade history
  local history
  if history=$($CLI contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    -- \
    get_upgrade_history --version "$current_version" 2>/dev/null); then
    log "INFO" "Upgrade history: $history"
  fi
}

execute_auto_rollback() {
  log "INFO" "Executing automatic rollback..."

  # Get admin address
  local admin_address
  admin_address=$($CLI keys address "$IDENTITY")

  # Execute rollback
  run_cmd $CLI contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- \
    rollback_upgrade \
    --admin "$admin_address"

  log "INFO" "Automatic rollback completed"
}

execute_manual_rollback() {
  log "INFO" "Executing manual rollback..."

  # This would contain custom rollback logic
  # For example: redeploying previous WASM, restoring state, etc.

  log "WARN" "Manual rollback not fully implemented"
  log "INFO" "Please implement custom rollback steps in this function"

  # Example steps (commented out):
  # 1. Find backup WASM
  # 2. Redeploy previous version
  # 3. Restore state from backup
  # 4. Update contract references

  error_exit "Manual rollback requires custom implementation"
}

execute_emergency_rollback() {
  log "INFO" "Executing emergency rollback..."

  if [[ $FORCE -eq 0 ]]; then
    if ! confirm "Emergency rollback is destructive. Continue?"; then
      log "INFO" "Emergency rollback cancelled"
      exit 0
    fi
  fi

  # Emergency rollback steps
  # This is more aggressive and may involve:
  # - Pausing the contract
  # - Redeploying to a known good state
  # - Updating all references

  log "WARN" "Emergency rollback not fully implemented"
  log "INFO" "Emergency rollback requires careful planning and testing"

  error_exit "Emergency rollback requires custom implementation"
}

validate_rollback_success() {
  log "INFO" "Validating rollback success..."

  # Check that rollback is no longer available
  local rollback_available
  rollback_available=$($CLI contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    -- \
    is_rollback_available 2>/dev/null)

  if [[ "$rollback_available" == "true" ]]; then
    log "WARN" "Rollback still available after execution"
  else
    log "INFO" "Rollback completed successfully"
  fi

  # Check contract version
  local version
  version=$($CLI contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    -- \
    get_current_version)

  log "INFO" "Contract version after rollback: $version"

  # Check contract functionality
  if $CLI contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    -- \
    get_current_version >/dev/null 2>&1; then
    log "INFO" "Contract functionality verified"
  else
    error_exit "Contract functionality check failed after rollback"
  fi
}

find_backup_files() {
  log "INFO" "Looking for backup files..."

  # Find recent backup directories
  local backup_dirs
  mapfile -t backup_dirs < <(find "$MIGRATION_DIR/backups" -type d -name "20*" 2>/dev/null | sort -r | head -5)

  if [[ ${#backup_dirs[@]} -eq 0 ]]; then
    log "WARN" "No backup directories found"
    return 1
  fi

  log "INFO" "Recent backup directories:"
  for dir in "${backup_dirs[@]}"; do
    log "INFO" "  $dir"
    if [[ -f "$dir/contract_info.txt" ]]; then
      log "INFO" "    Contents:"
      cat "$dir/contract_info.txt" | sed 's/^/      /' || true
    fi
  done

  return 0
}

main() {
  # Parse arguments
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --network) NETWORK="$2"; shift 2 ;;
      --identity) IDENTITY="$2"; shift 2 ;;
      --contract-id) CONTRACT_ID="$2"; shift 2 ;;
      --type) ROLLBACK_TYPE="$2"; shift 2 ;;
      --dry-run) DRY_RUN=1; shift ;;
      --non-interactive) NON_INTERACTIVE=1; shift ;;
      --force) FORCE=1; shift ;;
      -h|--help) usage; exit 0 ;;
      *) error_exit "Unknown option: $1" ;;
    esac
  done

  # Validate required arguments
  if [[ -z "$IDENTITY" || -z "$CONTRACT_ID" ]]; then
    usage
    error_exit "Missing required arguments: --identity, --contract-id"
  fi

  log "INFO" "Starting TeachLink contract rollback"
  log "INFO" "Type: $ROLLBACK_TYPE, Network: $NETWORK, Contract: $CONTRACT_ID"

  if ! confirm "Continue with rollback?"; then
    log "INFO" "Rollback cancelled by user"
    exit 0
  fi

  # Rollback preparation
  validate_rollback_prerequisites
  find_backup_files
  get_rollback_info

  # Check rollback availability for auto rollback
  if [[ "$ROLLBACK_TYPE" == "auto" ]]; then
    if ! check_rollback_availability; then
      if [[ $FORCE -eq 0 ]]; then
        error_exit "Automatic rollback not available. Use --force or different rollback type"
      else
        log "WARN" "Forcing rollback despite availability check failure"
      fi
    fi
  fi

  # Execute rollback based on type
  case "$ROLLBACK_TYPE" in
    auto) execute_auto_rollback ;;
    manual) execute_manual_rollback ;;
    emergency) execute_emergency_rollback ;;
    *) error_exit "Invalid rollback type: $ROLLBACK_TYPE" ;;
  esac

  # Validate rollback success
  validate_rollback_success

  log "INFO" "Rollback completed successfully"
}

# Run main function
main "$@"