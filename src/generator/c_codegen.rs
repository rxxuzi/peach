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
    function_table: HashMap<String, Vec<Parameter>>,
    temp_var_counter: usize,  // Counter for temporary variables in method chains
}

impl CCodeGenerator {
    pub fn new() -> Self {
        CCodeGenerator {
            output: String::new(),
            indent_level: 0,
            variable_types: HashMap::new(),
            struct_table: HashMap::new(),
            impl_table: HashMap::new(),
            function_table: HashMap::new(),
            temp_var_counter: 0,
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
        // Build function table for call-site array-to-slice conversion
        for function in &program.functions {
            self.function_table.insert(function.name.clone(), function.parameters.clone());
        }

        // Headers
        self.emit_line("#include <stdio.h>");
        self.emit_line("#include <stdlib.h>");
        self.emit_line("#include <stdint.h>");
        self.emit_line("#include <stdbool.h>");
        self.emit_line("#include <string.h>");  // For memcpy (copy! macro)
        self.emit_line("");

        // Generate slice struct definitions for array parameters
        self.generate_slice_structs(program)?;

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

    /// Collect all slice types and generate slice structs
    fn generate_slice_structs(&mut self, program: &Program) -> Result<(), String> {
        use std::collections::HashSet;
        let mut slice_types = HashSet::new();

        // Helper function to extract slice element type
        let extract_slice_type = |ty: &Type| -> Option<String> {
            match ty {
                Type::Reference { inner, .. } => {
                    match &**inner {
                        Type::Array { size: None, element_type } => {
                            Some(element_type.to_c_type())
                        }
                        _ => None,
                    }
                }
                _ => None,
            }
        };

        // Collect slice types from function parameters
        for function in &program.functions {
            for param in &function.parameters {
                if let Some(elem_type) = extract_slice_type(&param.param_type) {
                    slice_types.insert(elem_type);
                }
            }
        }

        // Collect slice types from method parameters
        for impl_block in &program.impls {
            for method in &impl_block.methods {
                for param in &method.parameters {
                    if let Some(elem_type) = extract_slice_type(&param.param_type) {
                        slice_types.insert(elem_type);
                    }
                }
                // Also check variable declarations in method bodies
                self.collect_slice_types_from_block(&method.body, &extract_slice_type, &mut slice_types);
            }
        }

        // Collect slice types from function bodies
        for function in &program.functions {
            self.collect_slice_types_from_block(&function.body, &extract_slice_type, &mut slice_types);
        }

        // Generate slice struct for each type
        for elem_type in slice_types {
            // Generate: typedef struct { T* data; size_t length; } Slice_T;
            let struct_name = format!("Slice_{}", elem_type.replace("*", "ptr").replace(" ", "_"));
            self.emit_line(&format!("typedef struct {{"));
            self.indent();
            self.emit_line(&format!("{}* data;", elem_type));
            self.emit_line("size_t length;");
            self.dedent();
            self.emit_line(&format!("}} {};", struct_name));
            self.emit_line("");
        }

        Ok(())
    }

    /// Helper function to collect slice types from a block's variable declarations
    fn collect_slice_types_from_block<F>(&self, block: &Block, extract_slice_type: &F, slice_types: &mut std::collections::HashSet<String>)
    where
        F: Fn(&Type) -> Option<String>,
    {
        for stmt in &block.statements {
            match stmt {
                Statement::VarDecl(var_decl) => {
                    if let Some(ref var_type) = var_decl.var_type {
                        if let Some(elem_type) = extract_slice_type(var_type) {
                            slice_types.insert(elem_type);
                        }
                    }
                }
                Statement::If(if_stmt) => {
                    self.collect_slice_types_from_block(&if_stmt.then_block, extract_slice_type, slice_types);
                    if let Some(ref else_block) = if_stmt.else_block {
                        self.collect_slice_types_from_block(else_block, extract_slice_type, slice_types);
                    }
                }
                Statement::While(while_stmt) => {
                    self.collect_slice_types_from_block(&while_stmt.body, extract_slice_type, slice_types);
                }
                Statement::Loop(loop_stmt) => {
                    self.collect_slice_types_from_block(&loop_stmt.body, extract_slice_type, slice_types);
                }
                Statement::For(for_stmt) => {
                    self.collect_slice_types_from_block(&for_stmt.body, extract_slice_type, slice_types);
                }
                _ => {}
            }
        }
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
        // Clear variable types for new function scope
        self.variable_types.clear();

        // Add function parameters to variable types for slice handling
        for param in &function.parameters {
            self.variable_types.insert(param.name.clone(), param.param_type.clone());
        }

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
                    Type::Array { element_type: _, size } => {
                        if let Some(_) = size {
                            // Fixed-size array (possibly multi-dimensional): int32_t arr[2][3]
                            let (dimensions, base_type) = self.collect_array_dimensions(&var_type);
                            let base_c_type = base_type.to_c_type();

                            // Build dimension string: [2][3][4]
                            let dimensions_str: String = dimensions.iter()
                                .map(|d| format!("[{}]", d))
                                .collect::<Vec<_>>()
                                .join("");

                            let mut line = format!("{}{} {}{}", const_keyword, base_c_type, var_decl.name, dimensions_str);

                            if let Some(ref init) = var_decl.initializer {
                                line.push_str(" = ");
                                line.push_str(&self.generate_method_chain_temps(init));
                            }

                            line.push(';');
                            self.emit_line(&line);
                        } else {
                            // Slice: Slice_int32_t arr
                            let c_type = var_type.to_c_type();  // Returns Slice_int32_t
                            let mut line = format!("{}{} {}", const_keyword, c_type, var_decl.name);

                            if let Some(ref init) = var_decl.initializer {
                                line.push_str(" = ");
                                line.push_str(&self.generate_method_chain_temps(init));
                            }

                            line.push(';');
                            self.emit_line(&line);
                        }
                    }
                    _ => {
                        // Regular variable declaration
                        let c_type = var_type.to_c_type();

                        let mut line = format!("{}{} {}", const_keyword, c_type, var_decl.name);

                        if let Some(ref init) = var_decl.initializer {
                            line.push_str(" = ");
                            line.push_str(&self.generate_method_chain_temps(init));
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
                        let index_code = self.expression_to_c(&index_expr.index);

                        // Check if this is a slice (reference to array) or regular array
                        let result = if let Expression::Variable(var_name) = &*index_expr.array {
                            if let Some(ty) = self.variable_types.get(var_name) {
                                match ty {
                                    Type::Reference { inner, .. } => {
                                        // Slice type: use array.data[index]
                                        match &**inner {
                                            Type::Array { .. } => {
                                                Some(format!("{}.data[{}]", var_name, index_code))
                                            }
                                            _ => None
                                        }
                                    }
                                    _ => None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        // Use result if we have one, otherwise generate regular array access
                        result.unwrap_or_else(|| {
                            let array_code = self.expression_to_c(&index_expr.array);
                            format!("{}[{}]", array_code, index_code)
                        })
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
                    ForIterable::Expression(expr) => {
                        // Get array expression and its type
                        let var = &for_stmt.variable;

                        match &**expr {
                            Expression::ArrayLiteral(array_lit) => {
                                // Array literal: create temporary array
                                let array_name = format!("__array_{}", var);
                                let len = array_lit.elements.len();

                                // Declare and initialize array
                                let elem_strings: Vec<String> = array_lit.elements.iter()
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
                            }
                            Expression::Variable(array_var) => {
                                // Array variable: use existing variable
                                // Get array size from variable type
                                let array_size = if let Some(Type::Array { size: Some(n), element_type }) = self.variable_types.get(array_var) {
                                    let elem_c_type = element_type.to_c_type();
                                    (*n, elem_c_type)
                                } else {
                                    return Err(format!("Cannot determine size of array '{}'", array_var));
                                };

                                // Iterate over array
                                self.emit_line(&format!(
                                    "for (size_t __i_{} = 0; __i_{} < {}; __i_{}++)",
                                    var, var, array_size.0, var
                                ));
                                self.emit_line("{");
                                self.indent();

                                // Declare loop variable
                                self.emit_line(&format!(
                                    "{} {} = {}[__i_{}];",
                                    array_size.1, var, array_var, var
                                ));
                            }
                            _ => {
                                return Err("For loop iterable must be array literal or array variable".to_string());
                            }
                        }

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
                // No automatic slice conversion - user must use & explicitly
                let args: Vec<String> = call.arguments.iter()
                    .map(|arg| self.expression_to_c(arg))
                    .collect();
                format!("{}({})", call.callee, args.join(", "))
            }

            Expression::FieldAccess(field_access) => {
                // Special handling for array.length
                if field_access.field == "length" {
                    // Check if the object is an array variable
                    if let Expression::Variable(var_name) = &*field_access.object {
                        if let Some(Type::Array { size: Some(n), .. }) = self.variable_types.get(var_name) {
                            // Return compile-time size as a constant
                            return format!("{}", n);
                        }
                    }
                }

                // Regular struct field access
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

                // Generate method arguments (object as first argument)
                let mut args = vec![format!("&{}", object_expr)];
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
                BuiltinMacroGenerator::generate_macro_call(
                    macro_call,
                    &|expr| self.expression_to_c(expr),
                    &|expr| self.infer_expression_type(expr),
                )
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
                let index_code = self.expression_to_c(&index_expr.index);

                // Check if this is a slice (reference to array) or regular array
                if let Expression::Variable(var_name) = &*index_expr.array {
                    if let Some(ty) = self.variable_types.get(var_name) {
                        match ty {
                            Type::Reference { inner, .. } => {
                                // Slice type: use array.data[index]
                                match &**inner {
                                    Type::Array { .. } => {
                                        return format!("{}.data[{}]", var_name, index_code);
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                }

                // Regular array: use array[index]
                let array_code = self.expression_to_c(&index_expr.array);
                format!("{}[{}]", array_code, index_code)
            }

            Expression::Reference(ref_expr) => {
                // &expr creates a slice from an array
                // Generate: (Slice_T){.data = expr, .length = N}
                match &*ref_expr.inner {
                    Expression::Variable(var_name) => {
                        // &arr where arr: [N]T
                        if let Some(Type::Array { size: Some(n), element_type }) = self.variable_types.get(var_name) {
                            let slice_type = format!("Slice_{}", element_type.to_c_type().replace("*", "ptr").replace(" ", "_"));
                            format!("({}){{.data = {}, .length = {}}}", slice_type, var_name, n)
                        } else {
                            format!("/* cannot take reference of non-array {} */", var_name)
                        }
                    }
                    Expression::ArrayLiteral(array_lit) => {
                        // &[1, 2, 3] or &[[1,2],[3,4]]
                        // Need to infer element type
                        if let Some(first_elem) = array_lit.elements.first() {
                            if let Some(elem_type) = self.infer_expression_type(first_elem) {
                                let len = array_lit.elements.len();
                                // Create temporary array
                                let elements_str: Vec<String> = array_lit.elements.iter()
                                    .map(|e| self.expression_to_c(e))
                                    .collect();

                                // Handle multi-dimensional arrays properly
                                match &elem_type {
                                    Type::Array { element_type: inner_elem, size: Some(inner_size) } => {
                                        // Multi-dimensional: &[[1,2],[3,4]]
                                        // elem_type is [2]i32, need to create Slice of [2]i32
                                        // Get the base type for the slice struct name
                                        let base_type = inner_elem.to_c_type();
                                        let slice_type = format!("Slice_{}", base_type.replace("*", "ptr").replace(" ", "_"));
                                        // C array type: int32_t[2]
                                        let c_array_elem_type = format!("{}[{}]", base_type, inner_size);
                                        let array_literal = format!("({}[]){{ {} }}", c_array_elem_type, elements_str.join(", "));
                                        format!("({}){{.data = {}, .length = {}}}", slice_type, array_literal, len)
                                    }
                                    _ => {
                                        // 1D array: &[1, 2, 3]
                                        let slice_type = format!("Slice_{}", elem_type.to_c_type().replace("*", "ptr").replace(" ", "_"));
                                        let array_literal = format!("({}[]){{ {} }}", elem_type.to_c_type(), elements_str.join(", "));
                                        format!("({}){{.data = {}, .length = {}}}", slice_type, array_literal, len)
                                    }
                                }
                            } else {
                                "/* cannot infer array element type */".to_string()
                            }
                        } else {
                            "/* empty array literal */".to_string()
                        }
                    }
                    Expression::Index(_) => {
                        // &matrix[i] where matrix: [N][M]T → creates slice of [M]T
                        // Need to infer the type of the indexed expression
                        if let Some(indexed_type) = self.infer_expression_type(&ref_expr.inner) {
                            match indexed_type {
                                Type::Array { element_type, size: Some(n) } => {
                                    // indexed_type is [N]T, create slice []T
                                    let slice_type = format!("Slice_{}", element_type.to_c_type().replace("*", "ptr").replace(" ", "_"));
                                    let index_c = self.expression_to_c(&ref_expr.inner);
                                    format!("({}){{.data = {}, .length = {}}}", slice_type, index_c, n)
                                }
                                _ => {
                                    format!("/* cannot create slice from indexed expression */")
                                }
                            }
                        } else {
                            format!("/* cannot infer type of indexed expression */")
                        }
                    }
                    _ => {
                        format!("/* unsupported reference expression */")
                    }
                }
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

    /// Collect all dimensions from a multi-dimensional array type
    /// Returns (dimensions, base_element_type)
    /// Example: [2][3][4]i32 -> ([2, 3, 4], i32)
    fn collect_array_dimensions(&self, ty: &Type) -> (Vec<usize>, Type) {
        let mut dimensions = vec![];
        let mut current = ty;

        loop {
            match current {
                Type::Array { element_type, size } => {
                    if let Some(s) = size {
                        dimensions.push(*s);
                        current = element_type;
                    } else {
                        // Slice type - stop collecting
                        break;
                    }
                }
                _ => break,
            }
        }

        (dimensions, current.clone())
    }

    fn indent(&mut self) {
        self.indent_level += 1;
    }

    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    /// Infer the type of an expression for builtin macros
    fn infer_expression_type(&self, expr: &Expression) -> Option<Type> {
        match expr {
            Expression::Variable(name) => self.variable_types.get(name).cloned(),
            Expression::IntLiteral(_) => Some(Type::I32),
            Expression::FloatLiteral(_) => Some(Type::F64),
            Expression::BoolLiteral(_) => Some(Type::Bool),
            Expression::StringLiteral(_) => Some(Type::String),
            Expression::FieldAccess(field_access) => {
                // Special case for array.length
                if field_access.field == "length" {
                    return Some(Type::I32);
                }

                // Infer struct field type
                if let Some(struct_name) = self.infer_expression_type_name(&field_access.object) {
                    if let Some(fields) = self.struct_table.get(&struct_name) {
                        if let Some(field) = fields.iter().find(|f| f.name == field_access.field) {
                            return Some(field.field_type.clone());
                        }
                    }
                }

                None
            }
            Expression::Index(index_expr) => {
                // arr[i] returns element type
                if let Expression::Variable(name) = &*index_expr.array {
                    if let Some(var_type) = self.variable_types.get(name) {
                        match var_type {
                            Type::Array { element_type, .. } => {
                                return Some((**element_type).clone());
                            }
                            Type::Reference { inner, .. } => {
                                if let Type::Array { element_type, .. } = &**inner {
                                    return Some((**element_type).clone());
                                }
                            }
                            _ => {}
                        }
                    }
                }
                None
            }
            _ => None,
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
            Expression::MethodCall(method_call) => {
                // Get the object's struct name
                if let Some(struct_name) = self.infer_expression_type_name(&method_call.object) {
                    // Look up the method in the impl table
                    if let Some(methods) = self.impl_table.get(&struct_name) {
                        // Find the method by name
                        if let Some(method) = methods.iter().find(|m| m.name == method_call.method) {
                            // Check if the return type is a struct
                            match &method.return_type {
                                Type::Struct(return_struct_name) => Some(return_struct_name.clone()),
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
            Expression::AssociatedCall(assoc_call) => {
                // Look up the associated function in the impl table
                if let Some(methods) = self.impl_table.get(&assoc_call.type_name) {
                    // Find the function by name
                    if let Some(method) = methods.iter().find(|m| m.name == assoc_call.function) {
                        // Check if the return type is a struct
                        match &method.return_type {
                            Type::Struct(return_struct_name) => Some(return_struct_name.clone()),
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Expression::Call(_) => {
                // Regular function calls - would need function_table access
                None
            }
            _ => None,
        }
    }

    /// Check if an expression is a method chain (method called on another method/function result)
    fn is_method_chain(&self, expr: &Expression) -> bool {
        match expr {
            Expression::MethodCall(method_call) => {
                // Check if the object is a method or associated call (not a simple variable)
                matches!(&*method_call.object,
                    Expression::MethodCall(_) | Expression::AssociatedCall(_))
            }
            _ => false,
        }
    }

    /// Generate temporary variables for method chain and return the final variable name
    /// This transforms: p.move_by(...).scale(...)
    /// Into: __tmp_0 = p.move_by(...); __tmp_1 = __tmp_0.scale(...); return "__tmp_1"
    fn generate_method_chain_temps(&mut self, expr: &Expression) -> String {
        if !self.is_method_chain(expr) {
            // Not a chain, generate normally
            return self.expression_to_c(expr);
        }

        // Collect all steps in the chain from innermost to outermost
        let mut steps = Vec::new();
        let mut current = expr;

        while let Expression::MethodCall(method_call) = current {
            steps.push(current);
            current = &method_call.object;
        }

        // Reverse to process from innermost (base) to outermost
        steps.reverse();

        // Generate the base expression (non-chained part)
        // If the base is an AssociatedCall, we need a temp for it too
        let mut prev_var = match current {
            Expression::AssociatedCall(assoc_call) => {
                // Generate temp variable for the associated call result
                let temp_name = format!("__tmp_{}", self.temp_var_counter);
                self.temp_var_counter += 1;

                // Get the return type
                let type_str = if let Some(methods) = self.impl_table.get(&assoc_call.type_name) {
                    if let Some(method) = methods.iter().find(|m| m.name == assoc_call.function) {
                        match &method.return_type {
                            Type::Struct(name) => name.clone(),
                            _ => "Unknown".to_string(),
                        }
                    } else {
                        "Unknown".to_string()
                    }
                } else {
                    "Unknown".to_string()
                };

                // Generate the call
                let call_str = self.expression_to_c(current);
                self.emit_line(&format!("{} {} = {};", type_str, temp_name, call_str));
                temp_name
            }
            _ => self.expression_to_c(current),
        };

        // Generate temps for each chained method call
        for step in steps.iter() {
            if let Expression::MethodCall(method_call) = step {
                let temp_name = format!("__tmp_{}", self.temp_var_counter);
                self.temp_var_counter += 1;

                // Get the struct name for the method (object's type)
                let object_struct_name = self.infer_expression_type_name(&method_call.object)
                    .unwrap_or_else(|| "Unknown".to_string());

                // Get the return type for the temp variable declaration
                let return_type = if let Some(methods) = self.impl_table.get(&object_struct_name) {
                    if let Some(method) = methods.iter().find(|m| m.name == method_call.method) {
                        method.return_type.clone()
                    } else {
                        Type::Void
                    }
                } else {
                    Type::Void
                };

                let type_c_str = return_type.to_c_type();

                // Generate method call with previous result
                let mut args = vec![format!("&{}", prev_var)];
                for arg in &method_call.arguments {
                    args.push(self.expression_to_c(arg));
                }

                let method_call_str = format!("{}_{}({})",
                    object_struct_name,
                    method_call.method,
                    args.join(", ")
                );

                // Emit temp variable declaration
                self.emit_line(&format!("{} {} = {};", type_c_str, temp_name, method_call_str));

                prev_var = temp_name;
            }
        }

        prev_var
    }
}