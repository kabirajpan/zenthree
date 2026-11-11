use std::collections::HashSet;
use std::path::Path;
use editor::buffer::Buffer;
use super::CompletionKind;

#[derive(Debug, Clone)]
pub struct BufferSymbol {
    pub name: String,
    pub kind: CompletionKind,
    pub preview: String,
    pub detail: String,
    pub documentation: Option<String>,
    pub source_file: String,
    pub line_number: usize,
    pub import_hint: Option<String>,
}

/// Backward compatible helper: extracts unique raw tokens.
pub fn scan_buffer_identifiers(buffer: &Buffer, active_word: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut tokens = Vec::with_capacity(64);

    for line in buffer.lines().iter().take(500) {
        let bytes = line.as_bytes();
        let mut start: Option<usize> = None;

        for (i, &b) in bytes.iter().enumerate() {
            let is_ident = b.is_ascii_alphanumeric() || b == b'_';
            match (start, is_ident) {
                (None, true) => {
                    if !b.is_ascii_digit() {
                        start = Some(i);
                    }
                }
                (Some(s), false) => {
                    let word = &line[s..i];
                    if word.len() >= 2 && word != active_word && !seen.contains(word) {
                        seen.insert(word.to_string());
                        tokens.push(word.to_string());
                    }
                    start = None;
                }
                _ => {}
            }
        }

        if let Some(s) = start {
            let word = &line[s..];
            if word.len() >= 2 && word != active_word && !seen.contains(word) {
                seen.insert(word.to_string());
                tokens.push(word.to_string());
            }
        }
    }

    tokens
}

/// Rich dynamic multi-language workspace symbol scanner.
/// Scans the active buffer for variable assignments, function headers/bodies, and class declarations.
/// Also inspects sibling files in the same directory for external functions and symbols.
pub fn scan_workspace_symbols(
    buffer: &Buffer,
    path: Option<&Path>,
    active_word: &str,
) -> Vec<BufferSymbol> {
    let mut symbols = Vec::with_capacity(64);
    let mut seen_names = HashSet::new();

    let active_filename = path
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("buffer");

    let ext = path
        .and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // 1. Scan current buffer lines (up to 600 lines for maximum speed)
    let lines = buffer.lines();
    let total_lines = lines.len().min(600);

    for idx in 0..total_lines {
        let line = &lines[idx];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        let _line_num = idx + 1;

        // Try extracting high-level declarations depending on language patterns
        if let Some(sym) = parse_line_declaration(trimmed, &lines, idx, active_filename, &ext) {
            if sym.name.len() >= 2 && !seen_names.contains(&sym.name) {
                seen_names.insert(sym.name.clone());
                symbols.push(sym);
            }
        }
    }

    // Also collect any general identifier words from current buffer that weren't captured as declarations
    for (line_idx, line) in lines.iter().enumerate().take(total_lines) {
        let bytes = line.as_bytes();
        let mut start: Option<usize> = None;

        for (i, &b) in bytes.iter().enumerate() {
            let is_ident = b.is_ascii_alphanumeric() || b == b'_';
            match (start, is_ident) {
                (None, true) => {
                    if !b.is_ascii_digit() {
                        start = Some(i);
                    }
                }
                (Some(s), false) => {
                    let word = &line[s..i];
                    if word.len() >= 2 && word != active_word && !seen_names.contains(word) {
                        seen_names.insert(word.to_string());
                        symbols.push(BufferSymbol {
                            name: word.to_string(),
                            kind: CompletionKind::BufferWord,
                            preview: line.trim().to_string(),
                            detail: active_filename.to_string(),
                            documentation: Some(format!("{} • Ln {}", active_filename, line_idx + 1)),
                            source_file: active_filename.to_string(),
                            line_number: line_idx + 1,
                            import_hint: None,
                        });
                    }
                    start = None;
                }
                _ => {}
            }
        }

        if let Some(s) = start {
            let word = &line[s..];
            if word.len() >= 2 && word != active_word && !seen_names.contains(word) {
                seen_names.insert(word.to_string());
                symbols.push(BufferSymbol {
                    name: word.to_string(),
                    kind: CompletionKind::BufferWord,
                    preview: line.trim().to_string(),
                    detail: active_filename.to_string(),
                    documentation: Some(format!("{} • Ln {}", active_filename, line_idx + 1)),
                    source_file: active_filename.to_string(),
                    line_number: line_idx + 1,
                    import_hint: None,
                });
            }
        }
    }

    // 2. Scan sibling project files in the same directory (multi-file awareness)
    if let Some(file_path) = path {
        if let Some(parent_dir) = file_path.parent() {
            scan_sibling_directory_files(
                parent_dir,
                file_path,
                &ext,
                active_word,
                &mut seen_names,
                &mut symbols,
            );
        }
    }

    symbols
}

