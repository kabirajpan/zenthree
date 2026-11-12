use std::path::Path;
use tree_sitter::{Language, Parser};

use tree_sitter_bash::language as tree_sitter_bash_lang;
use tree_sitter_c::language as tree_sitter_c_lang;
use tree_sitter_cpp::language as tree_sitter_cpp_lang;
use tree_sitter_css::language as tree_sitter_css_lang;
use tree_sitter_go::language as tree_sitter_go_lang;
use tree_sitter_html::language as tree_sitter_html_lang;
use tree_sitter_java::language as tree_sitter_java_lang;
use tree_sitter_javascript::language as tree_sitter_javascript_lang;
use tree_sitter_json::language as tree_sitter_json_lang;
use tree_sitter_md::language as tree_sitter_markdown_lang;
use tree_sitter_python::language as tree_sitter_python_lang;
use tree_sitter_rust::language as tree_sitter_rust_lang;
use tree_sitter_typescript::{
    language_tsx as tree_sitter_tsx_lang, language_typescript as tree_sitter_typescript_lang,
};
use tree_sitter_yaml::language as tree_sitter_yaml_lang;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LanguageId {
    Rust,
    JavaScript,
    Python,
    TypeScript,
    Tsx,
    Html,
    Css,
    Json,
    Yaml,
    Go,
    C,
    Cpp,
    Java,
    Markdown,
    Bash,
    Unknown,
}

#[derive(Clone)]
pub struct LanguageConfig {
    pub id: LanguageId,
    pub name: &'static str,
    pub language: Language,
    pub extensions: &'static [&'static str],
    pub indent_query: &'static str,
    pub highlight_query: &'static str,
    pub line_comment: &'static str,
}

impl LanguageConfig {
    pub fn rust() -> Self {
        Self {
            id: LanguageId::Rust,
            name: "Rust",
            language: tree_sitter_rust_lang(),
            extensions: &["rs"],
            indent_query: "",
            highlight_query: "",
            line_comment: "//",
        }
    }

    pub fn javascript() -> Self {
        Self {
            id: LanguageId::JavaScript,
            name: "JavaScript",
            language: tree_sitter_javascript_lang(),
            extensions: &["js", "jsx", "mjs", "cjs"],
            indent_query: "",
            highlight_query: "",
            line_comment: "//",
        }
    }

    pub fn python() -> Self {
        Self {
            id: LanguageId::Python,
            name: "Python",
            language: tree_sitter_python_lang(),
            extensions: &["py"],
            indent_query: "",
            highlight_query: "",
            line_comment: "#",
        }
    }

    pub fn typescript() -> Self {
        Self {
            id: LanguageId::TypeScript,
            name: "TypeScript",
            language: tree_sitter_typescript_lang(),
            extensions: &["ts", "mts", "cts"],
            indent_query: "",
            highlight_query: "",
            line_comment: "//",
        }
    }

    pub fn tsx() -> Self {
        Self {
            id: LanguageId::Tsx,
            name: "TSX",
            language: tree_sitter_tsx_lang(),
            extensions: &["tsx"],
            indent_query: "",
            highlight_query: "",
            line_comment: "//",
        }
    }

    pub fn html() -> Self {
        Self {
            id: LanguageId::Html,
            name: "HTML",
            language: tree_sitter_html_lang(),
            extensions: &["html", "htm"],
            indent_query: "",
            highlight_query: "",
            line_comment: "<!--",
        }
    }

    pub fn css() -> Self {
        Self {
            id: LanguageId::Css,
            name: "CSS",
            language: tree_sitter_css_lang(),
            extensions: &["css"],
            indent_query: "",
            highlight_query: "",
            line_comment: "/*",
        }
    }

    pub fn json() -> Self {
        Self {
            id: LanguageId::Json,
            name: "JSON",
            language: tree_sitter_json_lang(),
            extensions: &["json", "jsonc"],
            indent_query: "",
            highlight_query: "",
            line_comment: "",
        }
    }

    pub fn yaml() -> Self {
        Self {
            id: LanguageId::Yaml,
            name: "YAML",
            language: tree_sitter_yaml_lang(),
            extensions: &["yaml", "yml"],
            indent_query: "",
            highlight_query: "",
            line_comment: "#",
        }
    }

    pub fn go() -> Self {
        Self {
            id: LanguageId::Go,
            name: "Go",
            language: tree_sitter_go_lang(),
            extensions: &["go"],
            indent_query: "",
            highlight_query: "",
            line_comment: "//",
        }
    }

    pub fn c() -> Self {
        Self {
            id: LanguageId::C,
            name: "C",
            language: tree_sitter_c_lang(),
            extensions: &["c", "h"],
            indent_query: "",
            highlight_query: "",
            line_comment: "//",
        }
    }

