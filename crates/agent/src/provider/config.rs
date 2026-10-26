use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for a single provider entry in config.json.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ProviderConfig {
    /// API key for this provider.
    pub key: String,
    /// Base URL — required for custom providers, optional for known ones.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// Available models — required for custom providers, auto-filled for known ones.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_context_window: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_max_new_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_batch_size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_gpu_acceleration: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_offload_layers: Option<u32>,
}

/// Top-level structure of ~/.zenthree/config.json.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ZenthreeConfig {
    /// Map of provider name (e.g. "gemini", "groq", "openai") → config.
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

/// Known built-in providers and their default base URLs.
pub struct KnownProvider {
    pub name: &'static str,
    pub display_name: &'static str,
    pub base_url: &'static str,
    /// Env var name for backwards compatibility (e.g. "GEMINI_API_KEY").
    pub env_var: &'static str,
    pub models: &'static [&'static str],
}

pub const KNOWN_PROVIDERS: &[KnownProvider] = &[
    KnownProvider {
        name: "gemini",
        display_name: "Google Gemini",
        base_url: "https://generativelanguage.googleapis.com/v1beta",
        env_var: "GEMINI_API_KEY",
        models: &[
            "gemini-2.5-flash",
            "gemini-2.5-pro",
            "gemini-2.0-flash",
            "gemini-2.0-flash-lite",
        ],
    },
    KnownProvider {
        name: "groq",
        display_name: "Groq",
        base_url: "https://api.groq.com/openai/v1",
        env_var: "GROQ_API_KEY",
        models: &[
            "llama-3.3-70b-versatile",
            "deepseek-r1-distill-llama-70b",
            "llama-3.1-8b-instant",
            "gemma2-9b-it",
        ],
    },
];

impl ZenthreeConfig {
    /// Path to ~/.zenthree/config.json.
    pub fn config_path() -> PathBuf {
        if let Some(home) = dirs::home_dir() {
            home.join(".zenthree").join("config.json")
        } else {
            PathBuf::from("config.json")
        }
    }

    /// Load config from disk. Returns default if file doesn't exist or is malformed.
    pub fn load() -> Self {
        let path = Self::config_path();
        if !path.exists() {
            return Self::default();
        }
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    }

    /// Save config to disk.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())
    }

    /// Get a provider's API key by provider name (e.g. "gemini", "groq").
    pub fn get_provider_key(&self, provider: &str) -> Option<String> {
        let entry = self.providers.get(provider)?;
        if entry.key.is_empty() {
            None
        } else {
            Some(entry.key.clone())
        }
    }

    /// Set a provider's API key, creating the entry if it doesn't exist.
    pub fn set_provider_key(&mut self, provider: &str, key: &str) {
        let entry = self.providers.entry(provider.to_string()).or_default();
        entry.key = key.to_string();
    }

    /// Remove a provider's API key. Removes the entry if it has no other data.
    pub fn remove_provider_key(&mut self, provider: &str) {
        if let Some(entry) = self.providers.get_mut(provider) {
            entry.key.clear();
            if entry.base_url.is_none() && entry.models.is_none() {
                self.providers.remove(provider);
            }
        }
    }

    /// Add a fully custom provider with base URL and models.
    pub fn add_custom_provider(
        &mut self,
        name: &str,
        key: &str,
        base_url: &str,
        models: Vec<String>,
    ) {
        let entry = self.providers.entry(name.to_string()).or_default();
        entry.key = key.to_string();
        entry.base_url = Some(base_url.to_string());
        if !models.is_empty() {
            entry.models = Some(models);
        }
    }

    /// Returns all providers that have a key set (both known and custom).
    pub fn active_providers(&self) -> Vec<(&str, &ProviderConfig)> {
        self.providers
            .iter()
            .filter(|(_, v)| !v.key.is_empty())
            .map(|(k, v)| (k.as_str(), v))
            .collect()
    }

    /// Get models for a provider. Uses built-in list for known providers,
    /// config list for custom ones.
    pub fn get_models(&self, provider: &str) -> Vec<String> {
        // Check known providers first
        if let Some(known) = KNOWN_PROVIDERS.iter().find(|p| p.name == provider) {
            return known.models.iter().map(|s| s.to_string()).collect();
        }
        // Fall back to config-defined models
        if let Some(entry) = self.providers.get(provider) {
            if let Some(models) = &entry.models {
                return models.clone();
            }
        }
        Vec::new()
    }

    /// Lookup a key by env var name for backwards compatibility.
    /// e.g. "GEMINI_API_KEY" → checks "gemini" in config.
    pub fn get_key_by_env_var(&self, env_var: &str) -> Option<String> {
        let provider = KNOWN_PROVIDERS.iter().find(|p| p.env_var == env_var)?;
        self.get_provider_key(provider.name)
    }
}

#[cfg(test)]
mod tests {
    use super::ZenthreeConfig;

    #[test]
    fn older_provider_config_loads_without_local_runtime_options() {
        let config: ZenthreeConfig = serde_json::from_str(
            r#"{"providers":{"local":{"key":"","base_url":"http://127.0.0.1:8080/v1"}}}"#,
        )
        .expect("legacy provider config should remain valid");
        let local = config.providers.get("local").expect("local provider");
        assert_eq!(local.base_url.as_deref(), Some("http://127.0.0.1:8080/v1"));
        assert!(local.local_context_window.is_none());
        assert!(local.local_max_new_tokens.is_none());
    }
}
