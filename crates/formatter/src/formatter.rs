use std::path::Path;

#[derive(Debug, Clone)]
pub enum FormatError {
    ExecutionFailed(String),
    NotFound(String),
    InvalidOutput(String),
    UnsupportedLanguage(String),
}

pub type FormatResult = Result<String, FormatError>;

/// Trait that all formatter providers must implement
pub trait FormatterProvider: Send + Sync {
    /// Name of the formatter (e.g., "rustfmt", "prettier")
    fn name(&self) -> &str;

    /// File extensions this formatter supports (e.g., ["rs"], ["js", "ts"])
    fn supported_extensions(&self) -> &[&str];

    /// Check if the formatter binary is available
    fn is_available(&self) -> bool;

    /// Format the given text
    fn format(&self, text: &str, file_path: Option<&Path>) -> FormatResult;
}

/// Main formatter manager
pub struct Formatter {
    providers: Vec<Box<dyn FormatterProvider>>,
}

impl Formatter {
    pub fn new() -> Self {
        let mut formatter = Self {
            providers: Vec::new(),
        };
        formatter.register(Box::new(crate::providers::RustfmtProvider::new()));
        formatter.register(Box::new(crate::providers::TreesitterProvider::new()));
        formatter.register(Box::new(crate::providers::GenericProvider::new()));
        formatter
    }

    /// Register a formatter provider
    pub fn register(&mut self, provider: Box<dyn FormatterProvider>) {
        self.providers.push(provider);
    }

    /// Find appropriate formatter for a file
    pub fn find_provider(&self, file_path: &Path) -> Option<&dyn FormatterProvider> {
        let extension = file_path.extension().and_then(|e| e.to_str());

        if let Some(ext) = extension {
            if let Some(provider) = self
                .providers
                .iter()
                .find(|p| p.supported_extensions().contains(&ext) && p.is_available())
            {
                return Some(provider.as_ref());
            }
        }

        // Fallback to generic provider ("*")
        self.providers
            .iter()
            .find(|p| p.supported_extensions().contains(&"*") && p.is_available())
            .map(|p| p.as_ref())
    }

    /// Format text using the appropriate provider
    pub fn format_text(&self, text: &str, file_path: Option<&Path>) -> FormatResult {
        if let Some(path) = file_path {
            if let Some(provider) = self.find_provider(path) {
                return provider.format(text, Some(path));
            }
        }

        // If no file path or no matching provider found, fallback to generic provider
        if let Some(generic) = self
            .providers
            .iter()
            .find(|p| p.supported_extensions().contains(&"*") && p.is_available())
        {
            return generic.format(text, file_path);
        }

        Err(FormatError::UnsupportedLanguage(
            "No formatter found".to_string(),
        ))
    }

    /// Get list of available formatters
    pub fn available_formatters(&self) -> Vec<&str> {
        self.providers
            .iter()
            .filter(|p| p.is_available())
            .map(|p| p.name())
            .collect()
    }

    pub fn format_editor(
        &self,
        editor: &mut editor::editor::Editor,
        file_path: Option<&Path>,
    ) -> Result<(), String> {
        let dummy = std::path::PathBuf::from("untitled.rs");
        let path = file_path.or(Some(&dummy));
        let current_text = editor.buffer.to_string();
        let had_trailing_newline = current_text.ends_with('\n');
        match self.format_text(&current_text, path) {
            Ok(mut formatted_text) => {
                if !had_trailing_newline && formatted_text.ends_with('\n') {
                    formatted_text.pop();
                }
                if formatted_text != current_text {
                    editor.replace_all(&formatted_text);
                }
                Ok(())
            }
            Err(e) => Err(format!("Format failed: {:?}", e)),
        }
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_treesitter_formatting_java() {
        let formatter = Formatter::new();
        let unformatted = "public class Hello {\npublic static void main(String[] args) {\nSystem.out.println(\"Hi\");\n}\n}";
        let path = Path::new("Hello.java");
        let result = formatter.format_text(unformatted, Some(path)).unwrap();
        assert!(result.contains("    public static void main"));
        assert!(result.contains("        System.out.println"));
    }

    #[test]
    fn test_treesitter_formatting_python() {
        let formatter = Formatter::new();
        let unformatted = "def foo():\npass\n";
        let path = Path::new("main.py");
        let result = formatter.format_text(unformatted, Some(path)).unwrap();
        assert!(result.contains("    pass"));
    }

    #[test]
    fn test_treesitter_formatting_brace_in_string_literal() {
        let formatter = Formatter::new();
        let unformatted = "public class Hello {\nString s = \"}\";\nint x = 1;\n}";
        let path = Path::new("Hello.java");
        let result = formatter.format_text(unformatted, Some(path)).unwrap();
        assert!(result.contains("    String s = \"}\";"));
        assert!(result.contains("    int x = 1;"));
        assert!(result.ends_with("}\n"));
    }

    #[test]
    fn test_treesitter_formatting_javascript() {
        let formatter = Formatter::new();
        let unformatted = "function test() {\nconst a = 1;\nif (a) {\nconsole.log(a);\n}\n}";
        let path = Path::new("test.js");
        let result = formatter.format_text(unformatted, Some(path)).unwrap();
        assert!(result.contains("    const a = 1;"));
        assert!(result.contains("        console.log(a);"));
    }

    #[test]
    fn test_generic_fallback_for_unknown_language() {
        let formatter = Formatter::new();
        let unformatted = "func test() {\nreturn 42;\n}";
        let path = Path::new("file.customlang");
        let result = formatter.format_text(unformatted, Some(path)).unwrap();
        assert!(result.contains("    return 42;"));
    }
}
