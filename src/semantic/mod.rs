// Semantic analysis module
// Type checking, name resolution, etc.

pub mod type_checker;

use crate::parser::ast::Program;
use crate::helper::types::CompileError;
use type_checker::TypeChecker;

/// Perform semantic analysis on the AST with error reporting
pub fn analyze(program: &mut Program, _source: &str, filename: &str) -> Result<(), CompileError> {
    let mut type_checker = TypeChecker::new(filename.to_string());

    let result = type_checker.check_program(program);

    // Report all errors
    let errors = type_checker.get_errors();
    if !errors.is_empty() {
        // Print each error (they already have line numbers embedded)
        for error_msg in errors {
            eprintln!("{}", error_msg);
        }

        // Return the first error
        return Err(CompileError::new(
            errors[0].clone(),
            None,
            crate::helper::types::ErrorType::TypeError
        ));
    }

    result.map_err(|e| CompileError::new(
        e,
        None,
        crate::helper::types::ErrorType::TypeError
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;
    use crate::parser;

    #[test]
    fn test_type_check_simple_vars() {
        let source = r#"
def main() -> i32 {
    val x: i32 = 42;
    val y: i32 = 10;
    val z = x + y;
    return z;
}
"#;

        let tokens = lexer::tokenize(source).unwrap();
        let mut program = parser::parse(tokens).unwrap();
        let result = analyze(&mut program, source, "test.peach");

        assert!(result.is_ok());
    }

    #[test]
    fn test_type_check_undefined_variable() {
        let source = r#"
def main() -> i32 {
    val x = y + 1;
    return x;
}
"#;

        let tokens = lexer::tokenize(source).unwrap();
        let mut program = parser::parse(tokens).unwrap();
        let result = analyze(&mut program, source, "test.peach");

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Undefined variable"));
    }

    #[test]
    fn test_type_check_type_mismatch() {
        let source = r#"
def main() -> i32 {
    val x: i32 = 42;
    val y: bool = true;
    val z = x + y;
    return z;
}
"#;

        let tokens = lexer::tokenize(source).unwrap();
        let mut program = parser::parse(tokens).unwrap();
        let result = analyze(&mut program, source, "test.peach");

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Type mismatch"));
    }
}