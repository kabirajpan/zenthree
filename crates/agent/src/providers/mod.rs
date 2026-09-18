//! Multi-Model Streaming Providers Module
//!
//! Provides unified streaming model drivers with automatic retry, backoff,
//! and error categorization across Anthropic, OpenAI, Google Gemini, Groq,
//! and local offline runners (Ollama / Llama.cpp).

pub mod types;
pub mod errors;
pub mod keys;
pub mod anthropic;
pub mod openai;
pub mod gemini;
pub mod groq;
pub mod local;
pub mod chain;

pub use types::{
    ModelRequest, ModelResponse, ModelToolCall, ProviderMessage, ProviderRole, StreamChunk,
    TokenUsage,
};
pub use errors::{classify_provider_error, ProviderError};
pub use keys::{load_anthropic_api_key, load_gemini_api_key, load_groq_api_key, load_openai_api_key};
pub use anthropic::AnthropicProvider;
pub use openai::OpenAiProvider;
pub use gemini::{call_gemini_api, call_gemini_api_with_model, GeminiProvider};
pub use groq::{
    call_groq_api, call_groq_api_messages, call_groq_api_messages_with_model,
    call_groq_api_with_model, GroqProvider,
};
pub use local::LocalProvider;
pub use chain::ProviderChain;

use std::collections::HashMap;
use std::time::Duration;

/// Unified Trait for all Streaming Model Drivers
pub trait ModelProvider: Send + Sync {
    /// Dispatches a model completion request, invoking `on_chunk` as tokens arrive.
    fn stream(
        &self,
        req: &ModelRequest,
        on_chunk: &mut dyn FnMut(StreamChunk),
    ) -> Result<ModelResponse, String>;
}

impl ModelProvider for AnthropicProvider {
    fn stream(
        &self,
        req: &ModelRequest,
        on_chunk: &mut dyn FnMut(StreamChunk),
    ) -> Result<ModelResponse, String> {
        self.stream_completion(req, on_chunk)
    }
}

impl ModelProvider for OpenAiProvider {
    fn stream(
        &self,
        req: &ModelRequest,
        on_chunk: &mut dyn FnMut(StreamChunk),
    ) -> Result<ModelResponse, String> {
        self.stream_completion(req, on_chunk)
    }
}

impl ModelProvider for GeminiProvider {
    fn stream(
        &self,
        req: &ModelRequest,
        on_chunk: &mut dyn FnMut(StreamChunk),
    ) -> Result<ModelResponse, String> {
        self.stream_completion(req, on_chunk)
    }
}

impl ModelProvider for GroqProvider {
    fn stream(
        &self,
        req: &ModelRequest,
        on_chunk: &mut dyn FnMut(StreamChunk),
    ) -> Result<ModelResponse, String> {
        self.stream_completion(req, on_chunk)
    }
}

impl ModelProvider for LocalProvider {
    fn stream(
        &self,
        req: &ModelRequest,
        on_chunk: &mut dyn FnMut(StreamChunk),
    ) -> Result<ModelResponse, String> {
        self.stream_completion(req, on_chunk)
    }
}

/// Dynamic Provider Registry with automatic fallback and retry
#[derive(Default)]
pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn ModelProvider>>,
    fallback_order: Vec<String>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a provider instance under a given ID (e.g. "anthropic", "openai").
    pub fn register(&mut self, id: impl Into<String>, provider: Box<dyn ModelProvider>) {
        let id_str = id.into();
        self.providers.insert(id_str.clone(), provider);
        if !self.fallback_order.contains(&id_str) {
            self.fallback_order.push(id_str);
        }
    }

    /// Sets explicit fallback priority order (e.g. `["anthropic", "gemini", "groq"]`).
    pub fn set_fallback_order(&mut self, order: Vec<String>) {
        self.fallback_order = order;
    }

    /// Streams completion with automatic exponential backoff retry on HTTP 429/503.
    pub fn stream_with_retry(
        &self,
        provider_id: &str,
        req: &ModelRequest,
        max_retries: usize,
        on_chunk: &mut dyn FnMut(StreamChunk),
    ) -> Result<ModelResponse, String> {
        let provider = self.providers.get(provider_id).ok_or_else(|| {
            format!("Provider '{}' not found in ProviderRegistry", provider_id)
        })?;

        let mut attempts = 0;
        let mut backoff = Duration::from_millis(500);

        loop {
            attempts += 1;
            match provider.stream(req, on_chunk) {
                Ok(resp) => return Ok(resp),
                Err(err) => {
                    let is_rate_limit = err.contains("429") || err.contains("quota") || err.contains("rate_limit");
                    let is_overloaded = err.contains("503") || err.contains("overloaded");

                    if (is_rate_limit || is_overloaded) && attempts < max_retries {
                        log::warn!(
                            "Provider '{}' hit retryable error: {}. Retrying in {:?} (attempt {}/{})",
                            provider_id, err, backoff, attempts, max_retries
                        );
                        std::thread::sleep(backoff);
                        backoff *= 2;
                        continue;
                    }

                    return Err(err);
                }
            }
        }
    }

    /// Streams completion trying configured providers in priority fallback order.
    pub fn stream_with_fallback(
        &self,
        req: &ModelRequest,
        on_chunk: &mut dyn FnMut(StreamChunk),
    ) -> Result<ModelResponse, String> {
        let mut last_error = String::new();

        for provider_id in &self.fallback_order {
            if let Some(provider) = self.providers.get(provider_id) {
                match provider.stream(req, on_chunk) {
                    Ok(resp) => return Ok(resp),
                    Err(e) => {
                        log::warn!("Provider '{}' failed: {}. Falling back...", provider_id, e);
                        last_error = e;
                    }
                }
            }
        }

        Err(format!("All providers exhausted. Last error: {}", last_error))
    }
}
