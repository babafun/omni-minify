@echo off
REM Setup script for pre-commit hooks (Windows)

echo [SETUP] Configuring git pre-commit hooks for omni-minify...

REM Check if we're in a git repository
git rev-parse --git-dir >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Not in a git repository
    exit /b 1
)

REM Create .git/hooks directory if it doesn't exist
if not exist ".git\hooks" (
    mkdir ".git\hooks"
)

REM Copy the pre-commit script to git hooks
copy ".github\pre-commit\hook.bat" ".git\hooks\pre-commit.bat"
if errorlevel 1 (
    echo [ERROR] Failed to copy pre-commit script
    exit /b 1
)

REM Create a wrapper script that calls the batch file
echo @echo off > ".git\hooks\pre-commit"
echo call "%~dp0pre-commit.bat" >> ".git\hooks\pre-commit"

echo [SETUP] ✓ Pre-commit hook installed successfully!
echo [INFO] The hook will now run automatically before each commit.
echo [INFO] To bypass the hook temporarily, use: git commit --no-verify
echo [INFO] To test the hook manually, run: .github\pre-commit\hook.bat

exit /b 0