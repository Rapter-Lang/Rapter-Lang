# Rapter bootstrap plan

This folder will contain the self-hosted Rapter compiler written in Rapter.

Phases:
- 0: Minimal std
  - args: argc/argv (done)
  - fs: read_all/write_all (done)
  - char: is_alpha/is_digit/is_space (done)
  - str: length/equals (done)
- 1: Lexer (Rapter)
  - Token struct and DynamicArray[Token]
  - read file via std.fs, tokenize using std.char
- 2: Parser (Rapter)
  - AST structs; expressions/statements; precedence parsing
- 3: Semantic analysis (Rapter)
  - SymbolTable, type inference, checks (subset first)
- 4: C codegen (Rapter)
  - C emission matching current Rust generator
- 5: Stage-1 build
  - Use Rust compiler to compile Rapter compiler → link
- 6: Stage-2 build
  - Use stage-1 Rapter compiler to compile itself

Conventions
- Modules live under src/ with `export` as needed
- Use scripts/build.ps1 to link multiple modules

Try:
- scripts/build.ps1 -Modules 'src/std/args.rapt','src/std/fs.rapt','src/std/char.rapt','src/std/str.rapt' -Entrypoint 'bootstrap/src/main.rapt' -Out 'rapter-bootstrap.exe'
