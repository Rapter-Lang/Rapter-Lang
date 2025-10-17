# Rapter Programming Language

> A modern, statically-typed systems programming language that compiles itself

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/Rapter-Lang/Rapter-Lang)
[![Self-Hosting](https://img.shields.io/badge/self--hosting-✓-brightgreen.svg)](https://github.com/Rapter-Lang/Rapter-Lang)

## Self-Hosting Compiler Achieved!

**Rapter successfully compiles itself!** This milestone represents a fully functional compiler toolchain written in Rapter that can compile Rapter source code.

### What's Working Now
- ✅ **Self-hosting compiler** - 100-line bootstrap compiler written in pure Rapter
- ✅ **Full compiler pipeline** - 6,800+ lines of Rapter compiler code (lexer, parser, semantic analyzer, codegen)
- ✅ **Professional CLI** - Complete command-line interface with `build`, `run`, `compile`, `clean` commands
- ✅ **Rich type system** - Structs, enums, generics, pointers, arrays, pattern matching
- ✅ **Advanced features** - Error propagation (`?` operator), ternary operators, match expressions
- ✅ **Module system** - Import/export with qualified names
- ✅ **Standard library** - I/O, string utilities, filesystem, command-line args


## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/Rapter-Lang/Rapter-Lang.git
cd Rapter-Lang

# Install the Rapter CLI (adds to PATH)
.\install.ps1

# Build the Rust-based compiler
cargo build --release
```

### Your First Program

Create `hello.rapt`:
```rapter
extern fn printf(format: *char, ...) -> int;

fn main() -> int {
    printf("Hello, World!\n");
    return 0;
}
```

Build and run:
```bash
rapter build hello.rapt
rapter run
```

### CLI Commands

- `rapter build <file.rapt>` - Compile a Rapter program to C and executable
- `rapter run` - Run the most recently compiled program
- `rapter compile <file.rapt>` - Compile to C only (no executable)
- `rapter clean` - Remove build artifacts
- `rapter help` - Show all available commands

See [RAPTER_CLI.md](RAPTER_CLI.md) for detailed CLI documentation.

---

## Language Overview

Rapter is a modern systems programming language designed for clarity, safety, and performance. It compiles to C and then to native machine code.

### Design Philosophy
- **Explicit but concise** - Clear syntax without excessive boilerplate
- **Systems-level control** - Manual memory management, pointers, low-level operations
- **Modern conveniences** - Type inference, pattern matching, error handling operators
- **Self-hosting** - The compiler is written in Rapter itself

### Core Features

#### Type System
- **Static typing** with powerful type inference
- **Primitive types**: `int`, `float`, `bool`, `char`, `string`
- **Complex types**: Arrays, pointers, structs, enums
- **Generics**: Built-in support for `Option<T>` and `Result<T, E>`
- **Type annotations**: Optional (can be inferred in most cases)

#### Memory Management
- **Manual control** - Stack and heap allocation via `new`/`delete`
- **Pointers** - Full pointer arithmetic and dereferencing
- **No garbage collection** - Predictable performance
- **Safety features** - Compile-time checks for common errors

#### Pattern Matching
```rapter
let result: Result<int, string> = might_fail();

match result {
    Result::Ok(value) => printf("Success: %d\n", value),
    Result::Err(msg) => printf("Error: %s\n", msg),
}
```

#### Error Handling
```rapter
fn process() -> Result<int, string> {
    let value = might_fail()?;  // Propagate errors with ?
    return Result::Ok(value * 2);
}
```


## Code Examples

### Hello World
```rapter
extern fn printf(format: *char, ...) -> int;

fn main() -> int {
    printf("Hello, Rapter!\n");
    return 0;
}
```

### Variables and Control Flow
```rapter
extern fn printf(format: *char, ...) -> int;

fn fibonacci(n: int) -> int {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn main() -> int {
    let n = 10;
    
    for i : 0..n {
        let fib = fibonacci(i);
        printf("fib(%d) = %d\n", i, fib);
    }
    
    return 0;
}
```

### Structs and Memory Management
```rapter
extern fn printf(format: *char, ...) -> int;

struct Point {
    x: int,
    y: int
}

struct Rectangle {
    top_left: Point,
    width: int,
    height: int
}

fn area(rect: Rectangle) -> int {
    return rect.width * rect.height;
}

fn main() -> int {
    let rect = Rectangle {
        top_left: Point { x: 0, y: 0 },
        width: 10,
        height: 20
    };
    
    let a = area(rect);
    printf("Area: %d\n", a);
    
    return 0;
}
```

See the [examples/](examples/) directory for more complete programs.


## 🏗️ Project Structure

```
RapterLang/
├── src/                    # Rust-based compiler implementation
│   ├── main.rs            # Compiler entry point
│   ├── lexer.rs           # Tokenization
│   ├── parser.rs          # Syntax analysis
│   ├── semantic.rs        # Type checking & validation
│   ├── codegen.rs         # C code generation
│   ├── modules.rs         # Module resolution
│   └── std/               # Standard library (.rapt files)
│       ├── io.rapt        # I/O operations
│       ├── str.rapt       # String utilities
│       ├── char.rapt      # Character utilities
│       ├── fs.rapt        # Filesystem operations
│       └── args.rapt      # Command-line arguments
├── bootstrap/             # Self-hosting compiler
│   └── src/               # Rapter compiler written in Rapter
│       ├── rapter_bootstrap_v1.rapt
│       └── (25 compiler components)
├── lib/                   # Rapter runtime library
│   └── runtime.c          # C runtime support
├── examples/              # Example programs
│   ├── hello.rapt
│   ├── fibonacci.rapt
│   └── structs_demo.rapt
├── rapter.ps1            # CLI script (PowerShell)
├── rapter.bat            # CLI wrapper (Windows)
├── install.ps1           # PATH installer
└── Cargo.toml            # Rust project config
```

## Statistics

- **Rust compiler**: 6,603 lines (11 files)
- **Rapter bootstrap compiler**: 6,845 lines (25 files)  
- **Standard library**: 203 lines (5 modules)
- **Total self-hosted code**: 7,048 lines of Rapter

## Development

### Build from Source

```bash
# Clone the repository
git clone https://github.com/Rapter-Lang/Rapter-Lang.git
cd Rapter-Lang

# Build the Rust compiler
cargo build --release

# The compiler binary will be at:
# target/release/rapter-lang.exe (Windows)
# target/release/rapter-lang (Linux/Mac)
```

### Running Tests

```bash
# Run Rust tests
cargo test

# Test the compiler with examples
rapter build examples/hello.rapt
rapter run
```

### Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

- Report bugs via [GitHub Issues](https://github.com/Rapter-Lang/Rapter-Lang/issues)
- Submit pull requests for features or fixes
- Join discussions about language design

## Documentation

- **[QUICKSTART.md](QUICKSTART.md)** - Get started in 5 minutes
- **[RAPTER_CLI.md](RAPTER_CLI.md)** - Complete CLI reference
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Development guidelines
- **[CHANGELOG.md](CHANGELOG.md)** - Version history

## Roadmap

### Current Version: 0.1.0
- [x] Self-hosting compiler
- [x] Complete type system
- [x] Pattern matching
- [x] Generics (Option, Result)
- [x] Error propagation operator (`?`)
- [x] Module system
- [x] Standard library basics

### Version 0.2.0 (Next)
- [ ] **String improvements** - Methods, interpolation, better operations
- [ ] **Arrays enhancement** - Dynamic arrays, slices, methods
- [ ] **Error messages** - Better diagnostics with suggestions
- [ ] **More operators** - Compound assignment (+=, -=, etc.)
- [ ] **Loops** - Break/continue with labels
- [ ] **REPL** - Interactive interpreter

### Version 0.3.0
- [ ] **Closures** - Anonymous functions with captures
- [ ] **Traits** - Interface-like type system
- [ ] **Macros** - Compile-time code generation
- [ ] **Standard library expansion** - Collections, networking, etc.

### Version 1.0.0
- [ ] **Production ready** - Stable API and ABI
- [ ] **LSP server** - IDE/editor integration
- [ ] **Package manager** - Dependency management
- [ ] **Comprehensive docs** - Language reference manual

## Community & Support

- **GitHub**: [Rapter-Lang/Rapter-Lang](https://github.com/Rapter-Lang/Rapter-Lang)
- **Issues**: Report bugs or request features
- **Discussions**: Ask questions and share ideas

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Rapter draws inspiration from many great languages:
- **Rust** - Ownership concepts and error handling
- **C** - Systems programming philosophy
- **Swift** - Clean, modern syntax
- **Go** - Simplicity and pragmatism

---

**Built with by the Rapter community**

*Star ⭐ this repo if you find it interesting!*