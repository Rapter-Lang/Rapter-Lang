use crate::ast::*;
use crate::error::{CompilerError, ErrorKind, SourceLocation, Suggestion, type_mismatch, undefined_variable, duplicate_definition};
use crate::modules::{Symbol as ModuleSymbol, SymbolType as ModuleSymbolType};
use crate::builtins::BuiltinRegistry;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Function,
    Struct,
    Enum,
    Variable,
    Parameter,
}

pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
    // Map of struct name -> map of field name -> field type
    struct_defs: HashMap<String, HashMap<String, Type>>,
    // Map of enum name -> map of variant name -> variant value
    enum_defs: HashMap<String, HashMap<String, i64>>,
    // Built-in generic types (Option, Result, etc.)
    builtins: BuiltinRegistry,
    // Track current function's return type for ? operator validation
    current_function_return_type: Option<Type>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            scopes: vec![HashMap::new()],
            struct_defs: HashMap::new(),
            enum_defs: HashMap::new(),
            builtins: BuiltinRegistry::new(),
            current_function_return_type: None,
        }
    }
    
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    
    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }
    
    pub fn insert(&mut self, symbol: Symbol, file_path: &PathBuf) -> Result<(), CompilerError> {
        let name = symbol.name.clone();
        if self.scopes.last_mut().unwrap().contains_key(&name) {
            // For duplicate definition, we need the location of the previous definition
            // For now, we'll use a dummy location since we don't track locations in the symbol table
            let location = SourceLocation::new(file_path.clone(), 0, 0);
            return Err(duplicate_definition(&name, location.clone(), location));
        }
        self.scopes.last_mut().unwrap().insert(name, symbol);
        Ok(())
    }
    
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    pub fn insert_struct_def(&mut self, st: &Struct) {
        let mut fields_map = HashMap::new();
        for f in &st.fields {
            fields_map.insert(f.name.clone(), f.field_type.clone());
        }
        self.struct_defs.insert(st.name.clone(), fields_map);
    }

    pub fn get_struct_field_type(&self, struct_name: &str, field_name: &str) -> Option<&Type> {
        self.struct_defs
            .get(struct_name)
            .and_then(|m| m.get(field_name))
    }
    
    pub fn insert_enum_def(&mut self, enm: &Enum) {
        let mut variants_map = HashMap::new();
        for v in &enm.variants {
            if let Some(val) = v.value {
                variants_map.insert(v.name.clone(), val);
            }
        }
        self.enum_defs.insert(enm.name.clone(), variants_map);
    }
    
    pub fn get_enum_variant_value(&self, enum_name: &str, variant_name: &str) -> Option<&i64> {
        self.enum_defs
            .get(enum_name)
            .and_then(|m| m.get(variant_name))
    }
}

pub fn analyze(ast: &Program) -> Result<(), CompilerError> {
    analyze_with_imports(ast, &HashMap::new(), &PathBuf::from("<unknown>"))
}

pub fn analyze_with_imports(ast: &Program, imported_symbols: &HashMap<String, ModuleSymbol>, file_path: &PathBuf) -> Result<(), CompilerError> {
    let mut symbol_table = SymbolTable::new();
    
    // Add imported symbols to the symbol table
    // Note: ModuleResolver already provides both qualified and unqualified names
    for (name, symbol) in imported_symbols {
        let local_symbol = Symbol {
            name: name.clone(),
            symbol_type: match symbol.symbol_type {
                ModuleSymbolType::Function => SymbolType::Function,
                ModuleSymbolType::Struct => SymbolType::Struct,
                ModuleSymbolType::Enum => SymbolType::Enum,
                ModuleSymbolType::Variable => SymbolType::Variable,
                ModuleSymbolType::Parameter => SymbolType::Parameter,
            },
            ty: symbol.ty.clone(),
        };
        symbol_table.insert(local_symbol, file_path)?;
        
        // If this is an imported struct with fields metadata, register the fields
        if let ModuleSymbolType::Struct = symbol.symbol_type {
            if let Some(fields_map) = &symbol.fields {
                let struct_name = match &symbol.ty {
                    Type::Struct(n) => n.clone(),
                    _ => name.clone(),
                };
                symbol_table.struct_defs.insert(struct_name.clone(), fields_map.clone());
            }
        }
        
        // If this is an imported enum with variants metadata, register the variants
        if let ModuleSymbolType::Enum = symbol.symbol_type {
            if let Some(variants_map) = &symbol.fields {
                let enum_name = match &symbol.ty {
                    Type::Enum(n) => n.clone(),
                    _ => name.clone(),
                };
                // Register variant names with placeholder values (0)
                // The actual values don't matter for validation, only that the variants exist
                // The C compiler will use the actual enum values from the typedef
                let mut variants_with_values = HashMap::new();
                for (variant_name, _) in variants_map {
                    variants_with_values.insert(variant_name.clone(), 0i64);
                }
                symbol_table.enum_defs.insert(enum_name.clone(), variants_with_values);
            }
        }
    }
    
    // First pass: collect global declarations
    for ext_func in &ast.extern_functions {
        let symbol = Symbol {
            name: ext_func.name.clone(),
            symbol_type: SymbolType::Function,
            ty: ext_func.return_type.clone().unwrap_or(Type::Void),
        };
        symbol_table.insert(symbol, file_path)?;
    }
    
    for func in &ast.functions {
        let symbol = Symbol {
            name: func.name.clone(),
            symbol_type: SymbolType::Function,
            ty: func.return_type.clone().unwrap_or(Type::Void),
        };
        symbol_table.insert(symbol, file_path)?;
    }
    
    for st in &ast.structs {
        let symbol = Symbol {
            name: st.name.clone(),
            symbol_type: SymbolType::Struct,
            ty: Type::Struct(st.name.clone()),
        };
        symbol_table.insert(symbol, file_path)?;
        // record struct fields for semantic checks
        symbol_table.insert_struct_def(st);
    }
    
    for enm in &ast.enums {
        let symbol = Symbol {
            name: enm.name.clone(),
            symbol_type: SymbolType::Enum,
            ty: Type::Enum(enm.name.clone()),
        };
        symbol_table.insert(symbol, file_path)?;
        // record enum variants for semantic checks
        symbol_table.insert_enum_def(enm);
    }
    
    // Add global variables to symbol table
    for global_var in &ast.global_variables {
        let ty = if let Some(t) = &global_var.var_type {
            t.clone()
        } else if let Some(expr) = &global_var.initializer {
            infer_type(expr, &mut symbol_table, file_path)?
        } else {
            let location = SourceLocation::new(file_path.clone(), 0, 0);
            return Err(CompilerError::new(
                ErrorKind::InvalidSyntax,
                format!("global variable `{}` must have type annotation or initializer", global_var.name),
                location,
            ).with_suggestion(Suggestion::simple(
                "add a type annotation like `: int` or provide an initializer expression"
            )));
        };
        
        // Validate initializer if present
        if let Some(init) = &global_var.initializer {
            let init_ty = infer_type(init, &mut symbol_table, file_path)?;
            if !types_compatible(&ty, &init_ty) {
                let location = SourceLocation::new(file_path.clone(), 0, 0);
                return Err(CompilerError::new(
                    ErrorKind::TypeMismatch,
                    format!("global variable `{}` has type `{:?}` but initializer has type `{:?}`", 
                            global_var.name, ty, init_ty),
                    location,
                ));
            }
        }
        
        let symbol = Symbol {
            name: global_var.name.clone(),
            symbol_type: SymbolType::Variable,
            ty,
        };
        symbol_table.insert(symbol, file_path)?;
    }
    
    // Second pass: analyze function bodies
    for func in &ast.functions {
        analyze_function(func, &mut symbol_table, file_path)?;
    }
    
    Ok(())
}

