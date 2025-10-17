# Rapter Programming Language

> A statically-typed, compiled language with self-hosting capabilities

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/yourusername/rapter-lang)

## Self-Hosting Achievement!

**Rapter can now compile Rapter programs!** The bootstrap compiler (written in Rapter) successfully compiles Rapter source code.

### What's Working
- ✅ **Self-hosting compiler** - 100 lines of Rapter code that compiles Rapter
- ✅ **Full compiler pipeline** - 1,500+ lines of compiler components in Rapter
- ✅ **Professional CLI** - `rapter build`, `rapter run`, `rapter compile`, `rapter clean`
- ✅ **Complete language** - Functions, structs, generics, pattern matching, error handling

### Quick Start

```bash
# Install Rapter CLI
.\install.ps1

# Build a program
rapter build examples/hello_cli.rapt

# Run it
rapter run
```

See [RAPTER_CLI.md](RAPTER_CLI.md) for detailed CLI usage.

---

## Overview
Rapter is a statically-typed, compiled programming language designed to bridge the gap between low-level control and high-level expressiveness.

## Core Language Features

### Type System
- **Statically typed** with strong type inference
- Primitive types: `int`, `float`, `bool`, `char`, `string`
- Complex types: arrays, structs, enums, pointers
- Optional type annotations (inferred when omitted)
- Generics support for reusable code

