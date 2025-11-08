// C code generator

use crate::parser::ast::*;

pub struct CCodeGenerator {
    output: String,
    indent_level: usize,
}

impl CCodeGenerator {
    pub fn new() -> Self {
        CCodeGenerator {
            output: String::new(),
            indent_level: 0,
        }
    }

    pub fn generate(mut self, program: &Program) -> Result<String, String> {
        // Headers
        self.emit_line("#include <stdint.h>");
        self.emit_line("#include <stdbool.h>");
        self.emit_line("");

        // Functions
        for function in &program.functions {
            self.generate_function(function)?;
        }

        Ok(self.output)
    }

    fn generate_function(&mut self, function: &Function) -> Result<(), String> {
        // Function signature
        let return_type = function.return_type.to_c_type();
        let mut signature = format!("{} {}(", return_type, function.name);

        // Parameters
        if function.parameters.is_empty() {
            signature.push_str("void");
        } else {
            let params: Vec<String> = function.parameters.iter()
                .map(|p| format!("{} {}", p.param_type.to_c_type(), p.name))
                .collect();
            signature.push_str(&params.join(", "));
        }

        signature.push(')');
        self.emit_line(&signature);

        // Function body
        self.emit_line("{");
        self.indent();

        for statement in &function.body.statements {
            self.generate_statement(statement)?;
        }

        self.dedent();
        self.emit_line("}");

        Ok(())
    }

    fn generate_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Return(ret) => {
                if let Some(expr) = &ret.value {
                    let value = self.expression_to_c(expr);
                    self.emit_line(&format!("return {};", value));
                } else {
                    self.emit_line("return;");
                }
            }
            Statement::VarDecl(var_decl) => {
                let const_keyword = if var_decl.is_mutable { "" } else { "const " };

                // Type is now guaranteed to be present after semantic analysis
                let c_type = if let Some(ref var_type) = var_decl.var_type {
                    var_type.to_c_type()
                } else {
                    return Err(format!(
                        "Internal error: Variable '{}' has no type after semantic analysis",
                        var_decl.name
                    ));
                };

                // Generate declaration
                let mut line = format!("{}{} {}", const_keyword, c_type, var_decl.name);

                if let Some(ref init) = var_decl.initializer {
                    line.push_str(" = ");
                    line.push_str(&self.expression_to_c(init));
                }

                line.push(';');
                self.emit_line(&line);
            }
            Statement::Assignment(assignment) => {
                let value_code = self.expression_to_c(&assignment.value);
                self.emit_line(&format!("{} = {};", assignment.name, value_code));
            }
            Statement::If(if_stmt) => {
                let condition = self.expression_to_c(&if_stmt.condition);
                self.emit_line(&format!("if ({})", condition));
                self.emit_line("{");
                self.indent();

                for stmt in &if_stmt.then_block.statements {
                    self.generate_statement(stmt)?;
                }

                self.dedent();
                self.emit_line("}");

                if let Some(ref else_block) = if_stmt.else_block {
                    self.emit_line("else");
                    self.emit_line("{");
                    self.indent();

                    for stmt in &else_block.statements {
                        self.generate_statement(stmt)?;
                    }

                    self.dedent();
                    self.emit_line("}");
                }
            }
            Statement::While(while_stmt) => {
                let condition = self.expression_to_c(&while_stmt.condition);
                self.emit_line(&format!("while ({})", condition));
                self.emit_line("{");
                self.indent();

                for stmt in &while_stmt.body.statements {
                    self.generate_statement(stmt)?;
                }

                self.dedent();
                self.emit_line("}");
            }
            Statement::Loop(loop_stmt) => {
                self.emit_line("while (1)");
                self.emit_line("{");
                self.indent();

                for stmt in &loop_stmt.body.statements {
                    self.generate_statement(stmt)?;
                }

                self.dedent();
                self.emit_line("}");
            }
            Statement::For(for_stmt) => {
                match &for_stmt.iterable {
                    ForIterable::Range(start, end) => {
                        let start_expr = self.expression_to_c(start);
                        let end_expr = self.expression_to_c(end);
                        let var = &for_stmt.variable;

                        self.emit_line(&format!(
                            "for (int32_t {} = {}; {} < {}; {}++)",
                            var, start_expr, var, end_expr, var
                        ));
                        self.emit_line("{");
                        self.indent();

                        for stmt in &for_stmt.body.statements {
                            self.generate_statement(stmt)?;
                        }

                        self.dedent();
                        self.emit_line("}");
                    }
                    ForIterable::Array(elements) => {
                        // Generate array literal and iterate
                        let var = &for_stmt.variable;
                        let array_name = format!("__array_{}", var);
                        let len = elements.len();

                        // Declare and initialize array
                        let elem_strings: Vec<String> = elements.iter()
                            .map(|e| self.expression_to_c(e))
                            .collect();

                        self.emit_line(&format!(
                            "int32_t {}[] = {{{}}};",
                            array_name,
                            elem_strings.join(", ")
                        ));

                        // Iterate over array
                        self.emit_line(&format!(
                            "for (size_t __i_{} = 0; __i_{} < {}; __i_{}++)",
                            var, var, len, var
                        ));
                        self.emit_line("{");
                        self.indent();

                        // Declare loop variable
                        self.emit_line(&format!(
                            "int32_t {} = {}[__i_{}];",
                            var, array_name, var
                        ));

                        for stmt in &for_stmt.body.statements {
                            self.generate_statement(stmt)?;
                        }

                        self.dedent();
                        self.emit_line("}");
                    }
                }
            }
            Statement::Break(_) => {
                self.emit_line("break;");
            }
            Statement::Continue(_) => {
                self.emit_line("continue;");
            }
            Statement::Expression(expr) => {
                let expr_code = self.expression_to_c(expr);
                self.emit_line(&format!("{};", expr_code));
            }
        }
        Ok(())
    }

    fn expression_to_c(&self, expr: &Expression) -> String {
        match expr {
            Expression::IntLiteral(value) => value.to_string(),
            Expression::FloatLiteral(value) => value.to_string(),
            Expression::BoolLiteral(value) => if *value { "true" } else { "false" }.to_string(),
            Expression::Variable(name) => name.clone(),
            Expression::Binary(binary) => {
                let left = self.expression_to_c(&binary.left);
                let right = self.expression_to_c(&binary.right);
                let op = match binary.operator {
                    BinaryOperator::Add => "+",
                    BinaryOperator::Subtract => "-",
                    BinaryOperator::Multiply => "*",
                    BinaryOperator::Divide => "/",
                    BinaryOperator::Modulo => "%",
                    BinaryOperator::Equal => "==",
                    BinaryOperator::NotEqual => "!=",
                    BinaryOperator::Less => "<",
                    BinaryOperator::Greater => ">",
                    BinaryOperator::LessEqual => "<=",
                    BinaryOperator::GreaterEqual => ">=",
                    BinaryOperator::And => "&&",
                    BinaryOperator::Or => "||",
                };
                format!("({} {} {})", left, op, right)
            }
            Expression::Unary(unary) => {
                let operand = self.expression_to_c(&unary.operand);
                let op = match unary.operator {
                    UnaryOperator::Not => "!",
                    UnaryOperator::Negate => "-",
                };
                format!("({}{})", op, operand)
            }
            Expression::Call(call) => {
                let args: Vec<String> = call.arguments.iter()
                    .map(|arg| self.expression_to_c(arg))
                    .collect();
                format!("{}({})", call.callee, args.join(", "))
            }
        }
    }

    fn emit_line(&mut self, line: &str) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn indent(&mut self) {
        self.indent_level += 1;
    }

    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }
}