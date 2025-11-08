// Parser module
// Builds Abstract Syntax Tree from tokens

pub mod ast;
pub mod parser;

use crate::lexer::token::Token;
use ast::Program;
use parser::Parser;

/// Parse tokens into AST
pub fn parse(tokens: Vec<Token>) -> Result<Program, String> {
    let parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;

    #[test]
    fn test_parse_simple_main() {
        let source = r#"
def main() -> i32 {
    return 0;
}
"#;

        let tokens = lexer::tokenize(source).unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].name, "main");
        assert_eq!(program.functions[0].return_type, ast::Type::I32);
        assert_eq!(program.functions[0].body.statements.len(), 1);
    }
}