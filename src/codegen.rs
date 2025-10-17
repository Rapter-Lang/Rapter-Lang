use crate::ast::*;
use crate::error::{CompilerError, ErrorKind, SourceLocation, Suggestion};
use crate::modules::ModuleResolver;
use crate::builtins::BuiltinRegistry;
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};

pub struct CCodeGenerator {
    output: String,
    indent_level: usize,
    // Stack of variable type scopes for type-aware codegen
    var_types: Vec<HashMap<String, Type>>,
    // Known function return types (unqualified names)
    func_types: HashMap<String, Type>,
    // Current function's return type (for type inference in return statements)
    current_return_type: Option<Type>,
    // Counter for generating unique temporary variables
    temp_counter: usize,
    // Track generic type instantiations that need monomorphization
    generic_instantiations: HashSet<Type>,
    // Built-in types registry
    builtins: BuiltinRegistry,
}

impl CCodeGenerator {
    pub fn new() -> Self {
        CCodeGenerator {
            output: String::new(),
            indent_level: 0,
            var_types: Vec::new(),
            func_types: HashMap::new(),
            current_return_type: None,
            temp_counter: 0,
            generic_instantiations: HashSet::new(),
            builtins: BuiltinRegistry::new(),
        }
    }
    
    // Track a generic type instantiation for later generation
    fn track_generic_type(&mut self, ty: &Type) {
        if let Type::Generic { .. } = ty {
            self.generic_instantiations.insert(ty.clone());
        }
        // Also track nested generic types
        match ty {
            Type::Pointer(inner) => self.track_generic_type(inner),
            Type::Array(inner) => self.track_generic_type(inner),
            Type::DynamicArray(inner) => self.track_generic_type(inner),
            _ => {}
        }
    }
    
    // Collect all generic types used in an AST
    fn collect_generic_types(&mut self, ast: &Program) {
        // Collect from function signatures
        for func in &ast.functions {
            if let Some(ret_ty) = &func.return_type {
                self.track_generic_type(ret_ty);
            }
            for param in &func.parameters {
                self.track_generic_type(&param.param_type);
            }
            // Collect from function body
            for stmt in &func.body {
                self.collect_generic_types_from_stmt(stmt);
            }
        }
        
        // Collect from global variables
        for global in &ast.global_variables {
            if let Some(ty) = &global.var_type {
                self.track_generic_type(ty);
            }
        }
    }
    