fn analyze_function(func: &Function, symbol_table: &mut SymbolTable, file_path: &PathBuf) -> Result<(), CompilerError> {
    symbol_table.enter_scope();
    
    // Set current function return type for ? operator validation
    let expected_ret = func.return_type.clone().unwrap_or(Type::Void);
    symbol_table.current_function_return_type = Some(expected_ret.clone());
    
    // Add parameters to scope
    for param in &func.parameters {
        let symbol = Symbol {
            name: param.name.clone(),
            symbol_type: SymbolType::Parameter,
            ty: param.param_type.clone(),
        };
        symbol_table.insert(symbol, file_path)?;
    }
    
    // Analyze body
    for stmt in &func.body {
        analyze_statement(stmt, symbol_table, file_path, SourceLocation::new(file_path.clone(), 1, 1), &expected_ret)?;
    }
    // If function is non-void, ensure all paths return
    if expected_ret != Type::Void {
        if !block_returns(&func.body, symbol_table, file_path)? {
            let location = SourceLocation::new(file_path.clone(), 0, 0);
            return Err(CompilerError::new(
                ErrorKind::MissingReturnType,
                format!("function `{}` is declared to return `{:?}` but not all paths return a value", func.name, expected_ret),
                location,
            ).with_suggestion(Suggestion::simple(
                "ensure every execution path returns a value of the declared type"
            )));
        }
    }
    
    // Clear current function return type
    symbol_table.current_function_return_type = None;
    symbol_table.exit_scope();
    Ok(())
}

fn analyze_statement(stmt: &Statement, symbol_table: &mut SymbolTable, file_path: &PathBuf, stmt_location: SourceLocation, expected_return: &Type) -> Result<(), CompilerError> {
    match stmt {
        Statement::Let { name, var_type, mutable: _, initializer } => {
            let ty = if let Some(t) = var_type {
                t.clone()
            } else if let Some(expr) = initializer {
                infer_type(expr, symbol_table, file_path)?
            } else {
                return Err(CompilerError::new(
                    ErrorKind::InvalidSyntax,
                    "variable must have type annotation or initializer".to_string(),
                    stmt_location,
                ).with_suggestion(Suggestion::simple(
                    "add a type annotation like `: int` or provide an initializer expression"
                )));
            };
            
            if let Some(init) = initializer {
                // Special case: empty array literal with type annotation is allowed
                if let (Expression::ArrayLiteral(elements), Type::Array(_)) = (init, &ty) {
                    if elements.is_empty() {
                        // Empty array literal with type annotation is OK
                    } else {
                        // Check element types for non-empty arrays
                        let init_ty = infer_type(init, symbol_table, file_path)?;
                        if !types_compatible(&ty, &init_ty) {
                            return Err(type_mismatch(&format!("{:?}", ty), &format!("{:?}", init_ty), stmt_location)
                                .with_suggestion(Suggestion::simple(
                                    "ensure all array elements have the same type as the declared array type"
                                )));
                        }
                    }
                } else {
                    // Use type-aware inference when we have a type annotation
                    let init_ty = infer_type_with_hint(init, var_type.as_ref(), symbol_table, file_path)?;
                    if !types_compatible(&ty, &init_ty) {
                        return Err(type_mismatch(&format!("{:?}", ty), &format!("{:?}", init_ty), stmt_location)
                            .with_suggestion(Suggestion::simple(
                                "convert the initializer to match the declared type or change the type annotation"
                            )));
                    }
                }
            }
            
            let symbol = Symbol {
                name: name.clone(),
                symbol_type: SymbolType::Variable,
                ty,
            };
            symbol_table.insert(symbol, file_path)?;
        }
        Statement::Const { name, var_type, initializer } => {
            let ty = var_type.clone().unwrap_or_else(|| infer_type(initializer, symbol_table, file_path).unwrap());
            let init_ty = infer_type(initializer, symbol_table, file_path)?;
            if !types_compatible(&ty, &init_ty) {
                return Err(type_mismatch(&format!("{:?}", ty), &format!("{:?}", init_ty), stmt_location)
                    .with_suggestion(Suggestion::simple(
                        "ensure the initializer expression matches the declared constant type"
                    )));
            }
            
            let symbol = Symbol {
                name: name.clone(),
                symbol_type: SymbolType::Variable,
                ty,
            };
            symbol_table.insert(symbol, file_path)?;
        }
        Statement::Assignment { target, value } => {
            let target_ty = infer_type(target, symbol_table, file_path)?;
            let value_ty = infer_type(value, symbol_table, file_path)?;
            if !types_compatible(&target_ty, &value_ty) {
                return Err(type_mismatch(&format!("{:?}", target_ty), &format!("{:?}", value_ty), stmt_location)
                    .with_suggestion(Suggestion::simple(
                        "ensure the assigned value matches the target's type or convert it appropriately"
                    )));
            }
        }
        Statement::Return(value) => {
            match expected_return {
                Type::Void => {
                    if let Some(expr) = value {
                        let ret_ty = infer_type(expr, symbol_table, file_path)?;
                        return Err(CompilerError::new(
                            ErrorKind::TypeMismatch,
                            format!("returning a value of type `{:?}` from a void function", ret_ty),
                            stmt_location,
                        ).with_suggestion(Suggestion::simple(
                            "remove the return value or change the function return type"
                        )));
                    }
                }
                expected_ty => {
                    if let Some(expr) = value {
                        // Use type hint to help infer generic types like Result::Ok
                        let ret_ty = infer_type_with_hint(expr, Some(expected_ty), symbol_table, file_path)?;
                        if !types_compatible(expected_ty, &ret_ty) {
                            return Err(type_mismatch(&format!("{:?}", expected_ty), &format!("{:?}", ret_ty), stmt_location)
                                .with_suggestion(Suggestion::simple(
                                    "return a value that matches the function's declared return type"
                                )));
                        }
                    } else {
                        return Err(CompilerError::new(
                            ErrorKind::MissingReturnType,
                            format!("missing return value; function expects `{:?}`", expected_ty),
                            stmt_location,
                        ).with_suggestion(Suggestion::simple(
                            "provide a return value"
                        )));
                    }
                }
            }
        }
        Statement::If { condition, then_branch, else_branch } => {
            let cond_ty = infer_type(condition, symbol_table, file_path)?;
            if cond_ty != Type::Bool {
                return Err(type_mismatch("bool", &format!("{:?}", cond_ty), stmt_location)
                    .with_suggestion(Suggestion::simple(
                        "use a boolean expression in the if condition, such as a comparison or boolean variable"
                    )));
            }
            symbol_table.enter_scope();
            for stmt in then_branch {
                analyze_statement(stmt, symbol_table, file_path, stmt_location.clone(), expected_return)?;
            }
            symbol_table.exit_scope();
            if let Some(else_branch) = else_branch {
                symbol_table.enter_scope();
                for stmt in else_branch {
                    analyze_statement(stmt, symbol_table, file_path, stmt_location.clone(), expected_return)?;
                }
                symbol_table.exit_scope();
            }
        }
        Statement::While { condition, body } => {
            let cond_ty = infer_type(condition, symbol_table, file_path)?;
            if cond_ty != Type::Bool {
                return Err(type_mismatch("bool", &format!("{:?}", cond_ty), stmt_location)
                    .with_suggestion(Suggestion::simple(
                        "use a boolean expression in the while condition, such as a comparison or boolean variable"
                    )));
            }
            symbol_table.enter_scope();
            for stmt in body {
                analyze_statement(stmt, symbol_table, file_path, stmt_location.clone(), expected_return)?;
            }
            symbol_table.exit_scope();
        }
        Statement::For { variable, iterable, body } => {
            symbol_table.enter_scope();
            
            // Validate that iterable is a range or array
            let iterable_ty = infer_type(iterable, symbol_table, file_path)?;
            let loop_var_ty = match &iterable_ty {
                Type::Array(elem_ty) => *elem_ty.clone(),
                Type::DynamicArray(elem_ty) => *elem_ty.clone(),
                Type::Void => {
                    // Range expressions have Void type - loop variable is int
                    Type::Int
                }
                _ => {
                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                    return Err(CompilerError::new(
                        ErrorKind::InvalidOperation,
                        format!("cannot iterate over type `{:?}`", iterable_ty),
                        location,
                    ).with_suggestion(Suggestion::simple(
                        "for loops require an iterable type (range, array, or dynamic array)"
                    )));
                }
            };
            
            // Add the loop variable to the symbol table with correct type
            let symbol = Symbol {
                name: variable.clone(),
                ty: loop_var_ty,
                symbol_type: SymbolType::Variable,
            };
            symbol_table.insert(symbol, file_path)?;
            
            for stmt in body {
                analyze_statement(stmt, symbol_table, file_path, stmt_location.clone(), expected_return)?;
            }
            symbol_table.exit_scope();
        }
        Statement::Break | Statement::Continue => {
            // Break and continue are valid statements
            // Note: We could add loop context tracking here to ensure they're only used in loops
            // For now, we'll let the code generator handle that
        }
        Statement::Expression(expr) => {
            let _ = infer_type(expr, symbol_table, file_path)?;
        }
    }
    Ok(())
}

