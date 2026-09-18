//! Local Model Provider (Ollama & Llama.cpp)
//!
//! Connects to local offline inference servers (e.g. Ollama, Llama.cpp, LM Studio, vLLM)
//! running on `localhost:11434` or custom endpoints via OpenAI-compatible SSE streaming.

use std::time::Duration;

use super::openai::OpenAiProvider;
use super::types::{ModelRequest, ModelResponse, StreamChunk};

pub const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434/v1/chat/completions";

pub struct LocalProvider {
    inner: OpenAiProvider,
    endpoint_url: String,
}

impl LocalProvider {
    /// Creates a provider pointing to a local Ollama or Llama.cpp instance.
    pub fn new(endpoint_url: Option<&str>) -> Self {
        let url = endpoint_url.unwrap_or(DEFAULT_OLLAMA_URL).to_string();
        let inner = OpenAiProvider::new("local-no-key").with_custom_url(&url);
        Self {
            inner,
            endpoint_url: url,
        }
    }

    /// Tests if the local inference runner is currently running and responsive.
    pub fn check_liveness(&self) -> bool {
        // Attempt quick ping with 500ms timeout
        let test_url = self.endpoint_url.replace("/v1/chat/completions", "/api/version");
        ureq::get(&test_url)
            .timeout(Duration::from_millis(500))
            .call()
            .is_ok()
    }

    /// Streams completion from the local model runner.
    pub fn stream_completion(
        &self,
        req: &ModelRequest,
        on_chunk: impl FnMut(StreamChunk),
    ) -> Result<ModelResponse, String> {
        self.inner.stream_completion(req, on_chunk)
    }
}
