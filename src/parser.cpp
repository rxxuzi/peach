#include "parser.h"
#include <stdexcept>
#include <sstream>

Parser::Parser(const std::vector<Token>& toks) : tokens(toks), current(0) {}

bool Parser::isAtEnd() const {
    return peek().type == TokenType::END_OF_FILE;
}

Token Parser::peek() const {
    return tokens[current];
}

Token Parser::previous() const {
    return tokens[current - 1];
}

Token Parser::advance() {
    if (!isAtEnd()) current++;
    return previous();
}

bool Parser::check(TokenType type) const {
    if (isAtEnd()) return false;
    return peek().type == type;
}

bool Parser::match(TokenType type) {
    if (check(type)) {
        advance();
        return true;
    }
    return false;
}

bool Parser::match(std::initializer_list<TokenType> types) {
    for (TokenType type : types) {
        if (check(type)) {
            advance();
            return true;
        }
    }
    return false;
}

Token Parser::consume(TokenType type, const std::string& message) {
    if (check(type)) return advance();
    
    std::stringstream ss;
    ss << "Parse error at line " << peek().line << ", column " << peek().column 
       << ": " << message;
    throw std::runtime_error(ss.str());
}

void Parser::synchronize() {
    advance();
    
    while (!isAtEnd()) {
        if (previous().type == TokenType::SEMICOLON) return;
        
        switch (peek().type) {
            case TokenType::DEF:
            case TokenType::VAL:
            case TokenType::VAR:
            case TokenType::IF:
            case TokenType::WHILE:
            case TokenType::RETURN:
                return;
            default:
                advance();
        }
    }
}

TypeNodePtr Parser::parseType() {
    TypeNodePtr baseType;
    
    // Check for array type [N]T
    if (match(TokenType::LBRACKET)) {
        ExprNodePtr size = nullptr;
        if (!check(TokenType::RBRACKET)) {
            // Parse array size
            size = parseExpression();
        }
        consume(TokenType::RBRACKET, "Expected ']' after array size");
        
        // Parse element type
        TypeNodePtr elementType = parseType();
        return std::make_unique<ArrayTypeNode>(std::move(elementType), std::move(size));
    }
    
    // Check for pointer type *T
    if (match(TokenType::STAR)) {
        TypeNodePtr pointeeType = parseType();
        return std::make_unique<PointerTypeNode>(std::move(pointeeType));
    }
    
    // Basic types
    if (match({TokenType::INT_TYPE, TokenType::LONG_TYPE, TokenType::FLOAT_TYPE,
               TokenType::DOUBLE_TYPE, TokenType::BOOL_TYPE, TokenType::STRING_TYPE,
               TokenType::VOID})) {
        std::string typeName = previous().value;
        if (previous().type == TokenType::INT_TYPE) typeName = "int";
        else if (previous().type == TokenType::LONG_TYPE) typeName = "long";
        else if (previous().type == TokenType::FLOAT_TYPE) typeName = "float";
        else if (previous().type == TokenType::DOUBLE_TYPE) typeName = "double";
        else if (previous().type == TokenType::BOOL_TYPE) typeName = "bool";
        else if (previous().type == TokenType::STRING_TYPE) typeName = "string";
        else if (previous().type == TokenType::VOID) typeName = "void";
        
        baseType = std::make_unique<BasicTypeNode>(typeName);
    } else if (match(TokenType::IDENTIFIER)) {
        // This could be a struct type
        std::string typeName = previous().value;
        baseType = std::make_unique<StructTypeNode>(typeName);
    } else {
        throw std::runtime_error("Expected type");
    }
    
    return baseType;
}

ExprNodePtr Parser::parseExpression() {
    // For variable declarations, we need to be careful not to parse too far
    // This is a simple expression parser that stops at statement boundaries
    return parseAssignment();
}

ExprNodePtr Parser::parseAssignment() {
    ExprNodePtr expr = parseOr();
    
    if (match(TokenType::ASSIGN)) {
        // Check if the left side is a valid lvalue
        // For now, we'll allow any expression and let the code generator handle it
        // Non-associative: don't allow chained assignments in expressions
        ExprNodePtr value = parseOr(); // Changed from parseAssignment to parseOr
        return std::make_unique<BinaryOpNode>(std::move(expr), std::move(value), "=");
    }
    
    return expr;
}

