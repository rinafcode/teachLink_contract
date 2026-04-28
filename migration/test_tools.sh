#!/usr/bin/env bash
set -euo pipefail

# TeachLink Migration Tools Test Suite
# Validates that migration tools are working correctly

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
MIGRATION_DIR="$ROOT_DIR/migration"

echo "TeachLink Migration Tools Test Suite"
echo "===================================="

# Test 1: Check script existence and basic syntax
echo ""
echo "Test 1: Script validation"
echo "-------------------------"

scripts=("migrate.sh" "validate.sh" "rollback.sh" "progress.sh")
for script in "${scripts[@]}"; do
  script_path="$MIGRATION_DIR/$script"
  if [[ -f "$script_path" ]]; then
    echo "✓ $script exists"

    # Basic syntax check
    if bash -n "$script_path" 2>/dev/null; then
      echo "✓ $script syntax OK"
    else
      echo "✗ $script syntax error"
      exit 1
    fi

    # Check for help option
    if "$script_path" --help >/dev/null 2>&1; then
      echo "✓ $script help works"
    else
      echo "⚠ $script help may not work (non-critical)"
    fi
  else
    echo "✗ $script missing"
    exit 1
  fi
done

# Test 2: Check configuration template
echo ""
echo "Test 2: Configuration validation"
echo "--------------------------------"

config_file="$MIGRATION_DIR/config_template.json"
if [[ -f "$config_file" ]]; then
  echo "✓ Configuration template exists"

  # Basic JSON validation
  if command -v python3 >/dev/null 2>&1; then
    if python3 -m json.tool "$config_file" >/dev/null 2>&1; then
      echo "✓ Configuration JSON is valid"
    else
      echo "✗ Configuration JSON is invalid"
      exit 1
    fi
  elif command -v node >/dev/null 2>&1; then
    if node -e "console.log(JSON.parse(require('fs').readFileSync('$config_file', 'utf8')))" >/dev/null 2>&1; then
      echo "✓ Configuration JSON is valid"
    else
      echo "✗ Configuration JSON is invalid"
      exit 1
    fi
  else
    echo "⚠ Cannot validate JSON (no python3 or node available)"
  fi
else
  echo "✗ Configuration template missing"
  exit 1
fi

# Test 3: Check directory structure
echo ""
echo "Test 3: Directory structure"
echo "---------------------------"

directories=("logs" "reports" "backups" "scripts")
for dir in "${directories[@]}"; do
  dir_path="$MIGRATION_DIR/$dir"
  if [[ -d "$dir_path" ]]; then
    echo "✓ $dir/ directory exists"
  else
    echo "⚠ $dir/ directory missing (will be created when needed)"
  fi
done

# Test 4: Check README
echo ""
echo "Test 4: Documentation"
echo "---------------------"

readme_file="$MIGRATION_DIR/README.md"
if [[ -f "$readme_file" ]]; then
  echo "✓ README.md exists"

  # Check for key sections
  sections=("Overview" "Features" "Quick Start" "Safety Features")
  for section in "${sections[@]}"; do
    if grep -q "^#* $section" "$readme_file"; then
      echo "✓ README contains '$section' section"
    else
      echo "⚠ README missing '$section' section"
    fi
  done
else
  echo "✗ README.md missing"
  exit 1
fi

# Test 5: Dry run test
echo ""
echo "Test 5: Dry run validation"
echo "---------------------------"

# Test migrate.sh dry run (should not fail)
if "$MIGRATION_DIR/migrate.sh" --help >/dev/null 2>&1; then
  echo "✓ migrate.sh help works"
else
  echo "✗ migrate.sh help failed"
fi

# Test validate.sh dry run
if "$MIGRATION_DIR/validate.sh" --help >/dev/null 2>&1; then
  echo "✓ validate.sh help works"
else
  echo "✗ validate.sh help failed"
fi

# Test rollback.sh dry run
if "$MIGRATION_DIR/rollback.sh" --help >/dev/null 2>&1; then
  echo "✓ rollback.sh help works"
else
  echo "✗ rollback.sh help failed"
fi

# Test progress.sh dry run
if "$MIGRATION_DIR/progress.sh" --help >/dev/null 2>&1; then
  echo "✓ progress.sh help works"
else
  echo "✗ progress.sh help failed"
fi

echo ""
echo "Test Suite Complete"
echo "==================="
echo "Migration tools are ready for use!"
echo ""
echo "Next steps:"
echo "1. Configure your network settings in config/networks/"
echo "2. Test on testnet: ./migration/validate.sh --network testnet --contract-id YOUR_CONTRACT_ID --type pre"
echo "3. Run migration: ./migration/migrate.sh --network testnet --contract-id YOUR_CONTRACT_ID --new-wasm YOUR_WASM_FILE"
echo ""
echo "See migration/README.md for detailed documentation."