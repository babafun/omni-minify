# Pre-commit Hooks for omni-minify

This directory contains enterprise-grade pre-commit hooks that automatically fix code formatting and performance issues before each commit.

## Directory Structure

```
.github/pre-commit/
├── hook.bat          # Windows pre-commit script
├── hook.sh           # Unix/Linux/macOS pre-commit script  
├── setup.bat         # Windows installation script
├── setup.sh          # Unix/Linux/macOS installation script
└── README.md         # This documentation
```

## What the hooks do

The pre-commit scripts automatically:

1. **Apply rustfmt formatting** - Ensures consistent code style using hard tabs
2. **Apply clippy auto-fixes** - Fixes performance issues like `x = x + 0`, unnecessary clones, etc.
3. **Verify remaining issues** - Ensures no unfixable clippy warnings remain
4. **Run tests** - Confirms fixes didn't break functionality
5. **Check documentation** - Verifies docs still build correctly

## Installation

### Windows
```cmd
.github\pre-commit\setup.bat
```

### Unix/Linux/macOS
```bash
./.github/pre-commit/setup.sh
```

## Manual execution

You can run the pre-commit checks manually:

### Windows
```cmd
.github\pre-commit\hook.bat
```

### Unix/Linux/macOS
```bash
./.github/pre-commit/hook.sh
```

## What happens during commit

1. **Automatic fixes applied** - The hook will fix formatting and auto-fixable clippy issues
2. **If files are modified** - The commit is aborted with instructions to review and re-stage
3. **If tests fail** - The commit is aborted with error details
4. **If all passes** - The commit proceeds normally

## Bypassing the hook

In rare cases where you need to bypass the hook:

```bash
git commit --no-verify
```

**Note:** Only use `--no-verify` for emergency commits. The hook ensures code quality and performance.

## Troubleshooting

### Hook fails with "cargo not found"
Install the Rust toolchain: https://rustup.rs/

### Hook fails with clippy errors
Run `cargo clippy --all-targets --all-features` to see detailed error messages and fix them manually.

### Hook modifies files unexpectedly
Review the changes with `git diff`, then:
1. Stage the changes: `git add .`
2. Commit again: `git commit`

### Tests fail after auto-fixes
The auto-fixes may have revealed existing test issues. Review and fix the failing tests manually.

## Performance focus

The clippy configuration specifically catches performance issues like:
- Unnecessary allocations (`vec![]` instead of `Vec::new()`)
- Inefficient string operations (`string + &str` instead of `push_str`)
- Redundant operations (`x = x + 0`, `x * 1`)
- Poor iterator usage (`collect().len()` instead of `count()`)
- Large stack allocations
- Unnecessary clones and copies

This ensures every commit maintains optimal performance characteristics.