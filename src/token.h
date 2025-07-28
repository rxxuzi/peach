#pragma once
#include <string>

enum class TokenType {
    // Keywords
    VAL, VAR, DEF, IF, ELSE, WHILE, FOR, RETURN, TRUE, FALSE, VOID, STRUCT, UNION, IMPL,
    
    // Types
    INT_TYPE, LONG_TYPE, FLOAT_TYPE, DOUBLE_TYPE, BOOL_TYPE, STRING_TYPE,
    
    // Literals
    INT_LITERAL, LONG_LITERAL, FLOAT_LITERAL, DOUBLE_LITERAL, STRING_LITERAL,
    
    // Identifiers
    IDENTIFIER,
    
    // Operators
    PLUS, MINUS, STAR, SLASH, PERCENT,
    ASSIGN, EQ, NE, LT, GT, LE, GE,
    AND, OR, NOT,
    AMPERSAND, // &
    LEFT_ARROW, // <-
    
    // Delimiters
    LPAREN, RPAREN, LBRACE, RBRACE, LBRACKET, RBRACKET,
    SEMICOLON, COMMA, COLON, ARROW, DOT,
    
    // Special
    NEWLINE,
    END_OF_FILE,
    UNKNOWN
};

struct Token {
    TokenType type;
    std::string value;
    int line;
    int column;
    
    Token(TokenType t, const std::string& v, int l, int c) 
        : type(t), value(v), line(l), column(c) {}
};