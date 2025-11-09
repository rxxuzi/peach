// Type checker implementation

use crate::parser::ast::*;
use std::collections::HashMap;

pub struct TypeChecker {
    // Symbol table: variable name -> (type, is_mutable)
    symbol_table: HashMap<String, (Type, bool)>,
    // Function table: function name -> (param types, return type)
    function_table: HashMap<String, (Vec<Type>, Type)>,
    // Struct table: struct name -> fields
    struct_table: HashMap<String, Vec<StructField>>,
    // Impl table: struct name -> methods
    impl_table: HashMap<String, Vec<Method>>,
    // Error collection
    errors: Vec<String>,
    // Loop context tracking (for break/continue validation)
    loop_depth: usize,
    // Current method context (for checking self mutability)
    current_method_self_param: Option<SelfParam>,
    // Filename for error reporting with line numbers
    filename: String,
}

impl TypeChecker {
    pub fn new(filename: String) -> Self {
        TypeChecker {
            symbol_table: HashMap::new(),
            function_table: HashMap::new(),
            struct_table: HashMap::new(),
            impl_table: HashMap::new(),
            errors: Vec::new(),
            loop_depth: 0,
            current_method_self_param: None,
            filename,
        }
    }

    /// Helper to format error message with line number
    fn error_with_span(&self, message: String, span: Option<Span>) -> String {
        if let Some(s) = span {
            format!("{}:{}:{}: error: {}", self.filename, s.line, s.column, message)
        } else {
            format!("{}: error: {}", self.filename, message)
        }
    }

    pub fn check_program(&mut self, program: &mut Program) -> Result<(), String> {
        // First pass: collect all struct definitions
        for struct_def in &program.structs {
            if self.struct_table.contains_key(&struct_def.name) {
                self.errors.push(format!("Struct '{}' is already defined", struct_def.name));
            } else {
                self.struct_table.insert(
                    struct_def.name.clone(),
                    struct_def.fields.clone()
                );
            }
        }

        // Second pass: collect all impl blocks
        for impl_block in &program.impls {
            // Check that the struct exists
            if !self.struct_table.contains_key(&impl_block.struct_name) {
                self.errors.push(format!(
                    "Cannot implement methods for undefined struct '{}'",
                    impl_block.struct_name
                ));
                continue;
            }

            // Store methods for this struct
            if self.impl_table.contains_key(&impl_block.struct_name) {
                self.errors.push(format!(
                    "Struct '{}' already has an impl block",
                    impl_block.struct_name
                ));
            } else {
                self.impl_table.insert(
                    impl_block.struct_name.clone(),
                    impl_block.methods.clone()
                );
            }
        }

        // Third pass: collect all function signatures
        for function in &program.functions {
            let param_types: Vec<Type> = function.parameters.iter()
                .map(|p| p.param_type.clone())
                .collect();

            if self.function_table.contains_key(&function.name) {
                self.errors.push(format!("Function '{}' is already defined", function.name));
            } else {
                self.function_table.insert(
                    function.name.clone(),
                    (param_types, function.return_type.clone())
                );
            }
        }

        // Fourth pass: type check method bodies
        for impl_block in &mut program.impls {
            for method in &mut impl_block.methods {
                self.check_method(&impl_block.struct_name, method);
            }
        }

        // Fifth pass: type check function bodies
        for function in &mut program.functions {
            self.check_function(function);
        }

        // Return all errors or Ok
        if !self.errors.is_empty() {
            // Return only the first error for now
            // (multiple error reporting would require changing the return type)
            Err(self.errors[0].clone())
        } else {
            Ok(())
        }
    }

    /// Get all collected errors
    pub fn get_errors(&self) -> &Vec<String> {
        &self.errors
    }

