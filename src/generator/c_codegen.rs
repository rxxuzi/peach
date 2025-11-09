// C code generator

use crate::parser::ast::*;
use std::collections::HashMap;

pub struct CCodeGenerator {
    output: String,
    indent_level: usize,
    // Type tracking for method calls
    variable_types: HashMap<String, Type>,
    struct_table: HashMap<String, Vec<StructField>>,
    impl_table: HashMap<String, Vec<Method>>,
}

impl CCodeGenerator {
    pub fn new() -> Self {
        CCodeGenerator {
            output: String::new(),
            indent_level: 0,
            variable_types: HashMap::new(),
            struct_table: HashMap::new(),
            impl_table: HashMap::new(),
        }
    }

    pub fn generate(mut self, program: &Program) -> Result<String, String> {
        // Build struct and impl tables for type tracking
        for struct_def in &program.structs {
            self.struct_table.insert(struct_def.name.clone(), struct_def.fields.clone());
        }
        for impl_block in &program.impls {
            self.impl_table.insert(impl_block.struct_name.clone(), impl_block.methods.clone());
        }

        // Headers
        self.emit_line("#include <stdio.h>");
        self.emit_line("#include <stdlib.h>");
        self.emit_line("#include <stdint.h>");
        self.emit_line("#include <stdbool.h>");
        self.emit_line("");

        // Struct definitions (typedef struct)
        for struct_def in &program.structs {
            self.generate_struct(struct_def)?;
        }

        // Method declarations (forward declarations)
        for impl_block in &program.impls {
            self.generate_method_declarations(&impl_block.struct_name, &impl_block.methods)?;
        }

        // Functions
        for function in &program.functions {
            self.generate_function(function)?;
        }

        // Method implementations
        for impl_block in &program.impls {
            self.generate_methods(&impl_block.struct_name, &impl_block.methods)?;
        }

        Ok(self.output)
    }

    fn generate_struct(&mut self, struct_def: &Struct) -> Result<(), String> {
        self.emit_line(&format!("typedef struct {{"));
        self.indent();

        for field in &struct_def.fields {
            self.emit_line(&format!("{} {};", field.field_type.to_c_type(), field.name));
        }

        self.dedent();
        self.emit_line(&format!("}} {};", struct_def.name));
        self.emit_line("");

        Ok(())
    }

    fn generate_method_declarations(&mut self, struct_name: &str, methods: &[Method]) -> Result<(), String> {
        for method in methods {
            let return_type = method.return_type.to_c_type();
            let mut signature = format!("{} {}_{}(", return_type, struct_name, method.name);

            let mut params = Vec::new();

            // Add self parameter if present
            if let Some(ref self_param) = method.self_param {
                match self_param {
                    SelfParam::Ref => params.push(format!("const {}* self", struct_name)),
                    SelfParam::MutRef => params.push(format!("{}* self", struct_name)),
                    SelfParam::Owned => return Err("Owned self parameter not yet implemented".to_string()),
                }
            }

            // Add regular parameters
            for param in &method.parameters {
                params.push(format!("{} {}", param.param_type.to_c_type(), param.name));
            }

            if params.is_empty() {
                signature.push_str("void");
            } else {
                signature.push_str(&params.join(", "));
            }

            signature.push_str(");");
            self.emit_line(&signature);
        }

        self.emit_line("");
        Ok(())
    }

    fn generate_methods(&mut self, struct_name: &str, methods: &[Method]) -> Result<(), String> {
        for method in methods {
            self.generate_method(struct_name, method)?;
        }
        Ok(())
    }

