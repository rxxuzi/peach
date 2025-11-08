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

/// Compilation options
#[derive(Debug, Clone)]
pub struct CompileOptions {
    /// Show verbose output (progress, details)
    pub verbose: bool,
    /// Only perform type checking, don't generate code
    pub check_only: bool,
    /// Custom output file path
    pub output_path: Option<String>,
    /// Show warnings (placeholder for future)
    pub warnings: bool,
    /// Use build/ directory for output
    pub use_build_dir: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        CompileOptions {
            verbose: true,
            check_only: false,
            output_path: None,
            warnings: false,
            use_build_dir: true,
        }
    }
}

/// Main compilation function (backward compatible)
pub fn compile(input_file: &str) -> Result<(), String> {
    compile_with_options(input_file, CompileOptions::default())
}

/// Compilation with options
pub fn compile_with_options(input_file: &str, options: CompileOptions) -> Result<(), String> {
    // Read input file
    let source = read_file(input_file)
        .map_err(|e| format!("Failed to read file '{}': {}", input_file, e))?;

    // Create message formatter for error reporting
    let formatter = helper::MessageFormatter::new(source.clone(), input_file.to_string());

    // Lexer: Tokenize
    if options.verbose {
        println!("[1/4] Lexing...");
    }
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
    if options.verbose {
        println!("  ✓ {} tokens", tokens.len());
    }

    // Parser: Build AST
    if options.verbose {
        println!("[2/4] Parsing...");
    }
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
    if options.verbose {
        println!("  ✓ AST constructed");
    }

    // Semantic Analysis: Type checking
    if options.verbose {
        println!("[3/4] Semantic analysis...");
    }
    match semantic::analyze(&mut ast, &source, input_file) {
        Ok(_) => {},
        Err(_) => {
            // Error already reported by analyze
            return Err("Type checking failed".to_string());
        }
    };
    if options.verbose {
        println!("  ✓ Type checking passed");
    }

    // If check-only mode, stop here
    if options.check_only {
        return Ok(());
    }

    // Generator: Generate C code
    if options.verbose {
        println!("[4/4] Generating C code...");
    }
    let c_code = generator::generate(&ast)?;

    // Write output
    let output_file = if let Some(ref custom_path) = options.output_path {
        custom_path.clone()
    } else if options.use_build_dir {
        get_output_path_build(input_file)
    } else {
        get_output_path_current(input_file)
    };

    write_file(&output_file, &c_code)
        .map_err(|e| format!("Failed to write file '{}': {}", output_file, e))?;

    if options.verbose {
        println!("  ✓ Generated: {}", output_file);
    }

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

fn get_output_path_build(input_file: &str) -> String {
    let path = Path::new(input_file);
    let stem = path.file_stem().unwrap().to_str().unwrap();
    format!("build/{}.c", stem)
}

fn get_output_path_current(input_file: &str) -> String {
    let path = Path::new(input_file);
    let stem = path.file_stem().unwrap().to_str().unwrap();
    format!("{}.c", stem)
}