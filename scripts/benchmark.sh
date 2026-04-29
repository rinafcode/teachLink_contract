#!/bin/bash
set -e

# Benchmark Script for TeachLink Contract
# Includes gas usage benchmarking for performance regression detection

echo "============================================"
echo "  TeachLink Contract Benchmark Suite"
echo "============================================"

# 1. Build the Contract
echo ""
echo "[1/4] Building contract in release mode..."
cargo build --release --target wasm32-unknown-unknown -p teachlink-contract

# 2. Check WASM Size
echo ""
echo "[2/4] Checking WASM binary size..."
WASM_PATH="target/wasm32-unknown-unknown/release/teachlink_contract.wasm"
if [ -f "$WASM_PATH" ]; then
    SIZE=$(du -h "$WASM_PATH" | cut -f1)
    echo "  WASM Size: $SIZE"

    if command -v stat >/dev/null 2>&1; then
        if stat -f%z "$WASM_PATH" >/dev/null 2>&1; then
            SIZE_BYTES=$(stat -f%z "$WASM_PATH")
        else
            SIZE_BYTES=$(stat -c%s "$WASM_PATH")
        fi
    else
        echo "  Warning: stat command not available, skipping size check"
        SIZE_BYTES=0
    fi

    if [ "$SIZE_BYTES" -gt 307200 ] && [ "$SIZE_BYTES" -ne 0 ]; then
        echo "  FAIL: WASM size ($SIZE_BYTES bytes) exceeds 300 KB threshold!"
        WASM_OK=0
    elif [ "$SIZE_BYTES" -gt 256000 ] && [ "$SIZE_BYTES" -ne 0 ]; then
        echo "  WARNING: WASM size ($SIZE_BYTES bytes) approaching 300 KB limit"
        WASM_OK=1
    else
        echo "  PASS: WASM size is within limits ($SIZE_BYTES bytes)."
        WASM_OK=1
    fi
else
    echo "  Error: WASM file not found at $WASM_PATH"
    exit 1
fi

# 3. Run Gas Benchmarks
echo ""
echo "[3/4] Running gas usage benchmarks..."
start_time=$(date +%s)
cargo test --release -p teachlink-contract --test test_gas_benchmarks -- --nocapture 2>&1 | tee gas_output.txt
end_time=$(date +%s)
gas_duration=$((end_time - start_time))
echo "  Gas benchmarks completed in ${gas_duration}s"

# 4. Run Unit Tests (Performance Check)
echo ""
echo "[4/4] Running unit tests..."
start_time=$(date +%s)
cargo test --release --lib
end_time=$(date +%s)

duration=$((end_time - start_time))
echo "  Tests completed in ${duration}s"

# Summary
echo ""
echo "============================================"
echo "  Benchmark Summary"
echo "============================================"
echo "  WASM Binary:        $SIZE ($SIZE_BYTES bytes)"
echo "  Gas Benchmarks:     ${gas_duration}s"
echo "  Unit Tests:         ${duration}s"
echo ""

# Check for gas regressions
if [ -f gas_output.txt ]; then
    REGRESSIONS=$(grep -c "GAS REGRESSION" gas_output.txt || true)
    if [ "$REGRESSIONS" -gt 0 ]; then
        echo "  RESULT: FAILED - $REGRESSIONS gas regression(s) detected"
        echo "============================================"
        exit 1
    fi
fi

if [ "$WASM_OK" -eq 0 ]; then
    echo "  RESULT: FAILED - WASM size exceeds threshold"
    echo "============================================"
    exit 1
fi

echo "  RESULT: PASSED - All benchmarks within thresholds"
echo "============================================"
