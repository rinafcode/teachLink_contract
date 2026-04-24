#!/usr/bin/env bash
# Capture WASM size and test timing into baseline_current.json
set -euo pipefail

WASM="target/wasm32v1-none/release/teachlink_contract.wasm"

if [ ! -f "$WASM" ]; then
  echo "ERROR: WASM not found at $WASM" >&2
  exit 1
fi

wasm_bytes=$(stat -c%s "$WASM")

unit_secs="${unit_test_duration:-0}"
integration_secs="${integration_test_duration:-0}"

cat > baseline_current.json <<EOF
{
  "captured_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "commit": "${GITHUB_SHA:-local}",
  "wasm_size_bytes": ${wasm_bytes},
  "unit_test_duration_s": ${unit_secs},
  "integration_test_duration_s": ${integration_secs}
}
EOF

echo "Baseline captured:"
cat baseline_current.json
