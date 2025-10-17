# Contributing to Rapter

Thank you for your interest in contributing to Rapter! This document provides guidelines and instructions for contributing.

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on the best outcome for the project
- Show empathy towards other community members

## Getting Started

### Prerequisites

- Rust (latest stable version)
- GCC or Clang compiler
- Git

### Setting Up Development Environment

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/rapter-lang.git
   cd rapter-lang
   ```

2. Install Rapter CLI:
   ```powershell
   .\install.ps1
   ```

3. Build the compiler:
   ```bash
   cargo build
   ```

4. Run tests:
   ```bash
   cargo test
   ```

## Development Workflow

### Building and Testing

```bash
# Build the Rust compiler
cargo build

# Run the compiler
cargo run -- examples/hello_cli.rapt

# Use the CLI
rapter build examples/hello_cli.rapt
rapter run

# Clean build artifacts
rapter clean
```

### Project Structure

```
rapter-lang/
├── src/              # Rust implementation of the Rapter compiler
│   ├── main.rs       # Entry point
│   ├── lexer.rs      # Tokenization
│   ├── parser.rs     # Parsing
│   ├── ast.rs        # Abstract Syntax Tree
│   ├── typechecker.rs # Type checking
│   └── codegen.rs    # Code generation
├── bootstrap/        # Self-hosting Rapter compiler (written in Rapter)
│   └── src/          # Bootstrap compiler components
├── examples/         # Example Rapter programs
├── tests/            # Test suites
└── lib/              # Standard library (Rapter code)
```

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in Issues
2. Create a new issue with:
   - Clear, descriptive title
   - Steps to reproduce
   - Expected vs actual behavior
   - Rapter version and OS
   - Code sample demonstrating the issue

### Suggesting Features

1. Check existing issues for similar proposals
2. Create a new issue with:
   - Clear use case
   - Proposed syntax/API
   - Examples of how it would work
   - Why it benefits Rapter

### Submitting Code

1. **Fork the repository**

2. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**
   - Follow the coding style (see below)
   - Add tests for new features
   - Update documentation as needed

4. **Test your changes**
   ```bash
   cargo test
   cargo clippy
   ```

5. **Commit your changes**
   ```bash
   git commit -m "Add feature: brief description"
   ```
   
   Commit message format:
   - `Add feature: description` for new features
   - `Fix: description` for bug fixes
   - `Refactor: description` for code improvements
   - `Docs: description` for documentation
   - `Test: description` for tests

6. **Push to your fork**
   ```bash
   git push origin feature/your-feature-name
   ```

7. **Create a Pull Request**
   - Provide clear description of changes
   - Reference any related issues
   - Include test results

## Coding Style

### Rust Code

- Follow standard Rust style guidelines
- Use `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Add comments for complex logic
- Keep functions focused and modular

### Rapter Code

- Use clear, descriptive variable names
- Follow consistent indentation (4 spaces)
- Add comments for non-obvious code
- Keep functions under 50 lines when possible

### Example:
```rapter
// Good: Clear function with documentation
fn calculate_distance(x1: float, y1: float, x2: float, y2: float) -> float {
    let dx = x2 - x1;
    let dy = y2 - y1;
    return sqrt(dx * dx + dy * dy);
}

// Avoid: Unclear naming
fn calc(a: float, b: float, c: float, d: float) -> float {
    return sqrt((c-a)*(c-a)+(d-b)*(d-b));
}
```

## Testing

### Writing Tests

- Add unit tests for new functions
- Add integration tests for new features
- Ensure all tests pass before submitting PR

### Test Locations

- Rust tests: `#[test]` in source files or `tests/` directory
- Rapter tests: `examples/` directory with expected output

## Documentation

- Update README.md for user-facing changes
- Update code comments for implementation changes
- Add examples for new features
- Update CHANGELOG.md

## Questions?

- Open a discussion in GitHub Discussions
- Ask in Issues with the `question` label

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
