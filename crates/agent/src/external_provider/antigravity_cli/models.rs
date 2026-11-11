use serde::{Deserialize, Serialize};

/// Metadata and specifications for models supported by the Antigravity provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub max_tokens: usize,
    pub supports_tools: bool,
    pub supports_thinking: bool,
    pub description: String,
}

pub const DEFAULT_MODEL_ID: &str = "gemini-3.8-flash-high";

static DYNAMIC_MODELS: std::sync::Mutex<Vec<ModelInfo>> = std::sync::Mutex::new(Vec::new());

/// Store dynamically queried models from the external ACP provider.
pub fn set_dynamic_models(models: Vec<ModelInfo>) {
    if let Ok(mut m) = DYNAMIC_MODELS.lock() {
        *m = models;
    }
}

/// Dynamically discovered Antigravity models provided by the external ACP agent.
pub fn available_models() -> Vec<ModelInfo> {
    if let Ok(m) = DYNAMIC_MODELS.lock() {
        if !m.is_empty() {
            return m.clone();
        }
    }
    Vec::new()
}

/// Retrieve details for a given model ID or fall back to default model metadata.
pub fn get_model_info(model_id: &str) -> ModelInfo {
    available_models()
        .into_iter()
        .find(|m| m.id.eq_ignore_ascii_case(model_id))
        .unwrap_or_else(|| ModelInfo {
            id: model_id.to_string(),
            display_name: format!("{model_id} (Custom)"),
            max_tokens: 128_000,
            supports_tools: true,
            supports_thinking: false,
            description: "Custom Antigravity model endpoint.".to_string(),
        })
}
