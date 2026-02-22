#!/bin/bash
# Run all tests with coverage and reporting

set -e

echo "🧪 Running TeachLink Test Suite"
echo "================================"

# Run unit tests
echo "📦 Running unit tests..."
cargo test --lib --workspace

# Run integration tests
echo "🔗 Running integration tests..."
cargo test --test '*' --workspace

# Run with coverage
echo "📊 Generating coverage report..."
cargo tarpaulin --out Html --output-dir testing/reports/coverage

# Run benchmarks
echo "⚡ Running performance benchmarks..."
cargo bench --workspace

# Run security scan
echo "🔒 Running security scan..."
cargo audit

# Generate final report
echo "📄 Generating test report..."
./testing/scripts/generate_report.sh

echo "✅ All tests completed!"
