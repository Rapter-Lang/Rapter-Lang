# Rapter CLI Reference

The Rapter CLI provides a professional command-line interface for building and running Rapter programs.

## Installation

### Automatic Installation (Recommended)

Run the installer to add Rapter to your PATH:

```powershell
.\install.ps1
```

After installation, you can use `rapter` from anywhere!

### Manual Usage

If not installed, use the CLI from the Rapter directory:

```powershell
.\rapter.ps1 <command> [arguments]
```


## Commands

### `rapter build <file.rapt>`

Compiles a Rapter source file to C code and then to a native executable.

**Output:**
- `output.c` - Generated C code
- `output.exe` (Windows) / `output` (Linux/Mac) - Native executable

**Example:**
```bash
rapter build examples/hello.rapt
rapter build examples/fibonacci.rapt
```

**What it does:**
1. Compiles Rapter source to C using the Rust-based compiler
2. Compiles C code to executable using GCC
3. Reports any compilation errors with helpful messages

---

### `rapter run`

Executes the most recently built program.

**Example:**
```bash
rapter build examples/hello.rapt
rapter run
```

**Output:**
```
╭─────────────────────────────────────────╮
│ Running: output.exe                     │
╰─────────────────────────────────────────╯

Hello, World!
Welcome to Rapter!

╭─────────────────────────────────────────╮
│ Program finished                        │
│ Exit Code: 0                            │
╰─────────────────────────────────────────╯
```

---

### `rapter compile <file.rapt>`

Compiles Rapter source to C code only (no executable generated).

**Output:**
- `output.c` - Generated C code

**Example:**
```bash
rapter compile examples/structs_demo.rapt
# Now you can inspect output.c
```

**Use cases:**
- Debugging generated C code
- Understanding how Rapter compiles to C
- Integrating with custom build systems

---

### `rapter clean`

Removes all build artifacts.

**Removes:**
- `output.c` - Generated C code
- `output.exe` / `output` - Compiled executable
- `.build/` - Build cache directory

**Example:**
```bash
rapter clean
```

---

### `rapter help`

Shows all available commands and usage information.

**Example:**
```bash
rapter help
```


## Quick Start Workflow

### Your First Program

1. **Create a Rapter file** (`hello.rapt`):
```rapter
extern fn printf(format: *char, ...) -> int;

fn main() -> int {
    printf("Hello, Rapter!\n");
    return 0;
}
```

2. **Build it:**
```bash
rapter build hello.rapt
```

3. **Run it:**
```bash
rapter run
```

4. **Clean up:**
```bash
rapter clean
```

### Working with Examples

```bash
# Try the Fibonacci example
rapter build examples/fibonacci.rapt
rapter run

# Try the structs demonstration
rapter build examples/structs_demo.rapt
rapter run

# Clean up when done
rapter clean
```

## Advanced Usage

### Inspecting Generated C Code

```bash
# Compile to C only
rapter compile examples/hello.rapt

# View the generated C code
cat output.c
# or
notepad output.c
```

### Build and Run in One Line

```bash
rapter build examples/hello.rapt && rapter run
```

### Using Different Compilers

The CLI uses GCC by default. To use a different C compiler, modify `rapter.ps1`:

```powershell
# Change this line in rapter.ps1:
gcc output.c -o output.exe

# To use Clang instead:
clang output.c -o output.exe
```


## How It Works

### Build Process (`rapter build`)

1. **Lexical Analysis**
   - Tokenizes Rapter source code
   - Identifies keywords, operators, literals, identifiers

2. **Parsing**
   - Builds Abstract Syntax Tree (AST)
   - Validates syntax structure

3. **Semantic Analysis**
   - Type checking and inference
   - Variable resolution
   - Module imports

4. **Code Generation**
   - Generates C code from AST
   - Optimizes output
   - Adds runtime support

5. **Native Compilation**
   - Compiles C code with GCC
   - Links runtime library
   - Creates executable

### Run Process (`rapter run`)

- Executes the compiled binary
- Captures stdout/stderr
- Reports exit code
- Formats output with visual borders

### Compile Process (`rapter compile`)

- Performs steps 1-4 only
- Outputs `output.c`
- Useful for debugging and integration

## Troubleshooting

### Command Not Found

If `rapter` is not recognized:

1. **Run the installer:**
   ```powershell
   .\install.ps1
   ```

2. **Restart your terminal**

3. **Verify installation:**
   ```powershell
   rapter help
   ```

### GCC Not Found

The CLI requires GCC to compile C code to executables.

**Windows:**
- Install MinGW: http://www.mingw.org/
- Or install via Chocolatey: `choco install mingw`

**Linux:**
```bash
sudo apt-get install gcc  # Debian/Ubuntu
sudo yum install gcc      # Fedora/RHEL
```

**Mac:**
```bash
xcode-select --install
```

### Build Errors

If you encounter compilation errors:

1. **Check the error message** - Rapter provides detailed error reports
2. **Verify syntax** - Compare with examples in `examples/`
3. **Inspect generated C** - Use `rapter compile` to see output.c
4. **Report bugs** - https://github.com/Rapter-Lang/Rapter-Lang/issues

## File Locations

- **CLI Script**: `rapter.ps1` (PowerShell)
- **Windows Wrapper**: `rapter.bat`
- **Installer**: `install.ps1`
- **Compiler Binary**: `target/release/rapter-lang.exe`
- **Examples**: `examples/`
- **Standard Library**: `src/std/`
- **Build Artifacts**: `output.c`, `output.exe`


## Examples

### Basic Programs

**Hello World:**
```bash
rapter build examples/hello.rapt
rapter run
```

**Fibonacci Calculator:**
```bash
rapter build examples/fibonacci.rapt
rapter run
```

**Structs Demo:**
```bash
rapter build examples/structs_demo.rapt
rapter run
```

### Self-Hosting Compiler

The Rapter compiler is written in Rapter! Try compiling the bootstrap compiler:

```bash
# Build the self-hosting compiler
rapter build bootstrap/src/rapter_bootstrap_v1.rapt
rapter run
```

This demonstrates Rapter's self-hosting capability - a Rapter program that compiles Rapter programs! 🎉

## Tips & Best Practices

- **Use `rapter clean`** regularly to avoid stale build artifacts
- **Check `output.c`** with `rapter compile` to understand code generation
- **Read error messages carefully** - they include helpful suggestions
- **Start with examples** - All examples in `examples/` are tested and working
- **Report issues** - Help improve Rapter by reporting bugs

## Performance

- **Build times**: Typically < 1 second for small programs
- **Generated code**: Clean, readable C that compiles efficiently
- **Optimization**: Use GCC optimization flags in `rapter.ps1` for production builds

## See Also

- [QUICKSTART.md](QUICKSTART.md) - Get started in 5 minutes
- [README.md](README.md) - Language overview
- [CONTRIBUTING.md](CONTRIBUTING.md) - Development guidelines
- [examples/](examples/) - Working example programs

---

**Happy coding with Rapter!** 🚀
