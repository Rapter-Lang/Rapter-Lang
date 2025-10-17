# 🚀 Bootstrap Compiler - Tomorrow's Plan (Final 15%)

## Mission: Complete Self-Hosting Bootstrap Compiler

**Goal:** Take the bootstrap compiler from 85% → 100% and achieve self-hosting!

**Estimated Time:** 3-4 hours of focused work

---

## Phase 1: Complete Code Generator (Highest Priority)

### Current Status
- ✅ Basic framework exists (`codegen_v2.rapt` - 180 lines)
- ✅ Struct generation works
- ✅ Function signatures work
- ⚠️ Expression generation incomplete
- ⚠️ Statement generation incomplete

### What Needs to Be Done

#### 1.1 Expression Generation (Priority: CRITICAL)

**Goal:** Generate C code for all Rapter expressions

**Expressions to Handle:**
```rapter
// Integer literals - DONE
42 → "42"

// String literals - DONE  
"hello" → "\"hello\""

// Identifiers/Variables - DONE
x → "x"

// Binary operations - TODO
x + y → "(x + y)"
a * b + c → "((a * b) + c)"

// Function calls - TODO
foo(x, y) → "foo(x, y)"
printf("hello") → "printf(\"hello\")"

// Array access - TODO
arr[i] → "arr[i]"

// Struct field access - TODO
point.x → "point.x"

// Pointer dereference - TODO
*ptr → "(*ptr)"

// Address-of - TODO
&var → "(&var)"

// Type casts - TODO
x as int → "((int)(x))"
```

**Implementation Plan:**
```rapter
// Add to codegen_v2.rapt:

export fn gen_expression(expr: *ast.Expression) -> *char {
    let result = malloc(2048) as *char;
    
    if expr.is_int_literal == 1 {
        return gen_int_literal(expr.int_value);
    } else {
        if expr.is_string_literal == 1 {
            return gen_string_literal(expr.string_value);
        } else {
            if expr.is_identifier == 1 {
                return gen_identifier(expr.name);
            } else {
                if expr.is_binary_op == 1 {
                    let left = gen_expression(expr.left);
                    let right = gen_expression(expr.right);
                    return gen_binary_op(expr.op, left, right);
                } else {
                    if expr.is_function_call == 1 {
                        return gen_function_call_expr(expr);
                    } else {
                        // ... handle other expressions
                        return "/* TODO: expression */";
                    }
                }
            }
        }
    }
}

fn gen_function_call_expr(expr: *ast.Expression) -> *char {
    let result = malloc(2048) as *char;
    strcpy(result, expr.func_name);
    strcat(result, "(");
    
    let i = 0;
    while i < expr.args.length() {
        if i > 0 {
            strcat(result, ", ");
        }
        let arg = expr.args.get(i);
        let arg_code = gen_expression(arg);
        strcat(result, arg_code);
        i = i + 1;
    }
    
    strcat(result, ")");
    return result;
}
```

**Estimated Time:** 1-2 hours

---

#### 1.2 Statement Generation (Priority: CRITICAL)

**Goal:** Generate C code for all Rapter statements

**Statements to Handle:**
```rapter
// Return - PARTIAL
return 42; → "    return 42;"
return x + y; → "    return (x + y);"

// Let declarations - TODO
let x: int = 42; → "    int x = 42;"
let mut y = 10; → "    int y = 10;"

// Assignment - TODO
x = 5; → "    x = 5;"
arr[i] = 10; → "    arr[i] = 10;"

// If statements - TODO
if x > 5 { ... } → "    if (x > 5) { ... }"
if x > 5 { ... } else { ... } → with else block

// While loops - TODO
while x < 10 { ... } → "    while (x < 10) { ... }"

// For loops - TODO (can translate to while)

// Expression statements - TODO
foo(); → "    foo();"

// Match statements - TODO (translate to if/else chain or switch)
```

