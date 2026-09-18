use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooseConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub extensions: Vec<String>,
}

fn default_provider() -> String {
    "ollama".to_string()
}

fn default_model() -> String {
    "qwen2.5-coder".to_string()
}

impl Default for GooseConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            model: default_model(),
            host: Some("http://localhost:11434".to_string()),
            extensions: vec![
                "developer".to_string(),
                "memory".to_string(),
            ],
        }
    }
}

impl GooseConfig {
    /// Returns the standard goose config path (~/.config/goose/config.yaml)
    pub fn config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".config").join("goose").join("config.yaml"))
    }

    /// Loads the configuration or returns default if not found
    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str::<Self>(&content) {
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    /// Ensures the goose config directory and basic configuration exists
    pub fn ensure_configured(provider: &str, model: &str) -> Result<PathBuf, String> {
        let path = Self::config_path().ok_or("Cannot determine home directory for goose config")?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let config_json = format!(
            r#"{{
  "GOOSE_PROVIDER": "{}",
  "GOOSE_MODEL": "{}"
}}
"#,
            provider, model
        );

        fs::write(&path, config_json).map_err(|e| e.to_string())?;
        Ok(path)
    }
}