    fn check_method(&mut self, struct_name: &str, method: &mut Method) {
        // Clear symbol table for new method scope
        self.symbol_table.clear();

        // Set current method self param context
        self.current_method_self_param = method.self_param.clone();

        // Add self parameter if present
        if let Some(ref self_param) = method.self_param {
            // self is always a reference to the struct
            let is_mutable = matches!(self_param, SelfParam::MutRef);
            self.symbol_table.insert(
                "self".to_string(),
                (Type::Struct(struct_name.to_string()), is_mutable)
            );
        }

        // Add parameters to symbol table (parameters are immutable)
        for param in &method.parameters {
            self.symbol_table.insert(
                param.name.clone(),
                (param.param_type.clone(), false)
            );
        }

        // Check method body
        self.check_block(&mut method.body);

        // Clear current method context
        self.current_method_self_param = None;
    }

    fn check_function(&mut self, function: &mut Function) {
        // Clear symbol table for new function scope
        self.symbol_table.clear();

        // Clear method context (we're not in a method)
        self.current_method_self_param = None;

        // Add parameters to symbol table (parameters are immutable)
        for param in &function.parameters {
            self.symbol_table.insert(
                param.name.clone(),
                (param.param_type.clone(), false)
            );
        }

        // Check function body
        self.check_block(&mut function.body);
    }

    fn check_block(&mut self, block: &mut Block) {
        for statement in &mut block.statements {
            self.check_statement(statement);
        }
    }