    fn collect_generic_types_from_stmt(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let { var_type, .. } | Statement::Const { var_type, .. } => {
                if let Some(ty) = var_type {
                    self.track_generic_type(ty);
                }
            }
            Statement::If { then_branch, else_branch, .. } => {
                for s in then_branch {
                    self.collect_generic_types_from_stmt(s);
                }
                if let Some(else_stmts) = else_branch {
                    for s in else_stmts {
                        self.collect_generic_types_from_stmt(s);
                    }
                }
            }
            Statement::While { body, .. } => {
                for s in body {
                    self.collect_generic_types_from_stmt(s);
                }
            }
            Statement::For { body, .. } => {
                for s in body {
                    self.collect_generic_types_from_stmt(s);
                }
            }
            _ => {}
        }
    }
    
    // Generate C definitions for all tracked generic types
    fn generate_generic_type_defs(&mut self) -> Result<(), CompilerError> {
        let instantiations: Vec<Type> = self.generic_instantiations.iter().cloned().collect();
        for generic_ty in instantiations {
            if let Type::Generic { name, ref type_params } = generic_ty {
                if let Some(builtin) = self.builtins.get_generic(&name).cloned() {
                    self.generate_builtin_generic_def(&builtin, type_params)?;
                }
            }
        }
        Ok(())
    }
    
    // Generate C code for a built-in generic type (Option, Result)
    fn generate_builtin_generic_def(
        &mut self,
        builtin: &crate::builtins::BuiltinGenericType,
        type_params: &[Type],
    ) -> Result<(), CompilerError> {
        let mangled_name = self.type_to_c(&Type::Generic {
            name: builtin.name.clone(),
            type_params: type_params.to_vec(),
        });
        
        // Generate enum for variant tags
        self.output.push_str("// Generic type: ");
        self.output.push_str(&builtin.name);
        self.output.push_str("<");
        for (i, ty) in type_params.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str(&format!("{:?}", ty));
        }
        self.output.push_str(">\n");
        
        self.output.push_str("typedef enum {\n");
        for variant in &builtin.variants {
            self.output.push_str("    ");
            self.output.push_str(&mangled_name);
            self.output.push_str("_");
            self.output.push_str(&variant.name);
            self.output.push_str(",\n");
        }
        self.output.push_str("} ");
        self.output.push_str(&mangled_name);
        self.output.push_str("_Tag;\n\n");
        
        // Generate struct with tag and union for values
        self.output.push_str("typedef struct {\n");
        self.output.push_str("    ");
        self.output.push_str(&mangled_name);
        self.output.push_str("_Tag tag;\n");
        
        // Only add union if some variants have values
        let has_values = builtin.variants.iter().any(|v| v.has_value);
        if has_values {
            self.output.push_str("    union {\n");
            for variant in &builtin.variants {
                if variant.has_value {
                    if let Some(type_param_name) = &variant.value_type_param {
                        // Find which type parameter index this is
                        if let Some(idx) = builtin.type_params.iter().position(|p| p == type_param_name) {
                            if let Some(concrete_ty) = type_params.get(idx) {
                                self.output.push_str("        ");
                                self.output.push_str(&self.type_to_c(concrete_ty));
                                self.output.push_str(" ");
                                self.output.push_str(&variant.name.to_lowercase());
                                self.output.push_str("_value;\n");
                            }
                        }
                    }
                }
            }
            self.output.push_str("    } data;\n");
        }
        
        self.output.push_str("} ");
        self.output.push_str(&mangled_name);
        self.output.push_str(";\n\n");
        
        // Generate constructor macros for each variant
        for variant in &builtin.variants {
            let variant_upper = variant.name.to_uppercase();
            self.output.push_str("#define ");
            self.output.push_str(&builtin.name.to_uppercase());
            self.output.push_str("_");
            self.output.push_str(&variant_upper);
            self.output.push_str(" ((");
            self.output.push_str(&mangled_name);
            self.output.push_str("){ .tag = ");
            self.output.push_str(&mangled_name);
            self.output.push_str("_");
            self.output.push_str(&variant.name);
            self.output.push_str(" })\n");
        }
        self.output.push_str("\n");
        
        Ok(())
    }
    
    fn indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }
    
    pub fn generate(&mut self, ast: &Program, resolver: &mut ModuleResolver, _file_path: &PathBuf) -> Result<(), CompilerError> {
        // First pass: collect all generic type instantiations from main AST
        self.collect_generic_types(ast);
        
        // Also collect from imported modules
        for import in &ast.imports {
            let module = resolver.load_module(&import.module)?;
            self.collect_generic_types(&module.program);
        }
        
        // Add headers
        self.output.push_str("#include <stdio.h>\n");
        self.output.push_str("#include <stdlib.h>\n");
        self.output.push_str("#include <string.h>\n");
    self.output.push_str("#include <stddef.h>\n");
    self.output.push_str("#include <ctype.h>\n\n");
        let has_main = ast.functions.iter().any(|f| f.name == "main");
        if has_main {
            // Globals and accessors for command-line arguments (define once in entrypoint TU)
            self.output.push_str("static int __rapter_argc = 0;\n");
            self.output.push_str("static char** __rapter_argv = NULL;\n");
            self.output.push_str("int rapter_get_argc() { return __rapter_argc; }\n");
            self.output.push_str("char* rapter_get_argv(int i) { return (i >= 0 && i < __rapter_argc) ? __rapter_argv[i] : \"\"; }\n\n");
        }
        
        // Add typedefs for dynamic arrays
    self.output.push_str("typedef struct { int* data; size_t size; size_t capacity; } DynamicArray_int;\n");
    self.output.push_str("typedef struct { double* data; size_t size; size_t capacity; } DynamicArray_double;\n");
    self.output.push_str("typedef struct { char* data; size_t size; size_t capacity; } DynamicArray_char;\n\n");
        
        // Generate enums FIRST (before structs that might use them)
        for import in &ast.imports {
            let module = resolver.load_module(&import.module)?;
            for enm in &module.program.enums {
                self.generate_enum(enm)?;
                self.output.push_str("\n");
            }
        }
        for enm in &ast.enums {
            self.generate_enum(enm)?;
            self.output.push_str("\n");
        }
        
        // Define imported structs (typedef struct Name { ... } Name;) so we can construct values
        for import in &ast.imports {
            let module = resolver.load_module(&import.module)?;
            for st in &module.program.structs {
                self.generate_struct(st)?;
                self.output.push_str("\n");
            }
        }
        // Define local structs
        for st in &ast.structs {
            self.generate_struct(st)?;
            self.output.push_str("\n");
        }
        
        // Add typedefs for dynamic arrays of user-defined structs (local)
        for st in &ast.structs {
            self.output.push_str("typedef struct { ");
            self.output.push_str("struct ");
            self.output.push_str(&st.name);
            self.output.push_str("* data; size_t size; size_t capacity; } DynamicArray_");
            self.output.push_str(&st.name);
            self.output.push_str(";\n");
        }
        // Add typedefs for dynamic arrays of imported structs
        for import in &ast.imports {
            let module = resolver.load_module(&import.module)?;
            for st in &module.program.structs {
                self.output.push_str("typedef struct { ");
                self.output.push_str("struct ");
                self.output.push_str(&st.name);
                self.output.push_str("* data; size_t size; size_t capacity; } DynamicArray_");
                self.output.push_str(&st.name);
                self.output.push_str(";\n");
            }
        }
        self.output.push_str("\n");
        
        // Generate definitions for generic type instantiations (Option<int>, Result<T, E>, etc.)
        self.generate_generic_type_defs()?;
        
        // Declare external functions
        for ext_func in &ast.extern_functions {
            // Skip intrinsic functions - they're already in C stdlib headers
            if crate::intrinsics::is_intrinsic(&ext_func.name) {
                continue;
            }
            // Record return type for externs
            self.func_types.insert(ext_func.name.clone(), ext_func.return_type.clone().unwrap_or(Type::Void));
            self.declare_extern_function(ext_func)?;
            self.output.push_str(";\n");
        }

        if has_main {
            // Helper functions for std.fs (file I/O) bindings, define once in entrypoint TU
            self.output.push_str("int rapter_write_all(char* path, char* data) { FILE* f = fopen(path, \"wb\"); if (!f) return -1; size_t n = strlen(data); size_t w = fwrite(data, 1, n, f); fclose(f); return w == n ? 0 : -1; }\n");
            self.output.push_str("char* rapter_read_all(char* path) { FILE* f = fopen(path, \"rb\"); if (!f) { char* s = (char*)malloc(1); if (s) s[0] = 0; return s; } if (fseek(f, 0, SEEK_END) != 0) { fclose(f); char* s = (char*)malloc(1); if (s) s[0]=0; return s; } long sz = ftell(f); if (sz < 0) { fclose(f); char* s = (char*)malloc(1); if (s) s[0]=0; return s; } fseek(f, 0, SEEK_SET); char* buf = (char*)malloc((size_t)sz + 1); if (!buf) { fclose(f); return NULL; } size_t n = fread(buf, 1, (size_t)sz, f); fclose(f); buf[n] = 0; return buf; }\n\n");
            
            // String helper functions
            self.output.push_str("typedef struct { char** data; size_t size; size_t capacity; } DynamicArray_charptr;\n");
            self.output.push_str("char* rapter_substring(char* str, int start, int end) { if (!str) return NULL; int len = strlen(str); if (start < 0) start = 0; if (end > len) end = len; if (start >= end) return strdup(\"\"); int sublen = end - start; char* result = (char*)malloc(sublen + 1); if (!result) return NULL; strncpy(result, str + start, sublen); result[sublen] = 0; return result; }\n");
            self.output.push_str("char* rapter_trim(char* str) { if (!str) return NULL; while (*str && isspace((unsigned char)*str)) str++; if (!*str) return strdup(\"\"); char* end = str + strlen(str) - 1; while (end > str && isspace((unsigned char)*end)) end--; size_t len = end - str + 1; char* result = (char*)malloc(len + 1); if (!result) return NULL; memcpy(result, str, len); result[len] = 0; return result; }\n");
            self.output.push_str("DynamicArray_charptr rapter_split(char* str, char* delim) { DynamicArray_charptr arr; arr.size = 0; arr.capacity = 4; arr.data = (char**)malloc(arr.capacity * sizeof(char*)); if (!arr.data) return arr; char* copy = strdup(str); char* token = strtok(copy, delim); while (token) { if (arr.size >= arr.capacity) { arr.capacity *= 2; arr.data = (char**)realloc(arr.data, arr.capacity * sizeof(char*)); } arr.data[arr.size++] = strdup(token); token = strtok(NULL, delim); } free(copy); return arr; }\n\n");
        }
        
        // (structs already defined above)
        
        // Generate function declarations (for forward declarations if needed)
        for func in &ast.functions {
            // Record local function return types
            self.func_types.insert(func.name.clone(), func.return_type.clone().unwrap_or(Type::Void));
            self.declare_function(func)?;
            self.output.push_str(";\n");
        }
        
        // Generate declarations for imported functions
        for import in &ast.imports {
            let module = resolver.load_module(&import.module)?;
            for (name, symbol) in &module.exports {
                if let crate::modules::SymbolType::Function = symbol.symbol_type {
                    // Find the actual function in the module
                    if let Some(func) = module.program.functions.iter().find(|f| f.name == *name) {
                        // Record imported function return type by unqualified name
                        self.func_types.insert(func.name.clone(), func.return_type.clone().unwrap_or(Type::Void));
                        self.declare_function(func)?;
                        self.output.push_str(";\n");
                    }
                }
            }
        }
        self.output.push_str("\n");
        
        // Generate global variable definitions
        for global_var in &ast.global_variables {
            self.generate_global_variable(global_var)?;
        }
        if !ast.global_variables.is_empty() {
            self.output.push_str("\n");
        }
        
        // Generate function definitions
        for func in &ast.functions {
            self.generate_function(func)?;
            self.output.push_str("\n");
        }
        
        // Generate imported function definitions (for bootstrap simplicity)
        // TODO: In the future, compile modules separately and link
        for import in &ast.imports {
            let module = resolver.load_module(&import.module)?;
            // Generate ALL functions from the module (exported and internal)
            for func in &module.program.functions {
                self.generate_function(func)?;
                self.output.push_str("\n");
            }
        }
        
        // Generate main wrapper if there's a main function
        if has_main {
            self.generate_main_wrapper()?;
        }
        
        Ok(())
    }
    
    fn declare_extern_function(&mut self, func: &ExternFunction) -> Result<(), CompilerError> {
        let return_type = self.type_to_c(&func.return_type.clone().unwrap_or(Type::Void));
        self.output.push_str(&return_type);
        self.output.push_str(" ");
        self.output.push_str(&func.name);
        self.output.push_str("(");
        
        for (i, param) in func.parameters.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str(&self.type_to_c(&param.param_type));
            self.output.push_str(" ");
            self.output.push_str(&param.name);
        }
        
        if func.variadic {
            if !func.parameters.is_empty() {
                self.output.push_str(", ");
            }
            self.output.push_str("...");
        }
        
        self.output.push_str(")");
        Ok(())
    }
    
    fn declare_function(&mut self, func: &Function) -> Result<(), CompilerError> {
        let return_type = self.type_to_c(&func.return_type.clone().unwrap_or(Type::Void));
        self.output.push_str(&return_type);
        self.output.push_str(" ");
        let func_name = if func.name == "main" { "rapter_main" } else { &func.name };
        self.output.push_str(func_name);
        self.output.push_str("(");
        
        for (i, param) in func.parameters.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str(&self.type_to_c(&param.param_type));
            self.output.push_str(" ");
            self.output.push_str(&param.name);
        }
        self.output.push_str(")");
        Ok(())
    }
    
    fn generate_struct(&mut self, st: &Struct) -> Result<(), CompilerError> {
        self.output.push_str("typedef struct ");
        self.output.push_str(&st.name);
        self.output.push_str(" {\n");
        self.indent_level += 1;
        for field in &st.fields {
            self.indent();
            self.output.push_str(&self.type_to_c(&field.field_type));
            self.output.push_str(" ");
            self.output.push_str(&field.name);
            self.output.push_str(";\n");
        }
        self.indent_level -= 1;
        self.output.push_str("} ");
        self.output.push_str(&st.name);
        self.output.push_str(";\n");
        Ok(())
    }
    
    fn generate_enum(&mut self, enm: &Enum) -> Result<(), CompilerError> {
        // Generate C typedef enum with explicit values
        self.output.push_str("typedef enum {\n");
        self.indent_level += 1;
        
        for (i, variant) in enm.variants.iter().enumerate() {
            self.indent();
            // Use ENUM_VARIANT naming convention
            self.output.push_str(&format!("{}_{}", enm.name.to_uppercase(), variant.name.to_uppercase()));
            if let Some(val) = variant.value {
                self.output.push_str(&format!(" = {}", val));
            }
            if i < enm.variants.len() - 1 {
                self.output.push_str(",");
            }
            self.output.push_str("\n");
        }
        
        self.indent_level -= 1;
        self.output.push_str("} ");
        self.output.push_str(&enm.name);
        self.output.push_str(";\n");
        
        // Generate accessor functions for bootstrap compatibility
        // (so token.TK_EOF() works even though it should be TokenKind::EOF)
        for variant in &enm.variants {
            self.output.push_str(&format!("static inline {} TK_{}() {{ return {}_{};}} \n",
                enm.name,
                variant.name.to_uppercase(),
                enm.name.to_uppercase(),
                variant.name.to_uppercase()));
        }
        
        Ok(())
    }
    
    fn generate_global_variable(&mut self, global_var: &GlobalVariable) -> Result<(), CompilerError> {
        // Determine the type
        let ty = if let Some(t) = &global_var.var_type {
            t.clone()
        } else if let Some(expr) = &global_var.initializer {
            self.expr_type(expr).unwrap_or(Type::Int)
        } else {
            Type::Int // Should not happen as semantic analysis checks this
        };
        
        // Generate: static <type> <name> = <initializer>;
        // or: static <type> <name>;
        self.output.push_str("static ");
        self.output.push_str(&self.type_to_c(&ty));
        self.output.push_str(" ");
        self.output.push_str(&global_var.name);
        
        if let Some(init) = &global_var.initializer {
            self.output.push_str(" = ");
            self.generate_expression(init)?;
        }
        
        self.output.push_str(";\n");
        
        // Track global variable type for future references
        self.set_var_type(&global_var.name, ty);
        
        Ok(())
    }
    
    fn generate_function(&mut self, func: &Function) -> Result<(), CompilerError> {
        // Set the current return type for this function
        self.current_return_type = func.return_type.clone();
        
        self.declare_function(func)?;
        self.output.push_str(" {\n");
        self.indent_level += 1;
        // Enter a new variable type scope and record parameters
        self.enter_scope();
        for param in &func.parameters {
            self.set_var_type(&param.name, param.param_type.clone());
        }
        
        for stmt in &func.body {
            self.generate_statement(stmt)?;
        }
        
        // Exit function scope
        self.exit_scope();
        self.current_return_type = None;
        self.indent_level -= 1;
        self.output.push_str("}\n");
        Ok(())
    }
    
    fn generate_statement(&mut self, stmt: &Statement) -> Result<(), CompilerError> {
        self.indent();
        match stmt {
            Statement::Let { name, var_type, mutable: _, initializer } => {
                if let Some(ty) = var_type {
                    self.output.push_str(&self.type_to_c(ty));
                    // Track declared type
                    self.set_var_type(name, ty.clone());
                } else if let Some(expr) = initializer {
                    self.output.push_str(&self.infer_c_type(expr));
                    if let Some(inf_ty) = self.expr_type(expr) {
                        self.set_var_type(name, inf_ty);
                    }
                } else {
                    let location = SourceLocation::new(PathBuf::from("input.rap"), 0, 0);
                    return Err(CompilerError::new(
                        ErrorKind::UnsupportedFeature,
                        "variable must have type or initializer".to_string(),
                        location,
                    ).with_suggestion(Suggestion::simple(
                        "add an explicit type annotation or provide an initializer expression"
                    )));
                }
                self.output.push_str(" ");
                self.output.push_str(name);
                if let Some(expr) = initializer {
                    self.output.push_str(" = ");
                    self.generate_expression(expr)?;
                }
                self.output.push_str(";\n");
            }
            Statement::Const { name, var_type, initializer } => {
                if let Some(ty) = var_type {
                    self.output.push_str(&self.type_to_c(ty));
                    self.set_var_type(name, ty.clone());
                } else {
                    self.output.push_str(&self.infer_c_type(initializer));
                    if let Some(inf_ty) = self.expr_type(initializer) {
                        self.set_var_type(name, inf_ty);
                    }
                }
                self.output.push_str(" ");
                self.output.push_str(name);
                self.output.push_str(" = ");
                self.generate_expression(initializer)?;
                self.output.push_str(";\n");
            }
            Statement::Return(value) => {
                self.output.push_str("return");
                if let Some(expr) = value {
                    self.output.push_str(" ");
                    self.generate_expression(expr)?;
                }
                self.output.push_str(";\n");
            }
            Statement::Expression(expr) => {
                self.generate_expression(expr)?;
                self.output.push_str(";\n");
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.output.push_str("if (");
                self.generate_expression(condition)?;
                self.output.push_str(") {\n");
                self.indent_level += 1;
                self.enter_scope();
                for stmt in then_branch {
                    self.generate_statement(stmt)?;
                }
                self.exit_scope();
                self.indent_level -= 1;
                self.indent();
                self.output.push_str("}");
                
                if let Some(else_stmts) = else_branch {
                    self.output.push_str(" else {\n");
                    self.indent_level += 1;
                    self.enter_scope();
                    for stmt in else_stmts {
                        self.generate_statement(stmt)?;
                    }
                    self.exit_scope();
                    self.indent_level -= 1;
                    self.indent();
                    self.output.push_str("}");
                }
                self.output.push_str("\n");
            }
            Statement::While { condition, body } => {
                self.output.push_str("while (");
                self.generate_expression(condition)?;
                self.output.push_str(") {\n");
                self.indent_level += 1;
                self.enter_scope();
                for stmt in body {
                    self.generate_statement(stmt)?;
                }
                self.exit_scope();
                self.indent_level -= 1;
                self.indent();
                self.output.push_str("}\n");
            }
            Statement::Assignment { target, value } => {
                self.generate_expression(target)?;
                self.output.push_str(" = ");
                self.generate_expression(value)?;
                self.output.push_str(";\n");
            }
            Statement::For {
                variable,
                iterable,
                body,
            } => {
                // Assume iterable is a range like start..end
                if let Expression::Range { start, end } = iterable {
                    self.output.push_str("for (int ");
                    self.output.push_str(&variable);
                    self.output.push_str(" = ");
                    self.generate_expression(start)?;
                    self.output.push_str("; ");
                    self.output.push_str(&variable);
                    self.output.push_str(" < ");
                    self.generate_expression(end)?;
                    self.output.push_str("; ");
                    self.output.push_str(&variable);
                    self.output.push_str("++) {\n");
                    self.indent_level += 1;
                    // Scope for for-loop body; track loop variable as int
                    self.enter_scope();
                    self.set_var_type(variable, Type::Int);
                    for stmt in body {
                        self.generate_statement(stmt)?;
                    }
                    self.exit_scope();
                    self.indent_level -= 1;
                    self.output.push_str("}\n");
                } else {
                    // Fallback for other iterables
                    self.output.push_str("// TODO: implement for loop for non-range iterables\n");
                }
            }
            Statement::Break => {
                self.output.push_str("break;\n");
            }
            Statement::Continue => {
                self.output.push_str("continue;\n");
            }
        }
        Ok(())
    }
    
    fn generate_expression(&mut self, expr: &Expression) -> Result<(), CompilerError> {
        match expr {
            Expression::Literal(lit) => match lit {
                Literal::Integer(i) => self.output.push_str(&i.to_string()),
                Literal::Float(f) => self.output.push_str(&f.to_string()),
                Literal::Bool(b) => self.output.push_str(if *b { "1" } else { "0" }),
                Literal::Char(c) => {
                    // Escape special chars for valid C char literal
                    let esc: Option<&str> = match *c {
                        '\\' => Some("\\\\"),
                        '\'' => Some("\\'"),
                        '\n' => Some("\\n"),
                        '\t' => Some("\\t"),
                        '\r' => Some("\\r"),
                        '\0' => Some("\\0"),
                        _ => None,
                    };
                    self.output.push_str("'");
                    if let Some(e) = esc {
                        self.output.push_str(e);
                    } else {
                        self.output.push(*c);
                    }
                    self.output.push_str("'");
                }
                Literal::String(s) => {
                    self.output.push_str("\"");
                    // Escape special characters for C
                    for ch in s.chars() {
                        match ch {
                            '\n' => self.output.push_str("\\n"),
                            '\r' => self.output.push_str("\\r"),
                            '\t' => self.output.push_str("\\t"),
                            '\\' => self.output.push_str("\\\\"),
                            '"' => self.output.push_str("\\\""),
                            '\0' => self.output.push_str("\\0"),
                            _ => self.output.push(ch),
                        }
                    }
                    self.output.push_str("\"");
                }
            },
            Expression::Variable(name) => self.output.push_str(name),
            Expression::Binary { left, operator, right } => {
                // Special case: string concatenation
                if *operator == BinaryOp::Add && (self.contains_string_literal(left) || self.contains_string_literal(right)) {
                    // If either operand contains a string literal, treat this as string concatenation
                    self.generate_string_concatenation(left, right)?;
                } else {
                    self.output.push_str("(");
                    self.generate_expression(left)?;
                    self.output.push_str(" ");
                    let op_str = match operator {
                        BinaryOp::Add => "+",
                        BinaryOp::Subtract => "-",
                        BinaryOp::Multiply => "*",
                        BinaryOp::Divide => "/",
                        BinaryOp::Modulo => "%",
                        BinaryOp::Equal => "==",
                        BinaryOp::NotEqual => "!=",
                        BinaryOp::Less => "<",
                        BinaryOp::LessEqual => "<=",
                        BinaryOp::Greater => ">",
                        BinaryOp::GreaterEqual => ">=",
                        BinaryOp::And => "&&",
                        BinaryOp::Or => "||",
                    };
                    self.output.push_str(op_str);
                    self.output.push_str(" ");
                    self.generate_expression(right)?;
                    self.output.push_str(")");
                }
            }
            Expression::Unary { operator, operand } => {
                let op_str = match operator {
                    UnaryOp::Negate => "-",
                    UnaryOp::Not => "!",
                    UnaryOp::Dereference => "*",
                    UnaryOp::AddressOf => "&",
                };
                self.output.push_str(op_str);
                self.generate_expression(operand)?;
            }
            Expression::Call { callee, arguments } => {
                if let Expression::Variable(name) = &**callee {
                    if name == "print" {
                        if arguments.len() == 1 && self.is_array_expression(&arguments[0]) {
                            // Special handling for arrays - print each element
                            self.generate_array_print(&arguments[0], false)?;
                        } else {
                            // print(arg) -> printf("%d", arg) or similar based on type
                            self.output.push_str("printf(");
                            if arguments.len() == 1 {
                                // Try to infer the format based on the argument type
                                let format_spec = self.infer_printf_format(&arguments[0]);
                                self.output.push_str("\"");
                                self.output.push_str(&format_spec);
                                self.output.push_str("\"");
                                if arguments.len() > 0 {
                                    self.output.push_str(", ");
                                    self.generate_expression(&arguments[0])?;
                                }
                            }
                            self.output.push_str(")");
                        }
                    } else if name == "println" {
                        if arguments.len() == 1 && self.is_array_expression(&arguments[0]) {
                            // Special handling for arrays - print each element with newline
                            self.generate_array_print(&arguments[0], true)?;
                        } else {
                            // println(arg) -> printf("%d\n", arg) or similar
                            self.output.push_str("printf(");
                            if arguments.len() == 1 {
                                let format_spec = self.infer_printf_format(&arguments[0]);
                                self.output.push_str("\"");
                                self.output.push_str(&format_spec);
                                self.output.push_str("\\n\"");
                                if arguments.len() > 0 {
                                    self.output.push_str(", ");
                                    self.generate_expression(&arguments[0])?;
                                }
                            } else {
                                // Just print newline
                                self.output.push_str("\"\\n\"");
                            }
                            self.output.push_str(")");
                        }
                    } else if name == "len" {
                        // len(str) -> strlen(str) - built-in string length function
                        self.output.push_str("strlen(");
                        if arguments.len() == 1 {
                            self.generate_expression(&arguments[0])?;
                        } else {
                            self.output.push_str("\"\""); // Default empty string if no arguments
                        }
                        self.output.push_str(")");
                    } else {
                        // Regular function call
                        self.output.push_str(name);
                        self.output.push_str("(");
                        for (i, arg) in arguments.iter().enumerate() {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            self.generate_expression(arg)?;
                        }
                        self.output.push_str(")");
                    }
                } else if let Expression::StructAccess { object, field } = &**callee {
                    // Distinguish between module-qualified calls (module.func) and methods (obj.method)
                    if let Expression::Variable(obj_name) = &**object {
                        let mut obj_type = self.expr_type(object).unwrap_or(Type::Int);
                        
                        // Normalize str to String type
                        if let Type::Struct(ref name) = obj_type {
                            if name == "str" {
                                obj_type = Type::String;
                            }
                        }
                        
                        match (&obj_type, field.as_str()) {
                            // String methods
                            (&Type::String, "length") => {
                                self.output.push_str("strlen(");
                                self.generate_expression(object)?;
                                self.output.push_str(")");
                            }
                            (&Type::String, "substring") => {
                                self.output.push_str("rapter_substring(");
                                self.generate_expression(object)?;
                                self.output.push_str(", ");
                                self.generate_expression(&arguments[0])?;
                                self.output.push_str(", ");
                                self.generate_expression(&arguments[1])?;
                                self.output.push_str(")");
                            }
                            (&Type::String, "contains") => {
                                self.output.push_str("(strstr(");
                                self.generate_expression(object)?;
                                self.output.push_str(", ");
                                self.generate_expression(&arguments[0])?;
                                self.output.push_str(") != NULL ? 1 : 0)");
                            }
                            (&Type::String, "trim") => {
                                self.output.push_str("rapter_trim(");
                                self.generate_expression(object)?;
                                self.output.push_str(")");
                            }
                            (&Type::String, "split") => {
                                self.output.push_str("rapter_split(");
                                self.generate_expression(object)?;
                                self.output.push_str(", ");
                                self.generate_expression(&arguments[0])?;
                                self.output.push_str(")");
                            }
                            // Dynamic array methods
                            (&Type::DynamicArray(_), "push") => {
                                if arguments.len() == 1 {
                                    self.output.push_str("({ ");
                                    self.output.push_str("if (");
                                    self.output.push_str(obj_name);
                                    self.output.push_str(".size == ");
                                    self.output.push_str(obj_name);
                                    self.output.push_str(".capacity) { size_t new_cap = ");
                                    self.output.push_str(obj_name);
                                    self.output.push_str(".capacity ? ");
                                    self.output.push_str(obj_name);
                                    self.output.push_str(".capacity * 2 : 4; ");
                                    self.output.push_str(obj_name);
                                    self.output.push_str(".data = realloc(");
                                    self.output.push_str(obj_name);
                                    self.output.push_str(".data, new_cap * sizeof(");
                                    self.output.push_str(obj_name);
                                    self.output.push_str(".data[0])); ");
                                    self.output.push_str(obj_name);
                                    self.output.push_str(".capacity = new_cap; } ");
                                    self.output.push_str(obj_name);
                                    self.output.push_str(".data[");
                                    self.output.push_str(obj_name);
                                    self.output.push_str(".size++] = ");
                                    self.generate_expression(&arguments[0])?;
                                    self.output.push_str("; ");
                                    self.output.push_str(obj_name);
                                    self.output.push_str("; })");
                                } else {
                                    self.output.push_str("/* push expects 1 argument */");
                                }
                            }
                            (&Type::DynamicArray(_), "pop") => {
                                if arguments.is_empty() {
                                    self.output.push_str("(");
                                    self.output.push_str(obj_name);
                                    self.output.push_str(".size > 0 ? ");
                                    self.output.push_str(obj_name);
                                    self.output.push_str(".data[--");
                                    self.output.push_str(obj_name);
                                    self.output.push_str(".size] : 0)");
                                } else {
                                    self.output.push_str("/* pop expects no arguments */");
                                }
                            }
                            (&Type::DynamicArray(_), "length") => {
                                if arguments.is_empty() {
                                    self.output.push_str("(");
                                    self.output.push_str(obj_name);
                                    self.output.push_str(".size)");
                                } else {
                                    self.output.push_str("/* length expects no arguments */");
                                }
                            }
                            // Assume module-qualified function call like module.func
                            _ => {
                                self.output.push_str(field);
                                self.output.push_str("(");
                                for (i, arg) in arguments.iter().enumerate() {
                                    if i > 0 { self.output.push_str(", "); }
                                    self.generate_expression(arg)?;
                                }
                                self.output.push_str(")");
                            }
                        }
                    } else {
                        self.output.push_str("/* method calls on non-variables not supported */");
                    }
                } else if let Expression::EnumAccess { enum_name, variant } = &**callee {
                    // Enum variant constructor call: Option::Some(42), Result::Ok(value)
                    // Generate C code: (Option_int){ .tag = Option_int_Some, .data = { .some_value = 42 } }
                    
                    if arguments.len() == 1 {
                        // Try to use the current function's return type if it matches this generic type
                        let generic_type = if let Some(ret_ty) = &self.current_return_type {
                            if let Type::Generic { name, .. } = ret_ty {
                                if name == enum_name {
                                    // Use the full return type (has all type parameters)
                                    ret_ty.clone()
                                } else {
                                    // Fallback: infer from argument
                                    let arg_type = self.expr_type(&arguments[0]).unwrap_or(Type::Int);
                                    Type::Generic {
                                        name: enum_name.clone(),
                                        type_params: vec![arg_type.clone()],
                                    }
                                }
                            } else {
                                // Fallback
                                let arg_type = self.expr_type(&arguments[0]).unwrap_or(Type::Int);
                                Type::Generic {
                                    name: enum_name.clone(),
                                    type_params: vec![arg_type.clone()],
                                }
                            }
                        } else {
                            // No return type context, infer from argument
                            let arg_type = self.expr_type(&arguments[0]).unwrap_or(Type::Int);
                            Type::Generic {
                                name: enum_name.clone(),
                                type_params: vec![arg_type.clone()],
                            }
                        };
                        
                        let c_type = self.type_to_c(&generic_type);
                        let variant_tag = format!("{}_{}", c_type, variant);
                        let field_name = format!("{}_value", variant.to_lowercase());
                        
                        // Generate: (Option_int){ .tag = Option_int_Some, .data = { .some_value = 42 } }
                        self.output.push_str("((");
                        self.output.push_str(&c_type);
                        self.output.push_str("){ .tag = ");
                        self.output.push_str(&variant_tag);
                        self.output.push_str(", .data = { .");
                        self.output.push_str(&field_name);
                        self.output.push_str(" = ");
                        self.generate_expression(&arguments[0])?;
                        self.output.push_str(" } })");
                    } else {
                        self.output.push_str("/* enum variant construction expects 1 argument */");
                    }
                } else {
                    let location = SourceLocation::new(PathBuf::from("input.rap"), 0, 0);
                    return Err(CompilerError::new(
                        ErrorKind::UnsupportedFeature,
                        "function calls must be direct or method calls".to_string(),
                        location,
                    ).with_suggestion(Suggestion::simple(
                        "use direct function calls like `func()` or method calls like `obj.method()`"
                    )));
                }
            }
            Expression::ArrayLiteral(elements) => {
                self.output.push_str("(int[]){");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expression(elem)?;
                }
                self.output.push_str("}");
            }
            Expression::DynamicArrayLiteral { element_type, elements } => {
                // Generate initialized dynamic array with capacity and data copy using a GNU statement-expression
                let is_primitive = matches!(&**element_type, Type::Int | Type::Float | Type::Char);
                let elem_c_type = self.type_to_c(element_type);
                let count = elements.len();
                self.output.push_str("({ ");
                if is_primitive || matches!(&**element_type, Type::Struct(_)) {
                    let typedef_name = match &**element_type {
                        Type::Int => "DynamicArray_int",
                        Type::Float => "DynamicArray_double",
                        Type::Char => "DynamicArray_char",
                        Type::Struct(_) => {
                            // Use DynamicArray_<StructName>
                            self.output.push_str("");
                            // name used below
                            "" // placeholder, will be replaced
                        }
                        _ => unreachable!(),
                    };
                    if let Type::Struct(name) = &**element_type {
                        self.output.push_str("DynamicArray_");
                        self.output.push_str(name);
                    } else {
                        self.output.push_str(typedef_name);
                    }
                    self.output.push_str(" temp_arr; ");
                } else {
                    self.output.push_str("struct { ");
                    self.output.push_str(&elem_c_type);
                    self.output.push_str("* data; size_t size; size_t capacity; } temp_arr; ");
                }
                self.output.push_str("temp_arr.size = ");
                self.output.push_str(&count.to_string());
                self.output.push_str("; temp_arr.capacity = ");
                if count > 0 { self.output.push_str(&count.to_string()); } else { self.output.push_str("4"); }
                self.output.push_str("; temp_arr.data = malloc(sizeof(");
                self.output.push_str(&elem_c_type);
                self.output.push_str(") * temp_arr.capacity); ");
                for (i, elem) in elements.iter().enumerate() {
                    self.output.push_str("temp_arr.data[");
                    self.output.push_str(&i.to_string());
                    self.output.push_str("] = ");
                    self.generate_expression(elem)?;
                    self.output.push_str("; ");
                }
                self.output.push_str("temp_arr; })");
            }
            Expression::New(expr) => {
                // Generate: ({ type* p = malloc(sizeof(type)); *p = value; p; })
                self.output.push_str("({ ");
                let c_type = self.infer_c_type(expr);
                self.output.push_str(&c_type);
                self.output.push_str("* p = malloc(sizeof(");
                self.output.push_str(&c_type);
                self.output.push_str(")); *p = ");
                self.generate_expression(expr)?;
                self.output.push_str("; p; })");
            }
            Expression::Delete(expr) => {
                self.output.push_str("free(");
                self.generate_expression(expr)?;
                self.output.push_str(")");
            }
            Expression::ArrayAccess { array, index } => {
                // If the array is a dynamic array, index its .data field; otherwise use [] directly
                let is_dyn = match self.expr_type(array) { Some(Type::DynamicArray(_)) => true, _ => false };
                if is_dyn {
                    self.output.push_str("(");
                    self.generate_expression(array)?;
                    self.output.push_str(").data[");
                    self.generate_expression(index)?;
                    self.output.push_str("]");
                } else {
                    self.generate_expression(array)?;
                    self.output.push_str("[");
                    self.generate_expression(index)?;
                    self.output.push_str("]");
                }
            }
            Expression::StructAccess { object, field } => {
                // Check if object needs parentheses (e.g., for dereference)
                let needs_parens = matches!(&**object, Expression::Unary { operator: UnaryOp::Dereference, .. });
                
                if needs_parens {
                    self.output.push_str("(");
                }
                self.generate_expression(object)?;
                if needs_parens {
                    self.output.push_str(")");
                }
                self.output.push_str(".");
                self.output.push_str(field);
            }
            Expression::StructLiteral { name, fields } => {
                // Generate: (Name){ .field = value, ... }
                self.output.push_str("(");
                self.output.push_str(name);
                self.output.push_str("){ ");
                for (i, (fname, fexpr)) in fields.iter().enumerate() {
                    if i > 0 { self.output.push_str(", "); }
                    self.output.push_str(".");
                    self.output.push_str(fname);
                    self.output.push_str(" = ");
                    self.generate_expression(fexpr)?;
                }
                self.output.push_str(" }");
            }
            Expression::Cast { expression, target_type } => {
                // Generate C cast: (target_type)expression
                self.output.push_str("(");
                self.output.push_str(&self.type_to_c(target_type));
                self.output.push_str(")");
                self.generate_expression(expression)?;
            }
            Expression::Ternary { condition, true_expr, false_expr } => {
                // Generate: (condition ? true_expr : false_expr)
                self.output.push_str("(");
                self.generate_expression(condition)?;
                self.output.push_str(" ? ");
                self.generate_expression(true_expr)?;
                self.output.push_str(" : ");
                self.generate_expression(false_expr)?;
                self.output.push_str(")");
            }
            Expression::EnumAccess { enum_name, variant } => {
                // Generate enum variant as: ENUM_VARIANT_NAME
                // We'll use ALL_CAPS naming convention for enum variants in C
                self.output.push_str(&format!("{}_{}", enum_name.to_uppercase(), variant.to_uppercase()));
            }
            Expression::Match { scrutinee, arms } => {
                use crate::ast::Pattern;
                
                // For match expressions, we need to use a GCC statement expression ({ ... })
                // or generate a temporary function. For simplicity, we'll use statement expressions.
                self.output.push_str("({\n");
                self.indent_level += 1;
                
                // Generate a temporary variable for the scrutinee
                let temp_var = format!("__match_temp_{}", self.temp_counter);
                self.temp_counter += 1;
                
                self.indent();
                let scrutinee_type = self.expr_type(scrutinee).unwrap_or(Type::Int);
                self.output.push_str(&self.type_to_c(&scrutinee_type));
                self.output.push_str(&format!(" {} = ", temp_var));
                self.generate_expression(scrutinee)?;
                self.output.push_str(";\n");
                
                // Determine result type from arms - try all arms until we find one with an inferable type
                let result_type = arms.iter()
                    .filter_map(|arm| self.expr_type(&arm.expression))
                    .next()
                    .unwrap_or(Type::Int); // Default to int if no arm has inferable type
                let result_var = format!("__match_result_{}", self.temp_counter);
                self.temp_counter += 1;
                
                self.indent();
                self.output.push_str(&self.type_to_c(&result_type));
                self.output.push_str(&format!(" {};\n", result_var));
                
                // Check if we can use a switch statement (int/enum/char types)
                // Note: Due to parser limitations, enums might be typed as Struct, so we check both
                // Also handle Generic types (Option, Result, etc.)
                let use_switch = matches!(scrutinee_type, Type::Int | Type::Enum(_) | Type::Struct(_) | Type::Char | Type::Generic { .. });
                
                if use_switch {
                    // Generate switch statement
                    self.indent();
                    // For generic types, switch on the tag field
                    if matches!(scrutinee_type, Type::Generic { .. }) {
                        self.output.push_str(&format!("switch ({}.tag) {{\n", temp_var));
                    } else {
                        self.output.push_str(&format!("switch ({}) {{\n", temp_var));
                    }
                    self.indent_level += 1;
                    
                    for arm in arms {
                        match &arm.pattern {
                            Pattern::Wildcard => {
                                self.indent();
                                self.output.push_str("default:\n");
                                self.indent_level += 1;
                                self.indent();
                                self.output.push_str(&format!("{} = ", result_var));
                                self.generate_expression(&arm.expression)?;
                                self.output.push_str(";\n");
                                self.indent();
                                self.output.push_str("break;\n");
                                self.indent_level -= 1;
                            }
                            Pattern::Literal(lit) => {
                                self.indent();
                                self.output.push_str("case ");
                                match lit {
                                    crate::ast::Literal::Integer(val) => self.output.push_str(&val.to_string()),
                                    crate::ast::Literal::Char(ch) => {
                                        // Properly escape special chars in case labels
                                        let esc: Option<&str> = match *ch {
                                            '\\' => Some("\\\\"),
                                            '\'' => Some("\\'"),
                                            '\n' => Some("\\n"),
                                            '\t' => Some("\\t"),
                                            '\r' => Some("\\r"),
                                            '\0' => Some("\\0"),
                                            _ => None,
                                        };
                                        self.output.push_str("'");
                                        if let Some(e) = esc {
                                            self.output.push_str(e);
                                        } else {
                                            self.output.push(*ch);
                                        }
                                        self.output.push_str("'");
                                    },
                                    _ => self.output.push_str("/* unsupported literal */"),
                                }
                                self.output.push_str(":\n");
                                self.indent_level += 1;
                                self.indent();
                                self.output.push_str(&format!("{} = ", result_var));
                                self.generate_expression(&arm.expression)?;
                                self.output.push_str(";\n");
                                self.indent();
                                self.output.push_str("break;\n");
                                self.indent_level -= 1;
                            }
                            Pattern::EnumVariant { enum_name, variant, binding } => {
                                self.indent();
                                self.output.push_str("case ");
                                // For generic types, generate mangled enum variant names
                                if matches!(scrutinee_type, Type::Generic { ref name, .. } if name == enum_name) {
                                    // Generate: Option_int_Some
                                    self.output.push_str(&self.type_to_c(&scrutinee_type));
                                    self.output.push_str("_");
                                    self.output.push_str(variant);
                                } else {
                                    // Regular enum: OPTION_SOME (uppercase)
                                    self.output.push_str(&format!("{}_{}", enum_name.to_uppercase(), variant.to_uppercase()));
                                }
                                self.output.push_str(": {\n");
                                self.indent_level += 1;
                                
                                // If there's a binding (and it's not a wildcard), extract the value from the union
                                if let Some(binding_name) = binding {
                                    if binding_name != "_" {
                                        self.indent();
                                        // Get the type of the bound value
                                        if let Type::Generic { ref type_params, .. } = scrutinee_type {
                                            if !type_params.is_empty() {
                                                let value_type = &type_params[0];
                                                self.output.push_str(&self.type_to_c(value_type));
                                                self.output.push_str(" ");
                                                self.output.push_str(binding_name);
                                                self.output.push_str(" = ");
                                                self.output.push_str(&temp_var);
                                                self.output.push_str(".data.");
                                                self.output.push_str(&format!("{}_value", variant.to_lowercase()));
                                                self.output.push_str(";\n");
                                            }
                                        }
                                    }
                                }
                                
                                self.indent();
                                self.output.push_str(&format!("{} = ", result_var));
                                self.generate_expression(&arm.expression)?;
                                self.output.push_str(";\n");
                                self.indent();
                                self.output.push_str("break;\n");
                                self.indent_level -= 1;
                                self.indent();
                                self.output.push_str("}\n");
                            }
                        }
                    }
                    
                    self.indent_level -= 1;
                    self.indent();
                    self.output.push_str("}\n");
                } else {
                    // Generate if-else chain for other types
                    let mut first = true;
                    for arm in arms {
                        match &arm.pattern {
                            Pattern::Wildcard => {
                                // Default case
                                if !first {
                                    self.output.push_str(" else ");
                                }
                                self.output.push_str("{\n");
                                self.indent_level += 1;
                                self.indent();
                                self.output.push_str(&format!("{} = ", result_var));
                                self.generate_expression(&arm.expression)?;
                                self.output.push_str(";\n");
                                self.indent_level -= 1;
                                self.indent();
                                self.output.push_str("}\n");
                            }
                            Pattern::Literal(lit) => {
                                self.indent();
                                if !first {
                                    self.output.push_str("else ");
                                }
                                self.output.push_str(&format!("if ({} == ", temp_var));
                                match lit {
                                    crate::ast::Literal::String(s) => {
                                        self.output.push_str(&format!("strcmp({}, \"{}\") == 0", temp_var, s));
                                    }
                                    crate::ast::Literal::Float(f) => {
                                        self.output.push_str(&f.to_string());
                                    }
                                    crate::ast::Literal::Bool(b) => {
                                        self.output.push_str(if *b { "1" } else { "0" });
                                    }
                                    _ => self.output.push_str("/* unsupported */"),
                                }
                                self.output.push_str(") {\n");
                                self.indent_level += 1;
                                self.indent();
                                self.output.push_str(&format!("{} = ", result_var));
                                self.generate_expression(&arm.expression)?;
                                self.output.push_str(";\n");
                                self.indent_level -= 1;
                                self.indent();
                                self.output.push_str("}\n");
                                first = false;
                            }
                            Pattern::EnumVariant { .. } => {
                                // Should not happen for non-int/enum types
                            }
                        }
                    }
                }
                
                // Return the result
                self.indent();
                self.output.push_str(&format!("{};\n", result_var));
                
                self.indent_level -= 1;
                self.indent();
                self.output.push_str("})");
            }
            Expression::TryOperator { expression } => {
                // Desugar expr? into a match expression
                // For Result<T, E>: match expr { Ok(v) => v, Err(e) => return Err(e) }
                // For Option<T>: match expr { Some(v) => v, None => return None }
                
                // Get the type of the expression to know if it's Result or Option
                let expr_type = self.expr_type(expression).unwrap_or(Type::Int);
                
                if let Type::Generic { name, type_params } = expr_type {
                    // Generate unique temp variable
                    let temp_var = format!("__try_temp_{}", self.temp_counter);
                    self.temp_counter += 1;
                    let result_var = format!("__try_result_{}", self.temp_counter);
                    self.temp_counter += 1;
                    
                    // Generate compound expression: ({ Result temp = expr; match temp { ... }; })
                    self.output.push_str("({\n");
                    self.indent_level += 1;
                    
                    // Temporary variable to hold the Result/Option value
                    self.indent();
                    self.output.push_str(&self.type_to_c(&Type::Generic { 
                        name: name.clone(), 
                        type_params: type_params.clone() 
                    }));
                    self.output.push_str(&format!(" {} = ", temp_var));
                    self.generate_expression(expression)?;
                    self.output.push_str(";\n");
                    
                    // Generate the match
                    if name == "Result" {
                        // Result<T, E> case
                        let ok_type = if !type_params.is_empty() { &type_params[0] } else { &Type::Int };
                        let _err_type = if type_params.len() > 1 { &type_params[1] } else { &Type::Int };
                        
                        let c_type = self.type_to_c(&Type::Generic { 
                            name: name.clone(), 
                            type_params: type_params.clone() 
                        });
                        
                        // Declare result variable
                        self.indent();
                        self.output.push_str(&self.type_to_c(ok_type));
                        self.output.push_str(&format!(" {};\n", result_var));
                        
                        // Switch on tag
                        self.indent();
                        self.output.push_str(&format!("switch ({}.tag) {{\n", temp_var));
                        self.indent_level += 1;
                        
                        // Ok case: extract and assign value
                        self.indent();
                        self.output.push_str(&format!("case {}_Ok: {{\n", c_type));
                        self.indent_level += 1;
                        self.indent();
                        self.output.push_str(&format!("{} = {}.data.ok_value;\n", result_var, temp_var));
                        self.indent();
                        self.output.push_str("break;\n");
                        self.indent_level -= 1;
                        self.indent();
                        self.output.push_str("}\n");
                        
                        // Err case: propagate error by returning
                        // Use the current function's return type, not the expression's type
                        self.indent();
                        self.output.push_str(&format!("case {}_Err: {{\n", c_type));
                        self.indent_level += 1;
                        self.indent();
                        
                        // Get the function's return type for the early return
                        let return_c_type = if let Some(ret_ty) = &self.current_return_type {
                            self.type_to_c(ret_ty)
                        } else {
                            c_type.clone()
                        };
                        
                        self.output.push_str(&format!("return (({}) {{ .tag = {}_Err, .data = {{ .err_value = {}.data.err_value }} }});\n",
                            return_c_type, return_c_type, temp_var));
                        self.indent_level -= 1;
                        self.indent();
                        self.output.push_str("}\n");
                        
                        self.indent_level -= 1;
                        self.indent();
                        self.output.push_str("}\n");
                        
                        // Return the unwrapped value
                        self.indent();
                        self.output.push_str(&format!("{};\n", result_var));
                        
                    } else if name == "Option" {
                        // Option<T> case
                        let some_type = if !type_params.is_empty() { &type_params[0] } else { &Type::Int };
                        
                        let c_type = self.type_to_c(&Type::Generic { 
                            name: name.clone(), 
                            type_params: type_params.clone() 
                        });
                        
                        // Declare result variable
                        self.indent();
                        self.output.push_str(&self.type_to_c(some_type));
                        self.output.push_str(&format!(" {};\n", result_var));
                        
                        // Switch on tag
                        self.indent();
                        self.output.push_str(&format!("switch ({}.tag) {{\n", temp_var));
                        self.indent_level += 1;
                        
                        // Some case: extract and assign value
                        self.indent();
                        self.output.push_str(&format!("case {}_Some: {{\n", c_type));
                        self.indent_level += 1;
                        self.indent();
                        self.output.push_str(&format!("{} = {}.data.some_value;\n", result_var, temp_var));
                        self.indent();
                        self.output.push_str("break;\n");
                        self.indent_level -= 1;
                        self.indent();
                        self.output.push_str("}\n");
                        
                        // None case: propagate by returning None
                        // Use the current function's return type
                        self.indent();
                        self.output.push_str(&format!("case {}_None: {{\n", c_type));
                        self.indent_level += 1;
                        self.indent();
                        
                        // Get the function's return type for the early return
                        let return_c_type = if let Some(ret_ty) = &self.current_return_type {
                            self.type_to_c(ret_ty)
                        } else {
                            c_type.clone()
                        };
                        
                        self.output.push_str(&format!("return (({}) {{ .tag = {}_None }});\n", return_c_type, return_c_type));
                        self.indent_level -= 1;
                        self.indent();
                        self.output.push_str("}\n");
                        
                        self.indent_level -= 1;
                        self.indent();
                        self.output.push_str("}\n");
                        
                        // Return the unwrapped value
                        self.indent();
                        self.output.push_str(&format!("{};\n", result_var));
                    } else {
                        self.output.push_str("/* ? operator on unsupported type */");
                    }
                    
                    self.indent_level -= 1;
                    self.indent();
                    self.output.push_str("})");
                } else {
                    self.output.push_str("/* ? operator requires Result or Option */");
                }
            }
            Expression::MethodCall { object, method, arguments } => {
                // Method call: object.method(args)
                // Handle string methods and dynamic array methods
                let mut obj_type = self.expr_type(object).unwrap_or(Type::Int);
                
                // Normalize str to String type
                if let Type::Struct(ref name) = obj_type {
                    if name == "str" {
                        obj_type = Type::String;
                    }
                }
                
                match (&obj_type, method.as_str()) {
                    // String methods
                    (&Type::String, "length") => {
                        // string.length() -> strlen(string)
                        self.output.push_str("strlen(");
                        self.generate_expression(object)?;
                        self.output.push_str(")");
                    }
                    (&Type::String, "substring") => {
                        // string.substring(start, end) -> rapter_substring(string, start, end)
                        self.output.push_str("rapter_substring(");
                        self.generate_expression(object)?;
                        self.output.push_str(", ");
                        self.generate_expression(&arguments[0])?;
                        self.output.push_str(", ");
                        self.generate_expression(&arguments[1])?;
                        self.output.push_str(")");
                    }
                    (&Type::String, "contains") => {
                        // string.contains(needle) -> (strstr(string, needle) != NULL ? 1 : 0)
                        self.output.push_str("(strstr(");
                        self.generate_expression(object)?;
                        self.output.push_str(", ");
                        self.generate_expression(&arguments[0])?;
                        self.output.push_str(") != NULL ? 1 : 0)");
                    }
                    (&Type::String, "trim") => {
                        // string.trim() -> rapter_trim(string)
                        self.output.push_str("rapter_trim(");
                        self.generate_expression(object)?;
                        self.output.push_str(")");
                    }
                    (&Type::String, "split") => {
                        // string.split(delimiter) -> rapter_split(string, delimiter)
                        // Returns DynamicArray_charptr (array of strings)
                        self.output.push_str("rapter_split(");
                        self.generate_expression(object)?;
                        self.output.push_str(", ");
                        self.generate_expression(&arguments[0])?;
                        self.output.push_str(")");
                    }
                    // Dynamic array methods
                    (&Type::DynamicArray(_), "push") => {
                        // Convert to old-style struct access call for compatibility
                        if let Expression::Variable(obj_name) = &**object {
                            if arguments.len() == 1 {
                                self.output.push_str("({ ");
                                self.output.push_str("if (");
                                self.output.push_str(obj_name);
                                self.output.push_str(".size == ");
                                self.output.push_str(obj_name);
                                self.output.push_str(".capacity) { size_t new_cap = ");
                                self.output.push_str(obj_name);
                                self.output.push_str(".capacity ? ");
                                self.output.push_str(obj_name);
                                self.output.push_str(".capacity * 2 : 4; ");
                                self.output.push_str(obj_name);
                                self.output.push_str(".data = realloc(");
                                self.output.push_str(obj_name);
                                self.output.push_str(".data, new_cap * sizeof(");
                                self.output.push_str(obj_name);
                                self.output.push_str(".data[0])); ");
                                self.output.push_str(obj_name);
                                self.output.push_str(".capacity = new_cap; } ");
                                self.output.push_str(obj_name);
                                self.output.push_str(".data[");
                                self.output.push_str(obj_name);
                                self.output.push_str(".size++] = ");
                                self.generate_expression(&arguments[0])?;
                                self.output.push_str("; })");
                            } else {
                                self.output.push_str("/* push expects 1 argument */");
                            }
                        } else {
                            self.output.push_str("/* method calls on non-variables not supported */");
                        }
                    }
                    (&Type::DynamicArray(_), "pop") => {
                        if let Expression::Variable(obj_name) = &**object {
                            if arguments.is_empty() {
                                self.output.push_str("(");
                                self.output.push_str(obj_name);
                                self.output.push_str(".size > 0 ? ");
                                self.output.push_str(obj_name);
                                self.output.push_str(".data[--");
                                self.output.push_str(obj_name);
                                self.output.push_str(".size] : 0)");
                            } else {
                                self.output.push_str("/* pop expects no arguments */");
                            }
                        } else {
                            self.output.push_str("/* method calls on non-variables not supported */");
                        }
                    }
                    (&Type::DynamicArray(_), "length") => {
                        if let Expression::Variable(obj_name) = &**object {
                            if arguments.is_empty() {
                                self.output.push_str("(");
                                self.output.push_str(obj_name);
                                self.output.push_str(".size)");
                            } else {
                                self.output.push_str("/* length expects no arguments */");
                            }
                        } else {
                            self.output.push_str("/* method calls on non-variables not supported */");
                        }
                    }
                    _ => {
                        self.output.push_str("/* method not supported: ");
                        self.output.push_str(method);
                        self.output.push_str(" on ");
                        self.output.push_str(&format!("{:?}", obj_type));
                        self.output.push_str(" */");
                    }
                }
            }
            Expression::Range { start: _, end: _ } => {
                // Ranges are handled in for loops, not directly generated
                self.output.push_str("/* range not directly supported */");
            }
        }
        Ok(())
    }
    
    fn generate_main_wrapper(&mut self) -> Result<(), CompilerError> {
        self.output.push_str("int main(int argc, char* argv[]) {\n");
        self.indent_level += 1;
        self.indent();
        self.output.push_str("__rapter_argc = argc; __rapter_argv = argv;\n");
        self.indent();
        
        // Check if main returns void
        let main_return_type = self.func_types.get("main").cloned().unwrap_or(Type::Void);
        if main_return_type == Type::Void {
            self.output.push_str("rapter_main(argc, argv);\n");
            self.indent();
            self.output.push_str("return 0;\n");
        } else {
            self.output.push_str("return rapter_main(argc, argv);\n");
        }
        
        self.indent_level -= 1;
        self.output.push_str("}\n");
        Ok(())
    }
    
    fn type_to_c(&self, ty: &Type) -> String {
        match ty {
            Type::Int => "int".to_string(),
            Type::Float => "double".to_string(),
            Type::Bool => "int".to_string(), // C doesn't have bool, use int
            Type::Char => "char".to_string(),
            Type::String => "char*".to_string(),
            Type::Array(elem_ty) => format!("{}*", self.type_to_c(elem_ty)),
            Type::DynamicArray(elem_ty) => match &**elem_ty {
                Type::Int => "DynamicArray_int".to_string(),
                Type::Float => "DynamicArray_double".to_string(),
                Type::Char => "DynamicArray_char".to_string(),
                Type::String => "DynamicArray_charptr".to_string(),
                Type::Struct(name) if name == "str" => "DynamicArray_charptr".to_string(),
                Type::Struct(name) => format!("DynamicArray_{}", name),
                _ => format!("struct {{ {}* data; size_t size; size_t capacity; }}", self.type_to_c(elem_ty)),
            },
            Type::Pointer(pointee) => format!("{}*", self.type_to_c(pointee)),
            Type::Struct(name) => {
                // Special case: "str" should be char* in C
                if name == "str" {
                    "char*".to_string()
                } else {
                    name.clone()
                }
            },
            Type::Enum(_) => "int".to_string(), // Enums are represented as ints in C
            Type::Void => "void".to_string(),
            // Generic types are monomorphized: Option<int> -> Option_int
            Type::Generic { name, type_params } => {
                let param_names: Vec<String> = type_params.iter()
                    .map(|t| self.type_to_mangled_name(t))
                    .collect();
                format!("{}_{}", name, param_names.join("_"))
            },
            // Type parameters should have been substituted by monomorphization
            Type::TypeParam(name) => {
                panic!("Type parameter '{}' not substituted during monomorphization", name)
            },
        }
    }

    // Helper to create mangled names for generic type parameters
    fn type_to_mangled_name(&self, ty: &Type) -> String {
        match ty {
            Type::Int => "int".to_string(),
            Type::Float => "float".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Char => "char".to_string(),
            Type::String => "string".to_string(),
            Type::Pointer(inner) => format!("ptr_{}", self.type_to_mangled_name(inner)),
            Type::Struct(name) => {
                // Special case: "str" becomes "string" in mangled names
                if name == "str" {
                    "string".to_string()
                } else {
                    name.clone()
                }
            },
            Type::Enum(name) => name.clone(),
            Type::Array(elem) => format!("arr_{}", self.type_to_mangled_name(elem)),
            Type::DynamicArray(elem) => format!("vec_{}", self.type_to_mangled_name(elem)),
            Type::Void => "void".to_string(),
            Type::Generic { name, type_params } => {
                let params: Vec<String> = type_params.iter()
                    .map(|t| self.type_to_mangled_name(t))
                    .collect();
                format!("{}_{}", name, params.join("_"))
            },
            Type::TypeParam(name) => name.clone(), // Keep type param name for mangling
        }
    }
    
    fn infer_c_type(&self, expr: &Expression) -> String {
        // Simple type inference for literals
        match expr {
            Expression::Literal(Literal::Integer(_)) => "int".to_string(),
            Expression::Literal(Literal::Float(_)) => "double".to_string(),
            Expression::Literal(Literal::Bool(_)) => "int".to_string(),
            Expression::Literal(Literal::Char(_)) => "char".to_string(),
            Expression::Literal(Literal::String(_)) => "char*".to_string(),
            Expression::ArrayLiteral(_) => "int*".to_string(), // arrays decay to pointers
            Expression::DynamicArrayLiteral { element_type, .. } => {
                // Return the typedef name for dynamic arrays
                match &**element_type {
                    Type::Int => "DynamicArray_int".to_string(),
                    Type::Float => "DynamicArray_double".to_string(),
                    Type::Char => "DynamicArray_char".to_string(),
                    Type::String => "DynamicArray_charptr".to_string(),
                    Type::Struct(name) if name == "str" => "DynamicArray_charptr".to_string(),
                    Type::Struct(name) => format!("DynamicArray_{}", name),
                    _ => format!("struct {{ {}* data; size_t size; size_t capacity; }}", self.type_to_c(element_type)), // fallback
                }
            }
            Expression::New(inner) => format!("{}*", self.infer_c_type(inner)), // new returns a pointer to inner
            Expression::Call { callee, .. } => {
                // Try to resolve function return type
                if let Expression::Variable(name) = &**callee {
                    return self.type_to_c(self.func_types.get(name).unwrap_or(&Type::Int));
                } else if let Expression::StructAccess { field, .. } = &**callee {
                    return self.type_to_c(self.func_types.get(field).unwrap_or(&Type::Int));
                }
                "int".to_string()
            }
            Expression::ArrayAccess { array, .. } => {
                // Infer element C type for arrays and dynamic arrays
                if let Some(ty) = self.expr_type(array) {
                    match ty {
                        Type::Array(elem) | Type::DynamicArray(elem) | Type::Pointer(elem) => {
                            return self.type_to_c(&elem);
                        }
                        _ => {}
                    }
                }
                "int".to_string()
            }
            Expression::StructLiteral { name, .. } => {
                // Struct literals should use the struct type
                name.clone()
            }
            _ => {
                // Try to infer from expr_type as fallback
                if let Some(ty) = self.expr_type(expr) {
                    self.type_to_c(&ty)
                } else {
                    "int".to_string() // Last resort default
                }
            }
        }
    }
    
    fn infer_printf_format(&self, expr: &Expression) -> String {
        match expr {
            Expression::Literal(Literal::Integer(_)) => "%d".to_string(),
            Expression::Literal(Literal::Float(_)) => "%f".to_string(),
            Expression::Literal(Literal::Bool(_)) => "%d".to_string(),
            Expression::Literal(Literal::Char(_)) => "%c".to_string(),
            Expression::Literal(Literal::String(_)) => "%s".to_string(),
            Expression::Variable(name) => {
                if let Some(ty) = self.get_var_type(name) {
                    return match ty {
                        Type::Int | Type::Bool | Type::Enum(_) | Type::Pointer(_) => "%d".to_string(),
                        Type::Float => "%f".to_string(),
                        Type::Char => "%c".to_string(),
                        Type::String => "%s".to_string(),
                        Type::Array(_) | Type::DynamicArray(_) | Type::Struct(_) | Type::Void => "%d".to_string(),
                        Type::Generic { .. } => "%d".to_string(), // Generic types default to %d for now
                        Type::TypeParam(_) => "%d".to_string(),   // Type params default to %d for now
                    };
                }
                "%d".to_string()
            }
            Expression::Binary { left, operator, right } => {
                // Special case: if this is string concatenation, the result is a string
                if *operator == BinaryOp::Add && (self.contains_string_literal(left) || self.contains_string_literal(right)) {
                    "%s".to_string()
                } else {
                    "%d".to_string() // Default to int for other binary operations
                }
            }
            Expression::ArrayAccess { array, .. } => {
                // If we can infer element type, use it
                if let Some(ty) = self.expr_type(array) {
                    if let Type::DynamicArray(elem) | Type::Array(elem) = ty {
                        return match *elem {
                            Type::Int | Type::Bool | Type::Pointer(_) => "%d".to_string(),
                            Type::Float => "%f".to_string(),
                            Type::Char => "%c".to_string(),
                            Type::String => "%s".to_string(),
                            _ => "%d".to_string(),
                        };
                    }
                }
                "%d".to_string()
            }
            _ => "%d".to_string(), // Default fallback
        }
    }
    
    fn is_array_expression(&self, expr: &Expression) -> bool {
        match expr {
            Expression::ArrayLiteral(_) => true,
            Expression::DynamicArrayLiteral { .. } => true,
            Expression::Variable(var_name) => {
                // Use type info when available
                if let Some(ty) = self.get_var_type(var_name) {
                    matches!(ty, Type::Array(_) | Type::DynamicArray(_))
                } else { false }
            }
            _ => false,
        }
    }
    
    fn contains_string_literal(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Literal(Literal::String(_)) => true,
            Expression::Binary { left, operator: _, right } => {
                self.contains_string_literal(left) || self.contains_string_literal(right)
            }
            _ => false,
        }
    }
    
    fn generate_array_print(&mut self, expr: &Expression, add_newline: bool) -> Result<(), CompilerError> {
        match expr {
            Expression::ArrayLiteral(elements) => {
                // For array literals, we know the size at compile time
                self.output.push_str("printf(\"[\");\n");
                self.indent();
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str("printf(\", \");\n");
                        self.indent();
                    }
                    let format_spec = self.infer_printf_format(elem);
                    self.output.push_str("printf(\"");
                    self.output.push_str(&format_spec);
                    self.output.push_str("\"");
                    self.output.push_str(", ");
                    self.generate_expression(elem)?;
                    self.output.push_str(");\n");
                    self.indent();
                }
                self.output.push_str("printf(\"");
                self.output.push_str("]");
                if add_newline {
                    self.output.push_str("\\n");
                }
                self.output.push_str("\");");
            }
            Expression::Variable(var_name) => {
                // Attempt to print a dynamic array variable by iterating over its size
                if let Some(Type::DynamicArray(elem_ty)) = self.get_var_type(var_name) {
                    let elem_format = match &*elem_ty {
                        Type::Int => "%d",
                        Type::Float => "%f",
                        Type::Bool => "%d",
                        Type::Char => "%c",
                        Type::String => "%s",
                        _ => "%d",
                    };
                    self.output.push_str("printf(\"[\");\n");
                    self.indent();
                    self.output.push_str("for (size_t i = 0; i < ");
                    self.output.push_str(var_name);
                    self.output.push_str(".size; i++) {\n");
                    self.indent_level += 1;
                    self.indent();
                    self.output.push_str("if (i > 0) printf(\", \");\n");
                    self.indent();
                    self.output.push_str("printf(\"");
                    self.output.push_str(elem_format);
                    self.output.push_str("\", ");
                    self.output.push_str(var_name);
                    self.output.push_str(".data[i]);\n");
                    self.indent_level -= 1;
                    self.indent();
                    self.output.push_str("}\n");
                    self.indent();
                    self.output.push_str("printf(\"");
                    self.output.push_str("]");
                    if add_newline { self.output.push_str("\\n"); }
                    self.output.push_str("\");\n");
                } else {
                    // Unknown or non-array variable
                    self.output.push_str("printf(\"[array]\")");
                    if add_newline { self.output.push_str(";\n"); self.indent(); self.output.push_str("printf(\"\\n\")"); }
                    self.output.push_str(";");
                }
            }
            Expression::DynamicArrayLiteral { element_type, elements: _ } => {
                // For dynamic arrays, use the size field
                let elem_format = match &**element_type {
                    Type::Int => "%d",
                    Type::Float => "%f",
                    Type::Bool => "%d",
                    Type::Char => "%c",
                    _ => "%d", // fallback
                };
                
                // Generate a temporary variable to hold the array
                self.output.push_str("{\n");
                self.indent_level += 1;
                self.indent();
                self.output.push_str("DynamicArray_");
                match &**element_type {
                    Type::Int => self.output.push_str("int"),
                    Type::Float => self.output.push_str("double"),
                    Type::Char => self.output.push_str("char"),
                    _ => self.output.push_str("generic"),
                }
                self.output.push_str(" temp_arr = ");
                self.generate_expression(expr)?;
                self.output.push_str(";\n");
                self.indent();
                self.output.push_str("printf(\"[\");\n");
                self.indent();
                self.output.push_str("for (size_t i = 0; i < temp_arr.size; i++) {\n");
                self.indent_level += 1;
                self.indent();
                self.output.push_str("if (i > 0) printf(\", \");\n");
                self.indent();
                self.output.push_str("printf(\"");
                self.output.push_str(elem_format);
                self.output.push_str("\", temp_arr.data[i]);\n");
                self.indent_level -= 1;
                self.indent();
                self.output.push_str("}\n");
                self.indent();
                self.output.push_str("printf(\"");
                self.output.push_str("]");
                if add_newline {
                    self.output.push_str("\\n");
                }
                self.output.push_str("\");\n");
                self.indent_level -= 1;
                self.indent();
                self.output.push_str("}");
            }
            _ => {
                // Fallback - shouldn't happen if is_array_expression is correct
                self.output.push_str("printf(\"[array]\")");
                if add_newline {
                    self.output.push_str(";\n");
                    self.indent();
                    self.output.push_str("printf(\"\\n\")");
                }
                self.output.push_str(";");
            }
        }
        Ok(())
    }
    
    fn generate_string_concatenation(&mut self, left: &Expression, right: &Expression) -> Result<(), CompilerError> {
        // Generate: ({ char* result = malloc(strlen(left) + strlen(right) + 1); strcpy(result, left); strcat(result, right); result; })
        self.output.push_str("({");
        self.output.push_str("char* result = malloc(strlen(");
        self.generate_expression(left)?;
        self.output.push_str(") + strlen(");
        self.generate_expression(right)?;
        self.output.push_str(") + 1); ");
        self.output.push_str("strcpy(result, ");
        self.generate_expression(left)?;
        self.output.push_str("); ");
        self.output.push_str("strcat(result, ");
        self.generate_expression(right)?;
        self.output.push_str("); ");
        self.output.push_str("result;");
        self.output.push_str("})");
        Ok(())
    }
    
    pub fn get_output(&self) -> &str {
        &self.output
    }
    
    pub fn write_to_file(&self, filename: &str) -> Result<(), CompilerError> {
        std::fs::write(filename, &self.output)
            .map_err(|e| {
                let location = SourceLocation::new(PathBuf::from("input.rap"), 0, 0);
                CompilerError::new(
                    ErrorKind::InternalError,
                    format!("failed to write output file '{}': {}", filename, e),
                    location,
                )
            })
    }
}

