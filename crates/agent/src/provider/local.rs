use serde_json::{json, Value};
use crate::local_model::{find_catalog_model, LocalModelManager};
use crate::provider::config::ZenthreeConfig;
use crate::traits::provider::Provider;
use crate::types::error::ProviderError;
use crate::types::event::ProviderEvent;
use crate::types::message::{Message, Role};

/// Standard ports and paths for local LLM engines
#[allow(dead_code)]
const KNOWN_LOCAL_ENDPOINTS: &[&str] = &[
    "http://127.0.0.1:11434/v1", // Ollama
    "http://localhost:11434/v1",
    "http://127.0.0.1:8080/v1",  // llama-server / llama.cpp
    "http://localhost:8080/v1",
    "http://127.0.0.1:1234/v1",  // LM Studio
    "http://localhost:1234/v1",
    "http://127.0.0.1:5000/v1",  // LocalAI / vLLM
];



/// Local model LLM backend implementing the unified `Provider` trait.
pub struct LocalProvider {
    pub model: String,
    pub base_url: Option<String>,
}

impl LocalProvider {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            base_url: None,
        }
    }

    pub fn with_endpoint(model: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            base_url: Some(base_url.into()),
        }
    }

    /// Detect active local endpoint or fallback to primary default
    pub fn resolve_endpoint(&self) -> String {
        // 1. Explicit base URL if configured on the provider
        if let Some(ref url) = self.base_url {
            let trimmed = url.trim().trim_end_matches('/');
            if !trimmed.is_empty() {
                return if trimmed.ends_with("/v1") {
                    trimmed.to_string()
                } else {
                    format!("{}/v1", trimmed)
                };
            }
        }

        // 2. Base URL from ~/.zenthree/config.json under "local"
        let cfg = ZenthreeConfig::load();
        if let Some(entry) = cfg.providers.get("local") {
            if let Some(ref url) = entry.base_url {
                let trimmed = url.trim().trim_end_matches('/');
                if !trimmed.is_empty() {
                    return if trimmed.ends_with("/v1") {
                        trimmed.to_string()
                    } else {
                        format!("{}/v1", trimmed)
                    };
                }
            }
        }

        // 3. Probe known local ports (8080, 11434, 1234, 5000)
        if let Some(ep) = crate::local_model::LocalServerManager::probe_active_endpoint() {
            return ep;
        }

        // 4. If no server is running, attempt auto-starting local llama-server
        if let Ok(ep) = crate::local_model::LocalServerManager::ensure_server_running(&self.model) {
            return ep;
        }

        // 5. Fallback port
        if crate::local_model::LocalServerManager::is_runner_installed() {
            "http://127.0.0.1:8080/v1".to_string()
        } else {
            "http://127.0.0.1:11434/v1".to_string()
        }
    }

    /// Map catalog model ID or file to the name expected by the local server
    pub fn resolve_server_model_name(&self) -> String {
        let m = self.model.trim();

        // 1. If it's a catalog ID, prefer the ollama_tag or catalog id
        if let Some(cat) = find_catalog_model(m) {
            return cat.ollama_tag.to_string();
        }

        // 2. If it's a downloaded file on disk, return filename or tag
        if let Some(path) = LocalModelManager::get_model_path(m) {
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if let Some(cat) = find_catalog_model(filename) {
                    return cat.ollama_tag.to_string();
                }
                return filename.to_string();
            }
        }

        // 3. Fallback to raw string (e.g. "qwen2.5-coder:1.5b", "llama3.2:3b", etc.)
        m.to_string()
    }

    /// Helper to test if any local model server is reachable
    pub fn test_connection(&self) -> Result<String, String> {
        let endpoint = self.resolve_endpoint();
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
            .map_err(|e| e.to_string())?;

        let models_url = format!("{}/models", endpoint.trim_end_matches('/'));
        match client.get(&models_url).send() {
            Ok(resp) if resp.status().is_success() => {
                Ok(format!("Connected successfully to local server at {}", endpoint))
            }
            Ok(resp) => {
                Err(format!("Local server responded with status: {}", resp.status()))
            }
            Err(e) => {
                Err(format!("Could not connect to {}: {}", endpoint, e))
            }
        }
    }
}

impl Provider for LocalProvider {
    fn name(&self) -> &str {
        "Local Model"
    }