ExprNodePtr Parser::parseOr() {
    ExprNodePtr expr = parseAnd();
    
    while (match(TokenType::OR)) {
        std::string op = "||";
        ExprNodePtr right = parseAnd();
        expr = std::make_unique<BinaryOpNode>(std::move(expr), std::move(right), op);
    }
    
    return expr;
}

ExprNodePtr Parser::parseAnd() {
    ExprNodePtr expr = parseEquality();
    
    while (match(TokenType::AND)) {
        std::string op = "&&";
        ExprNodePtr right = parseEquality();
        expr = std::make_unique<BinaryOpNode>(std::move(expr), std::move(right), op);
    }
    
    return expr;
}

ExprNodePtr Parser::parseEquality() {
    ExprNodePtr expr = parseComparison();
    
    while (match({TokenType::EQ, TokenType::NE})) {
        std::string op = previous().type == TokenType::EQ ? "==" : "!=";
        ExprNodePtr right = parseComparison();
        expr = std::make_unique<BinaryOpNode>(std::move(expr), std::move(right), op);
    }
    
    return expr;
}

ExprNodePtr Parser::parseComparison() {
    ExprNodePtr expr = parseAddition();
    
    while (match({TokenType::GT, TokenType::GE, TokenType::LT, TokenType::LE})) {
        std::string op;
        switch (previous().type) {
            case TokenType::GT: op = ">"; break;
            case TokenType::GE: op = ">="; break;
            case TokenType::LT: op = "<"; break;
            case TokenType::LE: op = "<="; break;
            default: break;
        }
        ExprNodePtr right = parseAddition();
        expr = std::make_unique<BinaryOpNode>(std::move(expr), std::move(right), op);
    }
    
    return expr;
}

ExprNodePtr Parser::parseAddition() {
    ExprNodePtr expr = parseMultiplication();
    
    while (match({TokenType::PLUS, TokenType::MINUS})) {
        std::string op = previous().type == TokenType::PLUS ? "+" : "-";
        ExprNodePtr right = parseMultiplication();
        expr = std::make_unique<BinaryOpNode>(std::move(expr), std::move(right), op);
    }
    
    return expr;
}

ExprNodePtr Parser::parseMultiplication() {
    ExprNodePtr expr = parseUnary();
    
    while (match({TokenType::STAR, TokenType::SLASH, TokenType::PERCENT})) {
        std::string op;
        switch (previous().type) {
            case TokenType::STAR: op = "*"; break;
            case TokenType::SLASH: op = "/"; break;
            case TokenType::PERCENT: op = "%"; break;
            default: break;
        }
        ExprNodePtr right = parseUnary();
        expr = std::make_unique<BinaryOpNode>(std::move(expr), std::move(right), op);
    }
    
    return expr;
}

ExprNodePtr Parser::parseUnary() {
    if (match({TokenType::NOT, TokenType::MINUS})) {
        std::string op = previous().type == TokenType::NOT ? "!" : "-";
        ExprNodePtr right = parseUnary();
        return std::make_unique<UnaryOpNode>(std::move(right), op);
    }
    
    if (match(TokenType::AMPERSAND)) {
        ExprNodePtr operand = parseUnary();
        return std::make_unique<AddressOfNode>(std::move(operand));
    }
    
    if (match(TokenType::STAR)) {
        ExprNodePtr operand = parseUnary();
        return std::make_unique<DereferenceNode>(std::move(operand));
    }
    
    return parsePostfix();
}

ExprNodePtr Parser::parsePostfix() {
    ExprNodePtr expr = parsePrimary();
    
    while (true) {
        if (match(TokenType::LPAREN)) {
            // Function call
            auto args = parseArguments();
            if (auto* id = dynamic_cast<IdentifierNode*>(expr.get())) {
                expr = std::make_unique<CallNode>(id->name, std::move(args));
            } else {
                throw std::runtime_error("Invalid function call");
            }
        } else if (match(TokenType::LBRACKET)) {
            // Array indexing
            ExprNodePtr index = parseExpression();
            consume(TokenType::RBRACKET, "Expected ']' after array index");
            expr = std::make_unique<IndexNode>(std::move(expr), std::move(index));
        } else if (match(TokenType::DOT)) {
            // Field access or method call
            Token fieldName = consume(TokenType::IDENTIFIER, "Expected field or method name after '.'");
            
            if (match(TokenType::LPAREN)) {
                // Method call
                auto args = parseArguments();
                expr = std::make_unique<MethodCallNode>(std::move(expr), fieldName.value, std::move(args));
            } else {
                // Field access
                expr = std::make_unique<FieldAccessNode>(std::move(expr), fieldName.value);
            }
        } else {
            break;
        }
    }
    
    return expr;
}

