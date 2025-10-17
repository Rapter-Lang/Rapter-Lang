# 🦖 Rapter CLI - Quick Start Guide

The Rapter CLI makes it easy to build and run Rapter programs!

## Installation

The CLI is already set up! Just use it from the Rapter directory.

## Usage

### Build a Program
```bash
rapter build <file.rapt>
```

Compiles your Rapter source code into an executable (`output.exe`).

**Example:**
```bash
rapter build hello.rapt
rapter build examples/bootstrap_proof.rapt
```

### Run the Last Built Program
```bash
rapter run
```

Runs the most recently built executable.

**Example workflow:**
```bash
rapter build hello.rapt
rapter run
```

### Compile to C Only
```bash
rapter compile <file.rapt>
```

Only generates C code (`output.c`) without building the executable.

### Clean Build Artifacts
```bash
rapter clean
```

Removes all generated files (`output.c`, `output.exe`, `.build/`).

## Complete Example

```bash
# 1. Build a Rapter program
.\rapter.ps1 build examples/bootstrap_proof.rapt

# 2. Run it
.\rapter.ps1 run

# 3. Clean up when done
.\rapter.ps1 clean
```

## Adding to PATH (Optional)

To use `rapter` from anywhere:

1. Add the Rapter directory to your system PATH
2. Then you can just type:
   ```bash
   rapter build myprogram.rapt
   rapter run
   ```

## What Happens Under the Hood

1. **`rapter build`**:
   - Runs `cargo run <file.rapt>` to compile Rapter → C
   - Runs `gcc output.c -o output.exe` to compile C → executable
   - Saves the build info for `rapter run`

2. **`rapter run`**:
   - Executes `output.exe`
   - Shows the program output with nice formatting
   - Reports the exit code

3. **`rapter compile`**:
   - Only does step 1 (Rapter → C)
   - Useful for debugging generated C code

4. **`rapter clean`**:
   - Removes all generated files
   - Gives you a fresh start

## Tips

- Use `.\rapter.ps1` in PowerShell if `rapter` alone doesn't work
- The CLI remembers your last build, so `rapter run` just works!
- Build artifacts are cleaned up automatically on each new build
- Check the generated `output.c` to see what C code was generated

## Self-Hosting Note

The Rapter compiler itself is written in Rapter! See:
- `bootstrap/rapter_bootstrap_v1.rapt` - Bootstrap compiler
- Run it with: `rapter build bootstrap/rapter_bootstrap_v1.rapt`

This demonstrates Rapter's self-hosting capability! 🎉

---

**Happy Raptering!** 🦖✨
