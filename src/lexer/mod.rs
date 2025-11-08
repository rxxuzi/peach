// Lexer module
// Tokenizes Peach source code

pub mod token;
pub mod scanner;

use token::Token;
use scanner::Scanner;

/// Tokenize source code
pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let scanner = Scanner::new(source);
    scanner.scan_tokens()
}

#[cfg(test)]
mod tests {
    use super::*;
    use token::TokenType;

    #[test]
    fn test_simple_main() {
        let source = r#"
def main() -> i32 {
    return 0;
}
"#;

        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Def);
        assert_eq!(tokens[1].token_type, TokenType::Identifier("main".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::LeftParen);
        assert_eq!(tokens[3].token_type, TokenType::RightParen);
        assert_eq!(tokens[4].token_type, TokenType::Arrow);
        assert_eq!(tokens[5].token_type, TokenType::I32);
        assert_eq!(tokens[6].token_type, TokenType::LeftBrace);
        assert_eq!(tokens[7].token_type, TokenType::Return);
        assert_eq!(tokens[8].token_type, TokenType::IntLiteral("0".to_string()));
        assert_eq!(tokens[9].token_type, TokenType::Semicolon);
        assert_eq!(tokens[10].token_type, TokenType::RightBrace);
        assert_eq!(tokens[11].token_type, TokenType::Eof);
    }
}