std::vector<ExprNodePtr> Parser::parseArguments() {
    std::vector<ExprNodePtr> args;
    
    if (!check(TokenType::RPAREN)) {
        do {
            args.push_back(parseExpression());
        } while (match(TokenType::COMMA));
    }
    
    consume(TokenType::RPAREN, "Expected ')' after arguments");
    return args;
}

ExprNodePtr Parser::parsePrimary() {
    if (match(TokenType::TRUE)) {
        return std::make_unique<BoolLiteralNode>(true);
    }
    
    if (match(TokenType::FALSE)) {
        return std::make_unique<BoolLiteralNode>(false);
    }
    
    if (match(TokenType::INT_LITERAL)) {
        return std::make_unique<IntLiteralNode>(std::stoi(previous().value));
    }
    
    if (match(TokenType::LONG_LITERAL)) {
        return std::make_unique<LongLiteralNode>(std::stol(previous().value));
    }
    
    if (match(TokenType::FLOAT_LITERAL)) {
        return std::make_unique<FloatLiteralNode>(std::stof(previous().value));
    }
    
    if (match(TokenType::DOUBLE_LITERAL)) {
        return std::make_unique<DoubleLiteralNode>(std::stod(previous().value));
    }
    
    if (match(TokenType::STRING_LITERAL)) {
        return std::make_unique<StringLiteralNode>(previous().value);
    }
    
    if (match(TokenType::IDENTIFIER)) {
        std::string identifier = previous().value;
        
        // Check for struct initialization: StructName { ... }
        if (check(TokenType::LBRACE)) {
            advance(); // consume '{'
            
            std::vector<std::pair<std::string, ExprNodePtr>> fields;
            
            if (!check(TokenType::RBRACE)) {
                do {
                    // Parse field initialization: .fieldName = value or just value
                    if (match(TokenType::DOT)) {
                        Token fieldName = consume(TokenType::IDENTIFIER, "Expected field name after '.'");
                        consume(TokenType::ASSIGN, "Expected '=' after field name");
                        ExprNodePtr value = parseExpression();
                        fields.emplace_back(fieldName.value, std::move(value));
                    } else {
                        // Positional initialization (without field names)
                        ExprNodePtr value = parseExpression();
                        fields.emplace_back("", std::move(value)); // Empty field name for positional
                    }
                } while (match(TokenType::COMMA));
            }
            
            consume(TokenType::RBRACE, "Expected '}' after struct fields");
            return std::make_unique<StructInitNode>(identifier, std::move(fields));
        }
        
        return std::make_unique<IdentifierNode>(identifier);
    }
    
    if (match(TokenType::LBRACE)) {
        // Array literal
        std::vector<ExprNodePtr> elements;
        if (!check(TokenType::RBRACE)) {
            do {
                elements.push_back(parseExpression());
            } while (match(TokenType::COMMA));
        }
        consume(TokenType::RBRACE, "Expected '}' after array elements");
        return std::make_unique<ArrayLiteralNode>(std::move(elements));
    }
    
    if (match(TokenType::LPAREN)) {
        ExprNodePtr expr = parseExpression();
        consume(TokenType::RPAREN, "Expected ')' after expression");
        return expr;
    }
    
    throw std::runtime_error("Expected expression");
}

StmtNodePtr Parser::parseStatement() {
    if (match({TokenType::VAL, TokenType::VAR})) {
        current--; // Go back
        return parseVarDeclaration();
    }
    
    if (match(TokenType::LBRACE)) {
        return parseBlockStatement();
    }
    
    if (match(TokenType::IF)) {
        return parseIfStatement();
    }
    
    if (match(TokenType::WHILE)) {
        return parseWhileStatement();
    }
    
    if (match(TokenType::FOR)) {
        return parseForStatement();
    }
    
    if (match(TokenType::RETURN)) {
        return parseReturnStatement();
    }
    
    return parseExpressionStatement();
}

