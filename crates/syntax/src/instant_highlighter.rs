use regex::Regex;
use std::collections::HashMap;
use zenthra_core::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Highlight {
    COMMENT,
    KEYWORD,
    STRING,
    NUMBER,
    FUNCTION,
    TYPE,
    VARIABLE,
    CONSTANT,
    ATTRIBUTE,
    OPERATOR,
    PUNCTUATION,
}

impl Highlight {
    pub fn to_color(&self) -> Color {
        match self {
            Highlight::COMMENT => Color::rgb(100.0 / 255.0, 160.0 / 255.0, 100.0 / 255.0), // Green
            Highlight::KEYWORD => Color::rgb(200.0 / 255.0, 120.0 / 255.0, 200.0 / 255.0), // Purple
            Highlight::STRING => Color::rgb(200.0 / 255.0, 150.0 / 255.0, 100.0 / 255.0),  // Orange
            Highlight::NUMBER => Color::rgb(100.0 / 255.0, 180.0 / 255.0, 255.0 / 255.0),  // Blue
            Highlight::FUNCTION => Color::rgb(220.0 / 255.0, 220.0 / 255.0, 100.0 / 255.0), // Yellow
            Highlight::TYPE => Color::rgb(100.0 / 255.0, 200.0 / 255.0, 255.0 / 255.0),    // Light Blue
            Highlight::VARIABLE => Color::WHITE,                  // White
            Highlight::CONSTANT => Color::rgb(200.0 / 255.0, 100.0 / 255.0, 100.0 / 255.0), // Red
            Highlight::ATTRIBUTE => Color::rgb(200.0 / 255.0, 200.0 / 255.0, 100.0 / 255.0), // Light Yellow
            Highlight::OPERATOR => Color::rgb(200.0 / 255.0, 200.0 / 255.0, 200.0 / 255.0), // Light Gray
            Highlight::PUNCTUATION => Color::rgb(150.0 / 255.0, 150.0 / 255.0, 150.0 / 255.0), // Gray
        }
    }
}

#[derive(Debug, Clone)]
pub struct HighlightedRange {
    pub start: usize,
    pub end: usize,
    pub highlight: Highlight,
}

#[derive(Debug)]
pub struct InstantHighlighter {
    patterns: Vec<(Regex, Highlight)>,
    language_patterns: HashMap<String, Vec<(Regex, Highlight)>>,
}

impl InstantHighlighter {
    pub fn new() -> Self {
        let mut language_patterns = HashMap::new();

        // Python patterns
        language_patterns.insert("python".to_string(), Self::python_patterns());

        // JavaScript patterns
        language_patterns.insert("javascript".to_string(), Self::javascript_patterns());

        // Rust patterns
        language_patterns.insert("rust".to_string(), Self::rust_patterns());

        // TypeScript patterns
        language_patterns.insert("typescript".to_string(), Self::typescript_patterns());

        // Go patterns
        language_patterns.insert("go".to_string(), Self::go_patterns());

        // C / C++ patterns
        language_patterns.insert("c".to_string(), Self::cpp_patterns());
        language_patterns.insert("cpp".to_string(), Self::cpp_patterns());

        // HTML & CSS patterns
        language_patterns.insert("html".to_string(), Self::html_patterns());
        language_patterns.insert("css".to_string(), Self::css_patterns());

        // JSON & YAML patterns
        language_patterns.insert("json".to_string(), Self::json_patterns());
        language_patterns.insert("yaml".to_string(), Self::yaml_patterns());

        // Markdown patterns
        language_patterns.insert("markdown".to_string(), Self::markdown_patterns());

        // Shell patterns
        language_patterns.insert("shell".to_string(), Self::shell_patterns());

        // Lua patterns
        language_patterns.insert("lua".to_string(), Self::lua_patterns());

        // Generic patterns for unknown languages
        let generic_patterns = Self::generic_patterns();

        Self {
            patterns: generic_patterns,
            language_patterns,
        }
    }