pub fn generate(ast: &Program, resolver: &mut ModuleResolver, output_file: Option<&str>) -> Result<(), CompilerError> {
    let mut generator = CCodeGenerator::new();
    generator.generate(ast, resolver, &PathBuf::from("input.rap"))?;
    
    let output_path = output_file.unwrap_or("output.c");
    generator.write_to_file(output_path)?;
    
    // Only print to stdout if no output file specified
    if output_file.is_none() {
        println!("Generated C code:");
        println!("{}", generator.get_output());
    }
    
    Ok(())
}

// Helper methods for type-aware codegen
impl CCodeGenerator {
    fn enter_scope(&mut self) { self.var_types.push(HashMap::new()); }
    fn exit_scope(&mut self) { self.var_types.pop(); }
    fn set_var_type(&mut self, name: &str, ty: Type) {
        if let Some(scope) = self.var_types.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }
    fn get_var_type(&self, name: &str) -> Option<Type> {
        for scope in self.var_types.iter().rev() {
            if let Some(t) = scope.get(name) { return Some(t.clone()); }
        }
        None
    }
    fn expr_type(&self, expr: &Expression) -> Option<Type> {
        match expr {
            Expression::Literal(Literal::Integer(_)) => Some(Type::Int),
            Expression::Literal(Literal::Float(_)) => Some(Type::Float),
            Expression::Literal(Literal::Bool(_)) => Some(Type::Bool),
            Expression::Literal(Literal::Char(_)) => Some(Type::Char),
            Expression::Literal(Literal::String(_)) => Some(Type::String),
            Expression::Variable(name) => self.get_var_type(name),
            Expression::Unary { operator, operand } => match operator {
                UnaryOp::Dereference => {
                    if let Some(Type::Pointer(inner)) = self.expr_type(operand) { Some(*inner) } else { None }
                }
                UnaryOp::AddressOf => {
                    self.expr_type(operand).map(|t| Type::Pointer(Box::new(t)))
                }
                _ => self.expr_type(operand),
            },
            Expression::ArrayAccess { array, .. } => {
                match self.expr_type(array) {
                    Some(Type::Array(elem)) | Some(Type::DynamicArray(elem)) | Some(Type::Pointer(elem)) => Some(*elem),
                    _ => None,
                }
            }
            Expression::DynamicArrayLiteral { element_type, .. } => Some(Type::DynamicArray(element_type.clone())),
            Expression::ArrayLiteral(_) => None,
            Expression::StructAccess { .. } => None,
            Expression::StructLiteral { name, .. } => Some(Type::Struct(name.clone())),
            Expression::Binary { .. } => None,
            Expression::Call { callee, .. } => {
                if let Expression::Variable(name) = &**callee {
                    self.func_types.get(name).cloned()
                } else if let Expression::StructAccess { field, .. } = &**callee {
                    self.func_types.get(field).cloned()
                } else { None }
            }
            Expression::New(inner) => self.expr_type(inner).map(|t| Type::Pointer(Box::new(t))),
            Expression::Delete(_) => Some(Type::Void),
            Expression::Cast { target_type, .. } => Some(target_type.clone()),
            Expression::Ternary { true_expr, .. } => self.expr_type(true_expr),
            Expression::EnumAccess { enum_name, .. } => Some(Type::Enum(enum_name.clone())),
            Expression::Match { arms, .. } => {
                // Return type of first arm (all arms have compatible types)
                if !arms.is_empty() {
                    self.expr_type(&arms[0].expression)
                } else {
                    None
                }
            }
            Expression::TryOperator { expression } => {
                // ? operator unwraps Result<T, E> to T, or Option<T> to T
                if let Some(Type::Generic { type_params, .. }) = self.expr_type(expression) {
                    if !type_params.is_empty() {
                        Some(type_params[0].clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Expression::MethodCall { object, method, .. } => {
                // Return type based on method
                let mut obj_type = self.expr_type(object)?;
                
                // Normalize str to String type
                if let Type::Struct(ref name) = obj_type {
                    if name == "str" {
                        obj_type = Type::String;
                    }
                }
                
                match (&obj_type, method.as_str()) {
                    // String methods
                    (&Type::String, "length") => Some(Type::Int),
                    (&Type::String, "substring") => Some(Type::String),
                    (&Type::String, "contains") => Some(Type::Bool),
                    (&Type::String, "trim") => Some(Type::String),
                    (&Type::String, "split") => Some(Type::DynamicArray(Box::new(Type::String))),
                    // Dynamic array methods
                    (&Type::DynamicArray(_), "length") => Some(Type::Int),
                    (&Type::DynamicArray(ref elem_ty), "pop") => Some(*elem_ty.clone()),
                    (&Type::DynamicArray(_), "push") => Some(Type::Void),
                    _ => None,
                }
            }
            Expression::Range { .. } => Some(Type::Void),
        }
    }
}