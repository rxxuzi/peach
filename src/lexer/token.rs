// Token types for the Peach lexer

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Def,        // def
    Return,     // return
    Val,        // val
    Var,        // var
    Mut,        // mut
    If,         // if
    Else,       // else
    While,      // while
    Loop,       // loop
    For,        // for
    In,         // in
    Break,      // break
    Continue,   // continue
    Struct,     // struct
    Impl,       // impl
    SelfKeyword, // self (lowercase)

    // Types
    I8,         // i8
    I16,        // i16
    I32,        // i32
    I64,        // i64
    U8,         // u8
    U16,        // u16
    U32,        // u32
    U64,        // u64
    F32,        // f32
    F64,        // f64
    Bool,       // bool
    Void,       // void

    // Literals
    True,       // true
    False,      // false

    // Identifiers and literals
    Identifier(String),
    IntLiteral(String),
    FloatLiteral(String),

    // Arithmetic operators
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Percent,        // %

    // Comparison operators
    EqualEqual,     // ==
    BangEqual,      // !=
    Less,           // <
    Greater,        // >
    LessEqual,      // <=
    GreaterEqual,   // >=

    // Logical operators
    AmpAmp,         // &&
    PipePipe,       // ||
    Bang,           // !
    Amp,            // & (for references)

    // Assignment
    Equal,          // =

    // Symbols
    LeftParen,      // (
    RightParen,     // )
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    Arrow,          // ->
    LeftArrow,      // <-
    DotDot,         // ..
    Dot,            // .
    ColonColon,     // ::
    Colon,          // :
    Semicolon,      // ;
    Comma,          // ,

    // Special
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Token {
            token_type,
            lexeme,
            line,
            column,
        }
    }
}