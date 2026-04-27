#!/usr/bin/env bash
# Performance Budget Enforcement Script
#
# This script checks current performance metrics against defined budgets
# and fails if any critical thresholds are exceeded.
#
# Usage: ./scripts/enforce_budgets.sh [--warn-only] [--verbose]

set -euo pipefail

# Configuration
BUDGET_FILE="performance_budgets.toml"
GAS_OUTPUT="gas_output.txt"
WASM_PATH="target/wasm32v1-none/release/teachlink_contract.wasm"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Counters
WARNINGS=0
CRITICAL=0
PASSED=0

# Parse arguments
WARN_ONLY=false
VERBOSE=false

for arg in "$@"; do
  case $arg in
    --warn-only)
      WARN_ONLY=true
      shift
      ;;
    --verbose)
      VERBOSE=true
      shift
      ;;
    *)
      echo "Unknown argument: $arg"
      exit 1
      ;;
  esac
done

# Helper functions
log_info() {
  echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
  echo -e "${GREEN}[PASS]${NC} $1"
  PASSED=$((PASSED + 1))
}

log_warning() {
  echo -e "${YELLOW}[WARN]${NC} $1"
  WARNINGS=$((WARNINGS + 1))
}

log_critical() {
  echo -e "${RED}[CRITICAL]${NC} $1"
  CRITICAL=$((CRITICAL + 1))
}

log_verbose() {
  if [ "$VERBOSE" = true ]; then
    echo -e "${BLUE}[DEBUG]${NC} $1"
  fi
}

# Parse TOML value (simple parser for flat keys)
parse_budget_value() {
  local section="$1"
  local key="$2"
  
  if [ ! -f "$BUDGET_FILE" ]; then
    log_warning "Budget file not found: $BUDGET_FILE"
    echo "0"
    return
  fi

  # Extract value from TOML
  local value
  value=$(grep -A 100 "^\[$section\]" "$BUDGET_FILE" | \
          grep -m 1 "^$key" | \
          sed 's/.*= *//' | \
          sed 's/ *#.*//' | \
          tr -d '[:space:]')
  
  if [ -z "$value" ]; then
    log_verbose "Budget not defined: [$section] $key"
    echo "0"
  else
    echo "$value"
  fi
}

# Check WASM size budget
check_wasm_size() {
  log_info "Checking WASM size budget..."
  
  if [ ! -f "$WASM_PATH" ]; then
    log_warning "WASM file not found: $WASM_PATH"
    log_info "Run 'cargo build --release --target wasm32v1-none' first"
    return
  fi

  local size_bytes
  size_bytes=$(stat -f%z "$WASM_PATH" 2>/dev/null || stat -c%s "$WASM_PATH" 2>/dev/null || echo "0")
  
  local max_budget
  max_budget=$(parse_budget_value "size_budgets" "max_wasm_size")
  
  local target_budget
  target_budget=$(parse_budget_value "size_budgets" "target_wasm_size")

  log_verbose "WASM size: $size_bytes bytes"
  log_verbose "Max budget: $max_budget bytes"
  log_verbose "Target budget: $target_budget bytes"

  # Check against max budget
  if [ "$size_bytes" -gt "$max_budget" ]; then
    log_critical "WASM size ($size_bytes bytes) exceeds maximum budget ($max_budget bytes)"
  elif [ "$size_bytes" -gt "$target_budget" ]; then
    log_warning "WASM size ($size_bytes bytes) exceeds target ($target_budget bytes) but within max"
  else
    log_success "WASM size ($size_bytes bytes) within budget ($target_budget bytes)"
  fi
}

# Check gas budgets
check_gas_budgets() {
  log_info "Checking gas budgets..."
  
  if [ ! -f "$GAS_OUTPUT" ]; then
    log_warning "Gas output file not found: $GAS_OUTPUT"
    log_info "Run gas benchmarks first"
    return
  fi

  # Extract gas usage from output
  # Expected format: "gas_used: operation=value"
  while IFS= read -r line; do
    if [[ "$line" =~ gas_used:\ ([a-z_]+)=([0-9]+) ]]; then
      local operation="${BASH_REMATCH[1]}"
      local actual_gas="${BASH_REMATCH[2]}"
      
      local budget_key="$operation"
      local gas_budget
      gas_budget=$(parse_budget_value "gas_budgets" "$budget_key")
      
      if [ "$gas_budget" -eq 0 ]; then
        log_verbose "No budget defined for operation: $operation"
        continue
      fi

      # Calculate percentage
      local percentage=$((actual_gas * 100 / gas_budget))
      
      if [ "$actual_gas" -gt "$gas_budget" ]; then
        local over_by=$((actual_gas - gas_budget))
        log_critical "Gas for '$operation': ${actual_gas} exceeds budget ${gas_budget} by ${over_by} (${percentage}%)"
      elif [ "$percentage" -gt 90 ]; then
        log_warning "Gas for '$operation': ${actual_gas} approaching budget ${gas_budget} (${percentage}%)"
      else
        log_success "Gas for '$operation': ${actual_gas} within budget ${gas_budget} (${percentage}%)"
      fi
    fi
  done < "$GAS_OUTPUT"
}

# Check test execution time budgets
check_time_budgets() {
  log_info "Checking time budgets..."
  
  # This would typically be integrated with CI/CD timing data
  # For now, we provide a framework for future implementation
  log_verbose "Time budget enforcement requires CI/CD integration"
  log_info "Time budgets defined in performance_budgets.toml"
}

# Generate summary report
generate_report() {
  echo ""
  echo "============================================"
  echo "  Performance Budget Enforcement Report"
  echo "============================================"
  echo ""
  echo -e "  ${GREEN}Passed:${NC}   $PASSED"
  echo -e "  ${YELLOW}Warnings:${NC} $WARNINGS"
  echo -e "  ${RED}Critical:${NC} $CRITICAL"
  echo ""
  
  if [ "$CRITICAL" -gt 0 ] && [ "$WARN_ONLY" = false ]; then
    echo -e "${RED}❌ CRITICAL: Performance budgets exceeded!${NC}"
    echo "Please optimize before proceeding."
    exit 1
  elif [ "$WARNINGS" -gt 0 ]; then
    echo -e "${YELLOW}⚠️  WARNING: Some budgets are approaching limits${NC}"
    echo "Consider optimization to prevent future violations."
    if [ "$WARN_ONLY" = false ] && [ "$CRITICAL" -gt 0 ]; then
      exit 1
    fi
  else
    echo -e "${GREEN}✅ SUCCESS: All performance budgets met!${NC}"
  fi
  
  echo ""
}

# Main execution
main() {
  echo "============================================"
  echo "  Performance Budget Enforcement"
  echo "============================================"
  echo ""
  echo "Budget file: $BUDGET_FILE"
  echo "Mode: $([ "$WARN_ONLY" = true ] && echo 'WARN ONLY' || echo 'ENFORCE')"
  echo ""
  
  if [ ! -f "$BUDGET_FILE" ]; then
    log_critical "Budget file not found: $BUDGET_FILE"
    exit 1
  fi
  
  # Run all checks
  check_wasm_size
  echo ""
  check_gas_budgets
  echo ""
  check_time_budgets
  echo ""
  
  # Generate report
  generate_report
}

main "$@"
