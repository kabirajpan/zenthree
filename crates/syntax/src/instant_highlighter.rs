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
            (Regex::new(r"\b(function|class|const|let|var|if|else|for|while|return|import|export|from|default)\b").unwrap(), Highlight::KEYWORD),
            (Regex::new(r"\b(true|false|null|undefined)\b").unwrap(), Highlight::CONSTANT),
            (Regex::new(r"//[^\n]*").unwrap(), Highlight::COMMENT),
            (Regex::new(r"/\*[^*]*\*/").unwrap(), Highlight::COMMENT),
            (Regex::new(r#""[^"]*"|'[^']*'|`[^`]*`"#).unwrap(), Highlight::STRING),
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
                return match ext {
                    "rs" => "rust",
                    "py" => "python",
                    "js" | "jsx" | "ts" | "tsx" => "javascript",
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
