// Error types and structures

use std::fmt;

/// Represents a location in source code
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(line: usize, column: usize) -> Self {
        Span { line, column }
    }
}

/// Compilation error with location information
#[derive(Debug, Clone)]
pub struct CompileError {
    pub message: String,
    pub span: Option<Span>,
    pub error_type: ErrorType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    LexerError,
    ParseError,
    TypeError,
    NameError,
    GeneratorError,
}

impl CompileError {
    pub fn new(message: String, span: Option<Span>, error_type: ErrorType) -> Self {
        CompileError {
            message,
            span,
            error_type,
        }
    }

    pub fn lexer_error(message: String, line: usize, column: usize) -> Self {
        CompileError {
            message,
            span: Some(Span::new(line, column)),
            error_type: ErrorType::LexerError,
        }
    }

    pub fn parse_error(message: String, line: usize, column: usize) -> Self {
        CompileError {
            message,
            span: Some(Span::new(line, column)),
            error_type: ErrorType::ParseError,
        }
    }

    pub fn type_error(message: String, line: usize, column: usize) -> Self {
        CompileError {
            message,
            span: Some(Span::new(line, column)),
            error_type: ErrorType::TypeError,
        }
    }

    pub fn name_error(message: String, line: usize, column: usize) -> Self {
        CompileError {
            message,
            span: Some(Span::new(line, column)),
            error_type: ErrorType::NameError,
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CompileError {}