    fn check_statement(&mut self, statement: &mut Statement) {
        match statement {
            Statement::Return(ret) => {
                if let Some(ref expr) = ret.value {
                    if let Err(e) = self.infer_expression_type(expr) {
                        self.errors.push(e);
                    }
                }
            }

            Statement::VarDecl(var_decl) => {
                // Check for redeclaration FIRST
                if self.symbol_table.contains_key(&var_decl.name) {
                    self.errors.push(format!(
                        "Variable '{}' is already declared in this scope",
                        var_decl.name
                    ));
                    return; // Don't add to symbol table
                }

                let var_type = if let Some(ref declared_type) = var_decl.var_type {
                    // Type is explicitly declared
                    if let Some(ref init) = var_decl.initializer {
                        // Check that initializer matches declared type
                        match self.infer_expression_type(init) {
                            Ok(init_type) => {
                                if !self.types_match(&init_type, declared_type) {
                                    self.errors.push(format!(
                                        "Type mismatch: variable '{}' declared as '{}' but initialized with '{}'",
                                        var_decl.name,
                                        declared_type.to_string(),
                                        init_type.to_string()
                                    ));
                                }
                            }
                            Err(e) => {
                                self.errors.push(e);
                                // Continue with declared type even if initializer failed
                            }
                        }
                    }
                    declared_type.clone()
                } else if let Some(ref init) = var_decl.initializer {
                    // Infer type from initializer
                    match self.infer_expression_type(init) {
                        Ok(inferred_type) => {
                            // Update AST with inferred type
                            var_decl.var_type = Some(inferred_type.clone());
                            inferred_type
                        }
                        Err(e) => {
                            self.errors.push(e);
                            // Use default type (i32) to prevent cascading errors
                            let default_type = Type::I32;
                            var_decl.var_type = Some(default_type.clone());
                            default_type
                        }
                    }
                } else {
                    self.errors.push(format!(
                        "Variable '{}' must have either a type annotation or an initializer",
                        var_decl.name
                    ));
                    // Use default type to prevent cascading errors
                    let default_type = Type::I32;
                    var_decl.var_type = Some(default_type.clone());
                    default_type
                };

                // ALWAYS add variable to symbol table (even if there were errors)
                // This prevents cascading "undefined variable" errors
                self.symbol_table.insert(
                    var_decl.name.clone(),
                    (var_type, var_decl.is_mutable)
                );
            }

            Statement::Assignment(assignment) => {
                match &assignment.target {
                    AssignmentTarget::Variable(name) => {
                        // Check that variable exists
                        if let Some((var_type, is_mutable)) = self.symbol_table.get(name) {
                            // Check mutability
                            if !is_mutable {
                                self.errors.push(format!(
                                    "Cannot assign to immutable variable '{}'",
                                    name
                                ));
                            }

                            // Check type compatibility
                            match self.infer_expression_type(&assignment.value) {
                                Ok(value_type) => {
                                    if !self.types_match(&value_type, var_type) {
                                        self.errors.push(format!(
                                            "Type mismatch in assignment: variable '{}' has type '{}' but assigned value has type '{}'",
                                            name,
                                            var_type.to_string(),
                                            value_type.to_string()
                                        ));
                                    }
                                }
                                Err(e) => {
                                    self.errors.push(e);
                                }
                            }
                        } else {
                            self.errors.push(format!("Undefined variable '{}'", name));
                        }
                    }

                    AssignmentTarget::FieldAccess(field_access) => {
                        // Get the type of the field being assigned
                        match self.infer_expression_type(&Expression::FieldAccess(field_access.clone())) {
                            Ok(field_type) => {
                                // Check if we're assigning through an object
                                if let Expression::Variable(var_name) = &*field_access.object {
                                    // Special case for 'self'
                                    if var_name == "self" {
                                        // Check if current method has &mut self
                                        match &self.current_method_self_param {
                                            Some(SelfParam::MutRef) => {
                                                // OK: method has &mut self
                                            }
                                            Some(SelfParam::Ref) => {
                                                let msg = "Cannot assign to field of '&self' in method with immutable self reference. Use '&mut self' instead.".to_string();
                                                self.errors.push(self.error_with_span(msg, assignment.span.clone()));
                                            }
                                            Some(SelfParam::Owned) | None => {
                                                let msg = "Cannot assign to 'self' outside of method context".to_string();
                                                self.errors.push(self.error_with_span(msg, assignment.span.clone()));
                                            }
                                        }
                                    } else {
                                        // Regular variable - check mutability
                                        if let Some((_, is_mutable)) = self.symbol_table.get(var_name) {
                                            if !is_mutable {
                                                self.errors.push(format!(
                                                    "Cannot assign to field of immutable variable '{}'",
                                                    var_name
                                                ));
                                            }
                                        }
                                    }
                                }

                                // Check type compatibility with assigned value
                                match self.infer_expression_type(&assignment.value) {
                                    Ok(value_type) => {
                                        if !self.types_match(&value_type, &field_type) {
                                            self.errors.push(format!(
                                                "Type mismatch in field assignment: field has type '{}' but assigned value has type '{}'",
                                                field_type.to_string(),
                                                value_type.to_string()
                                            ));
                                        }
                                    }
                                    Err(e) => {
                                        self.errors.push(e);
                                    }
                                }
                            }
                            Err(e) => {
                                self.errors.push(e);
                            }
                        }
                    }
                }
            }

            Statement::If(if_stmt) => {
                // Check condition is boolean
                match self.infer_expression_type(&if_stmt.condition) {
                    Ok(cond_type) => {
                        if cond_type != Type::Bool {
                            self.errors.push(format!(
                                "If condition must be boolean, found '{}'",
                                cond_type.to_string()
                            ));
                        }
                    }
                    Err(e) => {
                        self.errors.push(e);
                    }
                }

                // Check then block
                self.check_block(&mut if_stmt.then_block);

                // Check else block if present
                if let Some(ref mut else_block) = if_stmt.else_block {
                    self.check_block(else_block);
                }
            }

            Statement::While(while_stmt) => {
                // Check condition is boolean
                match self.infer_expression_type(&while_stmt.condition) {
                    Ok(cond_type) => {
                        if cond_type != Type::Bool {
                            self.errors.push(format!(
                                "While condition must be boolean, found '{}'",
                                cond_type.to_string()
                            ));
                        }
                    }
                    Err(e) => {
                        self.errors.push(e);
                    }
                }

                // Check body in loop context
                self.loop_depth += 1;
                self.check_block(&mut while_stmt.body);
                self.loop_depth -= 1;
            }

            Statement::Loop(loop_stmt) => {
                // Check body in loop context
                self.loop_depth += 1;
                self.check_block(&mut loop_stmt.body);
                self.loop_depth -= 1;
            }

            Statement::For(for_stmt) => {
                // Save current symbol table state
                let saved_symbols = self.symbol_table.clone();

                // Check iterable and add loop variable
                match &for_stmt.iterable {
                    ForIterable::Range(start, end) => {
                        // Check that start and end are integers
                        if let Err(e) = self.infer_expression_type(start) {
                            self.errors.push(e);
                        }
                        if let Err(e) = self.infer_expression_type(end) {
                            self.errors.push(e);
                        }
                        // Loop variable is i32 for ranges
                        self.symbol_table.insert(
                            for_stmt.variable.clone(),
                            (Type::I32, false)  // loop variable is immutable
                        );
                    }
                    ForIterable::Array(elements) => {
                        // Infer array element type from first element
                        if let Some(first) = elements.first() {
                            match self.infer_expression_type(first) {
                                Ok(elem_type) => {
                                    // Check all elements have same type
                                    for (i, elem) in elements.iter().enumerate().skip(1) {
                                        match self.infer_expression_type(elem) {
                                            Ok(t) => {
                                                if !self.types_match(&t, &elem_type) {
                                                    self.errors.push(format!(
                                                        "Array element {} has type '{}', expected '{}'",
                                                        i,
                                                        t.to_string(),
                                                        elem_type.to_string()
                                                    ));
                                                }
                                            }
                                            Err(e) => self.errors.push(e),
                                        }
                                    }
                                    // Loop variable has element type
                                    self.symbol_table.insert(
                                        for_stmt.variable.clone(),
                                        (elem_type, false)  // loop variable is immutable
                                    );
                                }
                                Err(e) => self.errors.push(e),
                            }
                        }
                    }
                }

                // Check body in loop context
                self.loop_depth += 1;
                self.check_block(&mut for_stmt.body);
                self.loop_depth -= 1;

                // Restore symbol table (remove loop variable)
                self.symbol_table = saved_symbols;
            }

            Statement::Break(_) => {
                if self.loop_depth == 0 {
                    self.errors.push("'break' statement outside of loop".to_string());
                }
            }

            Statement::Continue(_) => {
                if self.loop_depth == 0 {
                    self.errors.push("'continue' statement outside of loop".to_string());
                }
            }

            Statement::Expression(expr) => {
                if let Err(e) = self.infer_expression_type(expr) {
                    self.errors.push(e);
                }
            }
        }
    }

