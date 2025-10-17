# Rapter Quick Start Guide

Get up and running with Rapter in 5 minutes!

## Prerequisites

- Windows (PowerShell)
- Rust (for building the compiler)
- GCC (for C compilation)

## Installation

### 1. Install Rapter CLI

Run the installer to add Rapter to your PATH:

```powershell
.\install.ps1
```

This allows you to use `rapter` from anywhere in your terminal.

**Note:** You may need to restart your terminal for PATH changes to take effect in new windows.

### 2. Verify Installation

```bash
rapter help
```

You should see the Rapter CLI help menu.

## Your First Program

### 1. Create a File

Create `hello.rapt`:

```rapter
import src.std.io

fn main() -> int {
    io.println("Hello, Rapter!");
    return 0;
}
```

### 2. Build It

```bash
rapter build hello.rapt
```

This will:
- Compile your Rapter code to C (`output.c`)
- Compile the C code to an executable (`output.exe`)

### 3. Run It

```bash
rapter run
```

Output:
```
Hello, Rapter!
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

When starting a new project, organize it like this:

```
my-rapter-project/
├── src/
│   ├── main.rapt       # Your main program
│   └── utils.rapt      # Helper functions
├── examples/
│   └── demo.rapt       # Example programs
└── rapter.toml         # Project configuration (future)
```

## Example Programs

Try the included examples:

```bash
# Simple hello world
rapter build examples/hello_cli.rapt
rapter run

# Compiler demonstration
rapter build examples/bootstrap_proof.rapt
rapter run
```

## Language Basics

### Variables
```rapter
let x: int = 42;           // Explicit type
let y = 3.14;              // Type inference
let mut counter = 0;       // Mutable variable
```

### Functions
```rapter
fn add(a: int, b: int) -> int {
    return a + b;
}

fn greet(name: string) {
    io.println("Hello, " + name);
}
```

### Control Flow
```rapter
// If statement
if x > 10 {
    io.println("Big number");
} else {
    io.println("Small number");
}

// While loop
while counter < 10 {
    counter = counter + 1;
}

// For loop
for i in 0..10 {
    io.println(i);
}
```

### Structs
```rapter
struct Point {
    x: float,
    y: float
}

let p = Point { x: 1.0, y: 2.0 };
io.println(p.x);
```

### Error Handling
```rapter
import src.std.result

fn divide(a: int, b: int) -> Result<int, string> {
    if b == 0 {
        return Result::Err("Division by zero");
    }
    return Result::Ok(a / b);
}

fn main() -> int {
    let result = divide(10, 2)?;  // Try operator
    io.println(result);
    return 0;
}
```

## Common Patterns

### Reading Command Line Arguments
```rapter
import src.std.args

fn main() -> int {
    let count = args.argc();
    
    for i in 0..count {
        let arg = args.argv(i);
        io.println(arg);
    }
    
    return 0;
}
```

### File I/O
```rapter
import src.std.fs

fn main() -> int {
    // Write to file
    fs.write_all("output.txt", "Hello, file!");
    
    // Read from file
    let content = fs.read_all("output.txt");
    io.println(content);
    
    return 0;
}
```

## Next Steps

1. **Explore Examples**: Check out `examples/` directory
2. **Read Documentation**: See `README.md` for full language spec
3. **Try Self-Hosting**: Build the bootstrap compiler!
   ```bash
   rapter build bootstrap/rapter_bootstrap_v1.rapt
   ```
4. **Contribute**: See `CONTRIBUTING.md` for guidelines

## Troubleshooting

### Command not found: rapter
- Make sure you ran `.\install.ps1`
- Restart your terminal
- Check that `C:\Users\therr\Desktop\RapterLang` is in your PATH

### Build fails with GCC error
- Make sure GCC is installed and in your PATH
- Try: `gcc --version`

### Import not found
- Standard library is in `lib/src/std/`
- Import with: `import src.std.modulename`

## Getting Help

- Run `rapter help` for CLI usage
- Check `RAPTER_CLI.md` for detailed CLI docs
- Read `README.md` for language specification
- See examples in `examples/` directory

## Project Commands

```bash
.\status.ps1     # View project statistics
.\cleanup.ps1    # Organize and clean project
.\install.ps1    # Install/reinstall CLI
```

---

**Happy coding with Rapter!** 🦖
