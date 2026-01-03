#!/bin/bash
# Enterprise-grade pre-commit script for omni-minify (Unix/Linux/macOS)
# Automatically fixes formatting and clippy issues before commit

set -e  # Exit on any error

echo "[PRE-COMMIT] Starting code quality checks and fixes..."

# Check if we're in a git repository
if ! git rev-parse --git-dir >/dev/null 2>&1; then
    echo "[ERROR] Not in a git repository"
    exit 1
fi

# Check if cargo is available
if ! command -v cargo >/dev/null 2>&1; then
    echo "[ERROR] Cargo not found. Please install Rust toolchain."
    exit 1
fi

# Apply rustfmt formatting fixes
echo "[PRE-COMMIT] Applying rustfmt formatting fixes..."
if ! cargo fmt --all; then
    echo "[ERROR] rustfmt failed"
    exit 1
fi

# Apply clippy auto-fixes for performance and correctness issues
echo "[PRE-COMMIT] Applying clippy auto-fixes..."
if ! cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged; then
    echo "[ERROR] clippy auto-fix failed"
    exit 1
fi

# Run clippy again to check for remaining issues
echo "[PRE-COMMIT] Checking for remaining clippy issues..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "[ERROR] Clippy found issues that require manual fixing:"
    echo "[INFO] Run 'cargo clippy --all-targets --all-features' to see details"
    exit 1
fi

# Check if any files were modified by the fixes (ignore line ending warnings)
if ! git diff --quiet; then
    echo "[PRE-COMMIT] Code formatting and fixes applied. Files modified:"
    git diff --name-only
    echo "[INFO] Please review changes and re-stage files for commit"
    echo "[INFO] Run 'git add .' to stage all changes, then commit again"
    echo "[NOTE] Line ending warnings are normal and handled by .gitattributes"
    exit 1
fi

# Run tests to ensure fixes didn't break anything
echo "[PRE-COMMIT] Running tests to verify fixes..."
if ! cargo test --all; then
    echo "[ERROR] Tests failed after applying fixes"
    echo "[INFO] Please review and fix test failures manually"
    exit 1
fi

# Check documentation builds
echo "[PRE-COMMIT] Verifying documentation builds..."
if ! cargo doc --all --no-deps --quiet; then
    echo "[ERROR] Documentation build failed"
    exit 1
fi

echo "[PRE-COMMIT] ✓ All checks passed! Code is ready for commit."
echo "[PRE-COMMIT] Summary:"
echo "  - Formatting: Applied and verified"
echo "  - Clippy: Auto-fixes applied, no remaining issues"
echo "  - Tests: All passing"
echo "  - Documentation: Builds successfully"
echo "  - Line endings: Normalized by .gitattributes"

exit 0