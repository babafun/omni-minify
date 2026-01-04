#!/bin/bash
# Setup script for pre-commit hooks (Unix/Linux/macOS)

set -e

echo "[SETUP] Configuring git pre-commit hooks for omni-minify..."

# Check if we're in a git repository
if ! git rev-parse --git-dir >/dev/null 2>&1; then
    echo "[ERROR] Not in a git repository"
    exit 1
fi

# Create .git/hooks directory if it doesn't exist
mkdir -p .git/hooks

# Copy the pre-commit script to git hooks and make it executable
cp .github/pre-commit/hook.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

echo "[SETUP] ✓ Pre-commit hook installed successfully!"
echo "[INFO] The hook will now run automatically before each commit."
echo "[INFO] To bypass the hook temporarily, use: git commit --no-verify"
echo "[INFO] To test the hook manually, run: ./.github/pre-commit/hook.sh"

exit 0