**Implementation Plan:**
```rapter
export fn gen_statement(sb: *StringBuilder, stmt: ast.Statement, indent: int) {
    let indent_str = make_indent(indent);
    
    if stmt.is_return == 1 {
        sb_append(sb, indent_str);
        sb_append(sb, "return ");
        if stmt.return_value != 0 as *void {
            let expr_code = gen_expression(stmt.return_value);
            sb_append(sb, expr_code);
        }
        sb_append_line(sb, ";");
    } else {
        if stmt.is_let == 1 {
            gen_let_declaration(sb, stmt.let_stmt, indent);
        } else {
            if stmt.is_if == 1 {
                gen_if_statement(sb, stmt.if_stmt, indent);
            } else {
                if stmt.is_while == 1 {
                    gen_while_statement(sb, stmt.while_stmt, indent);
                } else {
                    // ... handle other statements
                    sb_append(sb, indent_str);
                    sb_append_line(sb, "/* TODO: statement */");
                }
            }
        }
    }
}

fn gen_let_declaration(sb: *StringBuilder, let_stmt: *ast.LetStatement, indent: int) {
    let indent_str = make_indent(indent);
    sb_append(sb, indent_str);
    
    let var_type = type_to_c(let_stmt.var_type);
    sb_append(sb, var_type);
    sb_append(sb, " ");
    sb_append(sb, let_stmt.name);
    
    if let_stmt.init_value != 0 as *void {
        sb_append(sb, " = ");
        let init_code = gen_expression(let_stmt.init_value);
        sb_append(sb, init_code);
    }
    
    sb_append_line(sb, ";");
}
```

**Estimated Time:** 1 hour

---

#### 1.3 Complete Function Generation

**Current:** Basic signature + simple return
**Need:** Full function body with all statements

```rapter
export fn emit_function(sb: *StringBuilder, func: ast.Function) {
    // Signature
    let ret_type = type_to_c(func.return_type);
    let sig = malloc(1024) as *char;
    sprintf(sig, "%s %s(", ret_type, func.name);
    
    // Parameters
    let i = 0;
    while i < func.params.length() {
        if i > 0 {
            strcat(sig, ", ");
        }
        let param = func.params.get(i);
        let param_type = type_to_c(param.param_type);
        sprintf(sig + strlen(sig), "%s %s", param_type, param.name);
        i = i + 1;
    }
    
    strcat(sig, ")");
    sb_append(sb, sig);
    sb_append_line(sb, " {");
    
    // Body - generate all statements
    let j = 0;
    while j < func.body.length() {
        let stmt = func.body.get(j);
        gen_statement(sb, stmt, 1);  // indent level 1
        j = j + 1;
    }
    
    sb_append_line(sb, "}");
    sb_append_line(sb, "");
}
```

**Estimated Time:** 30 minutes

---

## Phase 2: Fix Module-Qualified Function Calls

### Current Issue
Calls like `module.function()` don't work in all contexts due to parser/codegen limitations.

### Solution Approach A: Parser Enhancement (Preferred)

**Modify:** `src/parser.rs` and `src/codegen.rs`

**Parser Changes:**
```rust
// In parse_call_expression() or similar
fn parse_module_qualified_call(&mut self) -> Result<Expression, String> {
    let module_or_name = self.expect_identifier()?;
    
    if self.peek().kind == TokenKind::Dot || self.peek().kind == TokenKind::ColonColon {
        // This is module.function()
        self.advance();
        let function_name = self.expect_identifier()?;
        
        // Parse arguments
        self.expect(TokenKind::LParen)?;
        let args = self.parse_arguments()?;
        self.expect(TokenKind::RParen)?;
        
        // Create qualified function call
        return Ok(Expression::ModuleCall {
            module: module_or_name,
            function: function_name,
            args,
        });
    } else {
        // Regular function call
        // ... existing logic
    }
}
```

**Codegen Changes:**
```rust
// In generate_expression()
Expression::ModuleCall { module, function, args } => {
    // Generate: module_function(args)
    // OR keep module:: prefix if needed
    format!("{}_{}", module, function)
}
```

**Estimated Time:** 1-2 hours

### Solution Approach B: Wrapper Functions (Quick Fix)

If parser enhancement is too complex, create wrapper layer:

```rapter
// In a helper module: bootstrap/src/wrappers.rapt

import bootstrap.src.lexer_v2 as lexer
import bootstrap.src.ast_v2 as ast
import bootstrap.src.parser_v2 as parser
import bootstrap.src.typechecker as tc
import bootstrap.src.codegen_v2 as codegen

// Wrapper functions that can be called without module prefix
export fn tokenize(input: *char) -> Result<DynamicArray[Token], str> {
    return lexer.tokenize_v2(input);
}

export fn parse(tokens: DynamicArray[Token]) -> Result<Program, str> {
    return parser.parse_program(tokens);
}

export fn typecheck(prog: Program) -> Result<int, str> {
    return tc.check_program(prog);
}

export fn generate_code(prog: Program) -> Result<*char, str> {
    return codegen.generate_c_code(prog);
}
```

Then use: `tokenize(input)` instead of `lexer.tokenize_v2(input)`

**Estimated Time:** 30 minutes

---

## Phase 3: Create Complete Pipeline Integration

### Goal: Connect All Components into Working Compiler