/// Parses a declaration from a line across languages (Python, Rust, TS/JS, Go, C/C++).
fn parse_line_declaration(
    trimmed: &str,
    lines: &[String],
    idx: usize,
    active_filename: &str,
    ext: &str,
) -> Option<BufferSymbol> {
    let line_num = idx + 1;

    // --- Python Declarations ---
    if ext == "py" || ext.is_empty() {
        if trimmed.starts_with("def ") {
            let rest = trimmed[4..].trim_start();
            if let Some(paren_pos) = rest.find('(') {
                let name = rest[..paren_pos].trim().to_string();
                let preview = extract_multiline_block(lines, idx, 4);
                return Some(BufferSymbol {
                    name,
                    kind: CompletionKind::Function,
                    preview,
                    detail: active_filename.to_string(),
                    documentation: Some(format!("{} • Ln {} (local function)", active_filename, line_num)),
                    source_file: active_filename.to_string(),
                    line_number: line_num,
                    import_hint: None,
                });
            }
        }

        if trimmed.starts_with("class ") {
            let rest = trimmed[6..].trim_start();
            let end_pos = rest.find(|c| c == '(' || c == ':').unwrap_or(rest.len());
            let name = rest[..end_pos].trim().to_string();
            let preview = extract_multiline_block(lines, idx, 3);
            return Some(BufferSymbol {
                name,
                kind: CompletionKind::Struct,
                preview,
                detail: active_filename.to_string(),
                documentation: Some(format!("{} • Ln {} (class definition)", active_filename, line_num)),
                source_file: active_filename.to_string(),
                line_number: line_num,
                import_hint: None,
            });
        }
    }

    // --- Rust Declarations ---
    if ext == "rs" {
        if trimmed.contains("fn ") {
            if let Some(fn_pos) = trimmed.find("fn ") {
                let rest = trimmed[fn_pos + 3..].trim_start();
                if let Some(paren_pos) = rest.find('(') {
                    let name = rest[..paren_pos].trim().to_string();
                    return Some(BufferSymbol {
                        name,
                        kind: CompletionKind::Function,
                        preview: trimmed.to_string(),
                        detail: active_filename.to_string(),
                        documentation: Some(format!("{} • Ln {} (function)", active_filename, line_num)),
                        source_file: active_filename.to_string(),
                        line_number: line_num,
                        import_hint: None,
                    });
                }
            }
        }

        for kw in &["struct ", "enum ", "trait ", "type "] {
            if let Some(pos) = trimmed.find(kw) {
                let rest = trimmed[pos + kw.len()..].trim_start();
                let end_pos = rest.find(|c: char| c.is_whitespace() || c == '{' || c == '<' || c == ';').unwrap_or(rest.len());
                let name = rest[..end_pos].trim().to_string();
                if !name.is_empty() {
                    return Some(BufferSymbol {
                        name,
                        kind: CompletionKind::Struct,
                        preview: trimmed.to_string(),
                        detail: active_filename.to_string(),
                        documentation: Some(format!("{} • Ln {} ({})", active_filename, line_num, kw.trim())),
                        source_file: active_filename.to_string(),
                        line_number: line_num,
                        import_hint: None,
                    });
                }
            }
        }
    }

    // --- JavaScript / TypeScript Declarations ---
    if ext == "js" || ext == "ts" || ext == "jsx" || ext == "tsx" {
        if trimmed.contains("function ") {
            if let Some(pos) = trimmed.find("function ") {
                let rest = trimmed[pos + 9..].trim_start();
                if let Some(paren_pos) = rest.find('(') {
                    let name = rest[..paren_pos].trim().to_string();
                    return Some(BufferSymbol {
                        name,
                        kind: CompletionKind::Function,
                        preview: trimmed.to_string(),
                        detail: active_filename.to_string(),
                        documentation: Some(format!("{} • Ln {} (function)", active_filename, line_num)),
                        source_file: active_filename.to_string(),
                        line_number: line_num,
                        import_hint: None,
                    });
                }
            }
        }

        for kw in &["class ", "interface ", "type "] {
            if let Some(pos) = trimmed.find(kw) {
                let rest = trimmed[pos + kw.len()..].trim_start();
                let end_pos = rest.find(|c: char| c.is_whitespace() || c == '{' || c == '<' || c == '=').unwrap_or(rest.len());
                let name = rest[..end_pos].trim().to_string();
                if !name.is_empty() {
                    return Some(BufferSymbol {
                        name,
                        kind: CompletionKind::Struct,
                        preview: trimmed.to_string(),
                        detail: active_filename.to_string(),
                        documentation: Some(format!("{} • Ln {} ({})", active_filename, line_num, kw.trim())),
                        source_file: active_filename.to_string(),
                        line_number: line_num,
                        import_hint: None,
                    });
                }
            }
        }
    }

    // --- Go Declarations ---
    if ext == "go" {
        if trimmed.starts_with("func ") {
            let rest = trimmed[5..].trim_start();
            let after_recv = if rest.starts_with('(') {
                if let Some(close_p) = rest.find(')') {
                    rest[close_p + 1..].trim_start()
                } else {
                    rest
                }
            } else {
                rest
            };
            if let Some(paren_pos) = after_recv.find('(') {
                let name = after_recv[..paren_pos].trim().to_string();
                return Some(BufferSymbol {
                    name,
                    kind: CompletionKind::Function,
                    preview: trimmed.to_string(),
                    detail: active_filename.to_string(),
                    documentation: Some(format!("{} • Ln {} (func)", active_filename, line_num)),
                    source_file: active_filename.to_string(),
                    line_number: line_num,
                    import_hint: None,
                });
            }
        }
    }

    // --- Universal Variable Assignment (e.g. `chai_varities = ["Green", "Red"]`, `const x = 10`, `let y = ...`) ---
    if let Some(eq_pos) = trimmed.find('=') {
        if eq_pos > 0 && !trimmed.starts_with("==") && !trimmed[eq_pos..].starts_with("==") {
            let before_eq = trimmed[..eq_pos].trim();
            // Clean modifiers like `const`, `let`, `var`, `let mut`, `pub const`
            let ident_candidate = before_eq
                .trim_start_matches("pub ")
                .trim_start_matches("let ")
                .trim_start_matches("mut ")
                .trim_start_matches("const ")
                .trim_start_matches("var ")
                .trim();

            if is_valid_ident(ident_candidate) {
                let preview = if trimmed.len() > 140 {
                    format!("{}…", &trimmed[..138])
                } else {
                    trimmed.to_string()
                };

                return Some(BufferSymbol {
                    name: ident_candidate.to_string(),
                    kind: CompletionKind::Variable,
                    preview,
                    detail: active_filename.to_string(),
                    documentation: Some(format!("{} • Ln {} (variable assignment)", active_filename, line_num)),
                    source_file: active_filename.to_string(),
                    line_number: line_num,
                    import_hint: None,
                });
            }
        }
    }

    // --- Go Walrus `:=` Assignment ---
    if let Some(walrus_pos) = trimmed.find(":=") {
        let before_walrus = trimmed[..walrus_pos].trim();
        if is_valid_ident(before_walrus) {
            return Some(BufferSymbol {
                name: before_walrus.to_string(),
                kind: CompletionKind::Variable,
                preview: trimmed.to_string(),
                detail: active_filename.to_string(),
                documentation: Some(format!("{} • Ln {} (assignment)", active_filename, line_num)),
                source_file: active_filename.to_string(),
                line_number: line_num,
                import_hint: None,
            });
        }
    }

    None
}

