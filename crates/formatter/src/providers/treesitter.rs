use crate::formatter::{FormatResult, FormatterProvider};
use std::path::Path;
use syntax::languages::LanguageRegistry;

pub struct TreesitterProvider {
    registry: LanguageRegistry,
    indent_width: usize,
}

impl TreesitterProvider {
    pub fn new() -> Self {
        Self {
            registry: LanguageRegistry::new(),
            indent_width: 4,
        }
    }

    pub fn with_indent_width(indent_width: usize) -> Self {
        Self {
            registry: LanguageRegistry::new(),
            indent_width,
        }
    }
}

impl Default for TreesitterProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatterProvider for TreesitterProvider {
    fn name(&self) -> &str {
        "tree-sitter"
    }

    fn supported_extensions(&self) -> &[&str] {
        &[
            "rs", "js", "jsx", "ts", "tsx", "py", "java", "cpp", "cc", "cxx", "c", "h",
            "hpp", "go", "html", "css", "json", "yaml", "yml", "sh", "bash", "md",
        ]
    }

    fn is_available(&self) -> bool {
        true // Always available directly compiled in-process
    }

    fn format(&self, text: &str, file_path: Option<&Path>) -> FormatResult {
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return Ok(String::new());
        }

        let is_python = file_path
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .map(|ext| ext == "py")
            .unwrap_or(false);

        // Optional tree-sitter parse to identify comments and strings
        let tree = file_path
            .and_then(|path| self.registry.detect_language(path))
            .and_then(|lang_config| {
                let mut parser = self.registry.create_parser(lang_config);
                parser.parse(text, None)
            });

        let mut formatted_lines: Vec<String> = Vec::with_capacity(lines.len());
        let mut depth: usize = 0;
        let mut consecutive_blanks = 0;
        let mut byte_offset = 0;

        for line in lines {
            let line_len_with_nl = line.len() + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                consecutive_blanks += 1;
                if consecutive_blanks <= 1 {
                    formatted_lines.push(String::new());
                }
                byte_offset += line_len_with_nl;
                continue;
            }
            consecutive_blanks = 0;

            if is_python {
                // Python indentation logic:
                let is_dedent_keyword = trimmed.starts_with("elif ")
                    || trimmed.starts_with("elif:")
                    || trimmed.starts_with("else:")
                    || trimmed.starts_with("except ")
                    || trimmed.starts_with("except:")
                    || trimmed.starts_with("finally:");

                let line_depth = if is_dedent_keyword && depth > 0 {
                    depth - 1
                } else {
                    depth
                };

                let indent_str = " ".repeat(line_depth * self.indent_width);
                formatted_lines.push(format!("{}{}", indent_str, trimmed));

                // If line ends with ':' and is not a comment, increase depth for next lines
                if trimmed.ends_with(':') && !trimmed.starts_with('#') {
                    depth = line_depth + 1;
                }
            } else {
                // Braced / delimited languages (Java, C, C++, Rust, JS, TS, Go, CSS, JSON, etc.):
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

                // Scan line characters for opens/closes, ignoring characters inside strings/comments
                let mut line_opens = 0;
                let mut line_closes = 0;
                let mut in_quote: Option<char> = None;
                let mut prev_c = ' ';

                for (col, c) in trimmed.char_indices() {
                    let char_byte = byte_offset + (line.find(trimmed).unwrap_or(0)) + col;
                    
                    // If Tree-sitter tree is available, check if this byte is in a comment or string
                    let is_in_ts_literal = tree.as_ref().map_or(false, |t| {
                        if let Some(node) = t.root_node().descendant_for_byte_range(char_byte, char_byte) {
                            let k = node.kind();
                            k.contains("string") || k.contains("comment") || k == "string_literal" || k == "character_literal"
                        } else {
                            false
                        }
                    });

                    if is_in_ts_literal {
                        prev_c = c;
                        continue;
                    }

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

                depth = depth.saturating_add(line_opens).saturating_sub(line_closes);
            }

            byte_offset += line_len_with_nl;
        }

        let mut result = formatted_lines.join("\n");
        if text.ends_with('\n') || !result.is_empty() {
            result.push('\n');
        }

        Ok(result)
    }
}