### Syntax Philosophy
- Clean and intuitive, drawing inspiration from modern languages
- Minimal boilerplate while maintaining explicitness where it matters
- Familiar control structures with consistent formatting
- Support for both functional and imperative paradigms
- **Statement termination**: Newline-based (like Python/Swift) with optional explicit continuation
  - Single statement per line (default)
  - Use `\` for line continuation when needed
  - Automatic semicolon insertion rules for edge cases

### Language Level
- **Medium-level positioning**
  - Low-level features: manual memory management, pointers, inline assembly
  - High-level features: closures, pattern matching, smart abstractions
  - Zero-cost abstractions where possible
- **Memory Management**
  - Manual allocation/deallocation (no garbage collection)
  - Stack-based allocation by default
  - Heap allocation via `new` operator
  - Deallocation via `delete` operator
  - Ownership system inspired by Rust (borrowing, moving)
  - Compile-time memory safety checks
  - RAII (Resource Acquisition Is Initialization) patterns

### Object-Oriented Features
- Classes with inheritance (single inheritance, multiple interfaces)
- Access modifiers: `public`, `private`, `protected`
- Methods, constructors, destructors
- Abstract classes and interfaces
- Operator overloading

### Error Handling System
Rapter's exceptional error messages include:
- **Precise location**: Line and column numbers
- **Visual indicators**: Colored output with pointer arrows (^)
- **Categorized errors**: `[E###]` for errors, `[W###]` for warnings
- **Context display**: Shows surrounding code with highlighting
- **Smart suggestions**: "Did you mean X?" for common mistakes
- **Error categories**:
  - `E001-E099`: Syntax errors
  - `E100-E199`: Type errors
  - `E200-E299`: Memory errors
  - `W001-W099`: Style warnings
  - `W100-W199`: Performance warnings

Example error output:
```
[E042] Type mismatch at src/main.rapt:12:18
   |
12 | let x: int = "hello";
   |              ^^^^^^^ expected type 'int', found 'string'
   |
   = help: Convert string to int using parse() method
```

## Version 0.1.0 Goals

### Core Components
- ✓ **Lexer**: Tokenize source code
- ✓ **Parser**: Build parse tree from tokens
- ✓ **AST Generator**: Create abstract syntax tree
- ✓ **Semantic Analyzer**: Type checking and validation
- ✓ **Error Handler**: Comprehensive error reporting system
- ✓ **Code Generator**: Emit target code (LLVM IR or native)
- ✓ **Standard Library (minimal)**: Basic I/O, string operations

### Language Features (v0.1.0)
- [x] File extension: `.rapt`
- [x] Variables and constants
- [x] Basic data types
- [x] Functions (with parameters and return values)
- [x] Control flow: `if`, `else`, `while`, `for`
- [x] Basic structs
- [x] Comments (single-line `//` and multi-line `/* */`)
- [x] Basic operators (arithmetic, logical, comparison)
- [x] Turing completeness verified

### Compiler Features
- Command-line interface: `rapter build`, `rapter run`
- Optimization levels: `-O0`, `-O1`, `-O2`, `-O3`
- Debug symbols generation
- Dependency management (basic)

## Syntax Examples

### Hello World
```rapter
import std.io

fn main() -> int {
    io.println("Hello, Rapter!")
    return 0
}
```

### Variables and Types
```rapter
fn example() {
    let x: int = 42           // explicit type
    let y = 3.14              // inferred as float
    const PI: float = 3.14159 // constant
    let mut counter = 0       // mutable variable
}
```

### Functions
```rapter
fn add(a: int, b: int) -> int {
    return a + b
}

fn greet(name: string) {
    io.println("Hello, " + name)
}

// Multi-line expressions use \ for continuation
fn complex_calculation(x: int) -> int {
    return x * 2 + \
           x * 3 + \
           x * 4
}
```

### Memory Management Examples
```rapter
fn memory_demo() {
    // Stack allocation (automatic cleanup)
    let x = 42
    
    // Heap allocation
    let ptr = new int(100)
    io.println(*ptr)
    delete ptr  // Manual cleanup required
    
    // Arrays on heap
    let arr = new [int; 10]
    arr[0] = 5
    delete arr
}

// Ownership and borrowing
fn ownership_example() {
    let data = new DataStruct()
    
    process_data(data)      // moves ownership to function
    // data is no longer valid here
    
    let data2 = new DataStruct()
    borrow_data(&data2)     // borrows reference
    // data2 is still valid here
    
    delete data2
}

fn process_data(owned: DataStruct) {
    // takes ownership, will be cleaned up when function exits
}

fn borrow_data(borrowed: &DataStruct) {
    // only borrows, doesn't take ownership
}
```

### Structs
```rapter
struct Point {
    x: float,
    y: float
}

fn distance(p1: Point, p2: Point) -> float {
    let dx = p2.x - p1.x
    let dy = p2.y - p1.y
    return sqrt(dx * dx + dy * dy)
}
```

### Classes (Basic)
```rapter
class Rectangle {
    private width: float
    private height: float
    
    public fn new(w: float, h: float) -> Rectangle {
        return Rectangle { width: w, height: h }
    }
    
    public fn area() -> float {
        return this.width * this.height
    }
}
```

### Statement Termination Rules
```rapter
// Single statements - newline terminates
let x = 5
let y = 10

// Expressions that clearly continue don't need \
let sum = x + y +
          z + w

// But for clarity, \ can be used
let result = calculate_value() \
             + another_value()

// Blocks don't need continuation
if x > 5 {
    do_something()
    do_another_thing()
}

// Chained method calls naturally continue
data.filter(predicate)
    .map(transform)
    .collect()
```

## Future Roadmap (Post v0.1.0)

### Version 0.2.0
- Full OOP implementation (inheritance, polymorphism)
- Pattern matching
- Advanced error recovery in parser
- Module system
- Package manager

### Version 0.3.0
- Generics
- Traits/Interfaces
- Async/await support
- Foreign Function Interface (FFI)

### Version 1.0.0
- Production-ready standard library
- Comprehensive testing framework
- Documentation generator
- IDE/LSP support
- Stable API

## Build System
- Project structure: `src/`, `tests/`, `docs/`
- Configuration file: `rapter.toml`
- Build artifacts: `build/` directory

## Development Priorities
1. Implement robust lexer and parser
2. Design comprehensive test suite
3. Create error message system early
4. Document language features as they're implemented
5. Establish coding standards and conventions

## Community & Philosophy
- Open source development
- Focus on developer experience
- Clear, comprehensive documentation
- Regular release cycle
- Community-driven feature requests