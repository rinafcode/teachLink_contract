#!/usr/bin/env bash
# Check gas benchmark results against baselines in gas_baseline.json.
# Exits non-zero if any operation exceeds its threshold.
set -euo pipefail

BASELINE="gas_baseline.json"
GAS_OUTPUT="gas_output.txt"
THRESHOLD_FILE="gas_thresholds.json"

if [ ! -f "$BASELINE" ]; then
  echo "No baseline file found ($BASELINE). Skipping regression check."
  exit 0
fi

if [ ! -f "$GAS_OUTPUT" ]; then
  echo "No gas output file found ($GAS_OUTPUT). Skipping regression check."
  exit 0
fi

REGRESSIONS=0

# Parse threshold values from gas_baseline.json (uses jq if available, else grep)
check_threshold() {
  local op="$1"
  local used="$2"
  local threshold

  if command -v jq &>/dev/null; then
    threshold=$(jq -r --arg op "$op" '.[$op].threshold // empty' "$BASELINE" 2>/dev/null)
  else
    threshold=$(grep -A2 "\"$op\"" "$BASELINE" | grep '"threshold"' | grep -o '[0-9]*' | head -1)
  fi

  if [ -z "$threshold" ] || [ "$threshold" = "0" ]; then
    return
  fi

  if [ "$used" -gt "$threshold" ]; then
    echo "GAS REGRESSION: $op used $used instructions (threshold: $threshold)"
    REGRESSIONS=$((REGRESSIONS + 1))
  else
    echo "OK: $op used $used instructions (threshold: $threshold)"
  fi
}

echo "=== Performance Regression Check ==="

# Extract gas usage lines from test output (format: "gas_used: <op>=<value>")
while IFS= read -r line; do
  if [[ "$line" =~ gas_used:[[:space:]]*([a-z_]+)=([0-9]+) ]]; then
    op="${BASH_REMATCH[1]}"
    used="${BASH_REMATCH[2]}"
    check_threshold "$op" "$used"
  fi
done < "$GAS_OUTPUT"

# Also check for explicit GAS REGRESSION markers from the test suite itself
SUITE_REGRESSIONS=$(grep -c "GAS REGRESSION" "$GAS_OUTPUT" 2>/dev/null || true)
REGRESSIONS=$((REGRESSIONS + SUITE_REGRESSIONS))

echo ""
if [ "$REGRESSIONS" -gt 0 ]; then
  echo "FAILED: $REGRESSIONS regression(s) detected."
  exit 1
else
  echo "PASSED: No performance regressions detected."
fi
