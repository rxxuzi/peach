// Builtin macro code generation

use crate::parser::ast::{MacroCallExpression, Expression, Type};

pub struct BuiltinMacroGenerator;

impl BuiltinMacroGenerator {
    pub fn generate_macro_call(
        macro_call: &MacroCallExpression,
        expression_to_c: &dyn Fn(&Expression) -> String,
        infer_type: &dyn Fn(&Expression) -> Option<Type>,
    ) -> String {
        match macro_call.macro_name.as_str() {
            "print" => Self::generate_print(macro_call, expression_to_c, infer_type),
            "println" => Self::generate_println(macro_call, expression_to_c, infer_type),
            "panic" => Self::generate_panic(macro_call, expression_to_c, infer_type),
            "format" => Self::generate_format(macro_call, expression_to_c, infer_type),
            "len" => Self::generate_len(macro_call, expression_to_c, infer_type),
            "copy" => Self::generate_copy(macro_call, expression_to_c, infer_type),
            _ => format!("/* Unknown macro: {} */", macro_call.macro_name),
        }
    }

    fn generate_print(
        macro_call: &MacroCallExpression,
        expression_to_c: &dyn Fn(&Expression) -> String,
        infer_type: &dyn Fn(&Expression) -> Option<Type>,
    ) -> String {
        // print!(...) → printf(...)
        if macro_call.arguments.is_empty() {
            return "printf(\"\")".to_string();
        }

        if macro_call.arguments.len() == 1 {
            // Single argument - infer type and generate appropriate printf
            let arg = &macro_call.arguments[0];
            let arg_c = expression_to_c(arg);

            // Check if it's a string literal
            if arg_c.starts_with('"') && arg_c.ends_with('"') {
                // String literal
                return format!("printf({})", arg_c);
            }

            // Infer the type
            if let Some(ty) = infer_type(arg) {
                let format_spec = Self::type_to_format_specifier(&ty);
                format!("printf(\"{}\", {})", format_spec, arg_c)
            } else {
                // Fallback to %s if type inference fails
                format!("printf(\"%s\", {})", arg_c)
            }
        } else {
            // Multiple arguments - first should be format string
            let args: Vec<String> = macro_call.arguments.iter()
                .map(|arg| expression_to_c(arg))
                .collect();
            format!("printf({})", args.join(", "))
        }
    }

    fn generate_println(
        macro_call: &MacroCallExpression,
        expression_to_c: &dyn Fn(&Expression) -> String,
        infer_type: &dyn Fn(&Expression) -> Option<Type>,
    ) -> String {
        // println!(...) → printf(...\n)
        if macro_call.arguments.is_empty() {
            return "printf(\"\\n\")".to_string();
        }

        if macro_call.arguments.len() == 1 {
            // Single argument - infer type and generate appropriate printf
            let arg = &macro_call.arguments[0];
            let arg_c = expression_to_c(arg);

            // Check if it's a string literal
            if arg_c.starts_with('"') && arg_c.ends_with('"') {
                // String literal - add \n before closing quote
                let without_quote = &arg_c[1..arg_c.len()-1];
                return format!("printf(\"{}\\n\")", without_quote);
            }

            // Infer the type
            if let Some(ty) = infer_type(arg) {
                let format_spec = Self::type_to_format_specifier(&ty);
                format!("printf(\"{}\\n\", {})", format_spec, arg_c)
            } else {
                // Fallback to %s if type inference fails
                format!("printf(\"%s\\n\", {})", arg_c)
            }
        } else {
            // Multiple arguments - first should be format string
            let args: Vec<String> = macro_call.arguments.iter()
                .map(|arg| expression_to_c(arg))
                .collect();

            let mut new_args = args.clone();
            let first = &args[0];
            if first.starts_with('"') && first.ends_with('"') {
                new_args[0] = format!("{}\\n\"", &first[..first.len()-1]);
            }
            format!("printf({})", new_args.join(", "))
        }
    }