**Create:** `bootstrap/src/compiler_main.rapt`

```rapter
// ============================================================================
// COMPILER_MAIN.RAPT - Bootstrap Compiler Main Entry Point
// ============================================================================

import bootstrap.src.wrappers as w

extern fn printf(format: *char, ...) -> int;
extern fn read_all(path: *char) -> *char;
extern fn write_all(path: *char, data: *char) -> int;

fn compile_file(input_path: *char, output_path: *char) -> Result<int, str> {
    printf("=== Rapter Bootstrap Compiler ===\n");
    printf("Input: %s\n", input_path);
    printf("Output: %s\n\n", output_path);
    
    // Step 1: Read source file
    printf("[1/5] Reading source file...\n");
    let source = read_all(input_path);
    
    // Step 2: Tokenize
    printf("[2/5] Tokenizing...\n");
    let tokens = w.tokenize(source)?;
    printf("      %d tokens generated\n", tokens.length());
    
    // Step 3: Parse
    printf("[3/5] Parsing...\n");
    let ast_prog = w.parse(tokens)?;
    printf("      AST constructed\n");
    
    // Step 4: Type check
    printf("[4/5] Type checking...\n");
    let _check_result = w.typecheck(ast_prog)?;
    printf("      Types validated\n");
    
    // Step 5: Generate code
    printf("[5/5] Generating C code...\n");
    let c_code = w.generate_code(ast_prog)?;
    printf("      C code generated\n");
    
    // Write output
    printf("\nWriting to %s...\n", output_path);
    let write_result = write_all(output_path, c_code);
    if write_result != 0 {
        return Result::Err("Failed to write output file");
    }
    
    printf("\n✅ Compilation successful!\n");
    return Result::Ok(0);
}

fn main() -> int {
    let input = "test.rapt";
    let output = "test.c";
    
    let result = compile_file(input, output);
    let status: int = match result {
        Result::Ok(s) => s,
        Result::Err(msg) => {
            printf("\n❌ Compilation failed: %s\n", msg);
            return 1;
        }
    };
    
    return status;
}
```

**Estimated Time:** 1 hour

---

## Phase 4: Self-Hosting Test

### Goal: Use Bootstrap Compiler to Compile Itself!

**Test Case 1: Compile a Simple Component**

```bash
# Use Rust compiler to compile bootstrap compiler
cargo run bootstrap/src/compiler_main.rapt

# Use compiled bootstrap to compile lexer_v2
./output bootstrap/src/lexer_v2.rapt -o lexer_v2.c

# Compare with Rust compiler output
cargo run bootstrap/src/lexer_v2.rapt
diff output.c lexer_v2.c
```

**Test Case 2: Compile Itself (Ultimate Test)**

```bash
# Bootstrap compiles itself!
./rapter_bootstrap bootstrap/src/compiler_main.rapt -o compiler_main_v2.c

# Compile the output
gcc compiler_main_v2.c -o rapter_bootstrap_v2

# Verify it still works
./rapter_bootstrap_v2 bootstrap/src/lexer_v2.rapt -o test.c
```

**Estimated Time:** 1-2 hours (includes debugging)

---

## Phase 5: Documentation and Polish

### 5.1 Update Documentation

**Files to Update:**
- `README.md` - Show 100% progress!
- `bootstrap/BOOTSTRAP_PROGRESS.md` - Mark all complete
- `bootstrap/SESSION_ACHIEVEMENTS.md` - Add final achievements

### 5.2 Create Self-Hosting Guide

**Create:** `bootstrap/SELF_HOSTING_GUIDE.md`

Contents:
- How to build the bootstrap compiler
- How to use it to compile Rapter code
- Comparison with Rust compiler
- Known limitations
- Future enhancements

### 5.3 Create Usage Examples

**Create:** `examples/self_hosting_demo.rapt`

Show:
- Compiling a simple program
- Full pipeline in action
- Error handling examples

**Estimated Time:** 1 hour

---

## Potential Challenges and Mitigation

### Challenge 1: Module-Qualified Calls Too Complex
**If:** Parser enhancement takes too long
**Then:** Use wrapper function approach (30 min vs 2 hours)

### Challenge 2: Expression Generation Edge Cases
**If:** Some expressions too hard to generate
**Then:** Generate placeholder comments, document TODOs

### Challenge 3: Type Compatibility Issues
**If:** Qualified types cause issues in codegen
**Then:** Use unqualified types in generated C code

### Challenge 4: Memory Management
**If:** Malloc/free cause issues
**Then:** Use static buffers (already done in some places)

---

## Success Criteria

