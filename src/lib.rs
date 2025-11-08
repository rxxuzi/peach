// Peach Compiler Library
// Author: rxxuzi
// Repository: github.com/rxxuzi/peach

pub mod lexer;
pub mod parser;
pub mod generator;
pub mod semantic;
pub mod cli;
pub mod helper;

use std::fs;
use std::io;
use std::path::Path;

/// Main compilation function
pub fn compile(input_file: &str) -> Result<(), String> {
    // Read input file
    let source = read_file(input_file)
        .map_err(|e| format!("Failed to read file '{}': {}", input_file, e))?;

    println!("Source code:");
    println!("{}", source);
    println!();

    // Create message formatter for error reporting
    let formatter = helper::MessageFormatter::new(source.clone(), input_file.to_string());

    // Lexer: Tokenize
    println!("[1/4] Lexing...");
    let tokens = match lexer::tokenize(&source) {
        Ok(tokens) => tokens,
        Err(e) => {
            let error = helper::CompileError::new(
                e.clone(),
                None,
                helper::ErrorType::LexerError
            );
            formatter.report(&error);
            return Err(e);
        }
    };
    println!("  ✓ {} tokens", tokens.len());

    // Parser: Build AST
    println!("[2/4] Parsing...");
    let mut ast = match parser::parse(tokens) {
        Ok(ast) => ast,
        Err(e) => {
            let error = helper::CompileError::new(
                e.clone(),
                None,
                helper::ErrorType::ParseError
            );
            formatter.report(&error);
            return Err(e);
        }
    };
    println!("  ✓ AST constructed");

    // Semantic Analysis: Type checking
    println!("[3/4] Semantic analysis...");
    match semantic::analyze(&mut ast, &source, input_file) {
        Ok(_) => {},
        Err(_) => {
            // Error already reported by analyze
            return Err("Type checking failed".to_string());
        }
    };
    println!("  ✓ Type checking passed");

    // Generator: Generate C code
    println!("[4/4] Generating C code...");
    let c_code = generator::generate(&ast)?;

    // Write output
    let output_file = get_output_path(input_file);
    write_file(&output_file, &c_code)
        .map_err(|e| format!("Failed to write file '{}': {}", output_file, e))?;

    println!("  ✓ Generated: {}", output_file);

    Ok(())
}

fn read_file(path: &str) -> io::Result<String> {
    fs::read_to_string(path)
}

fn write_file(path: &str, content: &str) -> io::Result<()> {
    // Create build directory if it doesn't exist
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}

fn get_output_path(input_file: &str) -> String {
    let path = Path::new(input_file);
    let stem = path.file_stem().unwrap().to_str().unwrap();
    format!("build/{}.c", stem)
}