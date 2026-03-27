#!/bin/bash
# Generate comprehensive test report

REPORT_DIR="testing/reports"
mkdir -p "$REPORT_DIR"

echo "Generating Test Report..."
echo "=========================" > "$REPORT_DIR/summary.txt"
echo "" >> "$REPORT_DIR/summary.txt"

# Test results
cargo test --workspace 2>&1 | tee "$REPORT_DIR/test_results.txt"

# Coverage
if command -v cargo-tarpaulin &> /dev/null; then
    cargo tarpaulin --out Json --output-dir "$REPORT_DIR"
fi

# Security audit
cargo audit --json > "$REPORT_DIR/security_audit.json" 2>&1 || true

echo "Report generated in $REPORT_DIR"
