use zenthra::Color;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionCategory {
    LanguagePack,
    Tooling,
}

#[derive(Debug, Clone)]
pub struct LanguageExtension {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub extensions: &'static [&'static str],
    pub version: &'static str,
    pub author: &'static str,
    pub glyph: &'static str,
    pub icon_color: Color,
    pub features: &'static [&'static str],
    pub tools: &'static [&'static str],
}

pub struct ExtensionCatalog;

impl ExtensionCatalog {
    pub fn all() -> Vec<LanguageExtension> {
        vec![
            LanguageExtension {
                id: "go",
                name: "Go",
                description: "Full syntax highlighting, code navigation, and formatting for Go (.go)",
                extensions: &["go"],
                version: "1.2.0",
                author: "Zenthra Official",
                glyph: "\u{e627}", // Devicon Go
                icon_color: Color::rgb(0.0 / 255.0, 172.0 / 255.0, 215.0 / 255.0), // Go cyan #00acd7
                features: &["Syntax Highlighting", "Formatter (gofmt)", "Indent Calculator"],
                tools: &["gofmt", "go-tools", "tree-sitter"],
            },
            LanguageExtension {
                id: "typescript",
                name: "TypeScript",
                description: "Type-aware syntax highlighting, TSX support, and Prettier integration (.ts, .tsx)",
                extensions: &["ts", "tsx", "mts", "cts"],
                version: "1.3.1",
                author: "Zenthra Official",
                glyph: "\u{e628}", // Devicon TypeScript
                icon_color: Color::rgb(49.0 / 255.0, 120.0 / 255.0, 198.0 / 255.0), // TS blue #3178c6
                features: &["Syntax Highlighting", "Prettier Formatter", "TSX Support"],
                tools: &["prettier", "tsc", "tree-sitter"],
            },
            LanguageExtension {
                id: "cpp",
                name: "C / C++",
                description: "Syntax highlighting, preprocessor macros, and Clang tooling for C/C++ (.c, .cpp, .h, .hpp)",
                extensions: &["c", "cpp", "cc", "cxx", "h", "hpp", "hxx"],
                version: "1.1.4",
                author: "Zenthra Official",
                glyph: "\u{e61d}", // Devicon C++
                icon_color: Color::rgb(0.0 / 255.0, 89.0 / 255.0, 156.0 / 255.0), // C++ blue #00599c
                features: &["Syntax Highlighting", "ClangFormat", "Macro Parsing"],
                tools: &["clang-format", "tree-sitter-cpp"],
            },
            LanguageExtension {
                id: "html_css",
                name: "HTML / CSS",
                description: "Rich web styling syntax, tags, attributes, and colors (.html, .css, .scss)",
                extensions: &["html", "htm", "css", "scss", "less"],
                version: "1.2.0",
                author: "Zenthra Official",
                glyph: "\u{e736}", // Devicon HTML5
                icon_color: Color::rgb(228.0 / 255.0, 77.0 / 255.0, 38.0 / 255.0), // HTML orange #e44d26
                features: &["Syntax Highlighting", "Prettier Formatter", "Color Preview"],
                tools: &["prettier", "stylelint"],
            },
            LanguageExtension {
                id: "json_yaml",
                name: "JSON / YAML",
                description: "Structured configuration highlighting and formatting (.json, .yaml, .yml)",
                extensions: &["json", "jsonc", "yaml", "yml"],
                version: "1.1.0",
                author: "Zenthra Official",
                glyph: "\u{e60b}", // Devicon JSON
                icon_color: Color::rgb(245.0 / 255.0, 180.0 / 255.0, 0.0 / 255.0), // Gold yellow
                features: &["Syntax Highlighting", "Key-Value Validation", "Prettier"],
                tools: &["prettier", "yaml-tools"],
            },
            LanguageExtension {
                id: "markdown",
                name: "Markdown",
                description: "Headers, tables, code blocks, bold/italics, and preview styling (.md, .markdown)",
                extensions: &["md", "markdown"],
                version: "1.2.5",
                author: "Zenthra Official",
                glyph: "\u{f4ed}", // Markdown icon
                icon_color: Color::rgb(142.0 / 255.0, 192.0 / 255.0, 124.0 / 255.0), // Aqua green
                features: &["Syntax Highlighting", "Document Formatting", "Table Alignment"],
                tools: &["prettier", "markdownlint"],
            },
            LanguageExtension {
                id: "shell",
                name: "Shell",
                description: "Command syntax, variables, pipes, and script execution tools (.sh, .bash, .zsh)",
                extensions: &["sh", "bash", "zsh"],
                version: "1.0.8",
                author: "Zenthra Official",
                glyph: "\u{f120}", // Terminal prompt icon
                icon_color: Color::rgb(74.0 / 255.0, 222.0 / 255.0, 128.0 / 255.0), // Terminal green
                features: &["Syntax Highlighting", "PTY Runner", "Shebang Detection"],
                tools: &["shfmt", "shellcheck"],
            },
            LanguageExtension {
                id: "lua",
                name: "Lua",
                description: "Lightweight scripting syntax, table definitions, and StyLua formatting (.lua)",
                extensions: &["lua"],
                version: "1.0.5",
                author: "Zenthra Official",
                glyph: "\u{e620}", // Devicon Lua
                icon_color: Color::rgb(0.0 / 255.0, 80.0 / 255.0, 180.0 / 255.0), // Lua blue
                features: &["Syntax Highlighting", "StyLua Formatter", "Neovim Config"],
                tools: &["stylua", "tree-sitter-lua"],
            },
            LanguageExtension {
                id: "rust",
                name: "Rust",
                description: "Tree-sitter native AST syntax highlighting, auto-indent, and rustfmt (.rs)",
                extensions: &["rs"],
                version: "2.1.0",
                author: "Zenthra Core",
                glyph: "\u{e7a8}", // Devicon Rust
                icon_color: Color::rgb(251.0 / 255.0, 146.0 / 255.0, 60.0 / 255.0), // Rust orange
                features: &["Tree-sitter AST", "rustfmt", "Semantic Context", "Auto-Indent"],
                tools: &["rustfmt", "cargo", "tree-sitter-rust"],
            },
            LanguageExtension {
                id: "python",
                name: "Python",
                description: "Tree-sitter syntax highlighting, indentation, and docstring formatting (.py)",
                extensions: &["py"],
                version: "2.0.2",
                author: "Zenthra Core",
                glyph: "\u{e73c}", // Devicon Python
                icon_color: Color::rgb(59.0 / 255.0, 130.0 / 255.0, 246.0 / 255.0), // Python blue/yellow
                features: &["Tree-sitter AST", "Black Formatter", "Semantic Context"],
                tools: &["black", "tree-sitter-python"],
            },
            LanguageExtension {
                id: "javascript",
                name: "JavaScript",
                description: "Modern ES6+ syntax highlighting, JSX support, and Prettier integration (.js, .jsx)",
                extensions: &["js", "jsx", "mjs", "cjs"],
                version: "1.5.0",
                author: "Zenthra Core",
                glyph: "\u{e74e}", // Devicon JavaScript
                icon_color: Color::rgb(250.0 / 255.0, 204.0 / 255.0, 21.0 / 255.0), // JS Yellow
                features: &["Tree-sitter AST", "Prettier", "JSX Support"],
                tools: &["prettier", "tree-sitter-javascript"],
            },
        ]
    }

    /// Find an extension matching the given file path
    pub fn find_by_path(path: &Path) -> Option<LanguageExtension> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        Self::find_by_extension(&ext)
    }

    /// Find an extension matching a file extension string
    pub fn find_by_extension(ext: &str) -> Option<LanguageExtension> {
        let ext_clean = ext.trim_start_matches('.').to_lowercase();
        Self::all()
            .into_iter()
            .find(|item| item.extensions.contains(&ext_clean.as_str()))
    }

    /// Find extension by ID
    pub fn find_by_id(id: &str) -> Option<LanguageExtension> {
        Self::all().into_iter().find(|item| item.id == id)
    }
}
