//! OpenAI Streaming Driver (GPT-4o, o1, o3-mini)
//!
//! Connects to OpenAI Chat Completions API over HTTP Server-Sent Events (SSE).
//! Handles token streaming, reasoning deltas, and parallel tool calling.

use std::io::{BufRead, BufReader};
use std::time::Duration;

use serde_json::{json, Value};

use super::types::{ModelRequest, ModelResponse, ModelToolCall, ProviderRole, StreamChunk, TokenUsage};

pub const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

pub struct OpenAiProvider {
    api_key: String,
    api_url: String,
}

impl OpenAiProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            api_url: OPENAI_API_URL.to_string(),
        }
    }

    pub fn with_custom_url(mut self, url: impl Into<String>) -> Self {
        self.api_url = url.into();
        self
    }

    /// Builds the standard OpenAI Chat Completions JSON request payload.
    pub fn build_payload(&self, req: &ModelRequest) -> Value {
        let mut messages = Vec::new();

        if let Some(ref sys) = req.system {
            messages.push(json!({
                "role": "system",
                "content": sys
            }));
        }

        for msg in &req.messages {
            let role = match msg.role {
                ProviderRole::System => "system",
                ProviderRole::User => "user",
                ProviderRole::Assistant => "assistant",
                ProviderRole::Tool => "tool",
            };

            let mut msg_obj = json!({
                "role": role,
                "content": msg.content
            });

            if let Some(ref call_id) = msg.tool_call_id {
                msg_obj["tool_call_id"] = json!(call_id);
            }

            messages.push(msg_obj);
        }

        let mut payload = json!({
            "model": req.model,
            "stream": true,
            "messages": messages,
        });

        if let Some(temp) = req.temperature {
            payload["temperature"] = json!(temp);
        }

        if let Some(max) = req.max_tokens {
            payload["max_tokens"] = json!(max);
        }

        if !req.tools.is_empty() {
            payload["tools"] = json!(req.tools);
        }

        payload
    }

    /// Streams completion chunks from OpenAI.
    pub fn stream_completion(
        &self,
        req: &ModelRequest,
        mut on_chunk: impl FnMut(StreamChunk),
    ) -> Result<ModelResponse, String> {
        let payload = self.build_payload(req);

        let http_resp = ureq::post(&self.api_url)
            .timeout(Duration::from_secs(60))
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .send_json(&payload)
            .map_err(|e| format!("OpenAI API error: {}", e))?;

        let reader = BufReader::new(http_resp.into_reader());
        let mut final_content = String::new();
        let mut final_thinking = String::new();
        let mut active_tools: std::collections::HashMap<usize, (String, String, String)> = std::collections::HashMap::new();
        let mut usage = TokenUsage::default();

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Error reading SSE stream: {}", e))?;
            let trimmed = line.trim();
            if !trimmed.starts_with("data:") {
                continue;
            }
            let data_str = trimmed["data:".len()..].trim();
            if data_str == "[DONE]" {
                break;
            }

            if let Ok(event) = serde_json::from_str::<Value>(data_str) {
                if let Some(choices) = event.get("choices").and_then(|v| v.as_array()) {
                    if let Some(choice) = choices.first() {
                        if let Some(delta) = choice.get("delta") {
                            // Text content
                            if let Some(content) = delta.get("content").and_then(|v| v.as_str()) {
                                final_content.push_str(content);
                                on_chunk(StreamChunk::Text { delta: content.to_string() });
                            }

                            // Reasoning / Thinking (o1, o3-mini, deepseek)
                            if let Some(reasoning) = delta.get("reasoning_content").or_else(|| delta.get("reasoning")).and_then(|v| v.as_str()) {
                                final_thinking.push_str(reasoning);
                                on_chunk(StreamChunk::Thinking { delta: reasoning.to_string() });
                            }

                            // Tool calls
                            if let Some(tool_calls) = delta.get("tool_calls").and_then(|v| v.as_array()) {
                                for tc in tool_calls {
                                    let index = tc.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                                    let entry = active_tools.entry(index).or_insert_with(|| (String::new(), String::new(), String::new()));

                                    if let Some(id) = tc.get("id").and_then(|v| v.as_str()) {
                                        entry.0 = id.to_string();
                                    }
                                    if let Some(func) = tc.get("function") {
                                        if let Some(name) = func.get("name").and_then(|v| v.as_str()) {
                                            entry.1.push_str(name);
                                        }
                                        if let Some(args_delta) = func.get("arguments").and_then(|v| v.as_str()) {
                                            entry.2.push_str(args_delta);
                                            on_chunk(StreamChunk::ToolCallChunk {
                                                id: entry.0.clone(),
                                                name: entry.1.clone(),
                                                arguments_delta: args_delta.to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Usage accounting
                if let Some(u) = event.get("usage") {
                    let prompt_tokens = u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let comp_tokens = u.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    usage = TokenUsage::new(prompt_tokens, comp_tokens);
                    on_chunk(StreamChunk::Usage {
                        prompt_tokens,
                        completion_tokens: comp_tokens,
                    });
                }
            }
        }

        on_chunk(StreamChunk::Done);

        let mut tool_calls = Vec::new();
        let mut sorted_indices: Vec<usize> = active_tools.keys().copied().collect();
        sorted_indices.sort_unstable();

        for idx in sorted_indices {
            if let Some((id, name, args_str)) = active_tools.remove(&idx) {
                let args_value: Value = serde_json::from_str(&args_str).unwrap_or(Value::Null);
                tool_calls.push(ModelToolCall {
                    id,
                    name,
                    arguments: args_value,
                });
            }
        }

        Ok(ModelResponse {
            content: final_content,
            thinking: if final_thinking.is_empty() { None } else { Some(final_thinking) },
            tool_calls,
            usage,
        })
    }
}
