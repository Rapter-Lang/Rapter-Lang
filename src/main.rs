use rapter_lang::compile;
use rapter_lang::lexer::tokenize;

use std::env;
use std::path::Path;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file.rapt> [-o output.c]", args[0]);
        std::process::exit(1);
    }
    
    let file_path = Path::new(&args[1]);
    if !file_path.exists() {
        eprintln!("File not found: {}", file_path.display());
        std::process::exit(1);
    }
    
    if args.len() > 2 && args[2] == "--tokens" {
        let source = fs::read_to_string(file_path).unwrap();
        let tokens = tokenize(&source, &file_path.to_path_buf()).unwrap();
        for token in tokens {
            println!("{:?}", token);
        }
        return;
    }
    
    // Parse -o flag for output file
    let output_file = if args.len() > 3 && args[2] == "-o" {
        Some(args[3].clone())
    } else {
        None
    };
    
    match compile(file_path, output_file.as_deref()) {
        Ok(_) => {
            if output_file.is_some() {
                eprintln!("Compilation successful!");
            } else {
                println!("Compilation successful!");
            }
        },
        Err(e) => {
            eprintln!("Compilation failed: {}", e);
            std::process::exit(1);
        }
    }
}
