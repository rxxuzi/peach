#include "lexer.h"
#include <cctype>
#include <stdexcept>

std::unordered_map<std::string, TokenType> Lexer::keywords = {
    {"val", TokenType::VAL},
    {"var", TokenType::VAR},
    {"def", TokenType::DEF},
    {"if", TokenType::IF},
    {"else", TokenType::ELSE},
    {"while", TokenType::WHILE},
    {"for", TokenType::FOR},
    {"return", TokenType::RETURN},
    {"true", TokenType::TRUE},
    {"false", TokenType::FALSE},
    {"void", TokenType::VOID},
    {"struct", TokenType::STRUCT},
    {"union", TokenType::UNION},
    {"impl", TokenType::IMPL},
    {"int", TokenType::INT_TYPE},
    {"long", TokenType::LONG_TYPE},
    {"float", TokenType::FLOAT_TYPE},
    {"double", TokenType::DOUBLE_TYPE},
    {"bool", TokenType::BOOL_TYPE},
    {"string", TokenType::STRING_TYPE}
};

Lexer::Lexer(const std::string& src) : source(src), current(0), line(1), column(1) {}

bool Lexer::isAtEnd() const {
    return current >= source.length();
}

char Lexer::advance() {
    char c = source[current++];
    if (c == '\n') {
        line++;
        column = 1;
    } else {
        column++;
    }
    return c;
}

char Lexer::peek() const {
    if (isAtEnd()) return '\0';
    return source[current];
}

char Lexer::peekNext() const {
    if (current + 1 >= source.length()) return '\0';
    return source[current + 1];
}

bool Lexer::match(char expected) {
    if (isAtEnd()) return false;
    if (source[current] != expected) return false;
    advance();
    return true;
}

void Lexer::skipWhitespace() {
    while (!isAtEnd()) {
        char c = peek();
        switch (c) {
            case ' ':
            case '\r':
            case '\t':
                advance();
                break;
            case '\n':
                // In Peach, newlines can be significant for statement separation
                // We'll still skip them here, but the parser needs to handle statement boundaries
                advance();
                break;
            case '/':
                if (peekNext() == '/') {
                    // Comment until end of line
                    while (peek() != '\n' && !isAtEnd()) advance();
                } else {
                    return;
                }
                break;
            default:
                return;
        }
    }
}

Token Lexer::makeToken(TokenType type) {
    return Token(type, "", line, column);
}

Token Lexer::makeToken(TokenType type, const std::string& value) {
    return Token(type, value, line, column);
}

Token Lexer::errorToken(const std::string& message) {
    return Token(TokenType::UNKNOWN, message, line, column);
}

bool Lexer::isDigit(char c) const {
    return c >= '0' && c <= '9';
}

bool Lexer::isAlpha(char c) const {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
}

bool Lexer::isAlphaNumeric(char c) const {
    return isAlpha(c) || isDigit(c);
}

Token Lexer::scanString() {
    std::string value;
    int startLine = line;
    int startCol = column;
    
    // Skip opening quote
    advance();
    
    while (peek() != '"' && !isAtEnd()) {
        if (peek() == '\n') {
            return errorToken("Unterminated string");
        }
        if (peek() == '\\') {
            advance();
            switch (peek()) {
                case 'n': value += '\n'; break;
                case 't': value += '\t'; break;
                case 'r': value += '\r'; break;
                case '\\': value += '\\'; break;
                case '"': value += '"'; break;
                default:
                    return errorToken("Invalid escape sequence");
            }
            advance();
        } else {
            value += advance();
        }
    }
    
    if (isAtEnd()) {
        return errorToken("Unterminated string");
    }
    
    // Skip closing quote
    advance();
    
    return Token(TokenType::STRING_LITERAL, value, startLine, startCol);
}

