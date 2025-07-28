#pragma once
#include <vector>
#include <memory>
#include "token.h"
#include "ast.h"

class Parser {
private:
    std::vector<Token> tokens;
    size_t current;
    
    bool isAtEnd() const;
    Token peek() const;
    Token previous() const;
    Token advance();
    bool check(TokenType type) const;
    bool match(TokenType type);
    bool match(std::initializer_list<TokenType> types);
    Token consume(TokenType type, const std::string& message);
    void synchronize();
    
    // Type parsing
    TypeNodePtr parseType();
    
    // Expression parsing
    ExprNodePtr parseExpression();
    ExprNodePtr parseAssignment();
    ExprNodePtr parseOr();
    ExprNodePtr parseAnd();
    ExprNodePtr parseEquality();
    ExprNodePtr parseComparison();
    ExprNodePtr parseAddition();
    ExprNodePtr parseMultiplication();
    ExprNodePtr parseUnary();
    ExprNodePtr parsePostfix();
    ExprNodePtr parsePrimary();
    
    // Statement parsing
    StmtNodePtr parseStatement();
    StmtNodePtr parseVarDeclaration();
    StmtNodePtr parseExpressionStatement();
    StmtNodePtr parseBlockStatement();
    StmtNodePtr parseIfStatement();
    StmtNodePtr parseWhileStatement();
    StmtNodePtr parseForStatement();
    StmtNodePtr parseReturnStatement();
    
    // Function parsing
    std::unique_ptr<FunctionNode> parseFunction();
    
    // Helper for parsing function calls
    std::vector<ExprNodePtr> parseArguments();
    
public:
    explicit Parser(const std::vector<Token>& toks);
    std::unique_ptr<ProgramNode> parse();
};