    fn generate_panic(
        macro_call: &MacroCallExpression,
        expression_to_c: &dyn Fn(&Expression) -> String,
        _infer_type: &dyn Fn(&Expression) -> Option<Type>,
    ) -> String {
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

    fn generate_format(
        _macro_call: &MacroCallExpression,
        _expression_to_c: &dyn Fn(&Expression) -> String,
        _infer_type: &dyn Fn(&Expression) -> Option<Type>,
    ) -> String {
        // format!(...) - For v0.2.1, just return empty string
        // TODO: Implement proper format string handling with string allocation
        "\"\"".to_string()
    }

    fn generate_len(
        macro_call: &MacroCallExpression,
        expression_to_c: &dyn Fn(&Expression) -> String,
        infer_type: &dyn Fn(&Expression) -> Option<Type>,
    ) -> String {
        // len!(arr) → returns array length as i32
        // For fixed arrays: return compile-time size
        // For slices: return slice.length
        if macro_call.arguments.is_empty() {
            return "0".to_string();
        }

        let arg = &macro_call.arguments[0];
        if let Some(ty) = infer_type(arg) {
            match ty {
                Type::Array { size: Some(n), .. } => {
                    // Fixed-size array: return compile-time size
                    format!("{}", n)
                }
                Type::Array { size: None, .. } => {
                    // Unsized array (shouldn't happen after v0.2.5, but kept for compatibility)
                    let arg_c = expression_to_c(arg);
                    format!("((int32_t)({}.length))", arg_c)
                }
                Type::Reference { ref inner, .. } => {
                    // Slice type (&[]T or &mut []T): return .length field
                    match &**inner {
                        Type::Array { .. } => {
                            let arg_c = expression_to_c(arg);
                            format!("((int32_t)({}.length))", arg_c)
                        }
                        _ => "0".to_string(),
                    }
                }
                _ => "0".to_string(),
            }
        } else {
            "0".to_string()
        }
    }

    fn generate_copy(
        macro_call: &MacroCallExpression,
        expression_to_c: &dyn Fn(&Expression) -> String,
        infer_type: &dyn Fn(&Expression) -> Option<Type>,
    ) -> String {
        // copy!(dst, src) → memcpy(dst, src, sizeof(src))
        if macro_call.arguments.len() != 2 {
            return "/* copy! expects 2 arguments */".to_string();
        }

        let dst = &macro_call.arguments[0];
        let src = &macro_call.arguments[1];

        let dst_c = expression_to_c(dst);
        let src_c = expression_to_c(src);

        // Get the type to determine size
        if let Some(ty) = infer_type(src) {
            match ty {
                Type::Array { element_type: _, size: Some(_) } => {
                    // Fixed-size array - use memcpy
                    // Use sizeof to get array size dynamically
                    format!("memcpy({}, {}, sizeof({}))", dst_c, src_c, src_c)
                }
                _ => {
                    // For slices or other types, generate error comment
                    "/* copy! currently only supports fixed-size arrays */".to_string()
                }
            }
        } else {
            "/* copy! type inference failed */".to_string()
        }
    }

    /// Get the appropriate printf format specifier for a type
    fn type_to_format_specifier(ty: &Type) -> &'static str {
        match ty {
            Type::I8 | Type::I16 | Type::I32 => "%d",
            Type::I64 => "%lld",
            Type::U8 | Type::U16 | Type::U32 => "%u",
            Type::U64 => "%llu",
            Type::F32 | Type::F64 => "%f",
            Type::Bool => "%d",  // bool prints as 0/1
            Type::String => "%s",
            Type::Array { .. } => "%p",  // Arrays print as pointer for now
            Type::Struct(_) => "%p",     // Structs print as pointer
            Type::Reference { .. } => "%p",  // Reference types (slices) print as pointer
            Type::Void => "",
        }
    }
}