Token Lexer::scanNumber() {
    std::string value;
    int startLine = line;
    int startCol = column;
    
    bool isFloat = false;
    bool isLong = false;
    
    while (isDigit(peek())) {
        value += advance();
    }
    
    // Look for decimal part
    if (peek() == '.' && isDigit(peekNext())) {
        isFloat = true;
        value += advance(); // consume '.'
        while (isDigit(peek())) {
            value += advance();
        }
    }
    
    // Check for type suffixes
    if (peek() == 'L' || peek() == 'l') {
        advance();
        isLong = true;
    } else if (peek() == 'f' || peek() == 'F') {
        advance();
        isFloat = true;
    } else if (peek() == 'd' || peek() == 'D') {
        advance();
        return Token(TokenType::DOUBLE_LITERAL, value, startLine, startCol);
    }
    
    if (isFloat) {
        return Token(TokenType::FLOAT_LITERAL, value, startLine, startCol);
    } else if (isLong) {
        return Token(TokenType::LONG_LITERAL, value, startLine, startCol);
    } else {
        return Token(TokenType::INT_LITERAL, value, startLine, startCol);
    }
}

Token Lexer::scanIdentifier() {
    std::string value;
    int startLine = line;
    int startCol = column;
    
    while (isAlphaNumeric(peek())) {
        value += advance();
    }
    
    // Check if it's a keyword
    auto it = keywords.find(value);
    if (it != keywords.end()) {
        return Token(it->second, value, startLine, startCol);
    }
    
    return Token(TokenType::IDENTIFIER, value, startLine, startCol);
}

Token Lexer::scanToken() {
    skipWhitespace();
    
    if (isAtEnd()) {
        return makeToken(TokenType::END_OF_FILE);
    }
    
    char c = advance();
    
    if (isAlpha(c)) {
        current--;
        column--;
        return scanIdentifier();
    }
    
    if (isDigit(c)) {
        current--;
        column--;
        return scanNumber();
    }
    
    switch (c) {
        case '(': return makeToken(TokenType::LPAREN);
        case ')': return makeToken(TokenType::RPAREN);
        case '{': return makeToken(TokenType::LBRACE);
        case '}': return makeToken(TokenType::RBRACE);
        case ';': return makeToken(TokenType::SEMICOLON);
        case ',': return makeToken(TokenType::COMMA);
        case ':': return makeToken(TokenType::COLON);
        case '.': return makeToken(TokenType::DOT);
        case '[': return makeToken(TokenType::LBRACKET);
        case ']': return makeToken(TokenType::RBRACKET);
        case '+': return makeToken(TokenType::PLUS);
        case '-':
            if (match('>')) {
                return makeToken(TokenType::ARROW);
            }
            return makeToken(TokenType::MINUS);
        case '*': return makeToken(TokenType::STAR);
        case '/': return makeToken(TokenType::SLASH);
        case '%': return makeToken(TokenType::PERCENT);
        case '&':
            if (match('&')) {
                return makeToken(TokenType::AND);
            }
            return makeToken(TokenType::AMPERSAND);
        case '|':
            if (match('|')) {
                return makeToken(TokenType::OR);
            }
            return errorToken("Unexpected character");
        case '!':
            if (match('=')) {
                return makeToken(TokenType::NE);
            }
            return makeToken(TokenType::NOT);
        case '=':
            if (match('=')) {
                return makeToken(TokenType::EQ);
            }
            return makeToken(TokenType::ASSIGN);
        case '<':
            if (match('=')) {
                return makeToken(TokenType::LE);
            } else if (match('-')) {
                return makeToken(TokenType::LEFT_ARROW);
            }
            return makeToken(TokenType::LT);
        case '>':
            if (match('=')) {
                return makeToken(TokenType::GE);
            }
            return makeToken(TokenType::GT);
        case '"':
            current--;
            column--;
            return scanString();
        default:
            return errorToken("Unexpected character");
    }
}

std::vector<Token> Lexer::tokenize() {
    std::vector<Token> tokens;
    
    while (!isAtEnd()) {
        Token token = scanToken();
        if (token.type == TokenType::UNKNOWN) {
            throw std::runtime_error("Lexical error at line " + std::to_string(token.line) + 
                                   ", column " + std::to_string(token.column) + 
                                   ": " + token.value);
        }
        tokens.push_back(token);
    }
    
    tokens.push_back(makeToken(TokenType::END_OF_FILE));
    return tokens;
}