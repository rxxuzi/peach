// Lexer scanner implementation

use super::token::{Token, TokenType};

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Scanner {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token>, String> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens.push(Token::new(
            TokenType::Eof,
            String::new(),
            self.line,
            self.column,
        ));

        Ok(self.tokens)
    }

    fn scan_token(&mut self) -> Result<(), String> {
        let c = self.advance();

        match c {
            ' ' | '\r' | '\t' => {
                // Skip whitespace
            }
            '\n' => {
                self.line += 1;
                self.column = 1;
            }
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            '[' => self.add_token(TokenType::LeftBracket),
            ']' => self.add_token(TokenType::RightBracket),
            ':' => {
                if self.match_char(':') {
                    self.add_token(TokenType::ColonColon);
                } else {
                    self.add_token(TokenType::Colon);
                }
            }
            ';' => self.add_token(TokenType::Semicolon),
            ',' => self.add_token(TokenType::Comma),
            '+' => self.add_token(TokenType::Plus),
            '*' => self.add_token(TokenType::Star),
            '%' => self.add_token(TokenType::Percent),
            '=' => {
                if self.match_char('=') {
                    self.add_token(TokenType::EqualEqual);
                } else {
                    self.add_token(TokenType::Equal);
                }
            }
            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenType::BangEqual);
                } else {
                    self.add_token(TokenType::Bang);
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.add_token(TokenType::LessEqual);
                } else if self.match_char('-') {
                    self.add_token(TokenType::LeftArrow);
                } else {
                    self.add_token(TokenType::Less);
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.add_token(TokenType::GreaterEqual);
                } else {
                    self.add_token(TokenType::Greater);
                }
            }
            '&' => {
                if self.match_char('&') {
                    self.add_token(TokenType::AmpAmp);
                } else {
                    self.add_token(TokenType::Amp);
                }
            }
            '|' => {
                if self.match_char('|') {
                    self.add_token(TokenType::PipePipe);
                } else {
                    return Err(format!("Unexpected character '|' at line {}", self.line));
                }
            }
            '/' => {
                if self.match_char('/') {
                    // Single-line comment
                    self.skip_line_comment();
                } else if self.match_char('*') {
                    // Multi-line comment
                    self.skip_block_comment()?;
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            '-' => {
                if self.match_char('>') {
                    self.add_token(TokenType::Arrow);
                } else {
                    self.add_token(TokenType::Minus);
                }
            }
            '.' => {
                if self.match_char('.') {
                    self.add_token(TokenType::DotDot);
                } else {
                    self.add_token(TokenType::Dot);
                }
            }
            _ => {
                if c.is_alphabetic() || c == '_' {
                    self.identifier();
                } else if c.is_ascii_digit() {
                    self.number();
                } else {
                    return Err(format!(
                        "Unexpected character '{}' at line {} column {}",
                        c, self.line, self.column
                    ));
                }
            }
        }

        Ok(())
    }

    fn skip_line_comment(&mut self) {
        while self.peek() != '\n' && !self.is_at_end() {
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> Result<(), String> {
        while !self.is_at_end() {
            if self.peek() == '*' {
                self.advance();
                if self.match_char('/') {
                    return Ok(());
                }
            } else if self.peek() == '\n' {
                self.line += 1;
                self.column = 0;
                self.advance();
            } else {
                self.advance();
            }
        }
        Err(format!("Unterminated block comment at line {}", self.line))
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text: String = self.source[self.start..self.current].iter().collect();
        let token_type = match text.as_str() {
            // Keywords
            "def" => TokenType::Def,
            "return" => TokenType::Return,
            "val" => TokenType::Val,
            "var" => TokenType::Var,
            "mut" => TokenType::Mut,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "loop" => TokenType::Loop,
            "for" => TokenType::For,
            "in" => TokenType::In,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "struct" => TokenType::Struct,
            "impl" => TokenType::Impl,
            "self" => TokenType::SelfKeyword,
            // Types
            "i8" => TokenType::I8,
            "i16" => TokenType::I16,
            "i32" => TokenType::I32,
            "i64" => TokenType::I64,
            "u8" => TokenType::U8,
            "u16" => TokenType::U16,
            "u32" => TokenType::U32,
            "u64" => TokenType::U64,
            "f32" => TokenType::F32,
            "f64" => TokenType::F64,
            "bool" => TokenType::Bool,
            "void" => TokenType::Void,
            // Literals
            "true" => TokenType::True,
            "false" => TokenType::False,
            // Identifier
            _ => TokenType::Identifier(text.clone()),
        };

        self.add_token(token_type);
    }

    fn number(&mut self) {
        // Integer part
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Check for decimal point
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Float
            self.advance(); // consume '.'

            while self.peek().is_ascii_digit() {
                self.advance();
            }

            let text: String = self.source[self.start..self.current].iter().collect();
            self.add_token(TokenType::FloatLiteral(text));
        } else {
            // Integer
            let text: String = self.source[self.start..self.current].iter().collect();
            self.add_token(TokenType::IntLiteral(text));
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        self.column += 1;
        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] != expected {
            return false;
        }

        self.current += 1;
        self.column += 1;
        true
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text: String = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token::new(
            token_type,
            text,
            self.line,
            self.column,
        ));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}