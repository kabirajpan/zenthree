pub mod config;
pub mod gemini;
pub mod groq;

pub use config::{ZenthreeConfig, KNOWN_PROVIDERS};

use std::fs;
use std::path::Path;
use crate::traits::provider::Provider;

pub use gemini::GeminiProvider;
pub use groq::GroqProvider;

/// Simple helper to get an API key.
/// Priority:
///   1. ~/.zenthree/config.json (primary)
///   2. Environment variable
///   3. .env files (CWD → parent dirs → ~/.config/zene/.env → ~/.zene/.env → ~/.zenthree/.env)
pub fn get_key(name: &str) -> Option<String> {
    // 1. config.json — look up by env var name (e.g. "GEMINI_API_KEY" → provider "gemini")
    let cfg = ZenthreeConfig::load();
    if let Some(val) = cfg.get_key_by_env_var(name) {
        return Some(val);
    }
    // Also allow direct provider name lookup (e.g. get_key("gemini"))
    if let Some(val) = cfg.get_provider_key(name) {
        return Some(val);
    }

    // 2. Environment variable
    if let Ok(val) = std::env::var(name) {
        if !val.trim().is_empty() {
            return Some(val.trim().to_string());
        }
    }

    let parse_env_file = |path: &Path| -> Option<String> {
        let content = fs::read_to_string(path).ok()?;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || !trimmed.contains('=') {
                continue;
            }
            let mut parts = trimmed.splitn(2, '=');
            let key = parts.next().unwrap_or("").trim();
            let val = parts.next().unwrap_or("").trim().trim_matches('"').trim_matches('\'');
            if key == name && !val.is_empty() {
                return Some(val.to_string());
            }
        }
        None
    };

    // 3. .env files — CWD and parent directories
    if let Ok(mut current) = std::env::current_dir() {
        for _ in 0..5 {
            let env_path = current.join(".env");
            if let Some(val) = parse_env_file(&env_path) {
                return Some(val);
            }
            if !current.pop() {
                break;
            }
        }
    }

    // 4. Global user configuration directories
    if let Some(config_dir) = dirs::config_dir() {
        let path = config_dir.join("zene").join(".env");
        if let Some(val) = parse_env_file(&path) {
            return Some(val);
        }
    }

    if let Some(home) = dirs::home_dir() {
        let zene_home_env = home.join(".zene").join(".env");
        if let Some(val) = parse_env_file(&zene_home_env) {
            return Some(val);
        }

        let zenthree_home_env = home.join(".zenthree").join(".env");
        if let Some(val) = parse_env_file(&zenthree_home_env) {
            return Some(val);
        }
    }

    None
}

/// Returns all available (provider_name, model_id) pairs based on configured keys.
/// For known providers, uses the built-in model list.
/// For custom providers, uses the config.json model list.
pub fn available_models() -> Vec<(String, String)> {
    let cfg = ZenthreeConfig::load();
    let mut result = Vec::new();
    // Known providers first
    for kp in KNOWN_PROVIDERS {
        if cfg.get_provider_key(kp.name).is_some() {
            for model in kp.models {
                result.push((kp.name.to_string(), model.to_string()));
            }
        }
    }
    // Custom providers
    for (name, entry) in &cfg.providers {
        if entry.key.is_empty() { continue; }
        // skip if it's a known provider (already added above)
        if KNOWN_PROVIDERS.iter().any(|kp| kp.name == name.as_str()) { continue; }
        if let Some(models) = &entry.models {
            for model in models {
                result.push((name.clone(), model.clone()));
            }
        }
    }
    result
}

/// Persist an API key to ~/.zenthree/config.json.
/// `provider_name` is the short name like "gemini", "groq", or custom.
pub fn set_key(provider_name: &str, value: &str) -> Result<(), std::io::Error> {
    let mut cfg = ZenthreeConfig::load();
    cfg.set_provider_key(provider_name, value);
    cfg.save().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

/// Remove an API key from ~/.zenthree/config.json.
/// `provider_name` is the short name like "gemini", "groq", or custom.
pub fn remove_key(provider_name: &str) -> Result<(), std::io::Error> {
    let mut cfg = ZenthreeConfig::load();
    cfg.remove_provider_key(provider_name);
    cfg.save().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

/// Run completion with Groq
pub fn complete_groq(prompt: &str) -> Result<String, String> {
    let key = get_key("GROQ_API_KEY").ok_or_else(|| "GROQ_API_KEY not found in env or .env".to_string())?;
    groq::chat_completion(&key, prompt)
}

/// Run completion with Gemini
pub fn complete_gemini(prompt: &str) -> Result<String, String> {
    let key = get_key("GEMINI_API_KEY").ok_or_else(|| "GEMINI_API_KEY not found in env or .env".to_string())?;
    gemini::generate_content(&key, prompt)
}

/// Run completion trying Groq first (fastest), falling back to Gemini if Groq fails or key is missing.
pub fn complete(prompt: &str) -> Result<String, String> {
    if get_key("GROQ_API_KEY").is_some() {
        match complete_groq(prompt) {
            Ok(res) => return Ok(res),
            Err(groq_err) => {
                if get_key("GEMINI_API_KEY").is_some() {
                    return complete_gemini(prompt).map_err(|gem_err| {
                        format!("Groq failed: {groq_err}\nGemini fallback also failed: {gem_err}")
                    });
                } else {
                    return Err(groq_err);
                }
            }
        }
    }

    if get_key("GEMINI_API_KEY").is_some() {
        return complete_gemini(prompt);
    }

    Err("Neither GROQ_API_KEY nor GEMINI_API_KEY found in .env or environment".to_string())
}

/// Automatically instantiate a Provider trait object from available API keys.
/// Prefers Gemini (1,000,000 TPM free tier) over Groq to avoid rate limiting.
/// Falls back to Groq if GEMINI_API_KEY is not set.
pub fn create_default_provider() -> Option<Box<dyn Provider>> {
    if let Some(key) = get_key("GROQ_API_KEY") {
        return Some(Box::new(GroqProvider::new(key)));
    }
    if let Some(key) = get_key("GEMINI_API_KEY") {
        return Some(Box::new(GeminiProvider::new(key)));
    }
    None
}

/// Instantiate a specific provider by name + model ID.
/// `provider_name`: `"gemini"` or `"groq"` (case-insensitive).
/// Falls back to `create_default_provider()` if the requested key is missing.
pub fn create_provider_for(provider_name: &str, model_id: &str) -> Option<Box<dyn Provider>> {
    match provider_name.to_lowercase().as_str() {
        "gemini" => {
            let key = get_key("GEMINI_API_KEY")?;
            Some(Box::new(GeminiProvider::with_model(key, model_id)))
        }
        "groq" => {
            let key = get_key("GROQ_API_KEY")?;
            Some(Box::new(GroqProvider::with_model(key, model_id)))
        }
        "antigravity" | "antigravity_cli" | "google_antigravity" => {
            Some(Box::new(crate::external_provider::create_antigravity_provider(Some(model_id))))
        }
        _ => create_default_provider(),
    }
}
