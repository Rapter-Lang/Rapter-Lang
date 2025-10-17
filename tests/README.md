# Tests Directory

This directory should contain organized test files for the Rapter language.

## Current Status

**Note:** Many test files are currently in the root directory and should be organized here.

## Recommended Test Structure

```
tests/
├── unit/              # Unit tests for individual components
│   ├── lexer/
│   ├── parser/
│   ├── typechecker/
│   └── codegen/
├── integration/       # Integration tests
│   ├── control_flow/
│   ├── type_system/
│   ├── generics/
│   └── error_handling/
├── stdlib/            # Standard library tests
│   ├── io/
│   ├── strings/
│   └── collections/
└── regression/        # Regression tests for fixed bugs
```

## Running Tests

### All Tests
```bash
cargo test
```

### Specific Test
```bash
cargo test test_name
```

### Rapter Program Tests
```bash
rapter build tests/integration/test_file.rapt
rapter run
```

## Writing Tests

### Rust Tests
Add tests in `src/` files or in this directory:

```rust
#[test]
fn test_example() {
    // Your test code
}
```

### Rapter Tests
Create `.rapt` files with expected behavior documented in comments:

```rapter
// Expected output: Hello, World!
import src.std.io

fn main() -> int {
    io.println("Hello, World!");
    return 0;
}
```

## TODO

- [ ] Organize root directory test files into this structure
- [ ] Add comprehensive test coverage
- [ ] Set up automated testing CI/CD
- [ ] Add test documentation
