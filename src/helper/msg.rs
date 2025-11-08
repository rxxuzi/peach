// Error message formatting (GCC/G++ style)

use colored::Colorize;
use super::types::{CompileError, ErrorType};

pub struct MessageFormatter {
    source: String,
    filename: String,
}

impl MessageFormatter {
    pub fn new(source: String, filename: String) -> Self {
        MessageFormatter { source, filename }
    }

    /// Format and display error in GCC/G++ style
    pub fn report(&self, error: &CompileError) {
        let error_label = self.get_error_label(&error.error_type);

        // Print error header
        if let Some(span) = error.span {
            eprintln!(
                "{}:{}:{}: {}: {}",
                self.filename.bold(),
                span.line,
                span.column,
                error_label,
                error.message
            );

            // Show source code context
            self.show_source_context(span);
        } else {
            // Try to find the error location in source by searching for keywords
            if let Some(span) = self.find_error_location(&error.message) {
                eprintln!(
                    "{}:{}:{}: {}: {}",
                    self.filename.bold(),
                    span.line,
                    span.column,
                    error_label,
                    error.message
                );
                self.show_source_context(span);
            } else {
                eprintln!(
                    "{}: {}: {}",
                    self.filename.bold(),
                    error_label,
                    error.message
                );
            }
        }

        eprintln!();
    }

    fn get_error_label(&self, error_type: &ErrorType) -> colored::ColoredString {
        match error_type {
            ErrorType::LexerError => "error".red().bold(),
            ErrorType::ParseError => "error".red().bold(),
            ErrorType::TypeError => "error".red().bold(),
            ErrorType::NameError => "error".red().bold(),
            ErrorType::GeneratorError => "error".red().bold(),
        }
    }

    fn show_source_context(&self, span: super::types::Span) {
        let lines: Vec<&str> = self.source.lines().collect();

        if span.line == 0 || span.line > lines.len() {
            return;
        }

        let line_idx = span.line - 1;
        let line_content = lines[line_idx];

        // Line number width (for alignment)
        let line_num_width = span.line.to_string().len().max(3);

        // Show context: previous line (if exists)
        if line_idx > 0 {
            eprintln!(
                "{:>width$} | {}",
                (span.line - 1).to_string().bright_blue(),
                lines[line_idx - 1].bright_black(),
                width = line_num_width
            );
        }

        // Show error line
        eprintln!(
            "{:>width$} | {}",
            span.line.to_string().bright_blue(),
            line_content,
            width = line_num_width
        );

        // Show error indicator (^~~~~)
        let spaces = " ".repeat(span.column.saturating_sub(1));
        let carets = self.get_error_highlight(line_content, span.column);
        eprintln!(
            "{:>width$} | {}{}",
            " ".bright_blue(),
            spaces,
            carets.red().bold(),
            width = line_num_width
        );

        // Show context: next line (if exists)
        if line_idx + 1 < lines.len() {
            eprintln!(
                "{:>width$} | {}",
                (span.line + 1).to_string().bright_blue(),
                lines[line_idx + 1].bright_black(),
                width = line_num_width
            );
        }
    }

    fn get_error_highlight(&self, line: &str, column: usize) -> String {
        if column == 0 || column > line.len() {
            return "^".to_string();
        }

        let start = column - 1;
        let chars: Vec<char> = line.chars().collect();

        // For operators in expressions, highlight the entire expression
        // Look backward to find the start of the expression
        let mut expr_start = start;
        while expr_start > 0 {
            let c = chars[expr_start - 1];
            if c.is_whitespace() {
                break;
            }
            expr_start -= 1;
        }

        // Look forward to find the end of the expression
        let mut expr_end = start;
        while expr_end < chars.len() {
            let c = chars[expr_end];
            if c == ';' || c == '{' || c == '}' || c == ',' {
                break;
            }
            expr_end += 1;
        }

        // Trim trailing whitespace
        while expr_end > start && chars[expr_end - 1].is_whitespace() {
            expr_end -= 1;
        }

        let length = expr_end - expr_start;
        if length <= 1 {
            return "^".to_string();
        }

        // Return tildes with caret at the beginning
        let tildes = "~".repeat(length - 1);
        format!("^{}", tildes)
    }

