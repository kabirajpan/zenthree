#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderError {
    Transient(String),
    Fatal(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::Transient(s) => write!(f, "Transient error: {}", s),
            ProviderError::Fatal(s) => write!(f, "Fatal error: {}", s),
        }
    }
}

impl std::error::Error for ProviderError {}

pub fn classify_provider_error(err_str: &str) -> ProviderError {
    let lower = err_str.to_lowercase();
    if lower.contains("429")
        || lower.contains("rate limit")
        || lower.contains("too many requests")
        || lower.contains("503")
        || lower.contains("500")
        || lower.contains("timeout")
        || lower.contains("resource exhausted")
        || lower.contains("temporarily unavailable")
    {
        ProviderError::Transient(err_str.to_string())
    } else {
        ProviderError::Fatal(err_str.to_string())
    }
}
