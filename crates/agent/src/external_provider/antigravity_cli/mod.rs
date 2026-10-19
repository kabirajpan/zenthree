pub mod auth;
pub mod models;
pub mod provider;

use std::sync::Arc;

pub use auth::{AntigravityAuthManager, OAuthToken};
pub use models::{available_models, get_model_info, ModelInfo, DEFAULT_MODEL_ID};
pub use super::acp::AcpClient as AntigravityClient;
pub use provider::AntigravityCliProvider;

/// Factory function to construct an `AntigravityCliProvider` ready for execution.
/// If `model_id` is None, defaults to `gemini-3.8-flash-high`.
pub fn create_antigravity_provider(model_id: Option<&str>) -> AntigravityCliProvider {
    let auth = Arc::new(AntigravityAuthManager::with_default_config());
    let model = model_id.unwrap_or(DEFAULT_MODEL_ID);
    AntigravityCliProvider::with_model(auth, model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_status() {
        let auth = AntigravityAuthManager::with_default_config();
        let _ = auth.load_token();
    }
}
