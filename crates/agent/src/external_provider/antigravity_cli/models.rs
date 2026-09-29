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

/// Catalogue of supported Antigravity models.
pub fn available_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "gemini-3.8-flash-high".to_string(),
            display_name: "Gemini 3.8 Flash (High)".to_string(),
            max_tokens: 1_048_576,
            supports_tools: true,
            supports_thinking: true,
            description: "High-reasoning ultra-fast model with dynamic thought reflection.".to_string(),
        },
        ModelInfo {
            id: "gemini-3.8-flash-medium".to_string(),
            display_name: "Gemini 3.8 Flash (Medium)".to_string(),
            max_tokens: 1_048_576,
            supports_tools: true,
            supports_thinking: true,
            description: "Medium-reasoning ultra-fast model for balanced coding tasks.".to_string(),
        },
        ModelInfo {
            id: "gemini-3.8-flash-low".to_string(),
            display_name: "Gemini 3.8 Flash (Low)".to_string(),
            max_tokens: 1_048_576,
            supports_tools: true,
            supports_thinking: false,
            description: "Low-latency fast model for simple queries and edits.".to_string(),
        },
        ModelInfo {
            id: "gemini-3.7-flash-high".to_string(),
            display_name: "Gemini 3.7 Flash (High)".to_string(),
            max_tokens: 1_048_576,
            supports_tools: true,
            supports_thinking: true,
            description: "Gemini 3.7 high-reasoning multimodal engine.".to_string(),
        },
        ModelInfo {
            id: "gemini-pro-agent".to_string(),
            display_name: "Gemini 3.1 Pro (High)".to_string(),
            max_tokens: 2_097_152,
            supports_tools: true,
            supports_thinking: true,
            description: "Deep reasoning model for massive multi-file codebases and complex refactors.".to_string(),
        },
        ModelInfo {
            id: "gemini-3.1-pro-low".to_string(),
            display_name: "Gemini 3.1 Pro (Low)".to_string(),
            max_tokens: 2_097_152,
            supports_tools: true,
            supports_thinking: false,
            description: "Pro-tier model optimized for fast responses.".to_string(),
        },
    ]
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