/// Inspects sibling files in the parent directory for multi-file symbols.
fn scan_sibling_directory_files(
    dir: &Path,
    current_file: &Path,
    ext: &str,
    active_word: &str,
    seen_names: &mut HashSet<String>,
    symbols: &mut Vec<BufferSymbol>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut scanned_count = 0;

    for entry in entries.flatten() {
        if scanned_count >= 12 {
            break;
        }

        let entry_path = entry.path();
        if entry_path == current_file || !entry_path.is_file() {
            continue;
        }

        // Only scan files with matching extension or common script extensions
        let entry_ext = entry_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !ext.is_empty() && entry_ext != ext {
            continue;
        }

        // Keep it fast: skip files larger than 120KB
        if let Ok(meta) = entry.metadata() {
            if meta.len() > 120_000 {
                continue;
            }
        }

        let sibling_filename = entry_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let stem = entry_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let content = match std::fs::read_to_string(&entry_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        scanned_count += 1;
        let file_lines: Vec<String> = content.lines().take(400).map(|s| s.to_string()).collect();

        for (idx, line) in file_lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
                continue;
            }

            if let Some(mut sym) = parse_line_declaration(trimmed, &file_lines, idx, &sibling_filename, &entry_ext) {
                if sym.name.len() >= 2 && sym.name != active_word && !seen_names.contains(&sym.name) {
                    seen_names.insert(sym.name.clone());

                    // Generate language-appropriate import hint
                    let import_hint = if entry_ext == "py" {
                        Some(format!("from {} import {}", stem, sym.name))
                    } else if entry_ext == "rs" {
                        Some(format!("use crate::{}::{};", stem, sym.name))
                    } else if entry_ext == "ts" || entry_ext == "js" {
                        Some(format!("import {{ {} }} from \"./{}\";", sym.name, stem))
                    } else {
                        None
                    };

                    sym.import_hint = import_hint.clone();
                    let doc_text = if let Some(ref imp) = import_hint {
                        format!("{} • Ln {}\n󰏪 {}", sibling_filename, idx + 1, imp)
                    } else {
                        format!("{} • Ln {}", sibling_filename, idx + 1)
                    };
                    sym.documentation = Some(doc_text);
                    sym.source_file = sibling_filename.clone();
                    sym.detail = sibling_filename.clone();

                    symbols.push(sym);
                }
            }
        }
    }
}