    fn complete_stream(
        &self,
        messages: &[Message],
        tools: &[Value],
        on_event: &mut dyn FnMut(ProviderEvent),
    ) -> Result<(), ProviderError> {
        let endpoint = self.resolve_endpoint();
        let server_model = self.resolve_server_model_name();

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let mut req_messages = Vec::new();
        for m in messages {
            let role_str = match m.role {
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::Tool => "tool",
                Role::System => "system",
            };

            let mut msg_obj = serde_json::Map::new();
            msg_obj.insert("role".to_string(), json!(role_str));
            msg_obj.insert("content".to_string(), json!(m.content));

            if let Some(tool_calls) = &m.tool_calls {
                let calls_json: Vec<Value> = tool_calls
                    .iter()
                    .map(|c| {
                        json!({
                            "id": c.id,
                            "type": "function",
                            "function": {
                                "name": c.name,
                                "arguments": c.args.to_string()
                            }
                        })
                    })
                    .collect();
                msg_obj.insert("tool_calls".to_string(), Value::Array(calls_json));
            }

            if let Some(tool_call_id) = &m.tool_call_id {
                msg_obj.insert("tool_call_id".to_string(), json!(tool_call_id));
            }

            req_messages.push(Value::Object(msg_obj));
        }

        // Ensure at least one user message
        let has_user_msg = req_messages
            .iter()
            .any(|m| m.get("role").and_then(|r| r.as_str()) == Some("user"));
        if !has_user_msg {
            let mut fallback = serde_json::Map::new();
            fallback.insert("role".to_string(), json!("user"));
            fallback.insert(
                "content".to_string(),
                json!("Proceed with the task using the provided context."),
            );
            let insert_pos = if req_messages
                .first()
                .and_then(|m| m.get("role"))
                .and_then(|r| r.as_str())
                == Some("system")
            {
                1
            } else {
                0
            };
            req_messages.insert(insert_pos, Value::Object(fallback));
        }

        // Optimize system prompt size for local model context windows
        for msg in &mut req_messages {
            if msg.get("role").and_then(|r| r.as_str()) == Some("system") {
                if let Some(c) = msg.get("content").and_then(|c| c.as_str()) {
                    if c.len() > 1500 {
                        msg["content"] = json!(
                            "You are a helpful and concise local AI coding assistant. Provide accurate, direct answers and clean code."
                        );
                    }
                }
            }
        }

        let mut body = json!({
            "model": server_model,
            "messages": req_messages,
            "stream": true,
        });

        // For small local models (<= 3B), limit tool schemas to prevent context thrashing
        if !tools.is_empty() {
            let is_small_model = self.model.contains("1.5b") || self.model.contains("1.7b") || self.model.contains("3b");
            if !is_small_model {
                body["tools"] = Value::Array(tools.to_vec());
            } else if tools.len() <= 4 {
                body["tools"] = Value::Array(tools.to_vec());
            }
        }

        let url = format!("{}/chat/completions", endpoint.trim_end_matches('/'));

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .json(&body)
            .send()
            .map_err(|e| {
                let model_path = LocalModelManager::get_model_path(&self.model);
                let path_hint = if let Some(p) = model_path {
                    format!("\nDownloaded model file: {}", p.display())
                } else {
                    String::new()
                };

                let runner_hint = if crate::local_model::LocalServerManager::is_runner_installed() {
                    "• Built-in runner is ready: Click 'Start Server' in AI Settings > Local Models\n"
                } else {
                    ""
                };

                ProviderError::Network(format!(
                    "Failed to connect to local model server at {url}.\n\
                    Error: {e}\n\
                    {runner_hint}\
                    Please start a local inference server:\n\
                    • Ollama: run 'ollama run {server_model}' (or 'ollama serve')\n\
                    • llama.cpp: run 'llama-server -m <path-to-model.gguf> --port 8080'\n\
                    • LM Studio: start local server on port 1234{path_hint}"
                ))
            })?;

        let status = response.status();
        if !status.is_success() {
            let resp_text = response
                .text()
                .unwrap_or_else(|_| "Unknown server error".to_string());
            return Err(ProviderError::Malformed(format!(
                "Local model server returned status {}: {}",
                status, resp_text
            )));
        }

        // Stream tokens in real time via Server-Sent Events (SSE)
        use std::io::BufRead;
        let reader = std::io::BufReader::new(response);
        let mut full_accumulated = String::new();

        for line_res in reader.lines() {
            let line = match line_res {
                Ok(l) => l,
                Err(_) => break,
            };

            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with(':') {
                continue;
            }

            if let Some(payload) = trimmed.strip_prefix("data: ") {
                let payload = payload.trim();
                if payload == "[DONE]" {
                    break;
                }

                if let Ok(chunk) = serde_json::from_str::<Value>(payload) {
                    if let Some(choices) = chunk.get("choices").and_then(|c| c.as_array()) {
                        if let Some(choice) = choices.first() {
                            let delta = choice.get("delta");
                            // 1. Text delta
                            if let Some(text) = delta.and_then(|d| d.get("content")).and_then(|c| c.as_str()) {
                                if !text.is_empty() {
                                    full_accumulated.push_str(text);
                                    on_event(ProviderEvent::TextDelta(text.to_string()));
                                }
                            }
                            // 2. Reasoning / Thinking delta (DeepSeek-R1 style)
                            if let Some(thought) = delta.and_then(|d| d.get("reasoning_content")).and_then(|c| c.as_str()) {
                                if !thought.is_empty() {
                                    on_event(ProviderEvent::ThoughtDelta(thought.to_string()));
                                }
                            }
                        }
                    }
                }
            } else if trimmed.starts_with('{') {
                // Fallback for non-SSE JSON response
                if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
                    if let Some(choices) = parsed.get("choices").and_then(|c| c.as_array()) {
                        if let Some(choice) = choices.first() {
                            if let Some(content) = choice.get("message").and_then(|m| m.get("content")).and_then(|c| c.as_str()) {
                                if !content.is_empty() {
                                    on_event(ProviderEvent::TextDelta(content.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }

        on_event(ProviderEvent::Done);
        Ok(())
    }
}
