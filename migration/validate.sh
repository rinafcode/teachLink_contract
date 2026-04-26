#!/usr/bin/env bash
set -euo pipefail

# TeachLink Contract Migration Validation Tools
# Comprehensive validation checks for contract migration

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
MIGRATION_DIR="$ROOT_DIR/migration"
CONFIG_DIR="$ROOT_DIR/config/networks"

NETWORK="testnet"
IDENTITY=""
CONTRACT_ID=""
VALIDATION_TYPE="pre"  # pre|post|full
OUTPUT_FORMAT="text"    # text|json
VERBOSE=0

usage() {
  cat <<USAGE
Usage: $0 [options]

TeachLink Contract Migration Validation

Options:
  --network <name>       Network to validate on (testnet|mainnet|local)
  --identity <name>      Identity name for validation
  --contract-id <id>     Contract ID to validate
  --type <type>          Validation type: pre|post|full (default: pre)
  --format <format>      Output format: text|json (default: text)
  --verbose              Enable verbose output
  -h, --help             Show this help

Validation Types:
  pre    - Pre-migration validation checks
  post   - Post-migration validation checks
  full   - Complete validation suite

Examples:
  $0 --network testnet --contract-id CB4HK... --identity deployer --type pre
  $0 --contract-id CB4HK... --type full --format json

USAGE
}

log() {
  local level="$1"
  local message="$2"
  if [[ $VERBOSE -eq 1 || "$level" != "DEBUG" ]]; then
    echo "[$level] $message"
  fi
}

error() {
  local message="$1"
  echo "[ERROR] $message" >&2
}

warning() {
  local message="$1"
  echo "[WARN] $message" >&2
}

success() {
  local message="$1"
  echo "[OK] $message"
}

json_output() {
  local data="$1"
  if [[ "$OUTPUT_FORMAT" == "json" ]]; then
    echo "$data"
  fi
}

validate_environment() {
  log "INFO" "Validating environment setup..."

  # Check CLI availability
  if command -v stellar >/dev/null 2>&1; then
    CLI=stellar
  elif command -v soroban >/dev/null 2>&1; then
    CLI=soroban
  else
    error "Missing CLI: install stellar or soroban"
    return 1
  fi

  # Check network config
  local config_file="$CONFIG_DIR/${NETWORK}.env"
  if [[ ! -f "$config_file" ]]; then
    error "Network config not found: $config_file"
    return 1
  fi

  # Load network config
  set -a
  source "$config_file"
  set +a

  # Check required variables
  if [[ -z "${SOROBAN_RPC_URL:-}" ]]; then
    error "Network config missing SOROBAN_RPC_URL"
    return 1
  fi

  success "Environment validation passed"
  return 0
}

validate_contract_connectivity() {
  log "INFO" "Validating contract connectivity..."

  # Test basic contract call
  if ! $CLI contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    -- \
    get_current_version >/dev/null 2>&1; then
    error "Cannot connect to contract $CONTRACT_ID"
    return 1
  fi

  success "Contract connectivity validated"
  return 0
}

validate_contract_version() {
  log "INFO" "Validating contract version..."

  local version
  if ! version=$($CLI contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    -- \
    get_current_version 2>/dev/null); then
    error "Failed to get contract version"
    return 1
  fi

  if [[ -z "$version" || "$version" -lt 1 ]]; then
    error "Invalid contract version: $version"
    return 1
  fi

  success "Contract version: $version"
  return 0
}

validate_contract_admin() {
  log "INFO" "Validating contract admin access..."

  if [[ -z "$IDENTITY" ]]; then
    warning "No identity provided, skipping admin validation"
    return 0
  fi

  # Try to get admin address (assuming contract has admin getter)
  local admin_address
  if admin_address=$($CLI contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    -- \
    get_admin 2>/dev/null); then

    local identity_address
    identity_address=$($CLI keys address "$IDENTITY" 2>/dev/null)

    if [[ "$admin_address" != "$identity_address" ]]; then
      warning "Identity $IDENTITY may not be contract admin"
      log "DEBUG" "Contract admin: $admin_address"
      log "DEBUG" "Identity address: $identity_address"
    else
      success "Admin access validated"
    fi
  else
    warning "Could not verify admin access (contract may not have get_admin function)"
  fi

  return 0
}

validate_contract_state() {
  log "INFO" "Validating contract state integrity..."

  # Check if contract has upgrade system initialized
  local upgrade_available
  if upgrade_available=$($CLI contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    -- \
    is_rollback_available 2>/dev/null); then
    success "Upgrade system available"
  else
    warning "Upgrade system not available or not initialized"
  fi

  # Check for emergency state (if contract has emergency module)
  local emergency_state
  if emergency_state=$($CLI contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    -- \
    is_paused 2>/dev/null); then
    if [[ "$emergency_state" == "true" ]]; then
      warning "Contract is in emergency/paused state"
    else
      success "Contract not in emergency state"
    fi
  fi

  return 0
}