    fn python_patterns() -> Vec<(Regex, Highlight)> {
        vec![
            (Regex::new(r"\b(def|class|if|else|elif|for|while|return|import|from|as|with|try|except|finally|raise)\b").unwrap(), Highlight::KEYWORD),
            (Regex::new(r"\b(True|False|None)\b").unwrap(), Highlight::CONSTANT),
            (Regex::new(r"\b(self|cls)\b").unwrap(), Highlight::VARIABLE),
            (Regex::new(r"#[^\n]*").unwrap(), Highlight::COMMENT),
            (Regex::new(r#""{3}[^"]*"{3}|'{3}[^']*'{3}"#).unwrap(), Highlight::COMMENT),
            (Regex::new(r#""[^"]*"|'[^']*'"#).unwrap(), Highlight::STRING),
            (Regex::new(r"\b\d+\.?\d*\b").unwrap(), Highlight::NUMBER),
        ]
    }

    fn javascript_patterns() -> Vec<(Regex, Highlight)> {
        vec![
            (Regex::new(r"\b(function|class|const|let|var|if|else|for|while|return|import|export|from|default|async|await)\b").unwrap(), Highlight::KEYWORD),
            (Regex::new(r"\b(true|false|null|undefined)\b").unwrap(), Highlight::CONSTANT),
            (Regex::new(r"//[^\n]*").unwrap(), Highlight::COMMENT),
            (Regex::new(r"/\*[^*]*\*/").unwrap(), Highlight::COMMENT),
            (Regex::new(r#""[^"]*"|'[^']*'|`[^`]*`"#).unwrap(), Highlight::STRING),
            (Regex::new(r"\b\d+\.?\d*\b").unwrap(), Highlight::NUMBER),
        ]
    }

    fn typescript_patterns() -> Vec<(Regex, Highlight)> {
        vec![
            (Regex::new(r"\b(function|class|const|let|var|if|else|for|while|return|import|export|from|default|type|interface|enum|implements|declare|namespace|as|keyof|readonly|async|await)\b").unwrap(), Highlight::KEYWORD),
            (Regex::new(r"\b(string|number|boolean|any|unknown|never|void|null|undefined|Record|Array|Promise)\b").unwrap(), Highlight::TYPE),
            (Regex::new(r"\b(true|false|null|undefined)\b").unwrap(), Highlight::CONSTANT),
            (Regex::new(r"//[^\n]*").unwrap(), Highlight::COMMENT),
            (Regex::new(r"/\*[^*]*\*/").unwrap(), Highlight::COMMENT),
            (Regex::new(r#""[^"]*"|'[^']*'|`[^`]*`"#).unwrap(), Highlight::STRING),
            (Regex::new(r"\b\d+\.?\d*\b").unwrap(), Highlight::NUMBER),
        ]
    }

    fn go_patterns() -> Vec<(Regex, Highlight)> {
        vec![
            (Regex::new(r"\b(package|import|func|struct|interface|var|const|type|return|if|else|for|range|switch|case|default|select|go|defer|chan)\b").unwrap(), Highlight::KEYWORD),
            (Regex::new(r"\b(string|int|int8|int16|int32|int64|uint|uint8|uint16|uint32|uint64|uintptr|float32|float64|bool|byte|rune|error|any)\b").unwrap(), Highlight::TYPE),
            (Regex::new(r"\b(true|false|nil|iota)\b").unwrap(), Highlight::CONSTANT),
            (Regex::new(r"\b(make|new|append|len|cap|delete|close|panic|recover|print|println)\b").unwrap(), Highlight::FUNCTION),
            (Regex::new(r"//[^\n]*").unwrap(), Highlight::COMMENT),
            (Regex::new(r"/\*[^*]*\*/").unwrap(), Highlight::COMMENT),
            (Regex::new(r#""[^"]*"|'[^']*'|`[^`]*`"#).unwrap(), Highlight::STRING),
            (Regex::new(r"\b\d+\.?\d*\b").unwrap(), Highlight::NUMBER),
        ]
    }

    fn cpp_patterns() -> Vec<(Regex, Highlight)> {
        vec![
            (Regex::new(r"\b(#include|#define|#ifdef|#ifndef|#endif|#pragma|class|struct|enum|union|template|typename|public|private|protected|virtual|override|namespace|using|const|static|constexpr|inline|auto|if|else|for|while|do|switch|case|default|return|break|continue|try|catch|throw|new|delete)\b").unwrap(), Highlight::KEYWORD),
            (Regex::new(r"\b(int|long|short|char|float|double|bool|void|size_t|uint32_t|int32_t|uint64_t|int64_t|uint8_t|int8_t|string|vector|map)\b").unwrap(), Highlight::TYPE),
            (Regex::new(r"\b(true|false|nullptr|NULL)\b").unwrap(), Highlight::CONSTANT),
            (Regex::new(r"//[^\n]*").unwrap(), Highlight::COMMENT),
            (Regex::new(r"/\*[^*]*\*/").unwrap(), Highlight::COMMENT),
            (Regex::new(r#""[^"]*"|'[^']*'"#).unwrap(), Highlight::STRING),
            (Regex::new(r"\b\d+\.?\d*\b").unwrap(), Highlight::NUMBER),
        ]
    }

    fn html_patterns() -> Vec<(Regex, Highlight)> {
        vec![
            (Regex::new(r"<!DOCTYPE[^>]*>").unwrap(), Highlight::CONSTANT),
            (Regex::new(r"<!--[^-]*-->").unwrap(), Highlight::COMMENT),
            (Regex::new(r"</?[a-zA-Z0-9_-]+").unwrap(), Highlight::KEYWORD),
            (Regex::new(r"\b[a-zA-Z0-9_-]+\s*=").unwrap(), Highlight::ATTRIBUTE),
            (Regex::new(r#""[^"]*"|'[^']*'"#).unwrap(), Highlight::STRING),
        ]
    }

    fn css_patterns() -> Vec<(Regex, Highlight)> {
        vec![
            (Regex::new(r"/\*[^*]*\*/").unwrap(), Highlight::COMMENT),
            (Regex::new(r"\b[a-z-]+\s*:").unwrap(), Highlight::ATTRIBUTE),
            (Regex::new(r":[a-z-]+").unwrap(), Highlight::FUNCTION),
            (Regex::new(r"#[0-9a-fA-F]{3,8}").unwrap(), Highlight::CONSTANT),
            (Regex::new(r"\b\d+\.?\d*(px|em|rem|%|vh|vw|s|ms|deg)?\b").unwrap(), Highlight::NUMBER),
            (Regex::new(r#""[^"]*"|'[^']*'"#).unwrap(), Highlight::STRING),
        ]
    }

    fn json_patterns() -> Vec<(Regex, Highlight)> {
        vec![
            (Regex::new(r#""[^"\\]*"\s*:"#).unwrap(), Highlight::KEYWORD),
            (Regex::new(r#""[^"\\]*""#).unwrap(), Highlight::STRING),
            (Regex::new(r"\b(true|false|null)\b").unwrap(), Highlight::CONSTANT),
            (Regex::new(r"\b-?\d+\.?\d*([eE][+-]?\d+)?\b").unwrap(), Highlight::NUMBER),
        ]
    }

    fn yaml_patterns() -> Vec<(Regex, Highlight)> {
        vec![
            (Regex::new(r"#[^\n]*").unwrap(), Highlight::COMMENT),
            (Regex::new(r"^[ \t]*[a-zA-Z0-9_.-]+\s*:").unwrap(), Highlight::KEYWORD),
            (Regex::new(r"\b(true|false|null|yes|no|on|off)\b").unwrap(), Highlight::CONSTANT),
            (Regex::new(r#""[^"]*"|'[^']*'"#).unwrap(), Highlight::STRING),
            (Regex::new(r"\b\d+\.?\d*\b").unwrap(), Highlight::NUMBER),
        ]
    }

    fn markdown_patterns() -> Vec<(Regex, Highlight)> {
        vec![
            (Regex::new(r"^#{1,6}\s+[^\n]+").unwrap(), Highlight::KEYWORD),
            (Regex::new(r"\*\*[^*]+\*\*|__[^_]+__").unwrap(), Highlight::FUNCTION),
            (Regex::new(r"\*[^*]+\*|_[^_]+_").unwrap(), Highlight::ATTRIBUTE),
            (Regex::new(r"`[^`]+`").unwrap(), Highlight::STRING),
            (Regex::new(r"\[[^\]]+\]\([^)]+\)").unwrap(), Highlight::TYPE),
            (Regex::new(r"^>[^\n]*").unwrap(), Highlight::COMMENT),
        ]
    }

    fn shell_patterns() -> Vec<(Regex, Highlight)> {
        vec![
            (Regex::new(r"\b(echo|export|source|if|then|elif|else|fi|for|in|do|done|while|until|case|esac|function|return|local|exit|set|unset)\b").unwrap(), Highlight::KEYWORD),
            (Regex::new(r"\$[a-zA-Z0-9_]+|\$\{[^}]+\}").unwrap(), Highlight::VARIABLE),
            (Regex::new(r"\b(true|false)\b").unwrap(), Highlight::CONSTANT),
            (Regex::new(r"#[^\n]*").unwrap(), Highlight::COMMENT),
            (Regex::new(r#""[^"]*"|'[^']*'"#).unwrap(), Highlight::STRING),
            (Regex::new(r"\b\d+\.?\d*\b").unwrap(), Highlight::NUMBER),
        ]
    }

    fn lua_patterns() -> Vec<(Regex, Highlight)> {
        vec![
            (Regex::new(r"\b(local|function|end|if|then|elseif|else|for|in|do|while|repeat|until|return|break|and|or|not)\b").unwrap(), Highlight::KEYWORD),
            (Regex::new(r"\b(nil|true|false)\b").unwrap(), Highlight::CONSTANT),
            (Regex::new(r"--\[\[.*?\]\]|--[^\n]*").unwrap(), Highlight::COMMENT),
            (Regex::new(r#""[^"]*"|'[^']*'|\[\[.*?\]\]"#).unwrap(), Highlight::STRING),
            (Regex::new(r"\b\d+\.?\d*\b").unwrap(), Highlight::NUMBER),
        ]
    }

    fn rust_patterns() -> Vec<(Regex, Highlight)> {
        vec![
            (Regex::new(r"\b(fn|struct|enum|impl|trait|let|mut|pub|if|else|for|while|match|return|use|mod|crate|super|self)\b").unwrap(), Highlight::KEYWORD),
            (Regex::new(r"\b(true|false|Some|None|Ok|Err)\b").unwrap(), Highlight::CONSTANT),
            (Regex::new(r"//[^\n]*").unwrap(), Highlight::COMMENT),
            (Regex::new(r"/\*[^*]*\*/").unwrap(), Highlight::COMMENT),
            (Regex::new(r#""[^"]*""#).unwrap(), Highlight::STRING),
            (Regex::new(r"\b\d+\.?\d*\b").unwrap(), Highlight::NUMBER),
        ]
    }

    fn generic_patterns() -> Vec<(Regex, Highlight)> {
        vec![
            (Regex::new(r"//[^\n]*|#[^\n]*").unwrap(), Highlight::COMMENT),
            (Regex::new(r"/\*[^*]*\*/").unwrap(), Highlight::COMMENT),
            (Regex::new(r#""[^"]*"|'[^']*'"#).unwrap(), Highlight::STRING),
            (Regex::new(r"\b\d+\.?\d*\b").unwrap(), Highlight::NUMBER),
        ]
    }

    /// 🚀 FAST: Highlight only visible region using regex
    pub fn highlight_visible_region(
        &self,
        content: &str,
        visible_start_byte: usize,
        visible_end_byte: usize,
        language: &str,
    ) -> Vec<HighlightedRange> {
        let mut ranges = Vec::new();

        // Use language-specific patterns if available
        let patterns_to_use = if let Some(lang_patterns) = self.language_patterns.get(language) {
            lang_patterns
        } else {
            &self.patterns
        };

        // Extract only the visible content
        let visible_content = if visible_end_byte <= content.len() {
            &content[visible_start_byte..visible_end_byte]
        } else {
            &content[visible_start_byte..]
        };

        // Apply regex patterns
        for (pattern, highlight_type) in patterns_to_use {
            for mat in pattern.find_iter(visible_content) {
                let start = visible_start_byte + mat.start();
                let end = visible_start_byte + mat.end();

                ranges.push(HighlightedRange {
                    start,
                    end,
                    highlight: *highlight_type,
                });
            }
        }

        // Sort by start position
        ranges.sort_by_key(|r| r.start);

        ranges
    }

    /// Detect language from file extension
    pub fn detect_language(file_path: Option<&std::path::Path>) -> &'static str {
        if let Some(path) = file_path {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                return match ext.to_lowercase().as_str() {
                    "rs" => "rust",
                    "py" => "python",
                    "js" | "jsx" | "mjs" | "cjs" => "javascript",
                    "ts" | "tsx" | "mts" | "cts" => "typescript",
                    "go" => "go",
                    "c" | "h" => "c",
                    "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp",
                    "html" | "htm" => "html",
                    "css" | "scss" | "less" => "css",
                    "json" | "jsonc" => "json",
                    "yaml" | "yml" => "yaml",
                    "md" | "markdown" => "markdown",
                    "sh" | "bash" | "zsh" => "shell",
                    "lua" => "lua",
                    _ => "unknown",
                };
            }
        }
        "unknown"
    }
}

impl Default for InstantHighlighter {
    fn default() -> Self {
        Self::new()
    }
}
