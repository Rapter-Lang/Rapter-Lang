# Changelog

All notable changes to the Rapter Language project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Self-hosting capability - Rapter can now compile Rapter code!
- Bootstrap compiler v1.0 (100 lines of Rapter code)
- Professional CLI system (`rapter build`, `rapter run`, `rapter compile`, `rapter clean`)
- PowerShell installer script for PATH setup
- Complete compiler pipeline in Rapter (1,500+ lines):
  - Lexer V2 (240 lines)
  - Parser V2 (312 lines)
  - AST V2 (357 lines)
  - Type Checker (230 lines)
  - Code Generator V2 (180 lines)

### Changed
- Removed emojis from CLI output for professional appearance
- Updated build workflow to use `rapter` command
- Improved error messages and formatting

### Fixed
- Module-qualified function call type mismatches (workaround implemented)
- Variable name conflicts in bootstrap compiler

## [0.1.0] - 2025-10-17

### Added
- Initial Rust implementation of Rapter compiler
- Basic lexer and tokenization
- Parser with full syntax support
- Abstract Syntax Tree (AST) generation
- Type checking and semantic analysis
- C code generation (transpiler backend)
- Standard library foundations:
  - File I/O (`src.std.fs`)
  - Command-line arguments (`src.std.args`)
- Core language features:
  - Variables and constants
  - Functions with parameters and return values
  - Control flow (`if`, `else`, `while`, `for`, `break`, `continue`)
  - Structs and enums
  - Arrays (static and dynamic)
  - String operations
  - Type casting
  - Generics (Option<T>, Result<T,E>)
  - Pattern matching (`match` expressions)
  - Try operator (`?`)
- Example programs demonstrating all features
- Comprehensive test suite

### Documentation
- README.md with language overview
- Self-hosting victory documentation
- Bootstrap progress reports
- CLI usage guide (RAPTER_CLI.md)

## Project Milestones

### Self-Hosting Achievement (October 17, 2025)
- Rapter successfully compiles Rapter programs
- Bootstrap compiler generates valid C code
- Complete compiler toolchain operational
- CLI automation system complete

---

## Version History

- **0.1.0**: Initial release with self-hosting capability
- **Unreleased**: Ongoing development and feature additions

[Unreleased]: https://github.com/yourusername/rapter-lang/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/rapter-lang/releases/tag/v0.1.0
