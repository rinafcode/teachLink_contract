#!/usr/bin/env bash
set -euo pipefail

# TeachLink Contract Migration Tools
# Comprehensive migration automation with validation, rollback, and progress tracking

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
MIGRATION_DIR="$ROOT_DIR/migration"
CONFIG_DIR="$ROOT_DIR/config/networks"
SCRIPTS_DIR="$ROOT_DIR/scripts"

# Default values
NETWORK="testnet"
IDENTITY=""
CONTRACT_ID=""
NEW_WASM_PATH=""
MIGRATION_CONFIG=""
DRY_RUN=0
NON_INTERACTIVE=0
SKIP_VALIDATION=0
FORCE_MIGRATION=0
ROLLBACK_ON_FAILURE=1

# Migration state tracking
MIGRATION_LOG="$MIGRATION_DIR/migration_$(date +%Y%m%d_%H%M%S).log"
PROGRESS_FILE="$MIGRATION_DIR/.migration_progress"
BACKUP_DIR="$MIGRATION_DIR/backups/$(date +%Y%m%d_%H%M%S)"

usage() {
  cat <<USAGE
Usage: $0 [options]

TeachLink Contract Migration Tools

Options:
  --network <name>           Network to migrate on (testnet|mainnet|local)
  --identity <name>          Deployer identity name
  --contract-id <id>         Current contract ID to migrate
  --new-wasm <path>          Path to new contract WASM file
  --config <path>            Migration configuration file
  --dry-run                  Print commands without executing them
  --non-interactive          Use defaults and do not prompt
  --skip-validation          Skip pre/post migration validation
  --force                    Force migration even if validation fails
  --no-rollback              Don't rollback on migration failure
  -h, --help                 Show this help

Migration Configuration:
  Create a migration config file with:
  - target_version: Target contract version
  - migration_script: Path to migration script (optional)
  - validation_checks: List of validation checks to run
  - rollback_timeout: Rollback window in seconds
  - backup_data: Whether to backup contract data

Examples:
  $0 --network testnet --contract-id CB4HK... --new-wasm ./target/new_contract.wasm --config migration/v2_config.json
  $0 --dry-run --network mainnet --contract-id CB4HK... --new-wasm ./target/contract.wasm

USAGE
}

log() {
  local level="$1"
  local message="$2"
  local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
  echo "[$timestamp] [$level] $message" | tee -a "$MIGRATION_LOG"
}

progress_update() {
  local step="$1"
  local status="$2"
  echo "$step:$status:$(date +%s)" >> "$PROGRESS_FILE"
  log "INFO" "Progress: $step - $status"
}

error_exit() {
  local message="$1"
  log "ERROR" "$message"
  if [[ $ROLLBACK_ON_FAILURE -eq 1 && -f "$PROGRESS_FILE" ]]; then
    log "INFO" "Attempting automatic rollback due to failure..."
    rollback_migration
  fi
  exit 1
}