/// Infer type with an expected type hint (for generic type inference)
fn infer_type_with_hint(
    expr: &Expression,
    expected: Option<&Type>,
    symbol_table: &mut SymbolTable,
    file_path: &PathBuf
) -> Result<Type, CompilerError> {
    // Special case: EnumAccess with expected generic type
    if let (Expression::EnumAccess { enum_name, variant }, Some(expected_ty)) = (expr, expected) {
        // If the expected type is a generic instantiation of this enum, use it
        if let Type::Generic { name, type_params: _ } = expected_ty {
            if name == enum_name && symbol_table.builtins.is_generic_builtin(enum_name) {
                let builtin = symbol_table.builtins.get_generic(enum_name).unwrap();
                
                // Validate variant exists
                if builtin.get_variant(variant).is_none() {
                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                    return Err(CompilerError::new(
                        ErrorKind::UndefinedType,
                        format!("type `{}` has no variant `{}`", enum_name, variant),
                        location,
                    ));
                }
                
                // Return the expected generic type
                return Ok(expected_ty.clone());
            }
        }
    }
    
    // Special case: Call with EnumAccess callee (variant construction with value)
    if let (Expression::Call { callee, arguments }, Some(expected_ty)) = (expr, expected) {
        if let Expression::EnumAccess { enum_name, variant } = &**callee {
            if let Type::Generic { name, type_params } = expected_ty {
                if name == enum_name && symbol_table.builtins.is_generic_builtin(enum_name) {
                    let builtin = symbol_table.builtins.get_generic(enum_name).unwrap();
                    
                    // Validate variant and argument
                    if let Some(variant_info) = builtin.get_variant(variant) {
                        if !variant_info.has_value {
                            let location = SourceLocation::new(file_path.clone(), 0, 0);
                            return Err(CompilerError::new(
                                ErrorKind::InvalidOperation,
                                format!("variant `{}::{}` does not take a value", enum_name, variant),
                                location,
                            ));
                        }
                        
                        if arguments.len() != 1 {
                            let location = SourceLocation::new(file_path.clone(), 0, 0);
                            return Err(CompilerError::new(
                                ErrorKind::WrongArgumentCount,
                                format!("{}::{} expects 1 argument, got {}", enum_name, variant, arguments.len()),
                                location,
                            ));
                        }
                        
                        // Get the expected type for the argument based on the variant's type parameter
                        if let Some(type_param_name) = &variant_info.value_type_param {
                            if let Some(param_idx) = builtin.type_params.iter().position(|p| p == type_param_name) {
                                if let Some(expected_arg_ty) = type_params.get(param_idx) {
                                    // Validate argument type matches
                                    let arg_ty = infer_type(&arguments[0], symbol_table, file_path)?;
                                    if !types_compatible(expected_arg_ty, &arg_ty) {
                                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                                        return Err(CompilerError::new(
                                            ErrorKind::TypeMismatch,
                                            format!("{}::{} expects argument of type `{:?}`, got `{:?}`", 
                                                enum_name, variant, expected_arg_ty, arg_ty),
                                            location,
                                        ));
                                    }
                                }
                            }
                        }
                        
                        // Return the expected generic type
                        return Ok(expected_ty.clone());
                    }
                }
            }
        }
    }
    
    // Otherwise, fall back to regular type inference
    infer_type(expr, symbol_table, file_path)
}

