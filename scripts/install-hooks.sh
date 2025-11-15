#!/bin/bash
# Install Git hooks for CLI Testing Specialist
#
# This script copies Git hooks from scripts/git-hooks/ to .git/hooks/
# Run this after cloning the repository to enable automatic code formatting.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOOKS_DIR="$SCRIPT_DIR/git-hooks"
GIT_HOOKS_DIR="$SCRIPT_DIR/../.git/hooks"

echo "📦 Installing Git hooks..."

# Check if .git directory exists
if [ ! -d "$GIT_HOOKS_DIR" ]; then
    echo "❌ Error: .git/hooks directory not found"
    echo "   Are you in the project root directory?"
    exit 1
fi

# Install pre-commit hook
if [ -f "$HOOKS_DIR/pre-commit" ]; then
    cp "$HOOKS_DIR/pre-commit" "$GIT_HOOKS_DIR/pre-commit"
    chmod +x "$GIT_HOOKS_DIR/pre-commit"
    echo "   ✓ pre-commit hook installed"
else
    echo "   ⚠ pre-commit hook not found"
fi

echo ""
echo "✅ Git hooks installed successfully!"
echo ""
echo "Installed hooks:"
echo "  • pre-commit: Auto-format Rust code with cargo fmt"
echo ""
echo "To skip hooks temporarily, use: git commit --no-verify"