validate_network_health() {
  log "INFO" "Validating network health..."

  # Check RPC connectivity
  if ! curl -s --max-time 10 "${SOROBAN_RPC_URL}/health" >/dev/null 2>&1; then
    if ! curl -s --max-time 10 "${SOROBAN_RPC_URL}" >/dev/null 2>&1; then
      error "Cannot connect to RPC endpoint: $SOROBAN_RPC_URL"
      return 1
    fi
  fi

  success "Network connectivity validated"
  return 0
}

validate_gas_estimates() {
  log "INFO" "Validating gas estimates for migration operations..."

  # Estimate gas for prepare_upgrade call
  local gas_estimate
  if gas_estimate=$($CLI contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    --dry-run \
    -- \
    prepare_upgrade \
    --admin "$($CLI keys address "$IDENTITY")" \
    --new_version 999 \
    --state_hash "0x1234" 2>&1 | grep -o "gas:[[:space:]]*[0-9]*" | grep -o "[0-9]*" || echo ""); then

    if [[ -n "$gas_estimate" && "$gas_estimate" -gt 0 ]]; then
      success "Gas estimation successful: $gas_estimate"
    else
      warning "Could not estimate gas for migration operations"
    fi
  else
    warning "Gas estimation failed"
  fi

  return 0
}

run_pre_migration_checks() {
  log "INFO" "Running pre-migration validation checks..."

  local checks_passed=0
  local total_checks=0

  ((total_checks++))
  if validate_environment; then
    ((checks_passed++))
  fi

  ((total_checks++))
  if validate_network_health; then
    ((checks_passed++))
  fi

  ((total_checks++))
  if validate_contract_connectivity; then
    ((checks_passed++))
  fi

  ((total_checks++))
  if validate_contract_version; then
    ((checks_passed++))
  fi

  ((total_checks++))
  if validate_contract_admin; then
    ((checks_passed++))
  fi

  ((total_checks++))
  if validate_contract_state; then
    ((checks_passed++))
  fi

  ((total_checks++))
  if validate_gas_estimates; then
    ((checks_passed++))
  fi

  log "INFO" "Pre-migration validation: $checks_passed/$total_checks checks passed"

  if [[ "$OUTPUT_FORMAT" == "json" ]]; then
    json_output "{\"validation_type\":\"pre\",\"checks_passed\":$checks_passed,\"total_checks\":$total_checks,\"success\":$((checks_passed == total_checks ? 1 : 0))}"
  fi

  return $((checks_passed == total_checks ? 0 : 1))
}

run_post_migration_checks() {
  log "INFO" "Running post-migration validation checks..."

  local checks_passed=0
  local total_checks=0

  ((total_checks++))
  if validate_contract_connectivity; then
    ((checks_passed++))
  fi

  ((total_checks++))
  if validate_contract_version; then
    ((checks_passed++))
  fi

  # Check that rollback is available
  ((total_checks++))
  local rollback_available
  if rollback_available=$($CLI contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    -- \
    is_rollback_available 2>/dev/null); then
    if [[ "$rollback_available" == "true" ]]; then
      success "Rollback available after migration"
      ((checks_passed++))
    else
      warning "Rollback not available after migration"
    fi
  else
    warning "Could not check rollback availability"
  fi

  # Check upgrade history
  ((total_checks++))
  local upgrade_history
  if upgrade_history=$($CLI contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    -- \
    get_upgrade_history --version 2 2>/dev/null); then
    if [[ -n "$upgrade_history" ]]; then
      success "Upgrade history recorded"
      ((checks_passed++))
    else
      warning "No upgrade history found"
    fi
  else
    warning "Could not retrieve upgrade history"
  fi

  log "INFO" "Post-migration validation: $checks_passed/$total_checks checks passed"

  if [[ "$OUTPUT_FORMAT" == "json" ]]; then
    json_output "{\"validation_type\":\"post\",\"checks_passed\":$checks_passed,\"total_checks\":$total_checks,\"success\":$((checks_passed == total_checks ? 1 : 0))}"
  fi

  return $((checks_passed == total_checks ? 0 : 1))
}

run_full_validation() {
  log "INFO" "Running full validation suite..."

  if ! run_pre_migration_checks; then
    error "Pre-migration checks failed"
    return 1
  fi

  if ! run_post_migration_checks; then
    error "Post-migration checks failed"
    return 1
  fi

  success "Full validation suite passed"
  return 0
}

main() {
  # Parse arguments
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --network) NETWORK="$2"; shift 2 ;;
      --identity) IDENTITY="$2"; shift 2 ;;
      --contract-id) CONTRACT_ID="$2"; shift 2 ;;
      --type) VALIDATION_TYPE="$2"; shift 2 ;;
      --format) OUTPUT_FORMAT="$2"; shift 2 ;;
      --verbose) VERBOSE=1; shift ;;
      -h|--help) usage; exit 0 ;;
      *) error "Unknown option: $1"; usage; exit 1 ;;
    esac
  done

  # Validate required arguments
  if [[ -z "$CONTRACT_ID" ]]; then
    error "Contract ID is required"
    usage
    exit 1
  fi

  case "$VALIDATION_TYPE" in
    pre) run_pre_migration_checks ;;
    post) run_post_migration_checks ;;
    full) run_full_validation ;;
    *) error "Invalid validation type: $VALIDATION_TYPE"; exit 1 ;;
  esac
}

# Run main function
main "$@"