fn infer_type(expr: &Expression, symbol_table: &mut SymbolTable, file_path: &PathBuf) -> Result<Type, CompilerError> {
    match expr {
        Expression::Literal(lit) => match lit {
            Literal::Integer(_) => Ok(Type::Int),
            Literal::Float(_) => Ok(Type::Float),
            Literal::Bool(_) => Ok(Type::Bool),
            Literal::Char(_) => Ok(Type::Char),
            Literal::String(_) => Ok(Type::String),
        },
        Expression::Variable(name) => {
            if let Some(symbol) = symbol_table.lookup(name) {
                // Normalize str to String type
                let mut ty = symbol.ty.clone();
                if let Type::Struct(ref type_name) = ty {
                    if type_name == "str" {
                        ty = Type::String;
                    }
                }
                Ok(ty)
            } else {
                let location = SourceLocation::new(file_path.clone(), 0, 0);
                Err(undefined_variable(name, location))
            }
        }
        Expression::Binary { left, operator, right } => {
            let left_ty = infer_type(left, symbol_table, file_path)?;
            let right_ty = infer_type(right, symbol_table, file_path)?;
            
            // Check for division/modulo by constant zero
            if matches!(operator, BinaryOp::Divide | BinaryOp::Modulo) {
                if let Expression::Literal(Literal::Integer(0)) = &**right {
                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                    let op_name = if *operator == BinaryOp::Divide { "division" } else { "modulo" };
                    return Err(CompilerError::new(
                        ErrorKind::InvalidOperation,
                        format!("{} by zero", op_name),
                        location,
                    ).with_suggestion(Suggestion::simple(
                        "division and modulo by zero will cause a runtime error"
                    )));
                }
            }
            
            match operator {
                BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => {
                    if left_ty == Type::Int && right_ty == Type::Int {
                        Ok(Type::Int)
                    } else if (left_ty == Type::Int || left_ty == Type::Float) && (right_ty == Type::Int || right_ty == Type::Float) {
                        Ok(Type::Float)
                    } else {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        Err(CompilerError::new(
                            ErrorKind::InvalidOperation,
                            format!("cannot apply arithmetic operator to types `{:?}` and `{:?}`", left_ty, right_ty),
                            location,
                        ).with_suggestion(Suggestion::simple(
                            "arithmetic operators require numeric operands (int or float)"
                        )))
                    }
                }
                BinaryOp::Equal | BinaryOp::NotEqual | BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => {
                    if types_compatible(&left_ty, &right_ty) {
                        Ok(Type::Bool)
                    } else {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        Err(CompilerError::new(
                            ErrorKind::InvalidOperation,
                            format!("cannot compare `{:?}` with `{:?}`", left_ty, right_ty),
                            location,
                        ).with_suggestion(Suggestion::simple(
                            "comparison operators require operands of compatible types"
                        )))
                    }
                }
                BinaryOp::And | BinaryOp::Or => {
                    if left_ty == Type::Bool && right_ty == Type::Bool {
                        Ok(Type::Bool)
                    } else {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        Err(CompilerError::new(
                            ErrorKind::InvalidOperation,
                            format!("logical operators require boolean operands, got `{:?}` and `{:?}`", left_ty, right_ty),
                            location,
                        ).with_suggestion(Suggestion::simple(
                            "use boolean expressions or variables with logical operators"
                        )))
                    }
                }
            }
        }
        Expression::Unary { operator, operand } => {
            let op_ty = infer_type(operand, symbol_table, file_path)?;
            match operator {
                UnaryOp::Negate => {
                    if op_ty == Type::Int || op_ty == Type::Float {
                        Ok(op_ty)
                    } else {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        Err(CompilerError::new(
                            ErrorKind::InvalidOperation,
                            format!("cannot negate type `{:?}`", op_ty),
                            location,
                        ).with_suggestion(Suggestion::simple(
                            "negation operator requires a numeric type (int or float)"
                        )))
                    }
                }
                UnaryOp::Not => {
                    if op_ty == Type::Bool {
                        Ok(Type::Bool)
                    } else {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        Err(CompilerError::new(
                            ErrorKind::InvalidOperation,
                            format!("cannot apply logical not to type `{:?}`", op_ty),
                            location,
                        ).with_suggestion(Suggestion::simple(
                            "logical not operator requires a boolean operand"
                        )))
                    }
                }
                UnaryOp::Dereference => {
                    if let Type::Pointer(pointee) = op_ty {
                        Ok(*pointee.clone())
                    } else {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        Err(CompilerError::new(
                            ErrorKind::InvalidOperation,
                            format!("cannot dereference type `{:?}`", op_ty),
                            location,
                        ).with_suggestion(Suggestion::simple(
                            "dereference operator (*) requires a pointer type"
                        )))
                    }
                }
                UnaryOp::AddressOf => {
                    // &expr returns pointer to expr type
                    Ok(Type::Pointer(Box::new(op_ty)))
                }
            }
        }
        Expression::Call { callee, arguments } => {
            match &**callee {
                Expression::Variable(name) => {
                    // Regular function call
                    if name == "print" || name == "println" {
                        // Built-in print functions - accept any argument type
                        Ok(Type::Void)
                    } else if name == "len" {
                        // Built-in len function - takes a string, returns int
                        if arguments.len() != 1 {
                            let location = SourceLocation::new(file_path.clone(), 0, 0);
                            return Err(CompilerError::new(
                                ErrorKind::WrongArgumentCount,
                                "len() function expects exactly 1 argument".to_string(),
                                location,
                            ));
                        }
                        // Validate argument is a string
                        let arg_ty = infer_type(&arguments[0], symbol_table, file_path)?;
                        if arg_ty != Type::String {
                            let location = SourceLocation::new(file_path.clone(), 0, 0);
                            return Err(CompilerError::new(
                                ErrorKind::TypeMismatch,
                                format!("len() expects a string argument, got `{:?}`", arg_ty),
                                location,
                            ).with_suggestion(Suggestion::simple(
                                "pass a string to len() to get its length"
                            )));
                        }
                        Ok(Type::Int)
                    } else if let Some(symbol) = symbol_table.lookup(name) {
                        if symbol.symbol_type == SymbolType::Function {
                            // TODO: check argument types
                            Ok(symbol.ty.clone())
                        } else {
                            let location = SourceLocation::new(file_path.clone(), 0, 0);
                            Err(CompilerError::new(
                                ErrorKind::InvalidOperation,
                                format!("`{}` is not a function", name),
                                location,
                            ).with_suggestion(Suggestion::simple(
                                "only functions can be called with parentheses"
                            )))
                        }
                    } else if crate::intrinsics::is_intrinsic(name) {
                        // Function is a built-in intrinsic - assume it returns int for now
                        // (Most intrinsics return int or pointer, this is a simplification)
                        Ok(Type::Int)
                    } else {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        Err(CompilerError::new(
                            ErrorKind::UndefinedFunction,
                            format!("cannot find function `{}` in this scope", name),
                            location,
                        ).with_suggestion(Suggestion::simple(
                            "check the function name for typos or ensure it's defined/imported"
                        )))
                    }
                }
                Expression::StructAccess { object, field } => {
                    // Check if this is a module-qualified call like math.add or token.TK_INTEGER
                    if let Expression::Variable(module_name) = &**object {
                        let qualified_name = format!("{}.{}", module_name, field);
                        if let Some(symbol) = symbol_table.lookup(&qualified_name) {
                            if symbol.symbol_type == SymbolType::Function {
                                // TODO: check argument types
                                Ok(symbol.ty.clone())
                            } else {
                                let location = SourceLocation::new(file_path.clone(), 0, 0);
                                Err(CompilerError::new(
                                    ErrorKind::InvalidOperation,
                                    format!("`{}.{}' is not a function", module_name, field),
                                    location,
                                ).with_suggestion(Suggestion::simple(
                                    "only functions can be called with parentheses"
                                )))
                            }
                        } else if symbol_table.lookup(module_name).is_some() {
                            // Object is a known variable, check if it's a method call
                            let mut object_ty = infer_type(object, symbol_table, file_path)?;
                            
                            // Normalize str to String type
                            if let Type::Struct(ref name) = object_ty {
                                if name == "str" {
                                    object_ty = Type::String;
                                }
                            }
                            
                            match (&object_ty, field.as_str()) {
                                // String methods
                                (&Type::String, "length") => {
                                    if !arguments.is_empty() {
                                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                                        return Err(CompilerError::new(
                                            ErrorKind::WrongArgumentCount,
                                            format!("length() expects 0 arguments, got {}", arguments.len()),
                                            location,
                                        ));
                                    }
                                    Ok(Type::Int)
                                }
                                (&Type::String, "substring") => {
                                    if arguments.len() != 2 {
                                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                                        return Err(CompilerError::new(
                                            ErrorKind::WrongArgumentCount,
                                            format!("substring() expects 2 arguments (start, end), got {}", arguments.len()),
                                            location,
                                        ).with_suggestion(Suggestion::simple(
                                            "usage: str.substring(start_index, end_index)"
                                        )));
                                    }
                                    Ok(Type::String)
                                }
                                (&Type::String, "contains") => {
                                    if arguments.len() != 1 {
                                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                                        return Err(CompilerError::new(
                                            ErrorKind::WrongArgumentCount,
                                            format!("contains() expects 1 argument, got {}", arguments.len()),
                                            location,
                                        ));
                                    }
                                    Ok(Type::Int)
                                }
                                (&Type::String, "trim") => {
                                    if !arguments.is_empty() {
                                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                                        return Err(CompilerError::new(
                                            ErrorKind::WrongArgumentCount,
                                            format!("trim() expects 0 arguments, got {}", arguments.len()),
                                            location,
                                        ));
                                    }
                                    Ok(Type::String)
                                }
                                (&Type::String, "split") => {
                                    if arguments.len() != 1 {
                                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                                        return Err(CompilerError::new(
                                            ErrorKind::WrongArgumentCount,
                                            format!("split() expects 1 argument, got {}", arguments.len()),
                                            location,
                                        ));
                                    }
                                    Ok(Type::DynamicArray(Box::new(Type::String)))
                                }
                                // Dynamic array methods
                                (&Type::DynamicArray(ref elem_ty), "push") => {
                                    // push(element) - validate argument count and type
                                    if arguments.len() != 1 {
                                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                                        return Err(CompilerError::new(
                                            ErrorKind::WrongArgumentCount,
                                            format!("push() expects 1 argument, got {}", arguments.len()),
                                            location,
                                        ));
                                    }
                                    let arg_ty = infer_type(&arguments[0], symbol_table, file_path)?;
                                    if !types_compatible(&*elem_ty, &arg_ty) {
                                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                                        return Err(CompilerError::new(
                                            ErrorKind::TypeMismatch,
                                            format!("push() expects element of type `{:?}`, got `{:?}`", elem_ty, arg_ty),
                                            location,
                                        ).with_suggestion(Suggestion::simple(
                                            "ensure the pushed element matches the array's element type"
                                        )));
                                    }
                                    // push returns the array (for chaining)
                                    Ok(object_ty)
                                }
                                (&Type::DynamicArray(ref elem_ty), "pop") => {
                                    // pop() - validate no arguments
                                    if arguments.len() != 0 {
                                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                                        return Err(CompilerError::new(
                                            ErrorKind::WrongArgumentCount,
                                            format!("pop() expects 0 arguments, got {}", arguments.len()),
                                            location,
                                        ));
                                    }
                                    // pop returns the element type
                                    Ok(*elem_ty.clone())
                                }
                                (&Type::DynamicArray(_), "length") => {
                                    // length() - validate no arguments
                                    if arguments.len() != 0 {
                                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                                        return Err(CompilerError::new(
                                            ErrorKind::WrongArgumentCount,
                                            format!("length() expects 0 arguments, got {}", arguments.len()),
                                            location,
                                        ));
                                    }
                                    // length returns int
                                    Ok(Type::Int)
                                }
                                _ => {
                                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                                    Err(CompilerError::new(
                                        ErrorKind::UndefinedFunction,
                                        format!("unknown method `{}` on type `{:?}`", field, object_ty),
                                        location,
                                    ).with_suggestion(Suggestion::simple(
                                        "check the method name or ensure the type supports this operation"
                                    )))
                                }
                            }
                        } else {
                            // Module name not found as variable, assume it's a module-qualified call
                            // that wasn't found. This allows module.function() even if module name isn't a variable
                            let location = SourceLocation::new(file_path.clone(), 0, 0);
                            Err(CompilerError::new(
                                ErrorKind::UndefinedFunction,
                                format!("function `{}.{}` not found", module_name, field),
                                location,
                            ).with_suggestion(Suggestion::simple(
                                format!("ensure `{}` is exported from module `{}`", field, module_name)
                            )))
                        }
                    } else {
                        // Regular struct field access used as function call - not allowed
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        Err(CompilerError::new(
                            ErrorKind::InvalidOperation,
                            "cannot call struct field as function".to_string(),
                            location,
                        ).with_suggestion(Suggestion::simple(
                            "struct fields cannot be called like functions"
                        )))
                    }
                }
                Expression::EnumAccess { enum_name, variant } => {
                    // Enum variant constructor call: Option::Some(42)
                    // Check if this is a built-in generic type
                    if symbol_table.builtins.is_generic_builtin(enum_name) {
                        let builtin = symbol_table.builtins.get_generic(enum_name).unwrap();
                        
                        // Check variant exists and can take a value
                        if let Some(variant_info) = builtin.get_variant(variant) {
                            if !variant_info.has_value {
                                let location = SourceLocation::new(file_path.clone(), 0, 0);
                                return Err(CompilerError::new(
                                    ErrorKind::InvalidOperation,
                                    format!("variant `{}::{}` does not take a value", enum_name, variant),
                                    location,
                                ).with_suggestion(Suggestion::simple(
                                    &format!("use `{}::{}` without parentheses", enum_name, variant)
                                )));
                            }
                            
                            // Validate argument count
                            if arguments.len() != 1 {
                                let location = SourceLocation::new(file_path.clone(), 0, 0);
                                return Err(CompilerError::new(
                                    ErrorKind::WrongArgumentCount,
                                    format!("{}::{} expects 1 argument, got {}", enum_name, variant, arguments.len()),
                                    location,
                                ));
                            }
                            
                            // Infer the argument type
                            let arg_ty = infer_type(&arguments[0], symbol_table, file_path)?;
                            
                            // TODO: We need to know what generic type parameters to use
                            // For now, construct a generic type from the argument
                            // This is simplified - proper inference would look at context
                            Ok(Type::Generic {
                                name: enum_name.clone(),
                                type_params: vec![arg_ty],
                            })
                        } else {
                            let location = SourceLocation::new(file_path.clone(), 0, 0);
                            return Err(CompilerError::new(
                                ErrorKind::UndefinedType,
                                format!("type `{}` has no variant `{}`", enum_name, variant),
                                location,
                            ));
                        }
                    } else {
                        // User-defined enum - check if it exists and supports construction
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        return Err(CompilerError::new(
                            ErrorKind::InvalidOperation,
                            format!("enum variant construction with values not yet supported for user-defined enums"),
                            location,
                        ).with_suggestion(Suggestion::simple(
                            "currently only built-in types like Option and Result support variant construction with values"
                        )));
                    }
                }
                _ => {
                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                    Err(CompilerError::new(
                        ErrorKind::InvalidOperation,
                        "invalid call expression".to_string(),
                        location,
                    ).with_suggestion(Suggestion::simple(
                        "only function names or method calls can be used as callees"
                    )))
                }
            }
        }
        Expression::ArrayLiteral(elements) => {
            if elements.is_empty() {
                let location = SourceLocation::new(file_path.clone(), 0, 0);
                return Err(CompilerError::new(
                    ErrorKind::InvalidSyntax,
                    "empty array literals need explicit type annotation".to_string(),
                    location,
                ).with_suggestion(Suggestion::simple(
                    "use explicit typing like `[]int` or provide array elements"
                )));
            }
            let first_ty = infer_type(&elements[0], symbol_table, file_path)?;
            for elem in &elements[1..] {
                let elem_ty = infer_type(elem, symbol_table, file_path)?;
                if !types_compatible(&first_ty, &elem_ty) {
                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                    return Err(CompilerError::new(
                        ErrorKind::TypeMismatch,
                        format!("array elements must have compatible types: `{:?}` vs `{:?}`", first_ty, elem_ty),
                        location,
                    ).with_suggestion(Suggestion::simple(
                        "ensure all array elements have the same type"
                    )));
                }
            }
            Ok(Type::Array(Box::new(first_ty)))
        }
        Expression::DynamicArrayLiteral { element_type, elements } => {
            // Check that all elements match the declared element type
            for elem in elements {
                let elem_ty = infer_type(elem, symbol_table, file_path)?;
                if !types_compatible(&element_type, &elem_ty) {
                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                    return Err(CompilerError::new(
                        ErrorKind::TypeMismatch,
                        format!("dynamic array elements must match declared type: `{:?}` vs `{:?}`", element_type, elem_ty),
                        location,
                    ).with_suggestion(Suggestion::simple(
                        "ensure all elements match the declared element type of the dynamic array"
                    )));
                }
            }
            Ok(Type::DynamicArray(element_type.clone()))
        }
        Expression::StructLiteral { name, fields } => {
            // Ensure struct exists and fields match
            if let Some(symbol) = symbol_table.lookup(name) {
                if symbol.symbol_type != SymbolType::Struct {
                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                    return Err(CompilerError::new(
                        ErrorKind::UndefinedType,
                        format!("`{}` is not a struct type", name),
                        location,
                    ));
                }
            } else {
                let location = SourceLocation::new(file_path.clone(), 0, 0);
                return Err(CompilerError::new(
                    ErrorKind::UndefinedType,
                    format!("unknown struct type `{}`", name),
                    location,
                ));
            }

            // Validate fields
            for (field_name, expr) in fields {
                let expr_ty = infer_type(expr, symbol_table, file_path)?;
                if let Some(expected) = symbol_table.get_struct_field_type(name, field_name) {
                    if !types_compatible(expected, &expr_ty) {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        return Err(CompilerError::new(
                            ErrorKind::TypeMismatch,
                            format!("field `{}` type mismatch: expected `{:?}`, found `{:?}`", field_name, expected, expr_ty),
                            location,
                        ));
                    }
                } else {
                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                    return Err(CompilerError::new(
                        ErrorKind::UndefinedVariable,
                        format!("unknown field `{}` on struct `{}`", field_name, name),
                        location,
                    ));
                }
            }

            Ok(Type::Struct(name.clone()))
        }
        Expression::ArrayAccess { array, index } => {
            let array_ty = infer_type(array, symbol_table, file_path)?;
            let index_ty = infer_type(index, symbol_table, file_path)?;
            if index_ty != Type::Int {
                let location = SourceLocation::new(file_path.clone(), 0, 0);
                return Err(type_mismatch("int", &format!("{:?}", index_ty), location)
                    .with_suggestion(Suggestion::simple(
                        "array indices must be integers"
                    )));
            }
            
            // Warn about negative constant indices
            if let Expression::Literal(Literal::Integer(n)) = &**index {
                if *n < 0 {
                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                    return Err(CompilerError::new(
                        ErrorKind::InvalidOperation,
                        format!("array index cannot be negative (got {})", n),
                        location,
                    ).with_suggestion(Suggestion::simple(
                        "array indices must be non-negative integers"
                    )));
                }
            }
            match array_ty {
                Type::Array(elem_ty) => Ok(*elem_ty),
                Type::DynamicArray(elem_ty) => Ok(*elem_ty),
                Type::Pointer(elem_ty) => Ok(*elem_ty),
                Type::String => Ok(Type::Char),
                _ => {
                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                    Err(CompilerError::new(
                        ErrorKind::InvalidOperation,
                        format!("cannot index type `{:?}`", array_ty),
                        location,
                    ).with_suggestion(Suggestion::simple(
                        "only arrays, dynamic arrays, pointers, and strings can be indexed with `[]`"
                    )))
                }
            }
        }
        Expression::StructAccess { object, field } => {
            let obj_ty = infer_type(object, symbol_table, file_path)?;
            if let Type::Struct(struct_name) = obj_ty {
                if let Some(fty) = symbol_table.get_struct_field_type(&struct_name, field) {
                    Ok(fty.clone())
                } else {
                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                    Err(CompilerError::new(
                        ErrorKind::UndefinedVariable,
                        format!("unknown field `{}.{}`", struct_name, field),
                        location,
                    ).with_suggestion(Suggestion::simple(
                        "check the field name or struct definition"
                    )))
                }
            } else {
                let location = SourceLocation::new(file_path.clone(), 0, 0);
                Err(CompilerError::new(
                    ErrorKind::InvalidOperation,
                    format!("cannot access field of non-struct type `{:?}`", obj_ty),
                    location,
                ).with_suggestion(Suggestion::simple(
                    "field access (.) is only valid on struct types"
                )))
            }
        }
        Expression::New(expr) => {
            // new returns a pointer to the allocated type
            let inner_ty = infer_type(expr, symbol_table, file_path)?;
            Ok(Type::Pointer(Box::new(inner_ty)))
        }
        Expression::Delete(expr) => {
            // delete returns void
            let _ = infer_type(expr, symbol_table, file_path)?;
            Ok(Type::Void)
        }
        Expression::Range { start, end } => {
            // ranges are used in for loops, type is not directly used
            let _ = infer_type(start, symbol_table, file_path)?;
            let _ = infer_type(end, symbol_table, file_path)?;
            Ok(Type::Void) // ranges don't have a specific type
        }
        Expression::Cast { expression, target_type } => {
            // Type casting: expr as Type
            let expr_ty = infer_type(expression, symbol_table, file_path)?;
            
            // Check if the cast is valid
            let valid_cast = match (&expr_ty, target_type) {
                // Numeric conversions
                (Type::Int, Type::Int) |
                (Type::Int, Type::Float) |
                (Type::Int, Type::Char) |
                (Type::Float, Type::Int) |
                (Type::Float, Type::Float) |
                (Type::Char, Type::Int) |
                (Type::Char, Type::Char) => true,
                
                // Pointer conversions
                (Type::Pointer(_), Type::Pointer(_)) => true,
                (Type::Int, Type::Pointer(_)) => true,
                (Type::Pointer(_), Type::Int) => true,
                
                // String to pointer conversions
                (Type::String, Type::Pointer(inner)) if **inner == Type::Char => true,
                
                // Allow casting between any two types (unsafe cast)
                // In a production compiler, you might want to restrict this more
                _ => false,
            };
            
            if !valid_cast {
                let location = SourceLocation::new(file_path.clone(), 0, 0);
                return Err(CompilerError::new(
                    ErrorKind::InvalidOperation,
                    format!("cannot cast from type `{:?}` to `{:?}`", expr_ty, target_type),
                    location,
                ).with_suggestion(Suggestion::simple(
                    "type casts are only valid between compatible types (numeric types, pointers, and int-pointer conversions)"
                )));
            }
            
            Ok(target_type.clone())
        }
        Expression::Ternary { condition, true_expr, false_expr } => {
            // Condition must be boolean
            let cond_ty = infer_type(condition, symbol_table, file_path)?;
            if cond_ty != Type::Bool {
                let location = SourceLocation::new(file_path.clone(), 0, 0);
                return Err(CompilerError::new(
                    ErrorKind::TypeMismatch,
                    format!("ternary condition must be boolean, got `{:?}`", cond_ty),
                    location,
                ).with_suggestion(Suggestion::simple(
                    "use a boolean expression for the ternary condition"
                )));
            }
            
            // Both branches must have compatible types
            let true_ty = infer_type(true_expr, symbol_table, file_path)?;
            let false_ty = infer_type(false_expr, symbol_table, file_path)?;
            
            if !types_compatible(&true_ty, &false_ty) {
                let location = SourceLocation::new(file_path.clone(), 0, 0);
                return Err(CompilerError::new(
                    ErrorKind::TypeMismatch,
                    format!("ternary branches must have compatible types: `{:?}` vs `{:?}`", true_ty, false_ty),
                    location,
                ).with_suggestion(Suggestion::simple(
                    "ensure both branches of the ternary operator return the same type"
                )));
            }
            
            // Return type of true branch (both are compatible)
            Ok(true_ty)
        }
        Expression::EnumAccess { enum_name, variant } => {
            // Check if this is a built-in generic type (Option, Result)
            if symbol_table.builtins.is_generic_builtin(enum_name) {
                let builtin = symbol_table.builtins.get_generic(enum_name).unwrap();
                
                // Check if variant exists in this built-in type
                if builtin.get_variant(variant).is_none() {
                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                    return Err(CompilerError::new(
                        ErrorKind::UndefinedType,
                        format!("type `{}` has no variant `{}`", enum_name, variant),
                        location,
                    ).with_suggestion(Suggestion::simple(
                        &format!("valid variants for {} are: {}", 
                                 enum_name, 
                                 builtin.variants.iter().map(|v| v.name.as_str()).collect::<Vec<_>>().join(", "))
                    )));
                }
                
                // For now, return a generic type with unspecified parameters
                // TODO: In a real implementation, we'd need type inference to determine the parameters
                // For now, we'll require explicit type annotations
                let location = SourceLocation::new(file_path.clone(), 0, 0);
                return Err(CompilerError::new(
                    ErrorKind::TypeMismatch,
                    format!("cannot use `{}::{}` without type parameters - use explicit type annotation", enum_name, variant),
                    location,
                ).with_suggestion(Suggestion::with_example(
                    "specify the type explicitly",
                    &format!("let x: {}<SomeType> = {}::{};", enum_name, enum_name, variant)
                )));
            }
            
            // Check if enum exists in symbol table (user-defined enums)
            if let Some(symbol) = symbol_table.lookup(enum_name) {
                if symbol.symbol_type != SymbolType::Enum {
                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                    return Err(CompilerError::new(
                        ErrorKind::TypeMismatch,
                        format!("`{}` is not an enum", enum_name),
                        location,
                    ));
                }
                
                // Check if variant exists in this enum
                if symbol_table.get_enum_variant_value(enum_name, variant).is_none() {
                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                    return Err(CompilerError::new(
                        ErrorKind::UndefinedType,
                        format!("enum `{}` has no variant `{}`", enum_name, variant),
                        location,
                    ).with_suggestion(Suggestion::simple(
                        "check the enum definition for valid variant names"
                    )));
                }
                
                // Return the enum type
                Ok(Type::Enum(enum_name.clone()))
            } else {
                let location = SourceLocation::new(file_path.clone(), 0, 0);
                Err(CompilerError::new(
                    ErrorKind::UndefinedType,
                    format!("enum `{}` not found", enum_name),
                    location,
                ).with_suggestion(Suggestion::simple(
                    "ensure the enum is defined before using it"
                )))
            }
        }
        Expression::Match { scrutinee, arms } => {
            use crate::ast::Pattern;
            
            // Infer the type of the scrutinee
            let scrutinee_ty = infer_type(scrutinee, symbol_table, file_path)?;
            
            if arms.is_empty() {
                let location = SourceLocation::new(file_path.clone(), 0, 0);
                return Err(CompilerError::new(
                    ErrorKind::InvalidSyntax,
                    "match expression must have at least one arm".to_string(),
                    location,
                ));
            }
            
            // Check each pattern is compatible with scrutinee type
            let mut has_wildcard = false;
            let mut matched_variants = std::collections::HashSet::new();
            
            for arm in arms {
                match &arm.pattern {
                    Pattern::Wildcard => {
                        has_wildcard = true;
                    }
                    Pattern::Literal(lit) => {
                        let pattern_ty = match lit {
                            Literal::Integer(_) => Type::Int,
                            Literal::Float(_) => Type::Float,
                            Literal::Bool(_) => Type::Bool,
                            Literal::Char(_) => Type::Char,
                            Literal::String(_) => Type::String,
                        };
                        if !types_compatible(&scrutinee_ty, &pattern_ty) {
                            let location = SourceLocation::new(file_path.clone(), 0, 0);
                            return Err(CompilerError::new(
                                ErrorKind::TypeMismatch,
                                format!("pattern type `{:?}` doesn't match scrutinee type `{:?}`", pattern_ty, scrutinee_ty),
                                location,
                            ));
                        }
                    }
                    Pattern::EnumVariant { enum_name, variant, binding } => {
                        // Check if this is a built-in generic type
                        if symbol_table.builtins.is_generic_builtin(enum_name) {
                            let builtin = symbol_table.builtins.get_generic(enum_name).unwrap();
                            
                            // Check variant exists
                            let variant_info = match builtin.get_variant(variant) {
                                Some(v) => v,
                                None => {
                                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                                    return Err(CompilerError::new(
                                        ErrorKind::UndefinedType,
                                        format!("type `{}` has no variant `{}`", enum_name, variant),
                                        location,
                                    ));
                                }
                            };
                            
                            // Validate binding matches variant requirements
                            if binding.is_some() && !variant_info.has_value {
                                let location = SourceLocation::new(file_path.clone(), 0, 0);
                                return Err(CompilerError::new(
                                    ErrorKind::InvalidSyntax,
                                    format!("variant `{}::{}` does not have a value to bind", enum_name, variant),
                                    location,
                                ).with_suggestion(Suggestion::simple(
                                    &format!("use `{}::{}` without a binding", enum_name, variant)
                                )));
                            }
                            
                            if binding.is_none() && variant_info.has_value {
                                let location = SourceLocation::new(file_path.clone(), 0, 0);
                                return Err(CompilerError::new(
                                    ErrorKind::InvalidSyntax,
                                    format!("variant `{}::{}` has a value that should be bound", enum_name, variant),
                                    location,
                                ).with_suggestion(Suggestion::simple(
                                    &format!("use `{}::{}(name)` to bind the value", enum_name, variant)
                                )));
                            }
                            
                            matched_variants.insert(variant.clone());
                            
                            // Check scrutinee type is compatible
                            if let Type::Generic { name, .. } = &scrutinee_ty {
                                if name != enum_name {
                                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                                    return Err(CompilerError::new(
                                        ErrorKind::TypeMismatch,
                                        format!("pattern type `{}` doesn't match scrutinee type `{:?}`", enum_name, scrutinee_ty),
                                        location,
                                    ));
                                }
                            } else {
                                let location = SourceLocation::new(file_path.clone(), 0, 0);
                                return Err(CompilerError::new(
                                    ErrorKind::TypeMismatch,
                                    format!("pattern expects generic type `{}`, but scrutinee is `{:?}`", enum_name, scrutinee_ty),
                                    location,
                                ));
                            }
                        }
                        // Check enum exists in symbol table (user-defined enum)
                        else if let Some(symbol) = symbol_table.lookup(enum_name) {
                            if symbol.symbol_type != SymbolType::Enum {
                                let location = SourceLocation::new(file_path.clone(), 0, 0);
                                return Err(CompilerError::new(
                                    ErrorKind::TypeMismatch,
                                    format!("`{}` is not an enum", enum_name),
                                    location,
                                ));
                            }
                            
                            if symbol_table.get_enum_variant_value(enum_name, variant).is_none() {
                                let location = SourceLocation::new(file_path.clone(), 0, 0);
                                return Err(CompilerError::new(
                                    ErrorKind::UndefinedType,
                                    format!("enum `{}` has no variant `{}`", enum_name, variant),
                                    location,
                                ));
                            }
                            
                            matched_variants.insert(variant.clone());
                            
                            // Check scrutinee is this enum type
                            let pattern_ty = Type::Enum(enum_name.clone());
                            if !types_compatible(&scrutinee_ty, &pattern_ty) {
                                let location = SourceLocation::new(file_path.clone(), 0, 0);
                                return Err(CompilerError::new(
                                    ErrorKind::TypeMismatch,
                                    format!("pattern type `{:?}` doesn't match scrutinee type `{:?}`", pattern_ty, scrutinee_ty),
                                    location,
                                ));
                            }
                        } else {
                            let location = SourceLocation::new(file_path.clone(), 0, 0);
                            return Err(CompilerError::new(
                                ErrorKind::UndefinedType,
                                format!("enum `{}` not found", enum_name),
                                location,
                            ));
                        }
                    }
                }
            }
            
            // Check exhaustiveness for enum matches
            if let Type::Enum(enum_name) = &scrutinee_ty {
                if !has_wildcard {
                    // Get all variants from the enum definition
                    if let Some(variants_map) = symbol_table.enum_defs.get(enum_name) {
                        let all_variants: std::collections::HashSet<_> = variants_map.keys().cloned().collect();
                        let missing: Vec<_> = all_variants.difference(&matched_variants).collect();
                        
                        if !missing.is_empty() {
                            let location = SourceLocation::new(file_path.clone(), 0, 0);
                            return Err(CompilerError::new(
                                ErrorKind::InvalidSyntax,
                                format!("non-exhaustive match on enum `{}`, missing variants: {:?}", enum_name, missing),
                                location,
                            ).with_suggestion(Suggestion::simple(
                                "add a wildcard pattern `_` or match all remaining variants"
                            )));
                        }
                    }
                }
            }
            
            // All arms must have compatible types
            // For each arm, we need to analyze the expression with bound variables in scope
            let first_arm_ty = {
                symbol_table.enter_scope();
                
                // Add bound variables from the pattern to the scope
                if let Pattern::EnumVariant { enum_name, variant: _, binding } = &arms[0].pattern {
                    if let Some(binding_name) = binding {
                        // Determine the type of the bound variable
                        let bound_type = if symbol_table.builtins.is_generic_builtin(enum_name) {
                            // For built-in generic types, extract the type parameter
                            if let Type::Generic { ref type_params, .. } = scrutinee_ty {
                                if !type_params.is_empty() {
                                    type_params[0].clone()
                                } else {
                                    Type::Int // Fallback
                                }
                            } else {
                                Type::Int // Fallback
                            }
                        } else {
                            Type::Int // User-defined enums don't support values yet
                        };
                        
                        let binding_symbol = Symbol {
                            name: binding_name.clone(),
                            symbol_type: SymbolType::Variable,
                            ty: bound_type,
                        };
                        
                        symbol_table.insert(binding_symbol, file_path)?;
                    }
                }
                
                let ty = infer_type(&arms[0].expression, symbol_table, file_path)?;
                symbol_table.exit_scope();
                ty
            };
            
            for arm in &arms[1..] {
                symbol_table.enter_scope();
                
                // Add bound variables from the pattern to the scope
                if let Pattern::EnumVariant { enum_name, variant: _, binding } = &arm.pattern {
                    if let Some(binding_name) = binding {
                        // Determine the type of the bound variable
                        let bound_type = if symbol_table.builtins.is_generic_builtin(enum_name) {
                            // For built-in generic types, extract the type parameter
                            if let Type::Generic { ref type_params, .. } = scrutinee_ty {
                                if !type_params.is_empty() {
                                    type_params[0].clone()
                                } else {
                                    Type::Int // Fallback
                                }
                            } else {
                                Type::Int // Fallback
                            }
                        } else {
                            Type::Int // User-defined enums don't support values yet
                        };
                        
                        let binding_symbol = Symbol {
                            name: binding_name.clone(),
                            symbol_type: SymbolType::Variable,
                            ty: bound_type,
                        };
                        
                        symbol_table.insert(binding_symbol, file_path)?;
                    }
                }
                
                let arm_ty = infer_type(&arm.expression, symbol_table, file_path)?;
                symbol_table.exit_scope();
                
                if !types_compatible(&first_arm_ty, &arm_ty) {
                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                    return Err(CompilerError::new(
                        ErrorKind::TypeMismatch,
                        format!("match arms must have compatible types: `{:?}` vs `{:?}`", first_arm_ty, arm_ty),
                        location,
                    ).with_suggestion(Suggestion::simple(
                        "ensure all match arms return the same type"
                    )));
                }
            }
            
            // Return type of first arm (all are compatible)
            Ok(first_arm_ty)
        }
        Expression::TryOperator { expression } => {
            // The ? operator unwraps Result<T, E> or Option<T> and propagates errors
            
            // First, infer the type of the expression being unwrapped
            let expr_ty = infer_type(expression, symbol_table, file_path)?;
            
            // Then check that we're in a function with a compatible return type
            let current_ret_ty = symbol_table.current_function_return_type.clone()
                .ok_or_else(|| {
                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                    CompilerError::new(
                        ErrorKind::InvalidSyntax,
                        "? operator can only be used inside functions".to_string(),
                        location,
                    )
                })?;
            
            // The expression must be Result<T, E> or Option<T>
            match &expr_ty {
                Type::Generic { name, type_params } if name == "Result" || name == "Option" => {
                    // Validate the function return type is compatible
                    match &current_ret_ty {
                        Type::Generic { name: ret_name, type_params: ret_params } 
                            if ret_name == name => {
                            // For Result, error types must match
                            if name == "Result" && type_params.len() >= 2 && ret_params.len() >= 2 {
                                if !types_compatible(&type_params[1], &ret_params[1]) {
                                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                                    return Err(CompilerError::new(
                                        ErrorKind::TypeMismatch,
                                        format!(
                                            "? operator error type mismatch: expression has error type `{:?}` but function returns error type `{:?}`",
                                            type_params[1], ret_params[1]
                                        ),
                                        location,
                                    ).with_suggestion(Suggestion::simple(
                                        "ensure the error types match between the expression and function return type"
                                    )));
                                }
                            }
                            
                            // The ? operator returns the unwrapped value type (T)
                            if !type_params.is_empty() {
                                Ok(type_params[0].clone())
                            } else {
                                Ok(Type::Int) // Fallback
                            }
                        }
                        _ => {
                            let location = SourceLocation::new(file_path.clone(), 0, 0);
                            Err(CompilerError::new(
                                ErrorKind::TypeMismatch,
                                format!(
                                    "? operator used on `{:?}` but function returns `{:?}`",
                                    expr_ty, current_ret_ty
                                ),
                                location,
                            ).with_suggestion(Suggestion::simple(
                                &format!("change function return type to `{:?}` or remove the ? operator", expr_ty)
                            )))
                        }
                    }
                }
                _ => {
                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                    Err(CompilerError::new(
                        ErrorKind::TypeMismatch,
                        format!("? operator can only be used on Result<T, E> or Option<T>, found `{:?}`", expr_ty),
                        location,
                    ).with_suggestion(Suggestion::simple(
                        "? operator is for error propagation and optional value handling"
                    )))
                }
            }
        }
        Expression::MethodCall { object, method, arguments } => {
            // Method call: object.method(args)
            let mut object_ty = infer_type(object, symbol_table, file_path)?;
            
            // Normalize str to String type
            if let Type::Struct(ref name) = object_ty {
                if name == "str" {
                    object_ty = Type::String;
                }
            }
            
            match (&object_ty, method.as_str()) {
                // String methods
                (&Type::String, "length") => {
                    if !arguments.is_empty() {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        return Err(CompilerError::new(
                            ErrorKind::WrongArgumentCount,
                            format!("length() expects 0 arguments, got {}", arguments.len()),
                            location,
                        ));
                    }
                    Ok(Type::Int)
                }
                (&Type::String, "substring") => {
                    if arguments.len() != 2 {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        return Err(CompilerError::new(
                            ErrorKind::WrongArgumentCount,
                            format!("substring() expects 2 arguments (start, end), got {}", arguments.len()),
                            location,
                        ).with_suggestion(Suggestion::simple(
                            "usage: str.substring(start_index, end_index)"
                        )));
                    }
                    // Validate arguments are integers
                    for (i, arg) in arguments.iter().enumerate() {
                        let arg_ty = infer_type(arg, symbol_table, file_path)?;
                        if arg_ty != Type::Int {
                            let location = SourceLocation::new(file_path.clone(), 0, 0);
                            return Err(CompilerError::new(
                                ErrorKind::TypeMismatch,
                                format!("substring() argument {} must be int, got `{:?}`", i + 1, arg_ty),
                                location,
                            ));
                        }
                    }
                    Ok(Type::String)
                }
                (&Type::String, "contains") => {
                    if arguments.len() != 1 {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        return Err(CompilerError::new(
                            ErrorKind::WrongArgumentCount,
                            format!("contains() expects 1 argument, got {}", arguments.len()),
                            location,
                        ).with_suggestion(Suggestion::simple(
                            "usage: str.contains(needle)"
                        )));
                    }
                    let arg_ty = infer_type(&arguments[0], symbol_table, file_path)?;
                    if arg_ty != Type::String {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        return Err(CompilerError::new(
                            ErrorKind::TypeMismatch,
                            format!("contains() expects string argument, got `{:?}`", arg_ty),
                            location,
                        ));
                    }
                    Ok(Type::Bool)
                }
                (&Type::String, "trim") => {
                    if !arguments.is_empty() {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        return Err(CompilerError::new(
                            ErrorKind::WrongArgumentCount,
                            format!("trim() expects 0 arguments, got {}", arguments.len()),
                            location,
                        ));
                    }
                    Ok(Type::String)
                }
                (&Type::String, "split") => {
                    if arguments.len() != 1 {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        return Err(CompilerError::new(
                            ErrorKind::WrongArgumentCount,
                            format!("split() expects 1 argument (delimiter), got {}", arguments.len()),
                            location,
                        ).with_suggestion(Suggestion::simple(
                            "usage: str.split(delimiter)"
                        )));
                    }
                    let arg_ty = infer_type(&arguments[0], symbol_table, file_path)?;
                    if arg_ty != Type::String && arg_ty != Type::Char {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        return Err(CompilerError::new(
                            ErrorKind::TypeMismatch,
                            format!("split() expects string or char delimiter, got `{:?}`", arg_ty),
                            location,
                        ));
                    }
                    // Returns a dynamic array of strings
                    Ok(Type::DynamicArray(Box::new(Type::String)))
                }
                // Dynamic array methods
                (&Type::DynamicArray(ref elem_ty), "push") => {
                    if arguments.len() != 1 {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        return Err(CompilerError::new(
                            ErrorKind::WrongArgumentCount,
                            format!("push() expects 1 argument, got {}", arguments.len()),
                            location,
                        ));
                    }
                    let arg_ty = infer_type(&arguments[0], symbol_table, file_path)?;
                    if !types_compatible(&elem_ty, &arg_ty) {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        return Err(CompilerError::new(
                            ErrorKind::TypeMismatch,
                            format!("push() expects element of type `{:?}`, got `{:?}`", elem_ty, arg_ty),
                            location,
                        ));
                    }
                    Ok(Type::Void)
                }
                (&Type::DynamicArray(ref elem_ty), "pop") => {
                    if !arguments.is_empty() {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        return Err(CompilerError::new(
                            ErrorKind::WrongArgumentCount,
                            format!("pop() expects 0 arguments, got {}", arguments.len()),
                            location,
                        ));
                    }
                    Ok(*elem_ty.clone())
                }
                (&Type::DynamicArray(_), "length") => {
                    if !arguments.is_empty() {
                        let location = SourceLocation::new(file_path.clone(), 0, 0);
                        return Err(CompilerError::new(
                            ErrorKind::WrongArgumentCount,
                            format!("length() expects 0 arguments, got {}", arguments.len()),
                            location,
                        ));
                    }
                    Ok(Type::Int)
                }
                _ => {
                    let location = SourceLocation::new(file_path.clone(), 0, 0);
                    Err(CompilerError::new(
                        ErrorKind::UndefinedFunction,
                        format!("unknown method `{}` on type `{:?}`", method, object_ty),
                        location,
                    ).with_suggestion(Suggestion::simple(
                        "check the method name or ensure the type supports this operation"
                    )))
                }
            }
        }
    }
}

