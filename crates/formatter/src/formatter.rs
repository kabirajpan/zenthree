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
        formatter.register(Box::new(crate::providers::PrettierProvider::new()));
        formatter
    }

    /// Register a formatter provider
    pub fn register(&mut self, provider: Box<dyn FormatterProvider>) {
        self.providers.push(provider);
    }

    /// Find appropriate formatter for a file
    pub fn find_provider(&self, file_path: &Path) -> Option<&dyn FormatterProvider> {
        let extension = file_path.extension()?.to_str()?;

        self.providers
            .iter()
            .find(|p| p.supported_extensions().contains(&extension))
            .map(|p| p.as_ref())
    }

    /// Format text using the appropriate provider
    pub fn format_text(&self, text: &str, file_path: Option<&Path>) -> FormatResult {
        if let Some(path) = file_path {
            if let Some(provider) = self.find_provider(path) {
                if !provider.is_available() {
                    return Err(FormatError::NotFound(format!(
                        "{} is not installed or not in PATH",
                        provider.name()
                    )));
                }
                return provider.format(text, Some(path));
            }
            return Err(FormatError::UnsupportedLanguage(format!(
                "No formatter found for {:?}",
                path.extension()
            )));
        }

        Err(FormatError::UnsupportedLanguage(
            "Cannot determine language without file path".to_string(),
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
