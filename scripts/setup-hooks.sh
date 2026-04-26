#!/bin/bash
# Setup script for pre-commit hooks
# Usage: ./scripts/setup-hooks.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
GIT_HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

echo "Setting up pre-commit hooks..."
echo ""

# Check if git directory exists
if [ ! -d "$PROJECT_ROOT/.git" ]; then
    echo "Error: Not in a git repository"
    exit 1
fi

# Make sure git hooks directory exists
mkdir -p "$GIT_HOOKS_DIR"

# Copy and make executable: pre-commit hook
echo "Installing pre-commit hook..."
cp "$SCRIPT_DIR/pre-commit" "$GIT_HOOKS_DIR/pre-commit"
chmod +x "$GIT_HOOKS_DIR/pre-commit"
echo "✓ Pre-commit hook installed"

# Copy and make executable: commit-msg hook
echo "Installing commit-msg hook..."
cp "$SCRIPT_DIR/commit-msg" "$GIT_HOOKS_DIR/commit-msg"
chmod +x "$GIT_HOOKS_DIR/commit-msg"
echo "✓ Commit-msg hook installed"

echo ""
echo "Pre-commit hooks installed successfully!"
echo ""
echo "Available hooks:"
echo "  • pre-commit:  Runs before committing (rustfmt, clippy, merge conflicts, etc.)"
echo "  • commit-msg:  Validates commit message format"
echo ""
echo "To run hooks manually:"
echo "  • Check specific hook: .git/hooks/pre-commit"
echo "  • Check commit message: .git/hooks/commit-msg <commit-msg-file>"
echo ""
echo "To bypass hooks (not recommended):"
echo "  • git commit --no-verify"
echo ""