/// Extracts multi-line block (e.g. function body or class header + initial statements).
fn extract_multiline_block(lines: &[String], start_idx: usize, max_lines: usize) -> String {
    let mut block = Vec::with_capacity(max_lines);
    let end_idx = (start_idx + max_lines).min(lines.len());

    for i in start_idx..end_idx {
        block.push(lines[i].as_str());
    }

    block.join("\n")
}

fn is_valid_ident(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_python_declarations() {
        let text = r#"
chai_varities = ["Green", "Red", "White"]
chai = "hello"

def afunc(name):
    print(f"Hello, {name}")
    return 42

class User:
    def __init__(self, name):
        self.name = name
"#;
        let buf = Buffer::from_text(text);
        let path = Path::new("list.py");
        let syms = scan_workspace_symbols(&buf, Some(path), "");

        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"chai_varities"));
        assert!(names.contains(&"chai"));
        assert!(names.contains(&"afunc"));
        assert!(names.contains(&"User"));

        let chai_var = syms.iter().find(|s| s.name == "chai_varities").unwrap();
        assert_eq!(chai_var.kind, CompletionKind::Variable);
        assert_eq!(chai_var.preview, r#"chai_varities = ["Green", "Red", "White"]"#);
        assert_eq!(chai_var.source_file, "list.py");

        let func = syms.iter().find(|s| s.name == "afunc").unwrap();
        assert_eq!(func.kind, CompletionKind::Function);
        assert!(func.preview.contains("def afunc(name):"));
    }

    #[test]
    fn test_scan_rust_declarations() {
        let text = r#"
pub fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}

pub struct Config {
    pub port: u16,
}

let timeout_ms = 5000;
"#;
        let buf = Buffer::from_text(text);
        let path = Path::new("main.rs");
        let syms = scan_workspace_symbols(&buf, Some(path), "");

        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"calculate_sum"));
        assert!(names.contains(&"Config"));
        assert!(names.contains(&"timeout_ms"));
    }
}
