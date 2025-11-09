// Builtin macro code generation

use crate::parser::ast::{MacroCallExpression, Expression};

pub struct BuiltinMacroGenerator;

impl BuiltinMacroGenerator {
    pub fn generate_macro_call(macro_call: &MacroCallExpression, expression_to_c: &dyn Fn(&Expression) -> String) -> String {
        match macro_call.macro_name.as_str() {
            "print" => Self::generate_print(macro_call, expression_to_c),
            "println" => Self::generate_println(macro_call, expression_to_c),
            "panic" => Self::generate_panic(macro_call, expression_to_c),
            "format" => Self::generate_format(macro_call, expression_to_c),
            _ => format!("/* Unknown macro: {} */", macro_call.macro_name),
        }
    }

    fn generate_print(macro_call: &MacroCallExpression, expression_to_c: &dyn Fn(&Expression) -> String) -> String {
        // print!(...) → printf(...)
        let args: Vec<String> = macro_call.arguments.iter()
            .map(|arg| expression_to_c(arg))
            .collect();

        if args.is_empty() {
            "printf(\"\")".to_string()
        } else if args.len() == 1 {
            // Single argument - check if it's a string literal or variable
            let first = &args[0];
            if first.starts_with('"') && first.ends_with('"') {
                // String literal
                format!("printf({})", first)
            } else {
                // Variable - use %s format
                format!("printf(\"%s\", {})", first)
            }
        } else {
            // Multiple arguments - first should be format string
            format!("printf({})", args.join(", "))
        }
    }

    fn generate_println(macro_call: &MacroCallExpression, expression_to_c: &dyn Fn(&Expression) -> String) -> String {
        // println!(...) → printf(...\n)
        let args: Vec<String> = macro_call.arguments.iter()
            .map(|arg| expression_to_c(arg))
            .collect();

        if args.is_empty() {
            "printf(\"\\n\")".to_string()
        } else if args.len() == 1 {
            // Single argument
            let first = &args[0];
            if first.starts_with('"') && first.ends_with('"') {
                // String literal - add \n before closing quote
                let without_quote = &first[1..first.len()-1];
                format!("printf(\"{}\\n\")", without_quote)
            } else {
                // Variable - use %s format with \n
                format!("printf(\"%s\\n\", {})", first)
            }
        } else {
            // Multiple arguments - modify format string to add \n
            let mut new_args = args.clone();
            let first = &args[0];
            if first.starts_with('"') && first.ends_with('"') {
                new_args[0] = format!("{}\\n\"", &first[..first.len()-1]);
            }
            format!("printf({})", new_args.join(", "))
        }
    }

    fn generate_panic(macro_call: &MacroCallExpression, expression_to_c: &dyn Fn(&Expression) -> String) -> String {
        // panic!(...) → fprintf(stderr, ...) + exit(1)
        let args: Vec<String> = macro_call.arguments.iter()
            .map(|arg| expression_to_c(arg))
            .collect();

        if args.is_empty() {
            "(fprintf(stderr, \"panic\\n\"), exit(1))".to_string()
        } else if args.len() == 1 {
            let first = &args[0];
            if first.starts_with('"') && first.ends_with('"') {
                // String literal - add \n
                let without_quote = &first[1..first.len()-1];
                format!("(fprintf(stderr, \"{}\\n\"), exit(1))", without_quote)
            } else {
                // Variable
                format!("(fprintf(stderr, \"%s\\n\", {}), exit(1))", first)
            }
        } else {
            format!("(fprintf(stderr, {}), exit(1))", args.join(", "))
        }
    }

    fn generate_format(_macro_call: &MacroCallExpression, _expression_to_c: &dyn Fn(&Expression) -> String) -> String {
        // format!(...) - For v0.2.1, just return empty string
        // TODO: Implement proper format string handling with string allocation
        "\"\"".to_string()
    }
}
