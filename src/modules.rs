use crate::ast::*;
use crate::lexer;
use crate::parser;
use crate::error::{CompilerError, ErrorKind, SourceLocation, Suggestion};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub program: Program,
    pub exports: HashMap<String, Symbol>,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub ty: Type,
    // For exported structs, include a map of field name -> field type
    pub fields: Option<std::collections::HashMap<String, Type>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Function,
    Struct,
    Enum,
    Variable,
    Parameter,
}

pub struct ModuleResolver {
    modules: HashMap<String, Module>,
    base_path: String,
}

impl ModuleResolver {
    pub fn new(base_path: &str) -> Self {
        ModuleResolver {
            modules: HashMap::new(),
            base_path: base_path.to_string(),
        }
    }

    pub fn load_module(&mut self, module_name: &str) -> Result<&Module, CompilerError> {
        if self.modules.contains_key(module_name) {
            return Ok(&self.modules[module_name]);
        }

        // Convert module name to file path (e.g., "std.io" -> "std/io.rapt")
        let file_path = module_name.replace(".", "/") + ".rapt";
        let full_path = Path::new(&self.base_path).join(&file_path);

        if !full_path.exists() {
            return Err(CompilerError::new(
                ErrorKind::ModuleNotFound,
                format!("Module '{}' not found", module_name),
                SourceLocation::new(full_path.clone(), 0, 0),
            ).with_suggestions(vec![Suggestion::simple(
                format!("Check if the module file exists at {}", full_path.display()),
            )]));
        }

        let source = fs::read_to_string(&full_path)
            .map_err(|e| CompilerError::new(
                ErrorKind::ModuleLoadError,
                format!("Failed to read module '{}': {}", module_name, e),
                SourceLocation::new(full_path.clone(), 0, 0),
            ).with_suggestions(vec![Suggestion::simple(
                format!("Check file permissions and ensure the file is not corrupted"),
            )]))?;

        let tokens = lexer::tokenize(&source, &full_path).map_err(|e| CompilerError::new(
            ErrorKind::ModuleLoadError,
            format!("Failed to tokenize module '{}': {}", module_name, e.message),
            e.location.clone(),
        ).with_suggestions(vec![Suggestion::simple(
            "Check the module file for syntax errors".to_string(),
        )]))?;
        let program = parser::parse(tokens, full_path.clone()).map_err(|e| CompilerError::new(
            ErrorKind::ModuleLoadError,
            format!("Failed to parse module '{}': {}", module_name, e.message),
            e.location.clone(),
        ).with_suggestions(vec![Suggestion::simple(
            "Check the module file for syntax errors".to_string(),
        )]))?;

        let exports = self.collect_exports(&program)?;

        let module = Module {
            name: module_name.to_string(),
            program,
            exports,
        };

        self.modules.insert(module_name.to_string(), module);
        Ok(&self.modules[module_name])
    }

    fn collect_exports(&self, program: &Program) -> Result<HashMap<String, Symbol>, CompilerError> {
        let mut exports = HashMap::new();

        for export in &program.exports {
            match &export.item {
                ExportItem::Function(name) => {
                    // Find the function in the program
                    if let Some(func) = program.functions.iter().find(|f| f.name == *name) {
                        let symbol = Symbol {
                            name: name.clone(),
                            symbol_type: SymbolType::Function,
                            ty: func.return_type.clone().unwrap_or(Type::Void),
                            fields: None,
                        };
                        exports.insert(name.clone(), symbol);
                    } else {
                        return Err(CompilerError::new(
                            ErrorKind::ModuleExportError,
                            format!("Exported function '{}' not found in module", name),
                            SourceLocation::new(PathBuf::from("<module>"), 0, 0),
                        ).with_suggestions(vec![Suggestion::simple(
                            format!("Ensure function '{}' is defined in the module", name),
                        )]));
                    }
                }
                ExportItem::Struct(name) => {
                    // Find the struct in the program
                    if let Some(st) = program.structs.iter().find(|s| s.name == *name) {
                        // Build field map for exported struct
                        let mut fields_map = HashMap::new();
                        for f in &st.fields {
                            fields_map.insert(f.name.clone(), f.field_type.clone());
                        }
                        let symbol = Symbol {
                            name: name.clone(),
                            symbol_type: SymbolType::Struct,
                            ty: Type::Struct(name.clone()),
                            fields: Some(fields_map),
                        };
                        exports.insert(name.clone(), symbol);
                    } else {
                        return Err(CompilerError::new(
                            ErrorKind::ModuleExportError,
                            format!("Exported struct '{}' not found in module", name),
                            SourceLocation::new(PathBuf::from("<module>"), 0, 0),
                        ).with_suggestions(vec![Suggestion::simple(
                            format!("Ensure struct '{}' is defined in the module", name),
                        )]));
                    }
                }
                ExportItem::Enum(name) => {
                    // Find the enum in the program
                    if let Some(enm) = program.enums.iter().find(|e| e.name == *name) {
                        // Collect variant names as Type::Int markers
                        // The fields map will be used to validate variant names exist
                        let mut variants_map = HashMap::new();
                        for variant in &enm.variants {
                            // Store Type::Int as a marker that this variant exists
                            variants_map.insert(variant.name.clone(), Type::Int);
                        }
                        
                        let symbol = Symbol {
                            name: name.clone(),
                            symbol_type: SymbolType::Enum,
                            ty: Type::Enum(name.clone()),
                            fields: Some(variants_map),
                        };
                        exports.insert(name.clone(), symbol);
                    } else {
                        return Err(CompilerError::new(
                            ErrorKind::ModuleExportError,
                            format!("Exported enum '{}' not found in module", name),
                            SourceLocation::new(PathBuf::from("<module>"), 0, 0),
                        ).with_suggestions(vec![Suggestion::simple(
                            format!("Ensure enum '{}' is defined in the module", name),
                        )]));
                    }
                }
            }
        }

        Ok(exports)
    }

    pub fn resolve_imports(&mut self, program: &Program) -> Result<HashMap<String, Symbol>, CompilerError> {
        let mut imported_symbols = HashMap::new();

        for import in &program.imports {
            let module = self.load_module(&import.module)?;

            // Determine the prefix for imported symbols
            let prefix = import.alias.as_ref().unwrap_or(&import.module);

            // Add all exported symbols in TWO forms:
            // 1. Unqualified name (direct access: Token, tokenize, etc.)
            // 2. Qualified name (module.symbol: token.Token, token.tokenize, etc.)
            for (name, symbol) in &module.exports {
                // Unqualified form - imports bring symbols directly into scope
                let unqualified_symbol = Symbol {
                    name: name.clone(),
                    symbol_type: symbol.symbol_type.clone(),
                    ty: symbol.ty.clone(),
                    fields: symbol.fields.clone(),
                };
                imported_symbols.insert(name.clone(), unqualified_symbol);
                
                // Qualified form - allows module.symbol for clarity/disambiguation
                let prefixed_name = format!("{}.{}", prefix, name);
                let prefixed_symbol = Symbol {
                    name: prefixed_name.clone(),
                    symbol_type: symbol.symbol_type.clone(),
                    ty: symbol.ty.clone(),
                    fields: symbol.fields.clone(),
                };
                imported_symbols.insert(prefixed_name, prefixed_symbol);
            }
        }

        Ok(imported_symbols)
    }
}