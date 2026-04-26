#!/usr/bin/env bash
# Generate a structured test report from test_output.txt and gas_output.txt.
set -euo pipefail

REPORT_DIR="reports"
mkdir -p "$REPORT_DIR"

TEST_OUTPUT="test_output.txt"
GAS_OUTPUT="gas_output.txt"
SUMMARY="$REPORT_DIR/summary.md"

# Counters
passed=0
failed=0
ignored=0

if [ -f "$TEST_OUTPUT" ]; then
  passed=$(grep -c "^test .* ok$" "$TEST_OUTPUT" 2>/dev/null || true)
  failed=$(grep -c "^test .* FAILED$" "$TEST_OUTPUT" 2>/dev/null || true)
  ignored=$(grep -c "^test .* ignored$" "$TEST_OUTPUT" 2>/dev/null || true)
fi

gas_regressions=0
if [ -f "$GAS_OUTPUT" ]; then
  gas_regressions=$(grep -c "GAS REGRESSION" "$GAS_OUTPUT" 2>/dev/null || true)
fi

total=$((passed + failed))
status="✅ PASSED"
if [ "$failed" -gt 0 ] || [ "$gas_regressions" -gt 0 ]; then
  status="❌ FAILED"
fi

cat > "$SUMMARY" <<EOF
## Test Report — $(date -u '+%Y-%m-%d %H:%M UTC')

**Status:** $status

### Test Results
| Result  | Count |
|---------|-------|
| Passed  | $passed |
| Failed  | $failed |
| Ignored | $ignored |
| Total   | $total |

### Performance
| Metric              | Value |
|---------------------|-------|
| Gas regressions     | $gas_regressions |

EOF

# Append failed test names if any
if [ "$failed" -gt 0 ] && [ -f "$TEST_OUTPUT" ]; then
  echo "### Failed Tests" >> "$SUMMARY"
  echo '```' >> "$SUMMARY"
  grep "^test .* FAILED$" "$TEST_OUTPUT" >> "$SUMMARY" || true
  echo '```' >> "$SUMMARY"
fi

# Append gas regression details if any
if [ "$gas_regressions" -gt 0 ] && [ -f "$GAS_OUTPUT" ]; then
  echo "### Gas Regressions" >> "$SUMMARY"
  echo '```' >> "$SUMMARY"
  grep "GAS REGRESSION" "$GAS_OUTPUT" >> "$SUMMARY" || true
  echo '```' >> "$SUMMARY"
fi

# Copy raw outputs into report dir
[ -f "$TEST_OUTPUT" ] && cp "$TEST_OUTPUT" "$REPORT_DIR/"
[ -f "$GAS_OUTPUT" ]  && cp "$GAS_OUTPUT"  "$REPORT_DIR/"

echo "Report written to $SUMMARY"
cat "$SUMMARY"
