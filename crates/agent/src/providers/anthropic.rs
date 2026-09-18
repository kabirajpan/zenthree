//! Anthropic Claude Streaming Driver (Claude 3.7 Sonnet, Thinking & 3.5 Sonnet)
//!
//! Connects to Anthropic's Messages API over HTTP Server-Sent Events (SSE).
//! Handles extended thinking blocks (`thinking_delta`), text streaming (`text_delta`),
//! and structured tool use (`tool_use`).

use std::io::{BufRead, BufReader};
use std::time::Duration;

use serde_json::{json, Value};

use super::types::{ModelRequest, ModelResponse, ModelToolCall, ProviderRole, StreamChunk, TokenUsage};

pub const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
pub const ANTHROPIC_VERSION: &str = "2023-06-01";

pub struct AnthropicProvider {
    api_key: String,
    api_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            api_url: ANTHROPIC_API_URL.to_string(),
        }
    }

    pub fn with_custom_url(mut self, url: impl Into<String>) -> Self {
        self.api_url = url.into();
        self
    }

    /// Converts a unified ModelRequest into the Anthropic Messages API JSON payload.
    pub fn build_payload(&self, req: &ModelRequest) -> Value {
        let max_tokens = req.max_tokens.unwrap_or(8192);

        let mut anthropic_messages = Vec::new();
        for msg in &req.messages {
            let role = match msg.role {
                ProviderRole::User => "user",
                ProviderRole::Assistant => "assistant",
                ProviderRole::System => continue, // System is top-level in Anthropic
                ProviderRole::Tool => "user",
            };
            anthropic_messages.push(json!({
                "role": role,
                "content": msg.content
            }));
        }

        let mut payload = json!({
            "model": req.model,
            "max_tokens": max_tokens,
            "stream": true,
            "messages": anthropic_messages,
        });

        if let Some(ref sys) = req.system {
            payload["system"] = json!(sys);
        }

        if let Some(budget) = req.thinking_budget {
            payload["thinking"] = json!({
                "type": "enabled",
                "budget_tokens": budget
            });
        }

        if !req.tools.is_empty() {
            let mut anthropic_tools = Vec::new();
            for tool in &req.tools {
                if let Some(func) = tool.get("function") {
                    anthropic_tools.push(json!({
                        "name": func["name"],
                        "description": func.get("description").unwrap_or(&Value::Null),
                        "input_schema": func.get("parameters").unwrap_or(&Value::Null)
                    }));
                } else if let Some(name) = tool.get("name") {
                    anthropic_tools.push(json!({
                        "name": name,
                        "description": tool.get("description").unwrap_or(&Value::Null),
                        "input_schema": tool.get("inputSchema").unwrap_or(&Value::Null)
                    }));
                }
            }
            if !anthropic_tools.is_empty() {
                payload["tools"] = json!(anthropic_tools);
            }
        }

        payload
    }

    /// Streams a completion, calling `on_chunk` for each incremental delta.
    pub fn stream_completion(
        &self,
        req: &ModelRequest,
        mut on_chunk: impl FnMut(StreamChunk),
    ) -> Result<ModelResponse, String> {
        let payload = self.build_payload(req);

        let http_resp = ureq::post(&self.api_url)
            .timeout(Duration::from_secs(60))
            .set("x-api-key", &self.api_key)
            .set("anthropic-version", ANTHROPIC_VERSION)
            .set("content-type", "application/json")
            .send_json(&payload)
            .map_err(|e| format!("Anthropic API error: {}", e))?;

        let reader = BufReader::new(http_resp.into_reader());
        let mut final_content = String::new();
        let mut final_thinking = String::new();
        let mut active_tool_calls: Vec<ModelToolCall> = Vec::new();
        let mut current_tool_id = String::new();
        let mut current_tool_name = String::new();
        let mut current_tool_args = String::new();
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
                let event_type = event.get("type").and_then(|v| v.as_str()).unwrap_or("");
                match event_type {
                    "content_block_start" => {
                        if let Some(block) = event.get("content_block") {
                            let block_type = block.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            if block_type == "tool_use" {
                                current_tool_id = block.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                current_tool_name = block.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                current_tool_args.clear();
                            }
                        }
                    }
                    "content_block_delta" => {
                        if let Some(delta) = event.get("delta") {
                            let delta_type = delta.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            if delta_type == "thinking_delta" {
                                if let Some(thinking) = delta.get("thinking").and_then(|v| v.as_str()) {
                                    final_thinking.push_str(thinking);
                                    on_chunk(StreamChunk::Thinking { delta: thinking.to_string() });
                                }
                            } else if delta_type == "text_delta" {
                                if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                                    final_content.push_str(text);
                                    on_chunk(StreamChunk::Text { delta: text.to_string() });
                                }
                            } else if delta_type == "input_json_delta" {
                                if let Some(partial_json) = delta.get("partial_json").and_then(|v| v.as_str()) {
                                    current_tool_args.push_str(partial_json);
                                    on_chunk(StreamChunk::ToolCallChunk {
                                        id: current_tool_id.clone(),
                                        name: current_tool_name.clone(),
                                        arguments_delta: partial_json.to_string(),
                                    });
                                }
                            }
                        }
                    }
                    "content_block_stop" => {
                        if !current_tool_name.is_empty() {
                            let args: Value = serde_json::from_str(&current_tool_args).unwrap_or(Value::Null);
                            active_tool_calls.push(ModelToolCall {
                                id: current_tool_id.clone(),
                                name: current_tool_name.clone(),
                                arguments: args,
                            });
                            current_tool_name.clear();
                            current_tool_id.clear();
                            current_tool_args.clear();
                        }
                    }
                    "message_delta" => {
                        if let Some(u) = event.get("usage") {
                            let out_tokens = u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                            usage.completion_tokens += out_tokens;
                            usage.total_tokens = usage.prompt_tokens + usage.completion_tokens;
                            on_chunk(StreamChunk::Usage {
                                prompt_tokens: usage.prompt_tokens,
                                completion_tokens: usage.completion_tokens,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        on_chunk(StreamChunk::Done);

        Ok(ModelResponse {
            content: final_content,
            thinking: if final_thinking.is_empty() { None } else { Some(final_thinking) },
            tool_calls: active_tool_calls,
            usage,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_payload_structure() {
        let provider = AnthropicProvider::new("sk-ant-mock");
        let req = ModelRequest::new("claude-3-7-sonnet-20250219")
            .with_system("System instructions")
            .with_message(ProviderRole::User, "Hello")
            .with_thinking(4096);

        let payload = provider.build_payload(&req);
        assert_eq!(payload["model"], "claude-3-7-sonnet-20250219");
        assert_eq!(payload["system"], "System instructions");
        assert_eq!(payload["thinking"]["type"], "enabled");
        assert_eq!(payload["thinking"]["budget_tokens"], 4096);
    }
}
