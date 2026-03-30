#!/bin/bash
set -e

# Generate API documentation for TeachLink smart contracts

echo "Generating documentation for TeachLink contracts..."

# Ensure we're in the project root
if [ ! -f "Cargo.toml" ]; then
    echo "Error: This script must be run from the project root."
    exit 1
fi

# Generate Rust documentation using cargo doc
echo "Running cargo doc..."
cargo doc --no-deps --document-private-items --workspace

# Check if browser is available to open (optional/skipped in headless)
# open target/doc/teachlink/index.html

echo "Documentation generated successfully in target/doc/"

# Optional: Generate Soroban spec (JSON) for each contract
# echo "Generating Soroban contract specs..."
# stellar contract spec json --wasm target/wasm32-unknown-unknown/release/teachlink.wasm > docs/teachlink_spec.json