StmtNodePtr Parser::parseVarDeclaration() {
    bool isConst = match(TokenType::VAL);
    if (!isConst) {
        consume(TokenType::VAR, "Expected 'val' or 'var'");
    }
    
    Token name = consume(TokenType::IDENTIFIER, "Expected variable name");
    
    TypeNodePtr type = nullptr;
    if (match(TokenType::COLON)) {
        type = parseType();
    }
    
    ExprNodePtr initializer = nullptr;
    if (match(TokenType::ASSIGN)) {
        initializer = parseExpression();
    } else if (isConst) {
        throw std::runtime_error("'val' declarations must be initialized");
    }
    
    // Consume optional semicolon for statement termination
    match(TokenType::SEMICOLON);
    return std::make_unique<VarDeclNode>(isConst, name.value, std::move(type), std::move(initializer));
}

StmtNodePtr Parser::parseExpressionStatement() {
    ExprNodePtr expr = parseExpression();
    // Expression statements are just expressions used as statements
    // Consume optional semicolon for statement termination
    match(TokenType::SEMICOLON);
    return std::make_unique<ExprStmtNode>(std::move(expr));
}

StmtNodePtr Parser::parseBlockStatement() {
    std::vector<StmtNodePtr> statements;
    
    while (!check(TokenType::RBRACE) && !isAtEnd()) {
        statements.push_back(parseStatement());
        // Each statement now handles its own semicolon consumption
    }
    
    consume(TokenType::RBRACE, "Expected '}' after block");
    return std::make_unique<BlockNode>(std::move(statements));
}

StmtNodePtr Parser::parseIfStatement() {
    consume(TokenType::LPAREN, "Expected '(' after 'if'");
    ExprNodePtr condition = parseExpression();
    consume(TokenType::RPAREN, "Expected ')' after condition");
    
    StmtNodePtr thenBranch = parseStatement();
    StmtNodePtr elseBranch = nullptr;
    
    if (match(TokenType::ELSE)) {
        elseBranch = parseStatement();
    }
    
    return std::make_unique<IfNode>(std::move(condition), std::move(thenBranch), std::move(elseBranch));
}

StmtNodePtr Parser::parseWhileStatement() {
    consume(TokenType::LPAREN, "Expected '(' after 'while'");
    ExprNodePtr condition = parseExpression();
    consume(TokenType::RPAREN, "Expected ')' after condition");
    
    StmtNodePtr body = parseStatement();
    
    return std::make_unique<WhileNode>(std::move(condition), std::move(body));
}

StmtNodePtr Parser::parseReturnStatement() {
    ExprNodePtr value = nullptr;
    if (!check(TokenType::SEMICOLON) && !check(TokenType::RBRACE)) {
        value = parseExpression();
    }
    // Consume optional semicolon for statement termination
    match(TokenType::SEMICOLON);
    return std::make_unique<ReturnNode>(std::move(value));
}

StmtNodePtr Parser::parseForStatement() {
    consume(TokenType::LPAREN, "Expected '(' after 'for'");
    
    Token iterator = consume(TokenType::IDENTIFIER, "Expected iterator name");
    consume(TokenType::LEFT_ARROW, "Expected '<-' after iterator name");
    
    ExprNodePtr collection = parseExpression();
    
    consume(TokenType::RPAREN, "Expected ')' after for clause");
    
    StmtNodePtr body = parseStatement();
    
    return std::make_unique<ForNode>(iterator.value, std::move(collection), std::move(body));
}

