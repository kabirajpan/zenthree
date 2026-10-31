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
            indent_query: include_str!("queries/rust/indents.scm"),
            highlight_query: include_str!("queries/rust/highlights.scm"),
            line_comment: "//",
        }
    }

    pub fn javascript() -> Self {
        Self {
            id: LanguageId::JavaScript,
            name: "JavaScript",
            language: tree_sitter_javascript_lang(),
            extensions: &["js", "jsx", "mjs", "cjs"],
            indent_query: include_str!("queries/javascript/indents.scm"),
            highlight_query: include_str!("queries/javascript/highlights.scm"),
            line_comment: "//",
        }
    }

    pub fn python() -> Self {
        Self {
            id: LanguageId::Python,
            name: "Python",
            language: tree_sitter_python_lang(),
            extensions: &["py"],
            indent_query: include_str!("queries/python/indents.scm"),
            highlight_query: include_str!("queries/python/highlights.scm"),
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
            highlight_query: include_str!("queries/typescript/highlights.scm"),
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
            highlight_query: include_str!("queries/typescript/highlights.scm"),
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
            highlight_query: include_str!("queries/html/highlights.scm"),
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
            highlight_query: include_str!("queries/css/highlights.scm"),
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
            highlight_query: include_str!("queries/json/highlights.scm"),
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
            highlight_query: include_str!("queries/yaml/highlights.scm"),
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
            highlight_query: include_str!("queries/go/highlights.scm"),
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
            highlight_query: include_str!("queries/c/highlights.scm"),
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
            highlight_query: include_str!("queries/cpp/highlights.scm"),
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
            highlight_query: include_str!("queries/java/highlights.scm"),
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
            highlight_query: include_str!("queries/markdown/highlights.scm"),
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
            highlight_query: include_str!("queries/bash/highlights.scm"),
            line_comment: "#",
        }
    }
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
    use tree_sitter::{Query, QueryCursor};

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
    fn registered_highlight_queries_compile_and_capture_tokens() {
        let registry = LanguageRegistry::new();
        let samples = [
            (LanguageId::Rust, "fn main() { let answer = 42; }"),
            (
                LanguageId::JavaScript,
                "function main() { const answer = 42; }",
            ),
            (LanguageId::Python, "def main():\n    answer = 42"),
            (LanguageId::TypeScript, "const answer: number = 42;"),
            (LanguageId::Tsx, "const view = <div title=\"hello\" />;"),
            (LanguageId::Html, "<div class=\"hello\">Text</div>"),
            (LanguageId::Css, ".hello { color: red; width: 12px; }"),
            (LanguageId::Json, "{\"answer\": 42, \"enabled\": true}"),
            (LanguageId::Yaml, "answer: 42\n# comment"),
            (
                LanguageId::Go,
                "package main\nfunc main() { println(\"hello\") }",
            ),
            (LanguageId::C, "int main(void) { return 42; }"),
            (LanguageId::Cpp, "class Example { int answer; };"),
            (LanguageId::Java, "class Example { int answer = 42; }"),
            (
                LanguageId::Markdown,
                "# Heading\n\n`code` [link](https://example.com)",
            ),
            (LanguageId::Bash, "echo \"hello\""),
        ];

        for (id, source) in samples {
            let config = registry
                .get_language(id)
                .unwrap_or_else(|| panic!("No configuration for {id:?}"));
            let query = Query::new(&config.language, config.highlight_query)
                .unwrap_or_else(|error| panic!("Invalid query for {}: {error}", config.name));
            let mut parser = registry.create_parser(config);
            let tree = parser
                .parse(source, None)
                .unwrap_or_else(|| panic!("Failed to parse {} sample", config.name));
            let mut cursor = QueryCursor::new();
            let has_captures = cursor
                .matches(&query, tree.root_node(), source.as_bytes())
                .any(|query_match| !query_match.captures.is_empty());

            assert!(has_captures, "Query matched no tokens for {}", config.name);
        }
    }
}