### Minimum Viable (MVP)
- ✅ Code generator handles basic expressions and statements
- ✅ Pipeline can compile simple Rapter programs
- ✅ Output C code compiles with gcc
- ✅ At least one self-hosting test passes

### Full Success
- ✅ All expression types generate correctly
- ✅ All statement types generate correctly
- ✅ Bootstrap compiler can compile itself
- ✅ Output matches Rust compiler (or close enough)
- ✅ Complete documentation

### Stretch Goals
- ✅ Optimize generated C code
- ✅ Better error messages
- ✅ Command-line argument parsing
- ✅ Multiple file compilation

---

## Timeline Estimate

| Phase | Task | Time | Cumulative |
|-------|------|------|------------|
| 1.1 | Expression generation | 1-2 hrs | 1-2 hrs |
| 1.2 | Statement generation | 1 hr | 2-3 hrs |
| 1.3 | Complete functions | 30 min | 2.5-3.5 hrs |
| 2 | Module calls (wrapper approach) | 30 min | 3-4 hrs |
| 3 | Pipeline integration | 1 hr | 4-5 hrs |
| 4 | Self-hosting test | 1-2 hrs | 5-7 hrs |
| 5 | Documentation | 1 hr | 6-8 hrs |

**Total Estimated Time:** 6-8 hours (can be done in one focused session)

**Realistic Timeline:** 
- Start: Morning
- Phase 1-2: Complete by lunch (3-4 hours)
- Phase 3-4: Complete by afternoon (2-3 hours)
- Phase 5: Polish in evening (1 hour)
- **DONE BY END OF DAY!** 🎉

---

## Order of Operations (Recommended)

1. **Start with wrappers** (30 min)
   - Quick fix for module calls
   - Unblocks integration testing

2. **Complete expression generation** (1-2 hrs)
   - Core functionality
   - Enables most code to work

3. **Complete statement generation** (1 hr)
   - Builds on expressions
   - Makes functions work fully

4. **Integrate pipeline** (1 hr)
   - Connect all pieces
   - Create compiler_main.rapt

5. **Test basic compilation** (30 min)
   - Simple test cases
   - Verify it works end-to-end

6. **Self-hosting test** (1-2 hrs)
   - The moment of truth!
   - Debug any issues

7. **Polish and document** (1 hr)
   - Update all docs
   - Create demo

---

## Files to Create Tomorrow

1. `bootstrap/src/wrappers.rapt` - Module call wrappers
2. `bootstrap/src/compiler_main.rapt` - Main compiler driver
3. `examples/simple_program.rapt` - Test case for compilation
4. `bootstrap/SELF_HOSTING_GUIDE.md` - How to use bootstrap compiler
5. `examples/self_hosting_demo.rapt` - Demonstration

## Files to Modify Tomorrow

1. `bootstrap/src/codegen_v2.rapt` - Add expression/statement generation
2. `README.md` - Update to 100% progress
3. `bootstrap/BOOTSTRAP_PROGRESS.md` - Mark complete
4. `bootstrap/SESSION_ACHIEVEMENTS.md` - Add final achievements

---

## The Moment of Truth

**Tomorrow we will run:**

```bash
# Build bootstrap compiler with Rust
cargo run bootstrap/src/compiler_main.rapt
gcc output.c -o rapter_bootstrap

# Use bootstrap to compile itself!
./rapter_bootstrap bootstrap/src/compiler_main.rapt -o self_compiled.c
gcc self_compiled.c -o rapter_bootstrap_v2

# Verify it works
./rapter_bootstrap_v2 examples/hello.rapt -o hello.c
gcc hello.c -o hello
./hello
# Output: Hello, Rapter!
```

**When this works:**
# 🎉🎉🎉 RAPTER IS SELF-HOSTING! 🎉🎉🎉

---

## Backup Plan

If full self-hosting proves too complex in one day:

**Plan B - Incremental Success:**
1. Get pipeline working on simple programs ✅
2. Generate C code that compiles ✅
3. Document known limitations ✅
4. Mark as "90% complete - self-hosting in progress"
5. Finish remaining 10% in next session

But honestly? **We're going to FINISH this tomorrow!** 🚀

The architecture is solid. The components are ready. We just need to connect the pieces and test!

---

## Motivation

We've already written **~1,429 lines** of bootstrap compiler code!

Tomorrow we finish the last **~200-300 lines** and achieve:

✨ **A complete Rapter compiler written in Rapter!**
✨ **Self-hosting capability!**
✨ **Proof that Rapter can compile itself!**

**From 85% → 100% in one day!**

Let's make it happen! 🎯🚀
