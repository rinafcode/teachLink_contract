@echo off
REM Test Coverage Script for TeachLink Contract (Windows)
REM This script runs comprehensive tests with coverage reporting

echo 🔍 Running TeachLink Contract Test Coverage Analysis...

REM Install coverage tools if not already installed
echo 📦 Installing coverage tools...
cargo install cargo-llvm-cov || echo cargo-llvm-cov already installed
cargo install cargo-tarpaulin || echo cargo-tarpaulin already installed

REM Clean previous coverage data
echo 🧹 Cleaning previous coverage data...
cargo clean
cargo llvm-cov clean --workspace

REM Run tests with coverage
echo 🧪 Running tests with coverage...
cargo llvm-cov --workspace --lib --bins --tests --all-features --lcov --output-path lcov.info

REM Generate HTML coverage report
echo 📊 Generating HTML coverage report...
cargo llvm-cov --workspace --lib --bins --tests --all-features --html

REM Run tarpaulin for additional coverage analysis
echo 📈 Running tarpaulin coverage analysis...
cargo tarpaulin --workspace --all-features --out Html --output-dir tarpaulin-report

echo 🎉 Test coverage analysis completed successfully!
echo 📁 HTML report available at: target\llvm-cov\html\index.html
echo 📁 Tarpaulin report available at: tarpaulin-report\tarpaulin-report.html
