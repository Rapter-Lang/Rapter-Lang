# Rapter Examples

This directory contains example programs demonstrating Rapter language features.

## Examples

### hello.rapt
Basic hello world program showing I/O operations.

```bash
rapter build examples/hello.rapt
rapter run
```

**Output:**
```
Hello, World!
Welcome to Rapter!
```

---

### fibonacci.rapt
Fibonacci sequence calculator demonstrating:
- Functions with parameters and return values
- Loops (`while`)
- Mutable variables
- Basic arithmetic

```bash
rapter build examples/fibonacci.rapt
rapter run
```

**Output:**
```
Fibonacci sequence:
F(0) = 0
F(1) = 1
F(2) = 1
F(3) = 2
...
```

---

### structs_demo.rapt
Struct demonstration showing:
- Struct definitions
- Nested structs
- Struct methods/functions
- Field access

```bash
rapter build examples/structs_demo.rapt
rapter run
```

**Output:**
```
Points:
Point(10, 20)
Point(5, 15)
Rectangle area: 1500
```


---

## Creating Your Own Examples

1. Create a `.rapt` file in this directory (or anywhere)
2. Write your Rapter code
3. Build and run:
   ```bash
   rapter build your_file.rapt
   rapter run
   ```

## Example Template

```rapter
import src.std.io

fn main() -> int {
    io.println("Your program here");
    return 0;
}
```

## Language Features Demonstrated

- **hello.rapt**: Basic I/O, imports, functions
- **fibonacci.rapt**: Loops, mutable variables, arithmetic, functions
- **structs_demo.rapt**: Structs, nested types, methods

## Need Help?

- See [QUICKSTART.md](../QUICKSTART.md) for language basics
- See [README.md](../README.md) for full language specification
- See [RAPTER_CLI.md](../RAPTER_CLI.md) for CLI usage