    pub fn cpp() -> Self {
        Self {
            id: LanguageId::Cpp,
            name: "C++",
            language: tree_sitter_cpp_lang(),
            extensions: &["cc", "cpp", "cxx", "hh", "hpp", "hxx"],
            indent_query: "",
            highlight_query: "",
            line_comment: "//",
        }
    }

    pub fn java() -> Self {
        Self {
            id: LanguageId::Java,
            name: "Java",
            language: tree_sitter_java_lang(),
            extensions: &["java"],
            indent_query: "",
            highlight_query: "",
            line_comment: "//",
        }
    }

    pub fn markdown() -> Self {
        Self {
            id: LanguageId::Markdown,
            name: "Markdown",
            language: tree_sitter_markdown_lang(),
            extensions: &["md", "markdown"],
            indent_query: "",
            highlight_query: "",
            line_comment: "",
        }
    }

    pub fn bash() -> Self {
        Self {
            id: LanguageId::Bash,
            name: "Bash",
            language: tree_sitter_bash_lang(),
            extensions: &["sh", "bash", "zsh"],
            indent_query: "",
            highlight_query: "",
            line_comment: "#",
        }
    }

    /// Dynamically loads any Tree-sitter query (highlights.scm, indents.scm, tags.scm, etc.)
    /// from the user's downloaded query directory (~/.local/share/zenthree/queries/<lang>/<filename>).
    /// Resolves Helix "; inherits: <parent1>, <parent2>" directives recursively in memory.
    pub fn resolved_query(&self, filename: &str) -> Option<String> {
        let lang_dir = match self.id {
            LanguageId::C => "c",
            LanguageId::Cpp => "cpp",
            LanguageId::Java => "java",
            LanguageId::Rust => "rust",
            LanguageId::Python => "python",
            LanguageId::Go => "go",
            LanguageId::JavaScript => "javascript",
            LanguageId::TypeScript => "typescript",
            LanguageId::Tsx => "tsx",
            LanguageId::Html => "html",
            LanguageId::Css => "css",
            LanguageId::Json => "json",
            LanguageId::Yaml => "yaml",
            LanguageId::Bash => "bash",
            LanguageId::Markdown => "markdown",
            LanguageId::Unknown => return None,
        };

        let data_dir = dirs::data_local_dir().or_else(dirs::data_dir)?;
        let queries_root = data_dir.join("zenthree").join("queries");
        let custom_path = queries_root.join(lang_dir).join(filename);

        if !custom_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&custom_path).ok()?;
        if content.trim().is_empty() {
            return None;
        }

        // Check for Helix style "; inherits: <parent1>, <parent2>" directive
        let mut prefix = String::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("; inherits:") || trimmed.starts_with(";inherits:") {
                if let Some(parents_str) = trimmed.split(':').nth(1) {
                    for parent_raw in parents_str.split(',') {
                        let parent = parent_raw.trim();
                        if !parent.is_empty() {
                            let parent_path = queries_root.join(parent).join(filename);
                            if let Ok(parent_content) = std::fs::read_to_string(&parent_path) {
                                prefix.push_str(&parent_content);
                                prefix.push_str("\n\n");
                            }
                        }
                    }
                }
            }
        }

        let mut raw_combined = if !prefix.is_empty() {
            format!("{}{}", prefix, content)
        } else {
            content
        };

        // Sanitize language-specific grammar version compatibility in memory
        if self.id == LanguageId::Cpp {
            raw_combined = raw_combined
                .replace("\"^^\"", "; \"^^\"")
                .replace("(splice_specifier", "; (splice_specifier")
                .replace("\"virtual\"", "(virtual)");
        }

        Some(raw_combined)
    }

    /// Dynamically loads the highlight query from ~/.local/share/zenthree/queries/<lang>/highlights.scm
    pub fn resolved_highlight_query(&self) -> Option<String> {
        self.resolved_query("highlights.scm")
    }

    /// Dynamically loads the indent query from ~/.local/share/zenthree/queries/<lang>/indents.scm
    pub fn resolved_indent_query(&self) -> Option<String> {
        self.resolved_query("indents.scm")
    }

    /// Dynamically loads the symbol tags query from ~/.local/share/zenthree/queries/<lang>/tags.scm
    pub fn resolved_tags_query(&self) -> Option<String> {
        self.resolved_query("tags.scm")
    }

    /// Dynamically loads the textobjects query from ~/.local/share/zenthree/queries/<lang>/textobjects.scm
    pub fn resolved_textobjects_query(&self) -> Option<String> {
        self.resolved_query("textobjects.scm")
    }

    /// Dynamically loads the injections query from ~/.local/share/zenthree/queries/<lang>/injections.scm
    pub fn resolved_injections_query(&self) -> Option<String> {
        self.resolved_query("injections.scm")
    }

    /// Dynamically extracts all keyword tokens defined in the Tree-sitter highlight query.
    pub fn dynamic_keywords(&self) -> Vec<String> {
        if let Some(query_str) = self.resolved_highlight_query() {
            extract_keywords_from_query(&query_str)
        } else {
            Vec::new()
        }
    }
}

