# Rapter Quick Start Guide

Get up and running with Rapter in 5 minutes!

## Prerequisites

- **Rust** - For building the compiler (`cargo build`)
- **GCC** - For compiling generated C code
- **Git** - For cloning the repository
- **Windows PowerShell** / **Linux/Mac bash** - For CLI

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/Rapter-Lang/Rapter-Lang.git
cd Rapter-Lang
```

### 2. Build the Compiler

```bash
cargo build --release
```

This builds the Rust-based Rapter compiler.

### 3. Install the CLI

```powershell
.\install.ps1
```

This adds `rapter` to your PATH, allowing you to use it from anywhere.

**Note:** Restart your terminal after installation for PATH changes to take effect.

### 4. Verify Installation

```bash
rapter help
```

You should see the Rapter CLI commands.


## Your First Program

### 1. Create a File

Create `hello.rapt`:

```rapter
extern fn printf(format: *char, ...) -> int;

fn main() -> int {
    printf("Hello, Rapter!\n");
    return 0;
}
```

### 2. Build It

```bash
rapter build hello.rapt
```

This will:
- Compile Rapter → C (`output.c`)
- Compile C → executable (`output.exe` on Windows, `output` on Linux/Mac)

### 3. Run It

```bash
rapter run
```

Output:
```
╭─────────────────────────────────────────╮
│ Running: output.exe                     │
╰─────────────────────────────────────────╯

Hello, Rapter!

╭─────────────────────────────────────────╮
│ Program finished                        │
│ Exit Code: 0                            │
╰─────────────────────────────────────────╯
```

### 4. Try the Examples

```bash
# Fibonacci calculator
rapter build examples/fibonacci.rapt
rapter run

# Structs demonstration
rapter build examples/structs_demo.rapt
rapter run
```

## CLI Commands

### Build and Run
```bash
rapter build myprogram.rapt    # Compile to executable
rapter run                     # Run the last built program
```

### Compile Only
```bash
rapter compile myprogram.rapt  # Generate C code only (output.c)
```

### Clean Up
```bash
rapter clean                   # Remove build artifacts
```


## Project Structure

When starting a new project:

```
my-rapter-project/
├── main.rapt           # Your main program
├── utils.rapt          # Helper functions
└── lib/                # Library modules (optional)
```

For now, Rapter programs are simple - just create `.rapt` files and compile them!

## Example Programs

Try the included examples:

```bash
# Hello World
rapter build examples/hello.rapt
rapter run

# Fibonacci calculator
rapter build examples/fibonacci.rapt
rapter run

# Structs demonstration
rapter build examples/structs_demo.rapt
rapter run
```


## Language Basics

### Variables

```rapter
extern fn printf(format: *char, ...) -> int;

fn main() -> int {
    let x: int = 42;           // Explicit type
    let y = 3.14;              // Type inference (float)
    let mut counter = 0;       // Mutable variable
    
    counter = counter + 1;
    printf("Counter: %d\n", counter);
    
    return 0;
}
```

### Functions

```rapter
extern fn printf(format: *char, ...) -> int;

fn add(a: int, b: int) -> int {
    return a + b;
}

fn greet(name: *char) {
    printf("Hello, %s!\n", name);
}

fn main() -> int {
    let sum = add(10, 20);
    printf("Sum: %d\n", sum);
    
    greet("Rapter");
    return 0;
}
```

### Control Flow

```rapter
extern fn printf(format: *char, ...) -> int;

fn main() -> int {
    let x = 15;
    
    // If statement
    if x > 10 {
        printf("Big number\n");
    } else {
        printf("Small number\n");
    }
    
    // While loop
    let mut counter = 0;
    while counter < 5 {
        printf("Count: %d\n", counter);
        counter = counter + 1;
    }
    
    // For loop with range
    for i : 0..5 {
        printf("Loop: %d\n", i);
    }
    
    return 0;
}
```

### Structs

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

### Enums and Pattern Matching

```rapter
extern fn printf(format: *char, ...) -> int;

enum Color {
    Red = 0,
    Green = 1,
    Blue = 2
}

fn main() -> int {
    let color = Color::Red;
    
    match color {
        Color::Red => printf("Color is red\n"),
        Color::Green => printf("Color is green\n"),
        Color::Blue => printf("Color is blue\n"),
    }
    
    return 0;
}
```


## Advanced Features

### Generics (Built-in Types)

```rapter
extern fn printf(format: *char, ...) -> int;

fn process() -> Result<int, string> {
    // Result type provides error handling
    return Result::Ok(42);
}

fn main() -> int {
    let result = process();
    
    match result {
        Result::Ok(value) => printf("Success: %d\n", value),
        Result::Err(msg) => printf("Error: %s\n", msg),
    }
    
    return 0;
}
```

### Error Propagation

```rapter
fn might_fail() -> Result<int, string> {
    return Result::Err("Something went wrong");
}

fn process() -> Result<int, string> {
    let value = might_fail()?;  // Propagate error with ?
    return Result::Ok(value * 2);
}
```

### Pointers and Memory

```rapter
extern fn printf(format: *char, ...) -> int;
extern fn malloc(size: int) -> *int;
extern fn free(ptr: *int);

fn main() -> int {
    let ptr = malloc(4);  // Allocate 4 bytes
    *ptr = 42;            // Dereference and assign
    
    printf("Value: %d\n", *ptr);
    
    free(ptr);            // Free memory
    return 0;
}
```


## Next Steps

1. **Explore Examples** - Check out all programs in `examples/`
2. **Read the Docs** - See [README.md](README.md) for full language overview
3. **Try Self-Hosting** - Build the Rapter compiler with Rapter!
   ```bash
   rapter build bootstrap/src/rapter_bootstrap_v1.rapt
   rapter run
   ```
4. **Join Development** - See [CONTRIBUTING.md](CONTRIBUTING.md)

## CLI Reference

```bash
rapter build <file>    # Compile Rapter → C → executable
rapter run             # Run last built program
rapter compile <file>  # Compile Rapter → C only
rapter clean           # Remove build artifacts
rapter help            # Show all commands
```

See [RAPTER_CLI.md](RAPTER_CLI.md) for detailed CLI documentation.

## Troubleshooting

### "Command not found: rapter"
```bash
# Run the installer
.\install.ps1

# Restart your terminal
# Verify installation
rapter help
```

### "GCC not found" or compilation errors
```bash
# Check GCC is installed
gcc --version

# Windows: Install MinGW or use Chocolatey
choco install mingw

# Linux
sudo apt-get install gcc

# Mac
xcode-select --install
```

### Build errors
- Check error messages - they include helpful suggestions
- Compare your code with working examples in `examples/`
- Use `rapter compile` to inspect generated C code
- Report bugs: https://github.com/Rapter-Lang/Rapter-Lang/issues

---

**Happy coding with Rapter!** 🦖
