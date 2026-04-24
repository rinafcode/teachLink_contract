#!/usr/bin/env bash
# Compare baseline_current.json against baseline_previous.json.
# Fails if WASM size or test duration regressed beyond thresholds.
set -euo pipefail

CURRENT="baseline_current.json"
PREVIOUS="baseline_previous.json"

# Thresholds
WASM_LIMIT_BYTES=307200          # 300 KB hard limit
WASM_REGRESSION_PCT=10           # fail if size grew >10% vs previous
DURATION_REGRESSION_PCT=50       # fail if test time grew >50% vs previous

if [ ! -f "$CURRENT" ]; then
  echo "ERROR: $CURRENT not found. Run perf-baseline.sh first." >&2
  exit 1
fi

read_field() { python3 -c "import json,sys; d=json.load(open('$1')); print(d.get('$2',0))"; }

cur_wasm=$(read_field "$CURRENT" wasm_size_bytes)
cur_unit=$(read_field "$CURRENT" unit_test_duration_s)
cur_intg=$(read_field "$CURRENT" integration_test_duration_s)

FAILED=0

echo "=== Regression Check ==="
echo "WASM size:            ${cur_wasm} bytes"
echo "Unit test duration:   ${cur_unit}s"
echo "Integration duration: ${cur_intg}s"

# Hard WASM size limit
if [ "$cur_wasm" -gt "$WASM_LIMIT_BYTES" ]; then
  echo "FAIL: WASM size ${cur_wasm} bytes exceeds hard limit ${WASM_LIMIT_BYTES} bytes"
  FAILED=1
fi

if [ -f "$PREVIOUS" ]; then
  prev_wasm=$(read_field "$PREVIOUS" wasm_size_bytes)
  prev_unit=$(read_field "$PREVIOUS" unit_test_duration_s)
  prev_intg=$(read_field "$PREVIOUS" integration_test_duration_s)

  echo ""
  echo "--- vs previous baseline ---"
  echo "Previous WASM:        ${prev_wasm} bytes"
  echo "Previous unit:        ${prev_unit}s"
  echo "Previous integration: ${prev_intg}s"

  check_regression() {
    local label=$1 cur=$2 prev=$3 pct=$4
    if [ "$prev" -gt 0 ]; then
      increase=$(python3 -c "print(int(($cur - $prev) * 100 / $prev))")
      if [ "$increase" -gt "$pct" ]; then
        echo "FAIL: $label increased by ${increase}% (threshold ${pct}%): ${prev} -> ${cur}"
        FAILED=1
      else
        echo "OK:   $label change: ${increase}% (${prev} -> ${cur})"
      fi
    fi
  }

  check_regression "WASM size"            "$cur_wasm" "$prev_wasm" "$WASM_REGRESSION_PCT"
  check_regression "Unit test duration"   "$cur_unit" "$prev_unit" "$DURATION_REGRESSION_PCT"
  check_regression "Integration duration" "$cur_intg" "$prev_intg" "$DURATION_REGRESSION_PCT"
else
  echo "(No previous baseline found — skipping comparison)"
fi

echo ""
if [ "$FAILED" -eq 1 ]; then
  echo "RESULT: REGRESSION DETECTED"
  exit 1
else
  echo "RESULT: PASSED"
fi
