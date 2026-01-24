#!/bin/bash
set -e

# Benchmark Script for TeachLink Contract

echo "Starting Benchmark..."

# 1. Build the Contract
echo "Building contract in release mode..."
cargo build --release --target wasm32-unknown-unknown -p teachlink-contract

# 2. Check WASM Size
WASM_PATH="target/wasm32-unknown-unknown/release/teachlink_contract.wasm"
if [ -f "$WASM_PATH" ]; then
    SIZE=$(du -h "$WASM_PATH" | cut -f1)
    echo "WASM Size: $SIZE"
    
    # Optional: Check against a limit (e.g., 300KB)
    SIZE_BYTES=$(stat -c%s "$WASM_PATH")
    if [ "$SIZE_BYTES" -gt 307200 ]; then
        echo "WARNING: WASM size exceeds 300KB!"
    else
        echo "WASM size is within limits."
    fi
else
    echo "Error: WASM file not found at $WASM_PATH"
    exit 1
fi

# 3. Run Tests (Performance Check)
echo "Running unit tests..."
start_time=$(date +%s%N)
cargo test --release
end_time=$(date +%s%N)

duration=$((end_time - start_time))
duration_ms=$((duration / 1000000))

echo "Tests completed in ${duration_ms} ms"

echo "Benchmark Complete."
