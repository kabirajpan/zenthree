use crate::languages::LanguageRegistry;
use std::path::Path;
use tree_sitter::{Query, QueryCursor};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Struct,
    Interface,
    Enum,
    Type,
    Constant,
    Variable,
    Module,
    Macro,
    Other(String),
}

impl SymbolKind {
    pub fn from_capture_name(name: &str) -> Self {
        if name.starts_with("definition.function") {
            SymbolKind::Function
        } else if name.starts_with("definition.method") {
            SymbolKind::Method
        } else if name.starts_with("definition.class") {
            SymbolKind::Class
        } else if name.starts_with("definition.struct") {
            SymbolKind::Struct
        } else if name.starts_with("definition.interface") || name.starts_with("definition.trait") {
            SymbolKind::Interface
        } else if name.starts_with("definition.enum") {
            SymbolKind::Enum
        } else if name.starts_with("definition.type") {
            SymbolKind::Type
        } else if name.starts_with("definition.constant") {
            SymbolKind::Constant
        } else if name.starts_with("definition.var") {
            SymbolKind::Variable
        } else if name.starts_with("definition.module") {
            SymbolKind::Module
        } else if name.starts_with("definition.macro") {
            SymbolKind::Macro
        } else {
            SymbolKind::Other(name.to_string())
        }
    }
}

#[derive(Debug, Clone)]
pub struct AstSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line_number: usize,
    pub start_byte: usize,
    pub end_byte: usize,
    pub preview: String,
}

#[derive(Clone)]
pub struct SymbolExtractor {
    registry: LanguageRegistry,
}

impl SymbolExtractor {
    pub fn new() -> Self {
        Self {
            registry: LanguageRegistry::new(),
        }
    }

    /// Extract document symbols using Helix tags.scm query and Tree-sitter.
    /// Returns None if no tags query is downloaded yet.
    pub fn extract_symbols(&self, code: &str, path: Option<&Path>) -> Option<Vec<AstSymbol>> {
        let path = path?;
        let config = self.registry.detect_language(path)?;
        let query_str = config.resolved_tags_query()?;
        if query_str.trim().is_empty() {
            return None;
        }

        let query = Query::new(&config.language, &query_str).ok()?;
        let mut parser = self.registry.create_parser(config);
        let tree = parser.parse(code, None)?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), code.as_bytes());

        let name_idx = query.capture_index_for_name("name");
        let lines: Vec<&str> = code.lines().collect();
        let mut symbols = Vec::new();

        for m in matches {
            let mut matched_name: Option<(String, usize, usize, usize)> = None;
            let mut symbol_kind = SymbolKind::Other("unknown".to_string());

            for capture in m.captures {
                let capture_name = &query.capture_names()[capture.index as usize];
                if Some(capture.index) == name_idx {
                    let node = capture.node;
                    let text = node.utf8_text(code.as_bytes()).unwrap_or("");
                    if !text.is_empty() {
                        let row = node.start_position().row;
                        matched_name = Some((text.to_string(), row, node.start_byte(), node.end_byte()));
                    }
                } else if capture_name.starts_with("definition.") {
                    symbol_kind = SymbolKind::from_capture_name(capture_name);
                }
            }

            if let Some((name, row, start_byte, end_byte)) = matched_name {
                let preview = lines.get(row).unwrap_or(&"").trim().to_string();
                symbols.push(AstSymbol {
                    name,
                    kind: symbol_kind,
                    line_number: row + 1,
                    start_byte,
                    end_byte,
                    preview,
                });
            }
        }

        Some(symbols)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_kind_mapping() {
        assert_eq!(SymbolKind::from_capture_name("definition.function"), SymbolKind::Function);
        assert_eq!(SymbolKind::from_capture_name("definition.method"), SymbolKind::Method);
        assert_eq!(SymbolKind::from_capture_name("definition.class"), SymbolKind::Class);
        assert_eq!(SymbolKind::from_capture_name("definition.struct"), SymbolKind::Struct);
        assert_eq!(SymbolKind::from_capture_name("definition.interface"), SymbolKind::Interface);
        assert_eq!(SymbolKind::from_capture_name("definition.enum"), SymbolKind::Enum);
        assert_eq!(SymbolKind::from_capture_name("definition.constant"), SymbolKind::Constant);
    }
}
