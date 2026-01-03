@echo off
REM Enterprise-grade pre-commit script for omni-minify (Windows)
REM Automatically fixes formatting and clippy issues before commit

echo [PRE-COMMIT] Starting code quality checks and fixes...

REM Check if we're in a git repository
git rev-parse --git-dir >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Not in a git repository
    exit /b 1
)

REM Check if cargo is available
cargo --version >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Cargo not found. Please install Rust toolchain.
    exit /b 1
)

REM Apply rustfmt formatting fixes
echo [PRE-COMMIT] Applying rustfmt formatting fixes...
cargo fmt --all
if errorlevel 1 (
    echo [ERROR] rustfmt failed
    exit /b 1
)

REM Apply clippy auto-fixes for performance and correctness issues
echo [PRE-COMMIT] Applying clippy auto-fixes...
cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged
if errorlevel 1 (
    echo [ERROR] clippy auto-fix failed
    exit /b 1
)

REM Run clippy again to check for remaining issues
echo [PRE-COMMIT] Checking for remaining clippy issues...
cargo clippy --all-targets --all-features -- -D warnings
if errorlevel 1 (
    echo [ERROR] Clippy found issues that require manual fixing:
    echo [INFO] Run 'cargo clippy --all-targets --all-features' to see details
    exit /b 1
)

REM Check if any files were modified by the fixes (ignore line ending warnings)
git diff --quiet
if errorlevel 1 (
    echo [PRE-COMMIT] Code formatting and fixes applied. Files modified:
    git diff --name-only
    echo [INFO] Please review changes and re-stage files for commit
    echo [INFO] Run 'git add .' to stage all changes, then commit again
    echo [NOTE] Line ending warnings are normal and handled by .gitattributes
    exit /b 1
)

REM Run tests to ensure fixes didn't break anything
echo [PRE-COMMIT] Running tests to verify fixes...
cargo test --all
if errorlevel 1 (
    echo [ERROR] Tests failed after applying fixes
    echo [INFO] Please review and fix test failures manually
    exit /b 1
)

REM Check documentation builds
echo [PRE-COMMIT] Verifying documentation builds...
cargo doc --all --no-deps --quiet
if errorlevel 1 (
    echo [ERROR] Documentation build failed
    exit /b 1
)

echo [PRE-COMMIT] ✓ All checks passed! Code is ready for commit.
echo [PRE-COMMIT] Summary:
echo   - Formatting: Applied and verified
echo   - Clippy: Auto-fixes applied, no remaining issues
echo   - Tests: All passing
echo   - Documentation: Builds successfully
echo   - Line endings: Normalized by .gitattributes

exit /b 0