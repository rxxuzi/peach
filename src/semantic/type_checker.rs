// Type checker implementation

use crate::parser::ast::*;
use std::collections::HashMap;

pub struct TypeChecker {
    // Symbol table: variable name -> (type, is_mutable)
    symbol_table: HashMap<String, (Type, bool)>,
    // Function table: function name -> (param types, return type)
    function_table: HashMap<String, (Vec<Type>, Type)>,
    // Error collection
    errors: Vec<String>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            symbol_table: HashMap::new(),
            function_table: HashMap::new(),
            errors: Vec::new(),
        }
    }

    pub fn check_program(&mut self, program: &mut Program) -> Result<(), String> {
        // First pass: collect all function signatures
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

        // Second pass: type check function bodies
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

    fn check_function(&mut self, function: &mut Function) {
        // Clear symbol table for new function scope
        self.symbol_table.clear();

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
                // Check that variable exists
                if let Some((var_type, is_mutable)) = self.symbol_table.get(&assignment.name) {
                    // Check mutability
                    if !is_mutable {
                        self.errors.push(format!(
                            "Cannot assign to immutable variable '{}'",
                            assignment.name
                        ));
                    }

                    // Check type compatibility
                    match self.infer_expression_type(&assignment.value) {
                        Ok(value_type) => {
                            if !self.types_match(&value_type, var_type) {
                                self.errors.push(format!(
                                    "Type mismatch in assignment: variable '{}' has type '{}' but assigned value has type '{}'",
                                    assignment.name,
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
                    self.errors.push(format!("Undefined variable '{}'", assignment.name));
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