fn types_compatible(left: &Type, right: &Type) -> bool {
    // Direct equality
    if left == right {
        return true;
    }
    
    // Handle struct/enum ambiguity:
    // Parser can't distinguish between enum and struct names in type annotations
    // So Type::Struct("Foo") and Type::Enum("Foo") should be compatible if they refer to the same type
    match (left, right) {
        (Type::Struct(name1), Type::Enum(name2)) | (Type::Enum(name1), Type::Struct(name2)) => {
            name1 == name2
        }
        // Handle str/String compatibility
        // Type annotation `str` becomes Type::Struct("str"), but string literals are Type::String
        (Type::Struct(name), Type::String) | (Type::String, Type::Struct(name)) => {
            name == "str"
        }
        // Handle DynamicArray with str/String element types
        (Type::DynamicArray(inner1), Type::DynamicArray(inner2)) => {
            types_compatible(inner1, inner2)
        }
        // Handle qualified vs unqualified type names
        // e.g., ast.AstType should match AstType
        (Type::Struct(name1), Type::Struct(name2)) => {
            // Check if one is qualified and one is not
            if name1.contains('.') && !name2.contains('.') {
                // name1 is qualified (e.g., "ast.AstType"), name2 is not (e.g., "AstType")
                name1.ends_with(&format!(".{}", name2))
            } else if !name1.contains('.') && name2.contains('.') {
                // name2 is qualified, name1 is not
                name2.ends_with(&format!(".{}", name1))
            } else {
                false
            }
        }
        // Handle pointers with qualified types
        (Type::Pointer(inner1), Type::Pointer(inner2)) => {
            types_compatible(inner1, inner2)
        }
        _ => false
    }
}

// Determine if a block of statements guarantees a return on all control-flow paths
fn block_returns(stmts: &Vec<Statement>, symbol_table: &mut SymbolTable, file_path: &PathBuf) -> Result<bool, CompilerError> {
    let mut guaranteed = false;
    for stmt in stmts {
        match stmt {
            Statement::Return(_) => { return Ok(true); }
            Statement::If { then_branch, else_branch, .. } => {
                let then_ret = block_returns(then_branch, symbol_table, file_path)?;
                let else_ret = if let Some(else_b) = else_branch {
                    block_returns(else_b, symbol_table, file_path)?
                } else { false };
                if then_ret && else_ret { return Ok(true); }
            }
            Statement::While { body, .. } => {
                // A while-loop alone cannot guarantee return (it may not execute)
                let _ = block_returns(body, symbol_table, file_path)?; // analyze nested but ignore for guarantee
            }
            Statement::For { body, .. } => {
                // Similarly, for-loops don't guarantee return by themselves
                let _ = block_returns(body, symbol_table, file_path)?;
            }
            _ => {}
        }
        guaranteed = false;
    }
    Ok(guaranteed)
}