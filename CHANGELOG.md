# Changelog

All notable changes to the Rapter Programming Language will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned for 0.2.0
- String improvements (methods, interpolation, multi-line strings)
- Array enhancements (methods, slices, better operations)
- Improved error messages with suggestions
- Compound assignment operators (+=, -=, *=, /=)
- Loop labels for break/continue
- REPL (interactive interpreter)

## [0.1.0] - 2025-10-17

### 🎉 Self-Hosting Milestone Achieved!

Rapter successfully compiles itself! This release marks the achievement of a fully functional, self-hosting compiler.

### Compiler Implementation

**Rust-based Compiler** (6,603 lines across 11 files):
- `lexer.rs` - Tokenization with comprehensive token types
- `parser.rs` - Full syntax parsing with error recovery
- `ast.rs` - Complete Abstract Syntax Tree definitions
- `semantic.rs` - Type checking and semantic analysis
- `codegen.rs` - C code generation
- `modules.rs` - Module system and import resolution
- `error.rs` - Rich error reporting with suggestions
- `builtins.rs` - Built-in generic types (Option, Result)
- `intrinsics.rs` - Standard C library intrinsics

**Bootstrap Compiler** (6,845 lines of Rapter code across 25 files):
- Self-hosting compiler written in Rapter
- Compiles Rapter source to C
- Demonstrates language capabilities
- Located in `bootstrap/src/`

**Standard Library** (203 lines across 5 modules):
- `io.rapt` - Input/output operations
- `str.rapt` - String utilities
- `char.rapt` - Character utilities
- `fs.rapt` - Filesystem operations
- `args.rapt` - Command-line argument access

### CLI System

- **PowerShell CLI** (`rapter.ps1`) - ~200 lines
- **Windows Wrapper** (`rapter.bat`)
- **PATH Installer** (`install.ps1`)
- **Commands**: `build`, `run`, `compile`, `clean`, `help`
- Professional output formatting
- Cross-platform support (Windows/Linux/Mac ready)

### Language Features

**Core Features:**
- Variables (`let`, `const`, `mut`)
- Functions with parameters and return types
- Type inference
- Comments (single-line `//` and multi-line `/* */`)

**Data Types:**
- Primitives: `int`, `float`, `bool`, `char`, `string`
- Arrays: Static `[T]` and dynamic `DynamicArray<T>`
- Pointers: `*T` with dereference and address-of operators
- Structs with field access
- Enums with numeric values

**Control Flow:**
- `if`/`else` conditionals
- `while` loops
- `for` loops with range syntax (`0..n`)
- `break` and `continue`
- Ternary operator (`condition ? true_val : false_val`)

**Advanced Features:**
- **Pattern Matching**: `match` expressions with enum and literal patterns
- **Generics**: Built-in `Option<T>` and `Result<T, E>` types
- **Error Propagation**: Try operator (`?`) for Result and Option
- **Module System**: Import/export with qualified names
- **Type Casting**: `as` operator for explicit conversions
- **Operator Overloading**: Binary and unary operators
- **Extern Functions**: C interop via `extern fn`

**Type System:**
- Static type checking
- Type inference
- Generic types (Option, Result)
- Pointer types
- Struct and enum types
- Type compatibility checking

### Examples

Three production-ready example programs:
- `examples/hello.rapt` - Hello World
- `examples/fibonacci.rapt` - Fibonacci calculator with loops
- `examples/structs_demo.rapt` - Struct definition and usage

### Documentation

- **README.md** - Comprehensive language overview
- **QUICKSTART.md** - 5-minute getting started guide
- **RAPTER_CLI.md** - Complete CLI reference
- **CONTRIBUTING.md** - Development guidelines
- **CHANGELOG.md** - This file
- **LICENSE** - MIT license

### Project Organization

- Clean `.gitignore` configuration
- Organized directory structure
- Separated development from public files
- Complete Cargo.toml metadata

### Statistics

- **Total Rapter Code**: 7,048 lines
- **Bootstrap Compiler**: 6,845 lines (25 files)
- **Standard Library**: 203 lines (5 modules)
- **Rust Compiler**: 6,603 lines (11 files)
- **Examples**: 3 working programs

### Known Limitations

- Module imports require workaround (use `extern fn` directly)
- `pub` keyword not yet supported in modules
- String operations limited (improvements planned for 0.2.0)
- No closure support yet
- No traits/interfaces yet

---

## Version History

- **0.1.0** (2025-10-17): Initial release - Self-hosting compiler achieved! 🎉

[Unreleased]: https://github.com/Rapter-Lang/Rapter-Lang/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Rapter-Lang/Rapter-Lang/releases/tag/v0.1.0