    fn generate_method(&mut self, struct_name: &str, method: &Method) -> Result<(), String> {
        let return_type = method.return_type.to_c_type();
        let mut signature = format!("{} {}_{}(", return_type, struct_name, method.name);

        let mut params = Vec::new();

        // Add self parameter if present
        if let Some(ref self_param) = method.self_param {
            match self_param {
                SelfParam::Ref => params.push(format!("const {}* self", struct_name)),
                SelfParam::MutRef => params.push(format!("{}* self", struct_name)),
                SelfParam::Owned => return Err("Owned self parameter not supported in v0.2.0".to_string()),
            }
        }

        // Add regular parameters
        for param in &method.parameters {
            params.push(format!("{} {}", param.param_type.to_c_type(), param.name));
        }

        if params.is_empty() {
            signature.push_str("void");
        } else {
            signature.push_str(&params.join(", "));
        }

        signature.push(')');
        self.emit_line(&signature);

        // Method body
        self.emit_line("{");
        self.indent();

        for statement in &method.body.statements {
            self.generate_statement(statement)?;
        }

        self.dedent();
        self.emit_line("}");
        self.emit_line("");

        Ok(())
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
                let var_type = if let Some(ref var_type) = var_decl.var_type {
                    var_type.clone()
                } else {
                    return Err(format!(
                        "Internal error: Variable '{}' has no type after semantic analysis",
                        var_decl.name
                    ));
                };

                // Track variable type for method calls
                self.variable_types.insert(var_decl.name.clone(), var_type.clone());

                // Special handling for array types
                match &var_type {
                    Type::Array { element_type, size } => {
                        // Generate C array declaration
                        let elem_c_type = element_type.to_c_type();
                        let size_str = if let Some(n) = size {
                            format!("[{}]", n)
                        } else {
                            // Size should have been inferred by semantic analysis
                            return Err(format!(
                                "Internal error: Array '{}' has no size after semantic analysis",
                                var_decl.name
                            ));
                        };

                        let mut line = format!("{}{} {}{}", const_keyword, elem_c_type, var_decl.name, size_str);

                        if let Some(ref init) = var_decl.initializer {
                            line.push_str(" = ");
                            line.push_str(&self.expression_to_c(init));
                        }

                        line.push(';');
                        self.emit_line(&line);
                    }
                    _ => {
                        // Regular variable declaration
                        let c_type = var_type.to_c_type();

                        let mut line = format!("{}{} {}", const_keyword, c_type, var_decl.name);

                        if let Some(ref init) = var_decl.initializer {
                            line.push_str(" = ");
                            line.push_str(&self.expression_to_c(init));
                        }

                        line.push(';');
                        self.emit_line(&line);
                    }
                }
            }
            Statement::Assignment(assignment) => {
                let value_code = self.expression_to_c(&assignment.value);
                let target_code = match &assignment.target {
                    AssignmentTarget::Variable(name) => name.clone(),
                    AssignmentTarget::FieldAccess(field_access) => {
                        let object = self.expression_to_c(&field_access.object);
                        // Use -> for self (pointer), . for regular variables
                        let accessor = if matches!(&*field_access.object, Expression::Variable(name) if name == "self") {
                            "->"
                        } else {
                            "."
                        };
                        format!("{}{}{}", object, accessor, field_access.field)
                    }
                    AssignmentTarget::Index(index_expr) => {
                        let array_code = self.expression_to_c(&index_expr.array);
                        let index_code = self.expression_to_c(&index_expr.index);
                        // For now, generate simple array indexing: array[index]
                        // Later when we implement slice structs, this will be: array.data[index]
                        format!("{}[{}]", array_code, index_code)
                    }
                };
                self.emit_line(&format!("{} = {};", target_code, value_code));
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
            Expression::StringLiteral(value) => {
                // Escape special characters for C string literal
                let escaped = value
                    .replace("\\", "\\\\")
                    .replace("\"", "\\\"")
                    .replace("\n", "\\n")
                    .replace("\t", "\\t")
                    .replace("\r", "\\r");
                format!("\"{}\"", escaped)
            }
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

            Expression::FieldAccess(field_access) => {
                let object = self.expression_to_c(&field_access.object);
                // Determine if we need . or ->
                // If the object is 'self' (in a method), it's always a pointer, so use ->
                let accessor = if matches!(&*field_access.object, Expression::Variable(name) if name == "self") {
                    "->"
                } else {
                    "."
                };
                format!("{}{}{}", object, accessor, field_access.field)
            }

            Expression::MethodCall(method_call) => {
                // Get the object expression
                let object_expr = self.expression_to_c(&method_call.object);

                // Infer the type of the object
                let struct_name = self.infer_expression_type_name(&method_call.object);

                // Generate method arguments
                let mut args = vec![format!("&{}", object_expr)];  // Pass object as pointer
                for arg in &method_call.arguments {
                    args.push(self.expression_to_c(arg));
                }

                // Generate: StructName_methodName(&object, args...)
                format!("{}_{}({})",
                    struct_name.unwrap_or_else(|| "Unknown".to_string()),
                    method_call.method,
                    args.join(", ")
                )
            }

            Expression::AssociatedCall(assoc_call) => {
                let args: Vec<String> = assoc_call.arguments.iter()
                    .map(|arg| self.expression_to_c(arg))
                    .collect();
                format!("{}_{}({})", assoc_call.type_name, assoc_call.function, args.join(", "))
            }

            Expression::StructLiteral(struct_literal) => {
                // Generate C struct initializer
                let fields: Vec<String> = struct_literal.fields.iter()
                    .map(|f| format!(".{} = {}", f.name, self.expression_to_c(&f.value)))
                    .collect();
                format!("({}){{{}}}", struct_literal.struct_name, fields.join(", "))
            }

            Expression::MacroCall(macro_call) => {
                use super::builtin::BuiltinMacroGenerator;
                BuiltinMacroGenerator::generate_macro_call(macro_call, &|expr| self.expression_to_c(expr))
            }

            Expression::ArrayLiteral(array_lit) => {
                if array_lit.elements.is_empty() {
                    // Empty array - this should have been caught by type checker
                    return "/* empty array */".to_string();
                }

                // Generate elements
                let elements: Vec<String> = array_lit.elements
                    .iter()
                    .map(|e| self.expression_to_c(e))
                    .collect();

                // For now, just generate the C array literal
                // Later we'll wrap this in a slice struct
                format!("{{{}}}", elements.join(", "))
            }

            Expression::Index(index_expr) => {
                let array_code = self.expression_to_c(&index_expr.array);
                let index_code = self.expression_to_c(&index_expr.index);

                // For now, generate simple array indexing: array[index]
                // Later when we implement slice structs, this will be: array.data[index]
                format!("{}[{}]", array_code, index_code)
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

    /// Infer the struct type name from an expression
    fn infer_expression_type_name(&self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::Variable(name) => {
                // Look up the variable's type
                if let Some(var_type) = self.variable_types.get(name) {
                    match var_type {
                        Type::Struct(struct_name) => Some(struct_name.clone()),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            Expression::StructLiteral(struct_lit) => {
                Some(struct_lit.struct_name.clone())
            }
            Expression::FieldAccess(field_access) => {
                // Get the type of the object, then look up the field type
                if let Some(struct_name) = self.infer_expression_type_name(&field_access.object) {
                    if let Some(fields) = self.struct_table.get(&struct_name) {
                        if let Some(field) = fields.iter().find(|f| f.name == field_access.field) {
                            match &field.field_type {
                                Type::Struct(name) => Some(name.clone()),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Expression::MethodCall(_method_call) => {
                // Method calls return values - would need full type inference
                // For now, not supported in method chains
                None
            }
            Expression::Call(_) | Expression::AssociatedCall(_) => {
                // Would need to look up function return types
                None
            }
            _ => None,
        }
    }
}