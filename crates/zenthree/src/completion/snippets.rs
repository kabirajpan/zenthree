use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SnippetTemplate {
    #[serde(alias = "prefix")]
    pub label: String,
    #[serde(default)]
    pub detail: String,
    pub body: String,
}

static SNIPPET_CACHE: RwLock<Option<HashMap<String, Vec<SnippetTemplate>>>> = RwLock::new(None);

/// Dynamically retrieves snippets for the file from ~/.local/share/zenthree/snippets/<lang>.json
/// Fully dynamic: No hardcoded snippet tables in the binary. Users can edit JSON directly.
pub fn get_snippets_for_path(path: Option<&Path>) -> Vec<SnippetTemplate> {
    let ext = path
        .and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext.is_empty() {
        return Vec::new();
    }

    let lang = match ext.as_str() {
        "py" => "python",
        "rs" => "rust",
        "js" | "jsx" | "mjs" | "cjs" => "javascript",
        "ts" | "tsx" | "mts" | "cts" => "typescript",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp",
        "java" => "java",
        "go" => "go",
        "html" | "htm" => "html",
        "css" | "scss" | "less" => "css",
        "json" | "jsonc" => "json",
        "yaml" | "yml" => "yaml",
        "sh" | "bash" | "zsh" => "bash",
        "lua" => "lua",
        other => other,
    };

    // Check memory cache
    if let Ok(read_guard) = SNIPPET_CACHE.read() {
        if let Some(ref map) = *read_guard {
            if let Some(cached) = map.get(lang) {
                return cached.clone();
            }
        }
    }

    // Load from disk: ~/.local/share/zenthree/snippets/<lang>.json
    let snippets = load_snippets_from_disk(lang);

    // Cache in memory
    if let Ok(mut write_guard) = SNIPPET_CACHE.write() {
        let map = write_guard.get_or_insert_with(HashMap::new);
        map.insert(lang.to_string(), snippets.clone());
    }

    snippets
}

fn load_snippets_from_disk(lang: &str) -> Vec<SnippetTemplate> {
    let Some(data_dir) = dirs::data_local_dir().or_else(dirs::data_dir) else {
        return Vec::new();
    };
    let snippet_file = data_dir.join("zenthree").join("snippets").join(format!("{}.json", lang));

    if let Ok(content) = std::fs::read_to_string(&snippet_file) {
        if let Ok(items) = serde_json::from_str::<Vec<SnippetTemplate>>(&content) {
            return items;
        }
    }

    Vec::new()
}

#[derive(Debug, Clone)]
pub struct ParsedSnippet {
    pub clean_text: String,
    pub placeholder_col_range: Option<(usize, usize)>,
    pub body_row_offset: Option<usize>,
    pub body_col: Option<usize>,
}

pub fn parse_snippet_template(template: &str) -> ParsedSnippet {
    if template.contains("${1:") || template.contains("$0") || template.contains("$1") {
        let mut clean = String::new();
        let mut placeholder_range = None;
        let mut body_row_offset = None;
        let mut body_col = None;

        let mut chars = template.chars().peekable();
        let mut current_row = 0;
        let mut current_col = 0;

        while let Some(ch) = chars.next() {
            if ch == '$' {
                if chars.peek() == Some(&'0') {
                    chars.next();
                    body_row_offset = Some(current_row);
                    body_col = Some(current_col);
                    continue;
                } else if chars.peek() == Some(&'{') {
                    let mut snippet_var = String::new();
                    chars.next(); // consume '{'
                    while let Some(c) = chars.next() {
                        if c == '}' {
                            break;
                        }
                        snippet_var.push(c);
                    }
                    if let Some(colon_pos) = snippet_var.find(':') {
                        let default_val = &snippet_var[colon_pos + 1..];
                        let start_col = current_col;
                        let end_col = current_col + default_val.chars().count();
                        if placeholder_range.is_none() {
                            placeholder_range = Some((start_col, end_col));
                        }
                        for c in default_val.chars() {
                            clean.push(c);
                            current_col += 1;
                        }
                    }
                    continue;
                }
            }

            clean.push(ch);
            if ch == '\n' {
                current_row += 1;
                current_col = 0;
            } else {
                current_col += 1;
            }
        }

        return ParsedSnippet {
            clean_text: clean,
            placeholder_col_range: placeholder_range,
            body_row_offset,
            body_col,
        };
    }

    let lines: Vec<&str> = template.lines().collect();
    let mut placeholder_col_range = None;
    let mut body_row_offset = None;
    let mut body_col = None;

    if !lines.is_empty() {
        let first_line = lines[0];
        if let Some(idx) = first_line.find("name") {
            placeholder_col_range = Some((idx, idx + 4));
        } else if let Some(idx) = first_line.find("Name") {
            placeholder_col_range = Some((idx, idx + 4));
        } else if let Some(idx) = first_line.find("func_name") {
            placeholder_col_range = Some((idx, idx + 9));
        } else if let Some(idx) = first_line.find("ClassName") {
            placeholder_col_range = Some((idx, idx + 9));
        } else if let Some(idx) = first_line.find("TypeName") {
            placeholder_col_range = Some((idx, idx + 8));
        } else if let Some(idx) = first_line.find("condition") {
            placeholder_col_range = Some((idx, idx + 9));
        }
    }

    if body_row_offset.is_none() && lines.len() == 1 {
        if let Some(quote_idx) = lines[0].find("\"\"") {
            body_row_offset = Some(0);
            body_col = Some(quote_idx + 1);
        } else if let Some(paren_idx) = lines[0].find("()") {
            body_row_offset = Some(0);
            body_col = Some(paren_idx + 1);
        }
    }

    if body_row_offset.is_none() {
        if lines.len() >= 3 {
            body_row_offset = Some(1);
            let second_line_indent = lines[1].chars().take_while(|c| c.is_whitespace()).count();
            body_col = Some(second_line_indent.max(4));
        } else if lines.len() == 2 && lines[1].trim().is_empty() {
            body_row_offset = Some(1);
            body_col = Some(4);
        }
    }

    ParsedSnippet {
        clean_text: template.to_string(),
        placeholder_col_range,
        body_row_offset,
        body_col,
    }
}
