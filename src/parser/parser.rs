// Parser implementation

use crate::lexer::token::{Token, TokenType};
use super::ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(mut self) -> Result<Program, String> {
        let mut functions = Vec::new();

        while !self.is_at_end() {
            if self.check(&TokenType::Eof) {
                break;
            }
            functions.push(self.function()?);
        }

        Ok(Program { functions })
    }

    fn function(&mut self) -> Result<Function, String> {
        // def
        self.consume(&TokenType::Def, "Expected 'def'")?;

        // function name
        let name = self.consume_identifier("Expected function name")?;

        // (
        self.consume(&TokenType::LeftParen, "Expected '('")?;

        // parameters
        let mut parameters = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                let param_name = self.consume_identifier("Expected parameter name")?;
                self.consume(&TokenType::Colon, "Expected ':' after parameter name")?;
                let param_type = self.parse_type()?;

                parameters.push(Parameter {
                    name: param_name,
                    param_type,
                });

                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }

        // )
        self.consume(&TokenType::RightParen, "Expected ')'")?;

        // ->
        self.consume(&TokenType::Arrow, "Expected '->'")?;

        // return type
        let return_type = self.parse_type()?;

        // {
        self.consume(&TokenType::LeftBrace, "Expected '{'")?;

        // body
        let body = self.block()?;

        // }
        self.consume(&TokenType::RightBrace, "Expected '}'")?;

        Ok(Function {
            name,
            parameters,
            return_type,
            body,
        })
    }

    fn block(&mut self) -> Result<Block, String> {
        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.statement()?);
        }

        Ok(Block { statements })
    }

    fn statement(&mut self) -> Result<Statement, String> {
        if self.match_token(&TokenType::Return) {
            return self.return_statement();
        }

        if self.check(&TokenType::Val) || self.check(&TokenType::Var) {
            return self.var_decl_statement();
        }

        if self.match_token(&TokenType::If) {
            return self.if_statement();
        }

        if self.match_token(&TokenType::While) {
            return self.while_statement();
        }

        if self.match_token(&TokenType::Loop) {
            return self.loop_statement();
        }

        if self.match_token(&TokenType::For) {
            return self.for_statement();
        }

        if self.match_token(&TokenType::Break) {
            self.consume(&TokenType::Semicolon, "Expected ';' after break")?;
            return Ok(Statement::Break(BreakStatement {}));
        }

        if self.match_token(&TokenType::Continue) {
            self.consume(&TokenType::Semicolon, "Expected ';' after continue")?;
            return Ok(Statement::Continue(ContinueStatement {}));
        }

        // Check for assignment vs expression statement
        // Look ahead: if we see "identifier = ", it's an assignment
        if let Some(token) = self.peek() {
            if let TokenType::Identifier(name) = &token.token_type {
                let name_clone = name.clone();
                self.advance(); // consume identifier

                if self.match_token(&TokenType::Equal) {
                    // It's an assignment: name = expr;
                    let value = self.expression()?;
                    self.consume(&TokenType::Semicolon, "Expected ';' after assignment")?;
                    return Ok(Statement::Assignment(AssignmentStatement {
                        name: name_clone,
                        value,
                    }));
                } else {
                    // Not an assignment, backtrack
                    self.current -= 1;
                }
            }
        }

        // Expression statement (for function calls)
        let expr = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Statement::Expression(expr))
    }

    fn var_decl_statement(&mut self) -> Result<Statement, String> {
        let is_mutable = if self.match_token(&TokenType::Var) {
            true
        } else if self.match_token(&TokenType::Val) {
            false
        } else {
            return Err("Expected 'val' or 'var'".to_string());
        };

        // variable name
        let name = self.consume_identifier("Expected variable name")?;

        // optional type annotation
        let var_type = if self.match_token(&TokenType::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // optional initializer
        let initializer = if self.match_token(&TokenType::Equal) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(&TokenType::Semicolon, "Expected ';' after variable declaration")?;

        Ok(Statement::VarDecl(VarDeclStatement {
            is_mutable,
            name,
            var_type,
            initializer,
        }))
    }

    fn return_statement(&mut self) -> Result<Statement, String> {
        let value = if !self.check(&TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(&TokenType::Semicolon, "Expected ';' after return")?;

        Ok(Statement::Return(ReturnStatement { value }))
    }

    fn if_statement(&mut self) -> Result<Statement, String> {
        // condition
        let condition = self.expression()?;

        // then block
        self.consume(&TokenType::LeftBrace, "Expected '{' after if condition")?;
        let then_block = self.block()?;
        self.consume(&TokenType::RightBrace, "Expected '}'")?;

        // else block (optional)
        let else_block = if self.match_token(&TokenType::Else) {
            self.consume(&TokenType::LeftBrace, "Expected '{' after else")?;
            let block = self.block()?;
            self.consume(&TokenType::RightBrace, "Expected '}'")?;
            Some(block)
        } else {
            None
        };

        Ok(Statement::If(IfStatement {
            condition,
            then_block,
            else_block,
        }))
    }

    fn while_statement(&mut self) -> Result<Statement, String> {
        // condition
        let condition = self.expression()?;

        // body
        self.consume(&TokenType::LeftBrace, "Expected '{' after while condition")?;
        let body = self.block()?;
        self.consume(&TokenType::RightBrace, "Expected '}'")?;

        Ok(Statement::While(WhileStatement {
            condition,
            body,
        }))
    }

    fn loop_statement(&mut self) -> Result<Statement, String> {
        // body
        self.consume(&TokenType::LeftBrace, "Expected '{' after loop")?;
        let body = self.block()?;
        self.consume(&TokenType::RightBrace, "Expected '}'")?;

        Ok(Statement::Loop(LoopStatement {
            body,
        }))
    }

    fn for_statement(&mut self) -> Result<Statement, String> {
        // variable name
        let variable = self.consume_identifier("Expected variable name in for loop")?;

        // <- or 'in' (we use <- for Scala style)
        if !self.match_token(&TokenType::LeftArrow) && !self.match_token(&TokenType::In) {
            return Err("Expected '<-' or 'in' in for loop".to_string());
        }

        // Parse iterable (range or array)
        let iterable = if self.match_token(&TokenType::LeftBracket) {
            // Array literal: [1, 2, 3, 4]
            let mut elements = Vec::new();

            if !self.check(&TokenType::RightBracket) {
                loop {
                    elements.push(self.expression()?);
                    if !self.match_token(&TokenType::Comma) {
                        break;
                    }
                }
            }

            self.consume(&TokenType::RightBracket, "Expected ']' after array elements")?;
            ForIterable::Array(elements)
        } else {
            // Range: start..end
            let start = self.expression()?;
            self.consume(&TokenType::DotDot, "Expected '..' for range")?;
            let end = self.expression()?;
            ForIterable::Range(Box::new(start), Box::new(end))
        };

        // body
        self.consume(&TokenType::LeftBrace, "Expected '{' after for iterable")?;
        let body = self.block()?;
        self.consume(&TokenType::RightBrace, "Expected '}'")?;

        Ok(Statement::For(ForStatement {
            variable,
            iterable,
            body,
        }))
    }

    fn expression(&mut self) -> Result<Expression, String> {
        self.logical_or()
    }

    fn logical_or(&mut self) -> Result<Expression, String> {
        let mut expr = self.logical_and()?;

        while self.match_token(&TokenType::PipePipe) {
            let right = self.logical_and()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expression, String> {
        let mut expr = self.equality()?;

        while self.match_token(&TokenType::AmpAmp) {
            let right = self.equality()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: BinaryOperator::And,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expression, String> {
        let mut expr = self.comparison()?;

        while self.match_token(&TokenType::EqualEqual) || self.match_token(&TokenType::BangEqual) {
            let operator = if self.previous().unwrap().token_type == TokenType::EqualEqual {
                BinaryOperator::Equal
            } else {
                BinaryOperator::NotEqual
            };

            let right = self.comparison()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression, String> {
        let mut expr = self.additive()?;

        while self.match_token(&TokenType::Less)
            || self.match_token(&TokenType::Greater)
            || self.match_token(&TokenType::LessEqual)
            || self.match_token(&TokenType::GreaterEqual) {
            let operator = match self.previous().unwrap().token_type {
                TokenType::Less => BinaryOperator::Less,
                TokenType::Greater => BinaryOperator::Greater,
                TokenType::LessEqual => BinaryOperator::LessEqual,
                TokenType::GreaterEqual => BinaryOperator::GreaterEqual,
                _ => unreachable!(),
            };

            let right = self.additive()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn additive(&mut self) -> Result<Expression, String> {
        let mut expr = self.multiplicative()?;

        while self.match_token(&TokenType::Plus) || self.match_token(&TokenType::Minus) {
            let operator = if self.previous().unwrap().token_type == TokenType::Plus {
                BinaryOperator::Add
            } else {
                BinaryOperator::Subtract
            };

            let right = self.multiplicative()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn multiplicative(&mut self) -> Result<Expression, String> {
        let mut expr = self.primary()?;

        while self.match_token(&TokenType::Star)
            || self.match_token(&TokenType::Slash)
            || self.match_token(&TokenType::Percent) {
            let operator = match self.previous().unwrap().token_type {
                TokenType::Star => BinaryOperator::Multiply,
                TokenType::Slash => BinaryOperator::Divide,
                TokenType::Percent => BinaryOperator::Modulo,
                _ => unreachable!(),
            };

            let right = self.primary()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expression, String> {
        // Unary operators
        if self.match_token(&TokenType::Bang) {
            let operand = self.primary()?;
            return Ok(Expression::Unary(UnaryExpression {
                operator: UnaryOperator::Not,
                operand: Box::new(operand),
            }));
        }

        if self.match_token(&TokenType::Minus) {
            let operand = self.primary()?;
            return Ok(Expression::Unary(UnaryExpression {
                operator: UnaryOperator::Negate,
                operand: Box::new(operand),
            }));
        }

        // Boolean literals
        if self.match_token(&TokenType::True) {
            return Ok(Expression::BoolLiteral(true));
        }
        if self.match_token(&TokenType::False) {
            return Ok(Expression::BoolLiteral(false));
        }

        // Number literals
        if let Some(token) = self.peek() {
            match &token.token_type {
                TokenType::IntLiteral(value) => {
                    let int_value = value.parse::<i64>()
                        .map_err(|_| format!("Invalid integer literal: {}", value))?;
                    self.advance();
                    return Ok(Expression::IntLiteral(int_value));
                }
                TokenType::FloatLiteral(value) => {
                    let float_value = value.parse::<f64>()
                        .map_err(|_| format!("Invalid float literal: {}", value))?;
                    self.advance();
                    return Ok(Expression::FloatLiteral(float_value));
                }
                TokenType::Identifier(name) => {
                    let var_name = name.clone();
                    self.advance();

                    // Check for function call
                    if self.match_token(&TokenType::LeftParen) {
                        let mut arguments = Vec::new();

                        if !self.check(&TokenType::RightParen) {
                            loop {
                                arguments.push(self.expression()?);
                                if !self.match_token(&TokenType::Comma) {
                                    break;
                                }
                            }
                        }

                        self.consume(&TokenType::RightParen, "Expected ')' after arguments")?;

                        return Ok(Expression::Call(CallExpression {
                            callee: var_name,
                            arguments,
                        }));
                    }

                    // Just a variable reference
                    return Ok(Expression::Variable(var_name));
                }
                _ => {}
            }
        }

        // Parenthesized expression
        if self.match_token(&TokenType::LeftParen) {
            let expr = self.expression()?;
            self.consume(&TokenType::RightParen, "Expected ')' after expression")?;
            return Ok(expr);
        }

        Err("Expected expression".to_string())
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        if let Some(token) = self.peek() {
            let ty = match &token.token_type {
                TokenType::I8 => Type::I8,
                TokenType::I16 => Type::I16,
                TokenType::I32 => Type::I32,
                TokenType::I64 => Type::I64,
                TokenType::U8 => Type::U8,
                TokenType::U16 => Type::U16,
                TokenType::U32 => Type::U32,
                TokenType::U64 => Type::U64,
                TokenType::F32 => Type::F32,
                TokenType::F64 => Type::F64,
                TokenType::Bool => Type::Bool,
                TokenType::Void => Type::Void,
                _ => return Err("Expected type".to_string()),
            };
            self.advance();
            return Ok(ty);
        }

        Err("Expected type".to_string())
    }

    // Helper methods

    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if let Some(token) = self.peek() {
            std::mem::discriminant(&token.token_type) == std::mem::discriminant(token_type)
        } else {
            false
        }
    }

    fn advance(&mut self) -> Option<Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn previous(&self) -> Option<Token> {
        if self.current > 0 {
            self.tokens.get(self.current - 1).cloned()
        } else {
            None
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().map_or(true, |t| matches!(t.token_type, TokenType::Eof))
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<Token, String> {
        if self.check(token_type) {
            Ok(self.advance().unwrap())
        } else {
            Err(format!("{} at line {}", message, self.peek().map_or(0, |t| t.line)))
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, String> {
        if let Some(token) = self.peek() {
            if let TokenType::Identifier(name) = &token.token_type {
                let name = name.clone();
                self.advance();
                return Ok(name);
            }
        }
        Err(message.to_string())
    }
}