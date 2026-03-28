#!/bin/bash

# Test Coverage Script for TeachLink Contract
# This script runs comprehensive tests with coverage reporting

set -e

echo "🔍 Running TeachLink Contract Test Coverage Analysis..."

# Install coverage tools if not already installed
echo "📦 Installing coverage tools..."
cargo install cargo-llvm-cov || echo "cargo-llvm-cov already installed"
cargo install cargo-tarpaulin || echo "cargo-tarpaulin already installed"

# Clean previous coverage data
echo "🧹 Cleaning previous coverage data..."
cargo clean
cargo llvm-cov clean --workspace

# Run tests with coverage
echo "🧪 Running tests with coverage..."
cargo llvm-cov --workspace --lib --bins --tests --all-features --lcov --output-path lcov.info

# Generate HTML coverage report
echo "📊 Generating HTML coverage report..."
cargo llvm-cov --workspace --lib --bins --tests --all-features --html

# Run tarpaulin for additional coverage analysis
echo "📈 Running tarpaulin coverage analysis..."
cargo tarpaulin --workspace --all-features --out Html --output-dir tarpaulin-report

# Check coverage thresholds
echo "✅ Checking coverage thresholds..."
COVERAGE_PERCENTAGE=$(cargo llvm-cov --workspace --lib --bins --tests --all-features --json | jq -r '.data[0].totals.percent_covered')

echo "📊 Current Coverage: ${COVERAGE_PERCENTAGE}%"

# Check if coverage meets minimum threshold (80%)
THRESHOLD=80
if (( $(echo "$COVERAGE_PERCENTAGE >= $THRESHOLD" | bc -l) )); then
    echo "✅ Coverage threshold met: ${COVERAGE_PERCENTAGE}% >= ${THRESHOLD}%"
else
    echo "❌ Coverage threshold NOT met: ${COVERAGE_PERCENTAGE}% < ${THRESHOLD}%"
    exit 1
fi

echo "🎉 Test coverage analysis completed successfully!"
echo "📁 HTML report available at: target/llvm-cov/html/index.html"
echo "📁 Tarpaulin report available at: tarpaulin-report/tarpaulin-report.html"