/// Dynamically parse keywords associated with `@keyword` in a Tree-sitter SCM query.
pub fn extract_keywords_from_query(query: &str) -> Vec<String> {
    let mut keywords = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for part in query.split("@keyword") {
        let trimmed = part.trim_end();
        if trimmed.ends_with(']') {
            if let Some(open_idx) = trimmed.rfind('[') {
                let inside = &trimmed[open_idx + 1..trimmed.len() - 1];
                for token in extract_quoted_strings(inside) {
                    if seen.insert(token.clone()) {
                        keywords.push(token);
                    }
                }
            }
        } else if let Some(last_quote_end) = trimmed.rfind('"') {
            if let Some(last_quote_start) = trimmed[..last_quote_end].rfind('"') {
                let token = &trimmed[last_quote_start + 1..last_quote_end];
                if !token.is_empty() && token.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                    if seen.insert(token.to_string()) {
                        keywords.push(token.to_string());
                    }
                }
            }
        }
    }

    keywords
}

fn extract_quoted_strings(s: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut in_quote = false;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        if c == '"' {
            if in_quote {
                let token = &s[start..i];
                if !token.is_empty() && token.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                    results.push(token.to_string());
                }
                in_quote = false;
            } else {
                in_quote = true;
                start = i + 1;
            }
        }
    }

    results
}

#[derive(Clone)]
pub struct LanguageRegistry {
    languages: Vec<LanguageConfig>,
}

impl LanguageRegistry {
    pub fn new() -> Self {
        Self {
            languages: vec![
                LanguageConfig::rust(),
                LanguageConfig::javascript(),
                LanguageConfig::python(),
                LanguageConfig::typescript(),
                LanguageConfig::tsx(),
                LanguageConfig::html(),
                LanguageConfig::css(),
                LanguageConfig::json(),
                LanguageConfig::yaml(),
                LanguageConfig::go(),
                LanguageConfig::c(),
                LanguageConfig::cpp(),
                LanguageConfig::java(),
                LanguageConfig::markdown(),
                LanguageConfig::bash(),
            ],
        }
    }

    pub fn all(&self) -> &[LanguageConfig] {
        &self.languages
    }

    pub fn detect_language(&self, path: &Path) -> Option<&LanguageConfig> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        self.languages
            .iter()
            .find(|lang| lang.extensions.contains(&extension.as_str()))
    }

    pub fn get_language(&self, id: LanguageId) -> Option<&LanguageConfig> {
        self.languages.iter().find(|lang| lang.id == id)
    }

    pub fn create_parser(&self, config: &LanguageConfig) -> Parser {
        let mut parser = Parser::new();
        parser.set_language(&config.language).unwrap();
        parser
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_registered_extensions_case_insensitively() {
        let registry = LanguageRegistry::new();
        let cases = [
            ("main.rs", LanguageId::Rust),
            ("app.jsx", LanguageId::JavaScript),
            ("app.cjs", LanguageId::JavaScript),
            ("script.py", LanguageId::Python),
            ("app.ts", LanguageId::TypeScript),
            ("component.tsx", LanguageId::Tsx),
            ("index.htm", LanguageId::Html),
            ("style.css", LanguageId::Css),
            ("data.JSON", LanguageId::Json),
            ("config.YML", LanguageId::Yaml),
            ("main.go", LanguageId::Go),
            ("main.c", LanguageId::C),
            ("main.hpp", LanguageId::Cpp),
            ("Main.java", LanguageId::Java),
            ("README.MD", LanguageId::Markdown),
            ("script.sh", LanguageId::Bash),
        ];

        for (path, expected_id) in cases {
            let config = registry
                .detect_language(Path::new(path))
                .unwrap_or_else(|| panic!("No language detected for {path}"));
            assert_eq!(config.id, expected_id, "Wrong language detected for {path}");
        }
    }

    #[test]
    fn test_dynamic_keyword_extraction_from_treesitter() {
        let sample_query = r#"
            [
              "fn"
              "let"
              "static"
              "def"
              "class"
              "public"
            ] @keyword
        "#;
        let kw = extract_keywords_from_query(sample_query);
        assert!(kw.contains(&"fn".to_string()), "Rust should contain 'fn'");
        assert!(kw.contains(&"let".to_string()), "Rust should contain 'let'");
        assert!(kw.contains(&"static".to_string()), "Rust should contain 'static'");
        assert!(kw.contains(&"public".to_string()), "Java should contain 'public'");
        assert!(kw.contains(&"class".to_string()), "Java should contain 'class'");
        assert!(kw.contains(&"def".to_string()), "Python should contain 'def'");
    }
}