    /// Show a simple note/hint message
    pub fn note(&self, message: &str) {
        eprintln!("{}: {}", "note".bright_cyan().bold(), message);
    }

    /// Show a help message
    pub fn help(&self, message: &str) {
        eprintln!("{}: {}", "help".bright_green().bold(), message);
    }

    /// Try to find the location of an error by searching the source code
    fn find_error_location(&self, error_message: &str) -> Option<super::types::Span> {
        let lines: Vec<&str> = self.source.lines().collect();

        // Look for specific patterns in error messages
        if error_message.contains("Type mismatch in binary expression") {
            // Extract operator from error message
            let operators = [" + ", " - ", " * ", " / ", " % "];

            // Search for val/var declarations with binary operations
            for (line_idx, line) in lines.iter().enumerate() {
                // Skip comment lines
                if line.trim_start().starts_with("//") {
                    continue;
                }

                // Look for lines with both "val"/"var" and an operator
                if (line.contains("val ") || line.contains("var ")) && line.contains(" = ") {
                    // Check if this line has a binary operation after the '='
                    if let Some(eq_pos) = line.find(" = ") {
                        let after_eq = &line[eq_pos + 3..];

                        // Check if there's an operator in the expression
                        let has_operator = operators.iter().any(|op| after_eq.contains(op));

                        if has_operator {
                            // Find the position of the first identifier after '='
                            let mut start_pos = eq_pos + 3;
                            let chars: Vec<char> = line.chars().collect();
                            while start_pos < chars.len() && chars[start_pos].is_whitespace() {
                                start_pos += 1;
                            }
                            return Some(super::types::Span::new(line_idx + 1, start_pos + 1));
                        }
                    }
                }
            }
        }

        if error_message.contains("Undefined variable") {
            // Extract variable name from error message
            if let Some(start) = error_message.find('\'') {
                if let Some(end) = error_message[start + 1..].find('\'') {
                    let var_name = &error_message[start + 1..start + 1 + end];

                    // Search for the variable in source (skip declarations)
                    for (line_idx, line) in lines.iter().enumerate() {
                        if let Some(pos) = line.find(var_name) {
                            // Make sure it's not in a comment
                            let before = &line[..pos];
                            if !before.contains("//") {
                                // Check if it's not a declaration
                                let trimmed_before = before.trim();
                                if !trimmed_before.ends_with("val ") &&
                                    !trimmed_before.ends_with("var ") &&
                                    !trimmed_before.ends_with("val") &&
                                    !trimmed_before.ends_with("var") {
                                    return Some(super::types::Span::new(line_idx + 1, pos + 1));
                                }
                            }
                        }
                    }
                }
            }
        }

        if error_message.contains("Cannot assign to immutable variable") {
            // Extract variable name from error message
            if let Some(start) = error_message.find('\'') {
                if let Some(end) = error_message[start + 1..].find('\'') {
                    let var_name = &error_message[start + 1..start + 1 + end];

                    // Search for assignment (not declaration)
                    for (line_idx, line) in lines.iter().enumerate() {
                        // Look for "varname = " pattern (not "val varname" or "var varname")
                        let pattern = format!("{} =", var_name);
                        if let Some(pos) = line.find(&pattern) {
                            let before = &line[..pos];
                            // Make sure it's not a declaration
                            if !before.contains("val ") && !before.contains("var ") {
                                return Some(super::types::Span::new(line_idx + 1, pos + 1));
                            }
                        }
                    }
                }
            }
        }

        if error_message.contains("already declared") {
            // Extract variable name from error message
            if let Some(start) = error_message.find('\'') {
                if let Some(end) = error_message[start + 1..].find('\'') {
                    let var_name = &error_message[start + 1..start + 1 + end];

                    // Find the SECOND declaration
                    let mut found_first = false;
                    for (line_idx, line) in lines.iter().enumerate() {
                        if (line.contains(&format!("val {}", var_name)) ||
                            line.contains(&format!("var {}", var_name))) &&
                            !line.trim_start().starts_with("//") {
                            if found_first {
                                // This is the second declaration - the error
                                if let Some(pos) = line.find(var_name) {
                                    return Some(super::types::Span::new(line_idx + 1, pos + 1));
                                }
                            } else {
                                found_first = true;
                            }
                        }
                    }
                }
            }
        }

        None
    }
}