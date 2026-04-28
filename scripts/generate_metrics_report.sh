#!/bin/bash

# Master script for Comprehensive Code Metrics Tracking

set -e

REPORTS_DIR="reports"
mkdir -p "$REPORTS_DIR"

echo "🚀 Starting Comprehensive Code Metrics Tracking..."
echo "================================================"

# 1. Run Test Coverage
echo "🧪 [1/4] Running Test Coverage..."
./scripts/coverage.sh || echo "⚠️ Coverage analysis failed, continuing..."

# 2. Run Duplication Check
echo "👥 [2/4] Monitoring Code Duplication..."
./scripts/metrics-duplication.sh || echo "⚠️ Duplication check failed, continuing..."

# 3. Run Complexity Analysis
# Since we implemented a Rust tool, we should build and run it.
# For now, we'll use clippy as a fallback or if the tool is ready.
echo "🧩 [3/4] Tracking Complexity Metrics..."
cargo clippy --workspace -- -W clippy::cognitive_complexity || echo "⚠️ Complexity check failed, continuing..."

# 4. Generate Consolidated Report
echo "📊 [4/4] Generating Consolidated Report..."

SUMMARY_FILE="$REPORTS_DIR/metrics_summary.md"

cat > "$SUMMARY_FILE" <<EOF
# Comprehensive Code Metrics Report
Generated on: $(date)

## 1. Test Coverage
- **LLVM Coverage**: [HTML Report](../target/llvm-cov/html/index.html)
- **Tarpaulin Report**: [HTML Report](../tarpaulin-report/tarpaulin-report.html)

## 2. Code Duplication
- **jscpd Report**: [HTML Report](duplication/jscpd-report.html)
- **Threshold**: 5%

## 3. Complexity
- **Cognitive Complexity**: Checked via Clippy
- **Cyclomatic Complexity**: TBD (Run quality tool)

## 4. Recommendations
EOF

# Add logic to extract values from JSON if they exist
if [ -f "lcov.info" ]; then
    echo "- ✅ Coverage data collected" >> "$SUMMARY_FILE"
fi

if [ -f "$REPORTS_DIR/duplication/jscpd-report.json" ]; then
    DUP_PERCENT=$(grep -o '"percentage":[0-9.]*' "$REPORTS_DIR/duplication/jscpd-report.json" | head -1 | cut -d: -f2)
    echo "- ✅ Duplication: ${DUP_PERCENT}%" >> "$SUMMARY_FILE"
fi

echo "================================================"
echo "✅ Comprehensive metrics tracking completed!"
echo "📁 Summary report: $SUMMARY_FILE"
EOF
