use crate::formatter::{FormatResult, FormatterProvider};
use std::path::Path;

pub struct GenericProvider {
    indent_width: usize,
}

impl GenericProvider {
    pub fn new() -> Self {
        Self { indent_width: 4 }
    }

    pub fn with_indent_width(indent_width: usize) -> Self {
        Self { indent_width }
    }
}

impl Default for GenericProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatterProvider for GenericProvider {
    fn name(&self) -> &str {
        "generic-indent"
    }

    fn supported_extensions(&self) -> &[&str] {
        &["*"] // Universal fallback for any custom language
    }

    fn is_available(&self) -> bool {
        true // Always available directly in-process
    }

    fn format(&self, text: &str, _file_path: Option<&Path>) -> FormatResult {
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return Ok(String::new());
        }

        let mut formatted_lines: Vec<String> = Vec::with_capacity(lines.len());
        let mut depth: usize = 0;
        let mut consecutive_blanks = 0;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                consecutive_blanks += 1;
                if consecutive_blanks <= 1 {
                    formatted_lines.push(String::new());
                }
                continue;
            }
            consecutive_blanks = 0;

            // Check if line starts with a closing bracket: decreases current line indent
            let starts_with_closer = trimmed.starts_with('}')
                || trimmed.starts_with(']')
                || trimmed.starts_with(')');

            let line_depth = if starts_with_closer && depth > 0 {
                depth - 1
            } else {
                depth
            };

            let indent_str = " ".repeat(line_depth * self.indent_width);
            formatted_lines.push(format!("{}{}", indent_str, trimmed));

            // Count openings and closings in this line
            let mut line_opens = 0;
            let mut line_closes = 0;
            let mut in_quote: Option<char> = None;
            let mut prev_c = ' ';

            for c in trimmed.chars() {
                if in_quote.is_none() {
                    if c == '"' || c == '\'' || c == '`' {
                        in_quote = Some(c);
                    } else if c == '{' || c == '[' || c == '(' {
                        line_opens += 1;
                    } else if c == '}' || c == ']' || c == ')' {
                        line_closes += 1;
                    }
                } else if in_quote == Some(c) && prev_c != '\\' {
                    in_quote = None;
                }
                prev_c = c;
            }

            // Update depth for subsequent lines
            depth = depth.saturating_add(line_opens).saturating_sub(line_closes);
        }

        let mut result = formatted_lines.join("\n");
        if text.ends_with('\n') || !result.is_empty() {
            result.push('\n');
        }

        Ok(result)
    }
}

