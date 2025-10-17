pub mod lexer;
pub mod parser;
pub mod ast;
pub mod semantic;
pub mod codegen;
pub mod modules;
pub mod error;
pub mod intrinsics;
pub mod builtins;

use std::fs;
use std::path::Path;

pub fn compile(file_path: &Path, output_file: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string(file_path)?;
    
    // Lexing
    let tokens = match lexer::tokenize(&source, &file_path.to_path_buf()) {
        Ok(tokens) => tokens,
        Err(error) => {
            error::report_error(&error);
            return Err(Box::new(error));
        }
    };
    
    // Parsing
    let ast = match parser::parse(tokens, file_path.to_path_buf()) {
        Ok(ast) => ast,
        Err(error) => {
            error::report_error(&error);
            return Err(Box::new(error));
        }
    };
    
    // Module resolution
    let cwd = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let mut resolver = modules::ModuleResolver::new(cwd.to_str().unwrap());
    let imported_symbols = match resolver.resolve_imports(&ast) {
        Ok(symbols) => symbols,
        Err(error) => {
            error::report_error(&error);
            return Err(Box::new(error));
        }
    };
    
    // Semantic analysis (with imported symbols)
    if let Err(error) = semantic::analyze_with_imports(&ast, &imported_symbols, &file_path.to_path_buf()) {
        error::report_error(&error);
        return Err(Box::new(error));
    }
    
    // Code generation
    if let Err(error) = codegen::generate(&ast, &mut resolver, output_file) {
        error::report_error(&error);
        return Err(Box::new(error));
    }
    
    Ok(())
}