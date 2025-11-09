// Parser implementation

use crate::lexer::token::{Token, TokenType};
use crate::helper::types::CompileError;
use super::ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    source: String,
    filename: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, source: String, filename: String) -> Self {
        Parser {
            tokens,
            current: 0,
            source,
            filename,
        }
    }

    pub fn parse(mut self) -> Result<Program, String> {
        let mut structs = Vec::new();
        let mut impls = Vec::new();
        let mut functions = Vec::new();

        while !self.is_at_end() {
            if self.check(&TokenType::Eof) {
                break;
            }

            // Check what kind of top-level item this is
            if self.check(&TokenType::Struct) {
                structs.push(self.struct_def()?);
            } else if self.check(&TokenType::Impl) {
                impls.push(self.impl_block()?);
            } else if self.check(&TokenType::Def) {
                functions.push(self.function()?);
            } else {
                return Err(format!("Expected struct, impl, or function definition"));
            }
        }

        Ok(Program { structs, impls, functions })
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

        // Parse what looks like an expression, then check if it's an assignment
        let expr = self.postfix()?;

        // Check if this is an assignment
        if self.match_token(&TokenType::Equal) {
            // Verify that expr is a valid assignment target
            let target = match expr {
                Expression::Variable(name) => AssignmentTarget::Variable(name),
                Expression::FieldAccess(field_access) => AssignmentTarget::FieldAccess(field_access),
                Expression::Index(index_expr) => AssignmentTarget::Index(index_expr),
                _ => {
                    return Err("Invalid assignment target".to_string());
                }
            };

            let value = self.expression()?;
            self.consume(&TokenType::Semicolon, "Expected ';' after assignment")?;
            return Ok(Statement::Assignment(AssignmentStatement {
                target,
                value,
                span: self.current_span(),
            }));
        }

        // Not an assignment, continue parsing as expression statement
        // We already parsed postfix, now parse the rest of the expression
        let full_expr = self.finish_expression(expr)?;
        self.consume(&TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Statement::Expression(full_expr))
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

        // Parse iterable (range, array literal, or array variable)
        // Special parsing to avoid conflict with { } block syntax
        let iterable = if self.match_token(&TokenType::LeftBracket) {
            // Array literal: [1, 2, 3]
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
            ForIterable::Expression(Box::new(Expression::ArrayLiteral(ArrayLiteralExpression {
                elements,
                span: self.current_span(),
            })))
        } else {
            // Parse iterable (variable or range)
            // We need to avoid struct literal parsing, so we handle this carefully
            // Check if it's a number (for range) or identifier (for variable/range)
            if self.check_number() {
                // Range starting with number: 0..n
                let start = self.comparison()?;
                self.consume(&TokenType::DotDot, "Expected '..' for range")?;
                let end = self.comparison()?;
                ForIterable::Range(Box::new(start), Box::new(end))
            } else {
                // Identifier - could be variable or range start
                let ident = self.consume_identifier("Expected array variable or range start")?;

                // Check for .length or other field access
                let expr = if self.match_token(&TokenType::Dot) {
                    let field = self.consume_identifier("Expected field name")?;
                    Expression::FieldAccess(FieldAccessExpression {
                        object: Box::new(Expression::Variable(ident)),
                        field,
                        span: self.current_span(),
                    })
                } else {
                    Expression::Variable(ident)
                };

                if self.match_token(&TokenType::DotDot) {
                    // Range: identifier..end or identifier.field..end
                    let end = self.comparison()?;
                    ForIterable::Range(Box::new(expr), Box::new(end))
                } else {
                    // Just an array variable reference
                    ForIterable::Expression(Box::new(expr))
                }
            }
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

    fn struct_def(&mut self) -> Result<Struct, String> {
        // struct keyword
        self.consume(&TokenType::Struct, "Expected 'struct'")?;

        // struct name
        let name = self.consume_identifier("Expected struct name")?;

        // {
        self.consume(&TokenType::LeftBrace, "Expected '{' after struct name")?;

        // Parse fields
        let mut fields = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let field_name = self.consume_identifier("Expected field name")?;
            self.consume(&TokenType::Colon, "Expected ':' after field name")?;
            let field_type = self.parse_type()?;

            fields.push(StructField {
                name: field_name,
                field_type,
            });

            // Optional comma
            if !self.match_token(&TokenType::Comma) {
                break;
            }
        }

        // }
        self.consume(&TokenType::RightBrace, "Expected '}' after struct fields")?;

        Ok(Struct { name, fields })
    }

    fn impl_block(&mut self) -> Result<Impl, String> {
        // impl keyword
        self.consume(&TokenType::Impl, "Expected 'impl'")?;

        // struct name
        let struct_name = self.consume_identifier("Expected struct name after 'impl'")?;

        // {
        self.consume(&TokenType::LeftBrace, "Expected '{' after impl struct name")?;

        // Parse methods
        let mut methods = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.method()?);
        }

        // }
        self.consume(&TokenType::RightBrace, "Expected '}' after impl methods")?;

        Ok(Impl { struct_name, methods })
    }

    fn method(&mut self) -> Result<Method, String> {
        // def keyword
        self.consume(&TokenType::Def, "Expected 'def' for method")?;

        // method name
        let name = self.consume_identifier("Expected method name")?;

        // (
        self.consume(&TokenType::LeftParen, "Expected '(' after method name")?;

        // Check for self parameter (&self, &mut self, or none)
        let mut self_param = None;
        let mut parameters = Vec::new();

        // Check if first parameter is a self parameter
        if self.match_token(&TokenType::Amp) {
            // & found - check for 'mut self' or just 'self'
            if self.match_token(&TokenType::Mut) {
                // &mut self
                self.consume(&TokenType::SelfKeyword, "Expected 'self' after '&mut'")?;
                self_param = Some(SelfParam::MutRef);
            } else if self.match_token(&TokenType::SelfKeyword) {
                // &self
                self_param = Some(SelfParam::Ref);
            } else {
                return Err("Expected 'self' or 'mut self' after '&'".to_string());
            }

            // If there are more parameters after self, expect a comma
            if !self.check(&TokenType::RightParen) {
                self.consume(&TokenType::Comma, "Expected ',' after self parameter")?;
            }
        } else if self.match_token(&TokenType::SelfKeyword) {
            // Owned self (not yet implemented)
            return Err("Owned 'self' parameter is not yet implemented. Use '&self' or '&mut self'.".to_string());
        }

        // Parse remaining parameters (or all parameters if no self)
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
        self.consume(&TokenType::RightParen, "Expected ')' after parameters")?;

        // -> return_type
        self.consume(&TokenType::Arrow, "Expected '->' after method parameters")?;
        let return_type = self.parse_type()?;

        // {
        self.consume(&TokenType::LeftBrace, "Expected '{' before method body")?;

        // body
        let body = self.block()?;

        // }
        self.consume(&TokenType::RightBrace, "Expected '}' after method body")?;

        Ok(Method {
            name,
            self_param,
            parameters,
            return_type,
            body,
        })
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

    /// Continue parsing an expression from a postfix expression
    fn finish_expression(&mut self, expr: Expression) -> Result<Expression, String> {
        // We have a postfix expression, now continue with binary operators
        // unary() -> multiplicative() -> additive() -> comparison() -> equality() -> logical_and() -> logical_or()
        let expr = self.finish_multiplicative(expr)?;
        let expr = self.finish_additive(expr)?;
        let expr = self.finish_comparison(expr)?;
        let expr = self.finish_equality(expr)?;
        let expr = self.finish_logical_and(expr)?;
        let expr = self.finish_logical_or(expr)?;
        Ok(expr)
    }

    fn finish_multiplicative(&mut self, mut expr: Expression) -> Result<Expression, String> {
        while self.match_token(&TokenType::Star)
            || self.match_token(&TokenType::Slash)
            || self.match_token(&TokenType::Percent) {
            let operator = match self.previous().unwrap().token_type {
                TokenType::Star => BinaryOperator::Multiply,
                TokenType::Slash => BinaryOperator::Divide,
                TokenType::Percent => BinaryOperator::Modulo,
                _ => unreachable!(),
            };

            let right = self.postfix()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn finish_additive(&mut self, mut expr: Expression) -> Result<Expression, String> {
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

    fn finish_comparison(&mut self, mut expr: Expression) -> Result<Expression, String> {
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

    fn finish_equality(&mut self, mut expr: Expression) -> Result<Expression, String> {
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

    fn finish_logical_and(&mut self, mut expr: Expression) -> Result<Expression, String> {
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

    fn finish_logical_or(&mut self, mut expr: Expression) -> Result<Expression, String> {
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

    fn multiplicative(&mut self) -> Result<Expression, String> {
        let mut expr = self.postfix()?;

        while self.match_token(&TokenType::Star)
            || self.match_token(&TokenType::Slash)
            || self.match_token(&TokenType::Percent) {
            let operator = match self.previous().unwrap().token_type {
                TokenType::Star => BinaryOperator::Multiply,
                TokenType::Slash => BinaryOperator::Divide,
                TokenType::Percent => BinaryOperator::Modulo,
                _ => unreachable!(),
            };

            let right = self.postfix()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn postfix(&mut self) -> Result<Expression, String> {
        let mut expr = self.primary()?;

        // Handle postfix operations: field access, method calls, array indexing
        loop {
            if self.match_token(&TokenType::Dot) {
                // Field access or method call
                let field_or_method = self.consume_identifier("Expected field or method name after '.'")?;

                // Check if it's a method call (followed by '(')
                if self.match_token(&TokenType::LeftParen) {
                    // Method call
                    let mut arguments = Vec::new();
                    if !self.check(&TokenType::RightParen) {
                        loop {
                            arguments.push(self.expression()?);
                            if !self.match_token(&TokenType::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume(&TokenType::RightParen, "Expected ')' after method arguments")?;

                    expr = Expression::MethodCall(MethodCallExpression {
                        object: Box::new(expr),
                        method: field_or_method,
                        arguments,
                        span: self.current_span(),
                    });
                } else {
                    // Field access
                    expr = Expression::FieldAccess(FieldAccessExpression {
                        object: Box::new(expr),
                        field: field_or_method,
                        span: self.current_span(),
                    });
                }
            } else if self.match_token(&TokenType::LeftBracket) {
                // Array indexing: arr[index]
                let index = self.expression()?;
                self.consume(&TokenType::RightBracket, "Expected ']' after array index")?;

                expr = Expression::Index(IndexExpression {
                    array: Box::new(expr),
                    index: Box::new(index),
                    span: self.current_span(),
                });
            } else {
                break;
            }
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

        // Reference operator (&) for creating slices
        if self.match_token(&TokenType::Amp) {
            let inner = self.primary()?;
            return Ok(Expression::Reference(ReferenceExpression {
                inner: Box::new(inner),
                span: self.current_span(),
            }));
        }

        // Boolean literals
        if self.match_token(&TokenType::True) {
            return Ok(Expression::BoolLiteral(true));
        }
        if self.match_token(&TokenType::False) {
            return Ok(Expression::BoolLiteral(false));
        }

        // self keyword (for method bodies)
        if self.match_token(&TokenType::SelfKeyword) {
            return Ok(Expression::Variable("self".to_string()));
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
                TokenType::StringLiteral(value) => {
                    let str_value = value.clone();
                    self.advance();
                    return Ok(Expression::StringLiteral(str_value));
                }
                TokenType::Identifier(name) => {
                    let var_name = name.clone();
                    self.advance();

                    // Check for macro call (identifier!(...))
                    if self.match_token(&TokenType::Bang) {
                        self.consume(&TokenType::LeftParen, "Expected '(' after '!' in macro call")?;

                        let mut arguments = Vec::new();
                        if !self.check(&TokenType::RightParen) {
                            loop {
                                arguments.push(self.expression()?);
                                if !self.match_token(&TokenType::Comma) {
                                    break;
                                }
                            }
                        }

                        self.consume(&TokenType::RightParen, "Expected ')' after macro arguments")?;

                        return Ok(Expression::MacroCall(MacroCallExpression {
                            macro_name: var_name,
                            arguments,
                            span: self.current_span(),
                        }));
                    }

                    // Check for associated function call (Type::function())
                    if self.match_token(&TokenType::ColonColon) {
                        let function_name = self.consume_identifier("Expected function name after '::'")?;
                        self.consume(&TokenType::LeftParen, "Expected '(' after associated function name")?;

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

                        return Ok(Expression::AssociatedCall(AssociatedCallExpression {
                            type_name: var_name,
                            function: function_name,
                            arguments,
                            span: self.current_span(),
                        }));
                    }

                    // Check for struct literal (Type { field: value, ... })
                    if self.match_token(&TokenType::LeftBrace) {
                        let mut fields = Vec::new();

                        if !self.check(&TokenType::RightBrace) {
                            loop {
                                let field_name = self.consume_identifier("Expected field name in struct literal")?;
                                self.consume(&TokenType::Colon, "Expected ':' after field name")?;
                                let field_value = self.expression()?;

                                fields.push(StructLiteralField {
                                    name: field_name,
                                    value: field_value,
                                });

                                if !self.match_token(&TokenType::Comma) {
                                    break;
                                }
                            }
                        }

                        self.consume(&TokenType::RightBrace, "Expected '}' after struct literal fields")?;

                        return Ok(Expression::StructLiteral(StructLiteralExpression {
                            struct_name: var_name,
                            fields,
                            span: self.current_span(),
                        }));
                    }

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

        // Array literals: [1, 2, 3] or {1, 2, 3}
        if self.match_token(&TokenType::LeftBracket) {
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

            return Ok(Expression::ArrayLiteral(ArrayLiteralExpression {
                elements,
                span: self.current_span(),
            }));
        }

        // Note: {} syntax for array literals is removed due to conflict with blocks
        // Use [] syntax for array literals instead

        // Parenthesized expression
        if self.match_token(&TokenType::LeftParen) {
            let expr = self.expression()?;
            self.consume(&TokenType::RightParen, "Expected ')' after expression")?;
            return Ok(expr);
        }

        // Error: unexpected token
        if let Some(token) = self.peek() {
            Err(self.create_parse_error("Expected expression", token.line, token.column))
        } else {
            Err("Expected expression".to_string())
        }
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        // Check for array type: [N]T or []T
        if self.match_token(&TokenType::LeftBracket) {
            // Parse optional size
            let size = if self.check(&TokenType::RightBracket) {
                None  // []T - size will be inferred
            } else {
                // [N]T - parse the size as an integer
                if let Some(token) = self.peek() {
                    if let TokenType::IntLiteral(value) = &token.token_type {
                        let size_val = value.parse::<usize>()
                            .map_err(|_| format!("Invalid array size: {}", value))?;
                        self.advance();
                        Some(size_val)
                    } else {
                        return Err("Expected integer size in array type".to_string());
                    }
                } else {
                    return Err("Expected integer size or ] in array type".to_string());
                }
            };

            self.consume(&TokenType::RightBracket, "Expected ']' in array type")?;

            // Parse element type
            let element_type = Box::new(self.parse_type()?);

            return Ok(Type::Array { element_type, size });
        }

        // Parse regular types
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
                TokenType::String => Type::String,
                TokenType::Identifier(name) => Type::Struct(name.clone()),
                _ => return Err("Expected type".to_string()),
            };
            self.advance();
            return Ok(ty);
        }

        Err("Expected type".to_string())
    }

    // Helper methods

    fn current_span(&self) -> Option<Span> {
        if let Some(token) = self.peek() {
            Some(Span::new(token.line, token.column))
        } else if self.current > 0 {
            let token = &self.tokens[self.current - 1];
            Some(Span::new(token.line, token.column))
        } else {
            None
        }
    }

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

    fn check_number(&self) -> bool {
        if let Some(token) = self.peek() {
            matches!(token.token_type, TokenType::IntLiteral(_) | TokenType::FloatLiteral(_))
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
            let error = if let Some(token) = self.peek() {
                self.create_parse_error(message, token.line, token.column)
            } else {
                format!("{}", message)
            };
            Err(error)
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, String> {
        if let Some(token) = self.peek() {
            if let TokenType::Identifier(name) = &token.token_type {
                let name = name.clone();
                self.advance();
                return Ok(name);
            }
            Err(self.create_parse_error(message, token.line, token.column))
        } else {
            Err(message.to_string())
        }
    }

    fn create_parse_error(&self, message: &str, line: usize, column: usize) -> String {
        use crate::helper::msg::MessageFormatter;
        let error = CompileError::parse_error(message.to_string(), line, column);
        let formatter = MessageFormatter::new(self.source.clone(), self.filename.clone());
        formatter.report(&error);
        message.to_string()
    }
}