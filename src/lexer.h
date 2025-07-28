#pragma once
#include <string>
#include <vector>
#include <unordered_map>
#include "token.h"

class Lexer {
private:
    std::string source;
    size_t current;
    int line;
    int column;
    
    static std::unordered_map<std::string, TokenType> keywords;
    
    bool isAtEnd() const;
    char advance();
    char peek() const;
    char peekNext() const;
    bool match(char expected);
    void skipWhitespace();
    void skipComment();
    bool isStatementEnd() const;  // New method to check for statement boundaries
    
    Token scanToken();
    Token makeToken(TokenType type);
    Token makeToken(TokenType type, const std::string& value);
    Token errorToken(const std::string& message);
    
    Token scanString();
    Token scanNumber();
    Token scanIdentifier();
    
    bool isDigit(char c) const;
    bool isAlpha(char c) const;
    bool isAlphaNumeric(char c) const;
    
public:
    explicit Lexer(const std::string& src);
    std::vector<Token> tokenize();
};