std::unique_ptr<FunctionNode> Parser::parseFunction() {
    consume(TokenType::DEF, "Expected 'def'");
    
    Token name = consume(TokenType::IDENTIFIER, "Expected function name");
    
    consume(TokenType::LPAREN, "Expected '(' after function name");
    
    std::vector<std::pair<std::string, TypeNodePtr>> parameters;
    
    // Handle void parameter or empty parameter list
    if (!check(TokenType::RPAREN)) {
        if (check(TokenType::VOID)) {
            advance();
            // void parameter, no actual parameters
        } else {
            // Parse parameters
            do {
                Token paramName = consume(TokenType::IDENTIFIER, "Expected parameter name");
                consume(TokenType::COLON, "Expected ':' after parameter name");
                TypeNodePtr paramType = parseType();
                parameters.push_back({paramName.value, std::move(paramType)});
            } while (match(TokenType::COMMA));
        }
    }
    
    consume(TokenType::RPAREN, "Expected ')' after parameters");
    
    // Parse return type if specified
    TypeNodePtr returnType = nullptr;
    if (match(TokenType::ARROW)) {
        returnType = parseType();
    }
    
    consume(TokenType::ASSIGN, "Expected '=' before function body");
    
    // Parse body
    StmtNodePtr body;
    if (check(TokenType::LBRACE)) {
        body = parseStatement(); // This will parse a block
    } else {
        // Single expression body
        ExprNodePtr expr = parseExpression();
        body = std::make_unique<ExprStmtNode>(std::move(expr));
    }
    
    return std::make_unique<FunctionNode>(name.value, std::move(parameters), 
                                          std::move(returnType), std::move(body));
}

std::unique_ptr<ProgramNode> Parser::parse() {
    auto program = std::make_unique<ProgramNode>();
    
    while (!isAtEnd()) {
        try {
            if (check(TokenType::DEF)) {
                program->functions.push_back(parseFunction());
            } else if (check(TokenType::VAL) || check(TokenType::VAR)) {
                program->globalDeclarations.push_back(parseVarDeclaration());
            } else if (check(TokenType::STRUCT)) {
                program->structs.push_back(parseStructDefinition());
            } else if (check(TokenType::IMPL)) {
                program->implBlocks.push_back(parseImplBlock());
            } else {
                throw std::runtime_error("Expected function, global declaration, struct, or impl block");
            }
        } catch (const std::exception& e) {
            // For now, just rethrow
            throw;
        }
    }
    
    return program;
}

std::unique_ptr<StructDefNode> Parser::parseStructDefinition() {
    consume(TokenType::STRUCT, "Expected 'struct'");
    Token nameToken = consume(TokenType::IDENTIFIER, "Expected struct name");
    consume(TokenType::LBRACE, "Expected '{' after struct name");
    
    std::vector<StructField> fields = parseStructFields();
    
    consume(TokenType::RBRACE, "Expected '}' after struct fields");
    
    return std::make_unique<StructDefNode>(nameToken.value, std::move(fields));
}

std::vector<StructField> Parser::parseStructFields() {
    std::vector<StructField> fields;
    
    while (!check(TokenType::RBRACE) && !isAtEnd()) {
        Token fieldName = consume(TokenType::IDENTIFIER, "Expected field name");
        consume(TokenType::COLON, "Expected ':' after field name");
        TypeNodePtr fieldType = parseType();
        
        fields.emplace_back(fieldName.value, std::move(fieldType));
        
        // Skip optional newlines between fields
        while (match(TokenType::NEWLINE)) {}
    }
    
    return fields;
}

std::unique_ptr<ImplBlockNode> Parser::parseImplBlock() {
    consume(TokenType::IMPL, "Expected 'impl'");
    
    ReceiverType receiverType = ReceiverType::Value;
    std::string structName;
    
    // Check for receiver type prefix
    if (match(TokenType::STAR)) {
        receiverType = ReceiverType::Pointer;
    } else if (match(TokenType::AMPERSAND)) {
        receiverType = ReceiverType::Reference;
    }
    
    Token nameToken = consume(TokenType::IDENTIFIER, "Expected struct name");
    structName = nameToken.value;
    
    consume(TokenType::LBRACE, "Expected '{' after impl declaration");
    
    std::vector<std::unique_ptr<FunctionNode>> methods;
    
    while (!check(TokenType::RBRACE) && !isAtEnd()) {
        if (check(TokenType::DEF)) {
            methods.push_back(parseFunction());
        } else {
            // Skip newlines
            if (match(TokenType::NEWLINE)) {
                continue;
            }
            throw std::runtime_error("Expected method definition in impl block");
        }
    }
    
    consume(TokenType::RBRACE, "Expected '}' after impl block");
    
    return std::make_unique<ImplBlockNode>(receiverType, structName, std::move(methods));
}