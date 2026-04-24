#!/usr/bin/env bash
# Generate regression_report.md from available artifacts.
set -euo pipefail

CURRENT="baseline_current.json"
PREVIOUS="baseline_previous.json"
REPORT="regression_report.md"
TEST_OUTPUT="ci_test_output.txt"

read_field() {
  if [ -f "$1" ]; then
    python3 -c "import json,sys; d=json.load(open('$1')); print(d.get('$2','N/A'))"
  else
    echo "N/A"
  fi
}

commit=$(read_field "$CURRENT" commit)
captured=$(read_field "$CURRENT" captured_at)
wasm_cur=$(read_field "$CURRENT" wasm_size_bytes)
unit_cur=$(read_field "$CURRENT" unit_test_duration_s)
intg_cur=$(read_field "$CURRENT" integration_test_duration_s)

wasm_prev=$(read_field "$PREVIOUS" wasm_size_bytes)
unit_prev=$(read_field "$PREVIOUS" unit_test_duration_s)
intg_prev=$(read_field "$PREVIOUS" integration_test_duration_s)

delta() {
  local cur=$1 prev=$2
  if [ "$prev" = "N/A" ] || [ "$prev" = "0" ]; then echo "—"; return; fi
  python3 -c "
cur, prev = $cur, $prev
d = cur - prev
sign = '+' if d >= 0 else ''
pct = d * 100 / prev if prev else 0
print(f'{sign}{d} ({sign}{pct:.1f}%)')
"
}

# Count test results
passed=0; failed=0
if [ -f "$TEST_OUTPUT" ]; then
  passed=$(grep -c "^test .* ok$" "$TEST_OUTPUT" 2>/dev/null || true)
  failed=$(grep -c "^test .* FAILED$" "$TEST_OUTPUT" 2>/dev/null || true)
fi

cat > "$REPORT" <<EOF
# Regression Report

**Commit:** \`${commit}\`
**Generated:** ${captured}

## Test Results

| Suite | Status |
|-------|--------|
| Unit tests | $([ "$failed" -eq 0 ] && echo "✅ Passed" || echo "❌ Failed") |
| Integration tests | $([ "$failed" -eq 0 ] && echo "✅ Passed" || echo "❌ Failed") |

Tests passed: **${passed}** | Failed: **${failed}**

## Performance

| Metric | Current | Previous | Delta |
|--------|---------|----------|-------|
| WASM size (bytes) | ${wasm_cur} | ${wasm_prev} | $(delta "$wasm_cur" "$wasm_prev") |
| Unit test time (s) | ${unit_cur} | ${unit_prev} | $(delta "$unit_cur" "$unit_prev") |
| Integration time (s) | ${intg_cur} | ${intg_prev} | $(delta "$intg_cur" "$intg_prev") |

## Thresholds

| Check | Threshold |
|-------|-----------|
| WASM hard limit | 300 KB (307200 bytes) |
| WASM size regression | +10% vs baseline |
| Test duration regression | +50% vs baseline |
EOF

echo "Report written to $REPORT"
cat "$REPORT"
