# Rapter Examples

This directory contains working example programs demonstrating Rapter language features.

## Running Examples

```bash
# Build any example
rapter build examples/<filename>.rapt

# Run it
rapter run

# Clean up
rapter clean
```

## Available Examples

### hello.rapt
**Basic hello world program**

Demonstrates:
- `extern fn` declarations for C interop
- `printf` for output
- Function definitions with return types
- String literals

```bash
rapter build examples/hello.rapt
rapter run
```

**Output:**
```
Hello, World!
Welcome to Rapter!
```

**Code snippet:**
```rapter
extern fn printf(format: *char, ...) -> int;

fn main() -> int {
    printf("Hello, World!\n");
    printf("Welcome to Rapter!\n");
    return 0;
}
```

---

### fibonacci.rapt
**Fibonacci sequence calculator**

Demonstrates:
- Recursive functions
- Conditional logic (`if`/`else`)
- For loops with ranges (`0..n`)
- Integer arithmetic
- Function calls with arguments

```bash
rapter build examples/fibonacci.rapt
rapter run
```

**Output:**
```
Fibonacci sequence:
fib(0) = 0
fib(1) = 1
fib(2) = 1
fib(3) = 2
fib(4) = 3
fib(5) = 5
fib(6) = 8
fib(7) = 13
fib(8) = 21
fib(9) = 34
```

**Key features:**
- Recursion for mathematical computation
- Loop iteration over ranges
- Formatted output with multiple arguments

---

### structs_demo.rapt
**Struct demonstration**

Demonstrates:
- Struct definitions with multiple fields
- Nested structs (Point inside Rectangle)
- Struct initialization with field syntax
- Functions taking struct parameters
- Field access with dot notation
- Computing with struct data

```bash
rapter build examples/structs_demo.rapt
rapter run
```

**Output:**
```
Rectangle dimensions:
  Top-left: (0, 0)
  Width: 10
  Height: 20
Area: 200
```

**Code snippet:**
```rapter
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
```

---

## Language Features Covered

### All Examples Demonstrate:
- ✅ Function definitions
- ✅ Return types
- ✅ Integer types
- ✅ C interop via `extern fn`
- ✅ Formatted output

### hello.rapt
- String literals
- Variadic functions (printf)
- Basic program structure

### fibonacci.rapt  
- Recursion
- Control flow (if/else)
- For loops
- Range syntax (0..n)
- Multiple function calls

### structs_demo.rapt
- Struct definitions
- Nested structures
- Struct initialization
- Field access
- Passing structs to functions


## Creating Your Own Programs

### Basic Template

```rapter
extern fn printf(format: *char, ...) -> int;

fn main() -> int {
    printf("Hello from my program!\n");
    return 0;
}
```

### Save and Run

1. **Create a file**: `my_program.rapt`
2. **Write your code** (use the template above)
3. **Build**: `rapter build my_program.rapt`
4. **Run**: `rapter run`

### Tips for Writing Rapter Programs

- **Start simple** - Begin with basic printf output
- **Use the examples** - Copy and modify existing examples
- **Check syntax** - Rapter has helpful error messages
- **Experiment** - Try different features from the language

## Common Patterns

### Declaring External Functions

```rapter
extern fn printf(format: *char, ...) -> int;
extern fn malloc(size: int) -> *int;
extern fn free(ptr: *int);
```

### Creating Functions

```rapter
fn add(a: int, b: int) -> int {
    return a + b;
}

fn greet() {
    printf("Hello!\n");
}
```

### Using Loops

```rapter
// For loop with range
for i : 0..10 {
    printf("%d\n", i);
}

// While loop
let mut counter = 0;
while counter < 10 {
    printf("%d\n", counter);
    counter = counter + 1;
}
```

### Working with Structs

```rapter
struct Person {
    age: int,
    height: int
}

fn main() -> int {
    let p = Person { age: 25, height: 180 };
    printf("Age: %d, Height: %d\n", p.age, p.height);
    return 0;
}
```

## More Examples Coming Soon

Future examples will cover:
- Pattern matching
- Enums
- Generic types (Option, Result)
- Error handling with `?` operator
- Dynamic arrays
- More advanced struct usage

## Getting Help

- **[QUICKSTART.md](../QUICKSTART.md)** - Language basics and quick start
- **[README.md](../README.md)** - Full language specification
- **[RAPTER_CLI.md](../RAPTER_CLI.md)** - CLI commands and usage

---

**Happy coding with Rapter!** 🚀
