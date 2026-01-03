# Test Files Organization

This directory contains all test files and sample data used for testing the omni-minify CLI and plugins.

## Directory Structure

- **`samples/`** - Sample files for testing plugin functionality
  - **`javascript/`** - JavaScript and TypeScript test files
  - **`css/`** - CSS test files  
  - **`html/`** - HTML test files
  - **`binary/`** - Binary format test files (GLB, etc.)
- **`integration/`** - Integration test files for end-to-end testing

## File Naming Conventions

- **Valid files**: `valid_*.ext`, `basic_*.ext`, `complex_*.ext`
- **Invalid files**: `malformed_*.ext`, `invalid_*.ext`, `broken_*.ext`
- **Edge cases**: `empty.ext`, `large_*.ext`, `minimal_*.ext`
- **Format-specific**: `typescript_*.ts`, `media_queries.css`, `binary_*.glb`

## Usage in Tests

All test files should be referenced using paths relative to the workspace root:

```rust
// Load test files from the proper directory structure
let valid_js = std::fs::read("tests/samples/javascript/basic_function.js")
    .expect("Test file should exist");

let malformed_css = std::fs::read("tests/samples/css/invalid_syntax.css")
    .expect("Test file should exist");
```

## Rules

- **NEVER place test files in the workspace root**
- **ALWAYS use the `tests/` directory** for any files used in testing
- **Organize by format and purpose** for easy discovery
- **Use descriptive filenames** that indicate test purpose
- **Include both positive and negative test cases**