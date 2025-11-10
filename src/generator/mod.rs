// Generator module
// Generates C code from AST

pub mod c_codegen;
pub mod builtin;

use crate::parser::ast::Program;
use c_codegen::CCodeGenerator;

/// Generate C code from AST
pub fn generate(program: &Program) -> Result<String, String> {
    let generator = CCodeGenerator::new();
    generator.generate(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;
    use crate::parser;

    #[test]
    fn test_generate_simple_main() {
        let source = r#"
def main() -> i32 {
    return 0;
}
"#;

        let tokens = lexer::tokenize(source).unwrap();
        let program = parser::parse(tokens, source.to_string(), "test.peach".to_string()).unwrap();
        let c_code = generate(&program).unwrap();

        assert!(c_code.contains("#include <stdint.h>"));
        assert!(c_code.contains("int32_t main(void)"));
        assert!(c_code.contains("return 0;"));
    }
}