    fn infer_expression_type(&self, expr: &Expression) -> Result<Type, String> {
        match expr {
            Expression::IntLiteral(_) => Ok(Type::I32),
            Expression::FloatLiteral(_) => Ok(Type::F64),
            Expression::BoolLiteral(_) => Ok(Type::Bool),
            Expression::StringLiteral(_) => Ok(Type::String),

            Expression::Variable(name) => {
                self.symbol_table.get(name)
                    .map(|(ty, _)| ty.clone())
                    .ok_or_else(|| {
                        // Try to find similar variable names
                        let similar = self.find_similar_variable(name);
                        if let Some(suggestion) = similar {
                            format!("Undefined variable '{}', did you mean '{}'?", name, suggestion)
                        } else {
                            format!("Undefined variable '{}'", name)
                        }
                    })
            }

            Expression::Binary(binary) => {
                let left_type = self.infer_expression_type(&binary.left)?;
                let right_type = self.infer_expression_type(&binary.right)?;

                match binary.operator {
                    // Comparison operators return bool
                    BinaryOperator::Equal | BinaryOperator::NotEqual |
                    BinaryOperator::Less | BinaryOperator::Greater |
                    BinaryOperator::LessEqual | BinaryOperator::GreaterEqual => {
                        if !self.types_match(&left_type, &right_type) {
                            return Err(format!(
                                "Type mismatch in comparison: '{}' {} '{}'",
                                left_type.to_string(),
                                self.operator_to_string(&binary.operator),
                                right_type.to_string()
                            ));
                        }
                        Ok(Type::Bool)
                    }

                    // Logical operators require bool operands and return bool
                    BinaryOperator::And | BinaryOperator::Or => {
                        if left_type != Type::Bool || right_type != Type::Bool {
                            return Err(format!(
                                "Logical operators require boolean operands, found '{}' {} '{}'",
                                left_type.to_string(),
                                self.operator_to_string(&binary.operator),
                                right_type.to_string()
                            ));
                        }
                        Ok(Type::Bool)
                    }

                    // Arithmetic operators
                    _ => {
                        if !self.types_match(&left_type, &right_type) {
                            return Err(format!(
                                "Type mismatch in binary expression: '{}' {} '{}'",
                                left_type.to_string(),
                                self.operator_to_string(&binary.operator),
                                right_type.to_string()
                            ));
                        }
                        Ok(left_type)
                    }
                }
            }

            Expression::Unary(unary) => {
                let operand_type = self.infer_expression_type(&unary.operand)?;

                match unary.operator {
                    UnaryOperator::Not => {
                        if operand_type != Type::Bool {
                            return Err(format!(
                                "Logical NOT requires boolean operand, found '{}'",
                                operand_type.to_string()
                            ));
                        }
                        Ok(Type::Bool)
                    }
                    UnaryOperator::Negate => {
                        // Check if type is numeric
                        match operand_type {
                            Type::I8 | Type::I16 | Type::I32 | Type::I64 |
                            Type::F32 | Type::F64 => Ok(operand_type),
                            _ => Err(format!(
                                "Negation requires numeric operand, found '{}'",
                                operand_type.to_string()
                            ))
                        }
                    }
                }
            }

            Expression::Call(call) => {
                // Look up function signature
                if let Some((param_types, return_type)) = self.function_table.get(&call.callee) {
                    // Check argument count
                    if call.arguments.len() != param_types.len() {
                        return Err(format!(
                            "Function '{}' expects {} arguments, but {} were provided",
                            call.callee,
                            param_types.len(),
                            call.arguments.len()
                        ));
                    }

                    // Check argument types
                    for (i, (arg, expected_type)) in call.arguments.iter().zip(param_types.iter()).enumerate() {
                        let arg_type = self.infer_expression_type(arg)?;
                        if !self.types_match(&arg_type, expected_type) {
                            return Err(format!(
                                "Type mismatch in argument {} of function '{}': expected '{}', found '{}'",
                                i + 1,
                                call.callee,
                                expected_type.to_string(),
                                arg_type.to_string()
                            ));
                        }
                    }

                    Ok(return_type.clone())
                } else {
                    Err(format!("Undefined function '{}'", call.callee))
                }
            }

            Expression::FieldAccess(field_access) => {
                // Infer type of the object
                let object_type = self.infer_expression_type(&field_access.object)?;

                // Extract struct name from type
                let struct_name = match object_type {
                    Type::Struct(name) => name,
                    _ => return Err(format!(
                        "Cannot access field '{}' on non-struct type '{}'",
                        field_access.field,
                        object_type.to_string()
                    )),
                };

                // Look up the struct definition
                if let Some(fields) = self.struct_table.get(&struct_name) {
                    // Find the field
                    if let Some(field) = fields.iter().find(|f| f.name == field_access.field) {
                        Ok(field.field_type.clone())
                    } else {
                        Err(format!(
                            "Struct '{}' has no field named '{}'",
                            struct_name,
                            field_access.field
                        ))
                    }
                } else {
                    Err(format!("Undefined struct '{}'", struct_name))
                }
            }

            Expression::MethodCall(method_call) => {
                // Infer type of the object
                let object_type = self.infer_expression_type(&method_call.object)?;

                // Extract struct name from type
                let struct_name = match object_type {
                    Type::Struct(name) => name,
                    _ => return Err(format!(
                        "Cannot call method '{}' on non-struct type '{}'",
                        method_call.method,
                        object_type.to_string()
                    )),
                };

                // Look up the method in the impl block
                if let Some(methods) = self.impl_table.get(&struct_name) {
                    // Find the method
                    if let Some(method) = methods.iter().find(|m| m.name == method_call.method) {
                        // Check that it's an instance method (has self parameter)
                        if method.self_param.is_none() {
                            return Err(format!(
                                "'{}::{}' is an associated function, not an instance method. Use '{}::{}()' instead.",
                                struct_name,
                                method_call.method,
                                struct_name,
                                method_call.method
                            ));
                        }

                        // Check argument count
                        if method_call.arguments.len() != method.parameters.len() {
                            return Err(format!(
                                "Method '{}.{}' expects {} arguments, but {} were provided",
                                struct_name,
                                method_call.method,
                                method.parameters.len(),
                                method_call.arguments.len()
                            ));
                        }

                        // Check argument types
                        for (i, (arg, param)) in method_call.arguments.iter().zip(method.parameters.iter()).enumerate() {
                            let arg_type = self.infer_expression_type(arg)?;
                            if !self.types_match(&arg_type, &param.param_type) {
                                return Err(format!(
                                    "Type mismatch in argument {} of '{}.{}': expected '{}', found '{}'",
                                    i + 1,
                                    struct_name,
                                    method_call.method,
                                    param.param_type.to_string(),
                                    arg_type.to_string()
                                ));
                            }
                        }

                        // Check if &mut self method is called on a mutable object
                        if let Some(SelfParam::MutRef) = method.self_param {
                            // Check if the object is mutable
                            if let Expression::Variable(var_name) = &*method_call.object {
                                if let Some((_, is_mutable)) = self.symbol_table.get(var_name) {
                                    if !is_mutable {
                                        let msg = format!(
                                            "Cannot call mutable method '{}.{}' on immutable variable '{}'. Consider using 'var' instead of 'val'.",
                                            struct_name,
                                            method_call.method,
                                            var_name
                                        );
                                        return Err(self.error_with_span(msg, method_call.span.clone()));
                                    }
                                }
                            }
                        }

                        Ok(method.return_type.clone())
                    } else {
                        Err(format!(
                            "Struct '{}' has no method named '{}'",
                            struct_name,
                            method_call.method
                        ))
                    }
                } else {
                    Err(format!("No methods defined for struct '{}'", struct_name))
                }
            }

            Expression::AssociatedCall(assoc_call) => {
                // Look up the struct
                if !self.struct_table.contains_key(&assoc_call.type_name) {
                    let msg = format!("Undefined type '{}'", assoc_call.type_name);
                    return Err(self.error_with_span(msg, assoc_call.span.clone()));
                }

                // Look up the method in the impl block
                if let Some(methods) = self.impl_table.get(&assoc_call.type_name) {
                    // Find the method
                    if let Some(method) = methods.iter().find(|m| m.name == assoc_call.function) {
                        // Check that it's an associated function (no self parameter)
                        if method.self_param.is_some() {
                            return Err(format!(
                                "'{}::{}' is an instance method, not an associated function",
                                assoc_call.type_name,
                                assoc_call.function
                            ));
                        }

                        // Check argument count
                        if assoc_call.arguments.len() != method.parameters.len() {
                            return Err(format!(
                                "Associated function '{}::{}' expects {} arguments, but {} were provided",
                                assoc_call.type_name,
                                assoc_call.function,
                                method.parameters.len(),
                                assoc_call.arguments.len()
                            ));
                        }

                        // Check argument types
                        for (i, (arg, param)) in assoc_call.arguments.iter().zip(method.parameters.iter()).enumerate() {
                            let arg_type = self.infer_expression_type(arg)?;
                            if !self.types_match(&arg_type, &param.param_type) {
                                return Err(format!(
                                    "Type mismatch in argument {} of '{}::{}': expected '{}', found '{}'",
                                    i + 1,
                                    assoc_call.type_name,
                                    assoc_call.function,
                                    param.param_type.to_string(),
                                    arg_type.to_string()
                                ));
                            }
                        }

                        Ok(method.return_type.clone())
                    } else {
                        Err(format!(
                            "Undefined associated function '{}::{}' for struct '{}'",
                            assoc_call.type_name,
                            assoc_call.function,
                            assoc_call.type_name
                        ))
                    }
                } else {
                    Err(format!("No methods defined for struct '{}'", assoc_call.type_name))
                }
            }

            Expression::StructLiteral(struct_literal) => {
                // Look up the struct definition
                if let Some(fields) = self.struct_table.get(&struct_literal.struct_name) {
                    // Check that all required fields are present
                    for field_def in fields {
                        if !struct_literal.fields.iter().any(|f| f.name == field_def.name) {
                            return Err(format!(
                                "Missing field '{}' in struct literal for '{}'",
                                field_def.name,
                                struct_literal.struct_name
                            ));
                        }
                    }

                    // Check that all provided fields exist and have correct types
                    for literal_field in &struct_literal.fields {
                        if let Some(field_def) = fields.iter().find(|f| f.name == literal_field.name) {
                            let value_type = self.infer_expression_type(&literal_field.value)?;
                            if !self.types_match(&value_type, &field_def.field_type) {
                                return Err(format!(
                                    "Type mismatch for field '{}' in struct '{}': expected '{}', found '{}'",
                                    literal_field.name,
                                    struct_literal.struct_name,
                                    field_def.field_type.to_string(),
                                    value_type.to_string()
                                ));
                            }
                        } else {
                            return Err(format!(
                                "Struct '{}' has no field named '{}'",
                                struct_literal.struct_name,
                                literal_field.name
                            ));
                        }
                    }

                    // Return the struct type
                    Ok(Type::Struct(struct_literal.struct_name.clone()))
                } else {
                    let msg = format!("Undefined struct '{}'", struct_literal.struct_name);
                    Err(self.error_with_span(msg, struct_literal.span.clone()))
                }
            }

            Expression::MacroCall(macro_call) => {
                // Validate macro name
                match macro_call.macro_name.as_str() {
                    "print" | "println" | "panic" => {
                        // These macros accept any number of arguments
                        // Type check all arguments
                        for arg in &macro_call.arguments {
                            self.infer_expression_type(arg)?;
                        }
                        Ok(Type::Void)
                    }
                    "format" => {
                        // format! returns a string
                        for arg in &macro_call.arguments {
                            self.infer_expression_type(arg)?;
                        }
                        Ok(Type::String)
                    }
                    _ => {
                        let msg = format!("Unknown builtin macro '{}'", macro_call.macro_name);
                        Err(self.error_with_span(msg, macro_call.span.clone()))
                    }
                }
            }
        }
    }

    fn types_match(&self, t1: &Type, t2: &Type) -> bool {
        t1 == t2
    }

    fn operator_to_string(&self, op: &BinaryOperator) -> &str {
        match op {
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
        }
    }

    /// Find similar variable name using Levenshtein distance
    fn find_similar_variable(&self, name: &str) -> Option<String> {
        let mut best_match: Option<(String, usize)> = None;

        for var_name in self.symbol_table.keys() {
            let distance = levenshtein_distance(name, var_name);

            // Only consider variables with distance <= 2
            if distance <= 2 && distance > 0 {
                if let Some((_, best_dist)) = &best_match {
                    if distance < *best_dist {
                        best_match = Some((var_name.clone(), distance));
                    }
                } else {
                    best_match = Some((var_name.clone(), distance));
                }
            }
        }

        best_match.map(|(name, _)| name)
    }
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(
                    matrix[i][j + 1] + 1,
                    matrix[i + 1][j] + 1
                ),
                matrix[i][j] + cost
            );
        }
    }

    matrix[len1][len2]
}