run_cmd() {
  local cmd="$*"
  log "CMD" "$cmd"
  if [[ $DRY_RUN -eq 1 ]]; then
    printf "> %s\n" "$cmd"
    return 0
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

load_migration_config() {
  local config_file="$1"
  if [[ ! -f "$config_file" ]]; then
    error_exit "Migration config file not found: $config_file"
  fi

  # Parse JSON config (simple implementation)
  TARGET_VERSION=$(grep -o '"target_version":[[:space:]]*[0-9]*' "$config_file" | grep -o '[0-9]*' || echo "")
  MIGRATION_SCRIPT=$(grep -o '"migration_script":[[:space:]]*"[^"]*"' "$config_file" | grep -o '"[^"]*"$' | tr -d '"' || echo "")
  VALIDATION_CHECKS=$(grep -o '"validation_checks":[[:space:]]*\[[^]]*\]' "$config_file" | grep -o '\[.*\]' || echo "[]")
  ROLLBACK_TIMEOUT=$(grep -o '"rollback_timeout":[[:space:]]*[0-9]*' "$config_file" | grep -o '[0-9]*' || echo "2592000")
  BACKUP_DATA=$(grep -o '"backup_data":[[:space:]]*true' "$config_file" | wc -l || echo "0")

  log "INFO" "Loaded migration config: version=$TARGET_VERSION, script=$MIGRATION_SCRIPT"
}

validate_environment() {
  log "INFO" "Validating environment..."

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

  # Check required variables
  if [[ -z "${SOROBAN_RPC_URL:-}" || -z "${NETWORK_PASSPHRASE:-}" ]]; then
    error_exit "Network config missing SOROBAN_RPC_URL or NETWORK_PASSPHRASE"
  fi

  # Check identity
  if ! $CLI keys address "$IDENTITY" >/dev/null 2>&1; then
    error_exit "Identity not found: $IDENTITY"
  fi

  # Check contract ID
  if [[ -z "$CONTRACT_ID" ]]; then
    error_exit "Contract ID not provided"
  fi

  # Check new WASM
  if [[ ! -f "$NEW_WASM_PATH" ]]; then
    error_exit "New WASM file not found: $NEW_WASM_PATH"
  fi

  progress_update "validate_environment" "completed"
}

backup_current_state() {
  log "INFO" "Backing up current contract state..."

  mkdir -p "$BACKUP_DIR"

  # Backup contract info
  echo "Contract ID: $CONTRACT_ID" > "$BACKUP_DIR/contract_info.txt"
  echo "Network: $NETWORK" >> "$BACKUP_DIR/contract_info.txt"
  echo "Identity: $IDENTITY" >> "$BACKUP_DIR/contract_info.txt"
  echo "Timestamp: $(date)" >> "$BACKUP_DIR/contract_info.txt"

  # Get current contract version
  local current_version
  current_version=$($CLI contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- \
    get_current_version)

  echo "Current Version: $current_version" >> "$BACKUP_DIR/contract_info.txt"

  # Backup contract WASM if possible
  if $CLI contract fetch --id "$CONTRACT_ID" --network "$NETWORK" --out "$BACKUP_DIR/current_contract.wasm" 2>/dev/null; then
    log "INFO" "Backed up current contract WASM"
  else
    log "WARN" "Could not backup current contract WASM"
  fi

  progress_update "backup_current_state" "completed"
}

run_pre_migration_validation() {
  if [[ $SKIP_VALIDATION -eq 1 ]]; then
    log "INFO" "Skipping pre-migration validation"
    return 0
  fi

  log "INFO" "Running pre-migration validation..."

  # Basic contract connectivity check
  if ! $CLI contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- \
    get_current_version >/dev/null 2>&1; then
    error_exit "Cannot communicate with contract $CONTRACT_ID"
  fi

  # Check if contract is paused (if emergency module exists)
  # This would need to be customized based on contract interface

  log "INFO" "Pre-migration validation passed"
  progress_update "pre_migration_validation" "completed"
}

prepare_migration() {
  log "INFO" "Preparing migration..."

  # Get current version
  local current_version
  current_version=$($CLI contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- \
    get_current_version)

  log "INFO" "Current contract version: $current_version"
  log "INFO" "Target version: $TARGET_VERSION"

  # Generate state hash (simplified - in practice would hash critical state)
  local state_hash="state_hash_$(date +%s)"
  local state_hash_bytes=$(echo -n "$state_hash" | xxd -p | tr -d '\n')

  # Prepare upgrade via contract
  run_cmd $CLI contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- \
    prepare_upgrade \
    --admin "$($CLI keys address "$IDENTITY")" \
    --new_version "$TARGET_VERSION" \
    --state_hash "0x$state_hash_bytes"

  progress_update "prepare_migration" "completed"
}

execute_migration() {
  log "INFO" "Executing migration..."

  # Deploy new contract WASM
  local new_contract_id
  new_contract_id=$(run_cmd $CLI contract deploy \
    --wasm "$NEW_WASM_PATH" \
    --source "$IDENTITY" \
    --network "$NETWORK")

  log "INFO" "New contract deployed with ID: $new_contract_id"

  # Generate migration hash
  local migration_hash="migration_$(date +%s)_v${TARGET_VERSION}"
  local migration_hash_bytes=$(echo -n "$migration_hash" | xxd -p | tr -d '\n')

  # Execute upgrade
  run_cmd $CLI contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- \
    execute_upgrade \
    --admin "$($CLI keys address "$IDENTITY")" \
    --new_version "$TARGET_VERSION" \
    --migration_hash "0x$migration_hash_bytes"

  # Update contract ID to new one (if upgrading to new contract)
  # In Soroban, upgrades typically update the existing contract
  # CONTRACT_ID="$new_contract_id"

  progress_update "execute_migration" "completed"
}

run_post_migration_validation() {
  if [[ $SKIP_VALIDATION -eq 1 ]]; then
    log "INFO" "Skipping post-migration validation"
    return 0
  fi

  log "INFO" "Running post-migration validation..."

  # Check new version
  local new_version
  new_version=$($CLI contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- \
    get_current_version)

  if [[ "$new_version" != "$TARGET_VERSION" ]]; then
    error_exit "Version mismatch after migration. Expected: $TARGET_VERSION, Got: $new_version"
  fi

  # Check rollback availability
  local rollback_available
  rollback_available=$($CLI contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- \
    is_rollback_available)

  if [[ "$rollback_available" != "true" ]]; then
    log "WARN" "Rollback not available after migration"
  fi

  log "INFO" "Post-migration validation passed"
  progress_update "post_migration_validation" "completed"
}

rollback_migration() {
  log "INFO" "Rolling back migration..."

  run_cmd $CLI contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- \
    rollback_upgrade \
    --admin "$($CLI keys address "$IDENTITY")"

  log "INFO" "Migration rolled back successfully"
  progress_update "rollback_migration" "completed"
}

run_custom_migration_script() {
  if [[ -n "$MIGRATION_SCRIPT" && -f "$MIGRATION_SCRIPT" ]]; then
    log "INFO" "Running custom migration script: $MIGRATION_SCRIPT"
    run_cmd bash "$MIGRATION_SCRIPT" "$CONTRACT_ID" "$NETWORK" "$IDENTITY"
    progress_update "custom_migration_script" "completed"
  fi
}

cleanup() {
  log "INFO" "Cleaning up migration artifacts..."
  # Remove temporary files, keep logs and backups
  progress_update "cleanup" "completed"
}

main() {
  # Parse arguments
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --network) NETWORK="$2"; shift 2 ;;
      --identity) IDENTITY="$2"; shift 2 ;;
      --contract-id) CONTRACT_ID="$2"; shift 2 ;;
      --new-wasm) NEW_WASM_PATH="$2"; shift 2 ;;
      --config) MIGRATION_CONFIG="$2"; shift 2 ;;
      --dry-run) DRY_RUN=1; shift ;;
      --non-interactive) NON_INTERACTIVE=1; shift ;;
      --skip-validation) SKIP_VALIDATION=1; shift ;;
      --force) FORCE_MIGRATION=1; shift ;;
      --no-rollback) ROLLBACK_ON_FAILURE=0; shift ;;
      -h|--help) usage; exit 0 ;;
      *) error_exit "Unknown option: $1" ;;
    esac
  done

  # Validate required arguments
  if [[ -z "$IDENTITY" || -z "$CONTRACT_ID" || -z "$NEW_WASM_PATH" ]]; then
    usage
    error_exit "Missing required arguments: --identity, --contract-id, --new-wasm"
  fi

  # Load migration config if provided
  if [[ -n "$MIGRATION_CONFIG" ]]; then
    load_migration_config "$MIGRATION_CONFIG"
  else
    TARGET_VERSION="${TARGET_VERSION:-2}"
  fi

  log "INFO" "Starting TeachLink contract migration"
  log "INFO" "Network: $NETWORK, Contract: $CONTRACT_ID, Target Version: $TARGET_VERSION"

  if ! confirm "Continue with migration?"; then
    log "INFO" "Migration cancelled by user"
    exit 0
  fi

  # Migration steps
  validate_environment
  backup_current_state
  run_pre_migration_validation
  prepare_migration
  run_custom_migration_script
  execute_migration
  run_post_migration_validation
  cleanup

  log "INFO" "Migration completed successfully!"
  echo "Migration log: $MIGRATION_LOG"
  echo "Backup location: $BACKUP_DIR"
}

# Run main function
main "$@"