pub mod buffer_indexer;
pub mod fuzzy;
pub mod keywords;
pub mod postfix;
pub mod snippets;
pub mod ui;

use std::path::Path;
use zenthra::Color;
use editor::buffer::{Buffer, Point};
use editor::editor::Editor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Method,
    Function,
    Variable,
    Field,
    Struct,
    Keyword,
    BufferWord,
    PostfixSnippet,
    Module,
    Property,
    Text,
}

impl CompletionKind {
    /// Returns (Nerd Font icon glyph, display color)
    pub fn icon_and_color(&self) -> (&'static str, Color) {
        match self {
            // Method / Function
            Self::Method | Self::Function => ("\u{f0871}", Color::rgb(0.70, 0.50, 0.95)),
            // Variable
            Self::Variable => ("\u{f0aef}", Color::rgb(0.40, 0.75, 0.95)),
            // Field / Property
            Self::Field | Self::Property => ("\u{f005}", Color::rgb(0.40, 0.75, 0.95)),
            // Struct / Class / Interface
            Self::Struct => ("\u{f13d2}", Color::rgb(0.35, 0.85, 0.55)),
            // Keyword
            Self::Keyword => ("\u{f0306}", Color::rgb(0.95, 0.65, 0.25)),
            // Buffer Token: [abc] in warm amber / orange matching LazyVim
            Self::BufferWord => ("abc", Color::rgb(0.95, 0.60, 0.25)),
            // Snippets: [T] in teal/cyan matching LazyVim nvim-cmp
            Self::PostfixSnippet => ("[T]", Color::rgb(0.35, 0.85, 0.70)),
            // Module / Namespace
            Self::Module => ("\u{f0115}", Color::rgb(0.95, 0.75, 0.35)),
            // Fallback Text
            Self::Text => ("abc", Color::rgb(0.65, 0.65, 0.70)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
    pub score: i32,
    pub is_postfix: bool,
    pub postfix_template: Option<String>,
}

#[derive(Default, Clone)]
pub struct CompletionState {
    pub active: bool,
    pub items: Vec<CompletionItem>,
    pub selected_idx: usize,
    pub prefix: String,
    pub cursor_point: Point,
    pub prev_expr: Option<String>,
    pub is_after_dot: bool,
}

impl CompletionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn dismiss(&mut self) {
        self.active = false;
        self.items.clear();
        self.prefix.clear();
        self.selected_idx = 0;
        self.prev_expr = None;
        self.is_after_dot = false;
    }

    /// Tier 1: Instant local buffer and keyword completion generator (0ms latency, runs synchronously on frame).
    pub fn populate_instant(
        &mut self,
        buffer: &Buffer,
        path: Option<&Path>,
        cursor: Point,
    ) {
        let (prefix, _prefix_col, is_after_dot, prev_expr, is_in_string_or_comment) = extract_prefix(buffer, cursor);
        self.prefix = prefix.clone();
        self.cursor_point = cursor;
        self.prev_expr = prev_expr.clone();
        self.is_after_dot = is_after_dot;

        // Auto-trigger rules: trigger if preceded by accessor (., ->, ::), or if prefix has at least 1 character!
        if !is_after_dot && prefix.is_empty() {
            self.dismiss();
            return;
        }

        // Inside string literals or comments: suppress code completions (matching VS Code, Zed, & Neovim)
        if is_in_string_or_comment {
            self.dismiss();
            return;
        }

        let mut candidates = Vec::with_capacity(64);

        // 1. Postfix snippets (JetBrains & LazyVim style snippets)
        if is_after_dot {
            if let Some(ref expr) = prev_expr {
                for template in postfix::get_postfix_templates(path) {
                    if let Some(score) = fuzzy::fuzzy_match_score(&prefix, template.label) {
                        let expanded = template.template.replace("{expr}", expr);
                        candidates.push(CompletionItem {
                            label: format!("{}~", template.label),
                            kind: CompletionKind::PostfixSnippet,
                            detail: Some(template.detail.to_string()),
                            documentation: Some(template.detail.to_string()),
                            insert_text: Some(expanded.clone()),
                            score: score + 120, // High ranking for postfix expressions
                            is_postfix: true,
                            postfix_template: Some(expanded),
                        });
                    }
                }
            }
        }

        // 2. Language Statement Snippets (LazyVim / VS Code style: if~, ifelse~, for~, etc.)
        if !is_after_dot {
            for snip in snippets::get_snippets_for_path(path) {
                if let Some(score) = fuzzy::fuzzy_match_score(&prefix, snip.label) {
                    candidates.push(CompletionItem {
                        label: format!("{}~", snip.label),
                        kind: CompletionKind::PostfixSnippet,
                        detail: Some("snippet".to_string()),
                        documentation: Some(snip.detail.to_string()),
                        insert_text: Some(snip.body.to_string()),
                        score: score + 110,
                        is_postfix: false,
                        postfix_template: Some(snip.body.to_string()),
                    });
                }
            }
        }

        // 3. Built-in language keywords (Tier 1) - only when not directly completing method after dot
        if !is_after_dot {
            for kw in keywords::get_keywords_for_path(path) {
                if let Some(score) = fuzzy::fuzzy_match_score(&prefix, kw.label) {
                    candidates.push(CompletionItem {
                        label: kw.label.to_string(),
                        kind: CompletionKind::Keyword,
                        detail: Some(kw.detail.to_string()),
                        documentation: Some(kw.detail.to_string()),
                        insert_text: Some(kw.label.to_string()),
                        score: score + 50,
                        is_postfix: false,
                        postfix_template: None,
                    });
                }
            }
        }

        // 3. Instant workspace & buffer symbols (Zed-style 0ms latency, multi-language dynamic)
        let symbols = buffer_indexer::scan_workspace_symbols(buffer, path, &prefix);
        for sym in symbols {
            if let Some(score) = fuzzy::fuzzy_match_score(&prefix, &sym.name) {
                candidates.push(CompletionItem {
                    label: sym.name.clone(),
                    kind: sym.kind,
                    detail: Some(sym.detail),
                    documentation: sym.documentation,
                    insert_text: Some(sym.name),
                    score: score + 45,
                    is_postfix: false,
                    postfix_template: Some(sym.preview),
                });
            }
        }

        // Sort descending by relevance score
        candidates.sort_by(|a, b| b.score.cmp(&a.score));

        // Deduplicate by label
        candidates.dedup_by(|a, b| a.label == b.label);

        if !candidates.is_empty() {
            self.items = candidates;
            self.selected_idx = 0;
            self.active = true;
        } else {
            self.dismiss();
        }
    }

    /// Tier 2: Merge async LSP completion results into the active list.
    pub fn merge_lsp_items(&mut self, lsp_items: &[crate::lsp::protocol::CompletionItem]) {
        if lsp_items.is_empty() {
            return;
        }

        // Map existing buffer items by label so we never lose their code preview or line documentation
        let mut existing_map: std::collections::HashMap<String, CompletionItem> = std::collections::HashMap::new();
        for item in self.items.drain(..) {
            existing_map.insert(item.label.clone(), item);
        }

        let mut enriched = Vec::with_capacity(lsp_items.len() + existing_map.len());

        for item in lsp_items {
            let label = &item.label;
            let score = fuzzy::fuzzy_match_score(&self.prefix, label).unwrap_or(300) + 500; // LSP items get +500 priority boost
            let kind = match item.kind {
                Some(k) => match k {
                    crate::lsp::protocol::CompletionItemKind::Method => CompletionKind::Method,
                    crate::lsp::protocol::CompletionItemKind::Function => CompletionKind::Function,
                    crate::lsp::protocol::CompletionItemKind::Variable => CompletionKind::Variable,
                    crate::lsp::protocol::CompletionItemKind::Field => CompletionKind::Field,
                    crate::lsp::protocol::CompletionItemKind::Class
                    | crate::lsp::protocol::CompletionItemKind::Struct
                    | crate::lsp::protocol::CompletionItemKind::Interface => CompletionKind::Struct,
                    crate::lsp::protocol::CompletionItemKind::Keyword => CompletionKind::Keyword,
                    crate::lsp::protocol::CompletionItemKind::Module => CompletionKind::Module,
                    crate::lsp::protocol::CompletionItemKind::Property => CompletionKind::Property,
                    crate::lsp::protocol::CompletionItemKind::Snippet => CompletionKind::PostfixSnippet,
                    _ => CompletionKind::Text,
                },
                None => CompletionKind::Method,
            };

            let doc_text = item.documentation.as_ref().and_then(|doc| {
                if let Some(s) = doc.as_str() {
                    Some(s.to_string())
                } else if let Some(obj) = doc.as_object() {
                    obj.get("value").and_then(|v| v.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            });

            let display_label = if kind == CompletionKind::PostfixSnippet && !label.ends_with('~') {
                format!("{}~", label)
            } else {
                label.clone()
            };

            // Check if we already indexed this symbol locally in the buffer
            let existing_item = existing_map.remove(label);
            let preserved_preview = existing_item.as_ref().and_then(|ex| ex.postfix_template.clone());
            let final_doc = doc_text.or_else(|| existing_item.as_ref().and_then(|ex| ex.documentation.clone()));
            let final_detail = item.detail.clone().or_else(|| existing_item.as_ref().and_then(|ex| ex.detail.clone()));
            let is_postfix = existing_item.as_ref().map_or(false, |ex| ex.is_postfix);

            enriched.push(CompletionItem {
                label: display_label,
                kind,
                detail: final_detail,
                documentation: final_doc,
                insert_text: item.insert_text.clone().or_else(|| Some(label.clone())),
                score,
                is_postfix,
                postfix_template: preserved_preview,
            });
        }

        // Retain any existing buffer/keyword items that aren't shadowed by LSP
        for (_label, existing) in existing_map {
            enriched.push(existing);
        }

        enriched.sort_by(|a, b| b.score.cmp(&a.score));
        enriched.dedup_by(|a, b| a.label == b.label);

        if !enriched.is_empty() {
            self.items = enriched;
            self.active = true;
        }
    }
}

/// Extracts the active identifier prefix and preceding expression before the cursor.
/// Returns (prefix, word_start_col, is_after_dot, prev_expr, is_in_string_or_comment)
pub fn extract_prefix(buffer: &Buffer, cursor: Point) -> (String, usize, bool, Option<String>, bool) {
    let line_str = match buffer.line(cursor.row) {
        Some(s) => s,
        None => return (String::new(), 0, false, None, false),
    };

    let col = cursor.column.min(line_str.len());
    let before_cursor = &line_str[..col];

    // Detect if cursor is inside an unclosed string literal or comment
    let mut in_quote: Option<char> = None;
    let mut is_in_comment = false;
    let mut prev_char = ' ';

    for ch in before_cursor.chars() {
        if in_quote.is_none() {
            if ch == '"' || ch == '\'' {
                in_quote = Some(ch);
            } else if (prev_char == '/' && ch == '/') || ch == '#' {
                is_in_comment = true;
                break;
            }
        } else if in_quote == Some(ch) && prev_char != '\\' {
            in_quote = None;
        }
        prev_char = ch;
    }

    let is_in_string_or_comment = in_quote.is_some() || is_in_comment;

    // Find the word prefix right before cursor
    let chars: Vec<char> = before_cursor.chars().collect();
    let mut word_start = chars.len();

    while word_start > 0 {
        let prev = chars[word_start - 1];
        if prev.is_ascii_alphanumeric() || prev == '_' {
            word_start -= 1;
        } else {
            break;
        }
    }

    let prefix: String = chars[word_start..].iter().collect();

    // Check if directly preceded by an accessor: dot '.', arrow '->', or double colon '::'
    let is_after_dot = if word_start > 0 {
        if chars[word_start - 1] == '.' {
            true
        } else if word_start >= 2 && chars[word_start - 1] == '>' && chars[word_start - 2] == '-' {
            true
        } else if word_start >= 2 && chars[word_start - 1] == ':' && chars[word_start - 2] == ':' {
            true
        } else {
            false
        }
    } else {
        false
    };

    // Extract previous expression before dot (for postfix templates like `my_result.match`)
    let prev_expr = if is_after_dot && word_start >= 1 {
        let before_dot = &chars[..word_start - 1];
        let mut expr_start = before_dot.len();
        while expr_start > 0 {
            let c = before_dot[expr_start - 1];
            if c.is_ascii_alphanumeric() || c == '_' || c == ')' || c == ']' {
                expr_start -= 1;
            } else {
                break;
            }
        }
        let expr: String = before_dot[expr_start..].iter().collect();
        if !expr.trim().is_empty() {
            Some(expr.trim().to_string())
        } else {
            None
        }
    } else {
        None
    };

    (prefix, word_start, is_after_dot, prev_expr, is_in_string_or_comment)
}

/// Cleanly commits a completion item into the editor by removing the typed prefix and inserting the suggestion.
pub fn commit_completion(
    editor: &mut Editor,
    item: &CompletionItem,
    prefix: &str,
    prev_expr: Option<&str>,
    is_postfix: bool,
    path: Option<&Path>,
    indent_calc: Option<&dyn editor::editor::IndentProvider>,
) {
    if is_postfix {
        if let Some(ref template) = item.postfix_template {
            let expr_str = prev_expr.unwrap_or("");
            let total_chars_to_remove = expr_str.chars().count() + 1 + prefix.chars().count(); // expr + '.' + prefix
            for _ in 0..total_chars_to_remove {
                editor.backspace();
            }
            editor.insert(template, path, indent_calc);
            return;
        }
    }

    // Normal clean range replacement: backspace the exact prefix typed so far
    let prefix_char_count = prefix.chars().count();
    for _ in 0..prefix_char_count {
        editor.backspace();
    }

    let text_to_insert = item.insert_text.as_deref().unwrap_or(&item.label);
    editor.insert(text_to_insert, path, indent_calc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_prefix_identifier() {
        let buf = Buffer::from_text("let my_variable_name = 42;");
        let (prefix, start_col, is_after_dot, prev_expr, is_in_string_or_comment) = extract_prefix(&buf, Point::new(0, 16));
        assert_eq!(prefix, "my_variable_");
        assert_eq!(start_col, 4);
        assert!(!is_after_dot);
        assert_eq!(prev_expr, None);
        assert!(!is_in_string_or_comment);
    }

    #[test]
    fn test_extract_prefix_after_dot() {
        let buf = Buffer::from_text("let res = user_record.to_st");
        let (prefix, _, is_after_dot, prev_expr, is_in_string_or_comment) = extract_prefix(&buf, Point::new(0, 28));
        assert_eq!(prefix, "to_st");
        assert!(is_after_dot);
        assert_eq!(prev_expr, Some("user_record".to_string()));
        assert!(!is_in_string_or_comment);
    }

    #[test]
    fn test_populate_instant_keywords_and_buffer_tokens() {
        let mut state = CompletionState::new();
        let buf = Buffer::from_text("fn calculate_sum() {\n    let total_cost = 10;\n    tot\n}");
        // Cursor at line 2 right after "tot" (column 7)
        state.populate_instant(&buf, Some(Path::new("src/main.rs")), Point::new(2, 7));
        assert!(state.active);
        assert!(!state.items.is_empty());

        // Should include total_cost from buffer tokens
        let has_total_cost = state.items.iter().any(|item| item.label == "total_cost");
        assert!(has_total_cost);
    }

    #[test]
    fn test_populate_instant_postfix_templates() {
        let mut state = CompletionState::new();
        let buf = Buffer::from_text("let x = my_value.mat");
        state.populate_instant(&buf, Some(Path::new("test.rs")), Point::new(0, 20));
        assert!(state.active);

        let match_snippet = state.items.iter().find(|i| i.label.starts_with("match"));
        assert!(match_snippet.is_some());
        let item = match_snippet.unwrap();
        assert!(item.is_postfix);
        assert!(item.insert_text.as_ref().unwrap().contains("match my_value {"));
    }

    #[test]
    fn test_commit_completion_no_prefix_duplication() {
        let mut editor = Editor::new(Buffer::new());
        editor.insert("user.to_", None, None);

        let item = CompletionItem {
            label: "to_string".to_string(),
            kind: CompletionKind::Method,
            detail: Some("fn to_string(&self) -> String".to_string()),
            documentation: None,
            insert_text: Some("to_string()".to_string()),
            score: 950,
            is_postfix: false,
            postfix_template: None,
        };

        commit_completion(&mut editor, &item, "to_", None, false, None, None);
        assert_eq!(editor.buffer.to_string(), "user.to_string()");
    }

    #[test]
    fn test_commit_completion_postfix_expansion() {
        let mut editor = Editor::new(Buffer::new());
        editor.insert("my_result.mat", None, None);

        let item = CompletionItem {
            label: "match~".to_string(),
            kind: CompletionKind::PostfixSnippet,
            detail: Some("match expression { ... }".to_string()),
            documentation: Some("match expression { ... }".to_string()),
            insert_text: Some("match my_result {\n    _ => {}\n}".to_string()),
            score: 900,
            is_postfix: true,
            postfix_template: Some("match my_result {\n    _ => {}\n}".to_string()),
        };

        commit_completion(&mut editor, &item, "mat", Some("my_result"), true, None, None);
        assert_eq!(editor.buffer.to_string(), "match my_result {\n    _ => {}\n}");
    }

    #[test]
    fn test_merge_lsp_items_prioritizes_lsp() {
        let mut state = CompletionState::new();
        let buf = Buffer::from_text("let x = user_val.to");
        state.populate_instant(&buf, Some(Path::new("src/main.rs")), Point::new(0, 19));

        let lsp_items = vec![
            crate::lsp::protocol::CompletionItem {
                label: "to_owned".to_string(),
                kind: Some(crate::lsp::protocol::CompletionItemKind::Method),
                detail: Some("fn to_owned(&self) -> T".to_string()),
                documentation: None,
                insert_text: Some("to_owned()".to_string()),
            }
        ];

        state.merge_lsp_items(&lsp_items);
        assert!(state.active);
        assert_eq!(state.items[0].label, "to_owned");
        assert_eq!(state.items[0].kind, CompletionKind::Method);
    }

    #[test]
    fn test_populate_instant_single_character_triggers_realtime() {
        let mut state = CompletionState::new();
        let buf = Buffer::from_text("f");
        // User typed a single letter 'f'
        state.populate_instant(&buf, Some(Path::new("src/main.rs")), Point::new(0, 1));
        assert!(state.active);
        assert!(!state.items.is_empty());
        // Should contain keywords like 'fn' or 'for'
        assert!(state.items.iter().any(|i| i.label == "fn" || i.label == "for"));
    }

    #[test]
    fn test_extract_prefix_double_colon() {
        let buf = Buffer::from_text("std::io::wr");
        let (prefix, _, is_after_dot, _prev_expr, is_in_string_or_comment) = extract_prefix(&buf, Point::new(0, 11));
        assert_eq!(prefix, "wr");
        assert!(is_after_dot);
        assert!(!is_in_string_or_comment);
    }

    #[test]
    fn test_inside_string_dismisses_completion() {
        let mut state = CompletionState::new();
        let buf = Buffer::from_text("println!(\"h\")");
        // Cursor at column 11 (right after the 'h' inside the quotes)
        state.populate_instant(&buf, Some(Path::new("a.rs")), Point::new(0, 11));
        assert!(!state.active);
        assert!(state.items.is_empty());
    }

    #[test]
    fn test_populate_instant_statement_snippets_python() {
        let mut state = CompletionState::new();
        let buf = Buffer::from_text("if");
        state.populate_instant(&buf, Some(Path::new("list.py")), Point::new(0, 2));
        assert!(state.active);
        let labels: Vec<&str> = state.items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"if~"));
        assert!(labels.contains(&"ifelse~"));
        assert!(labels.contains(&"ifmain~"));

        let ifelse_item = state.items.iter().find(|i| i.label == "ifelse~").unwrap();
        assert!(ifelse_item.postfix_template.as_ref().unwrap().contains("else:"));
    }
}

