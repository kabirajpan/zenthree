use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::traits::provider::Provider;
use crate::types::error::ProviderError;
use crate::types::event::ProviderEvent;
use crate::types::message::{Message, Role};

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Option<Vec<ChatChoice>>,
    error: Option<serde_json::Value>,
}

/// Simple one-shot text completion helper.
pub fn chat_completion(api_key: &str, prompt: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let req_body = ChatRequest {
        model: "openai/gpt-oss-120b".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
    };

    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key.trim()))
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()
        .map_err(|e| format!("Groq request failed: {e}"))?;

    let status = response.status();
    let resp_text = response
        .text()
        .map_err(|e| format!("Failed to read Groq response: {e}"))?;

    if !status.is_success() {
        return Err(format!("Groq API error (status {status}): {resp_text}"));
    }

    let parsed: ChatResponse = serde_json::from_str(&resp_text)
        .map_err(|e| format!("Failed to parse Groq response: {e}. Raw: {resp_text}"))?;

    if let Some(choices) = parsed.choices {
        if let Some(first) = choices.into_iter().next() {
            return Ok(first.message.content.trim().to_string());
        }
    }

    if let Some(err) = parsed.error {
        return Err(format!("Groq error response: {err}"));
    }

    Err("Empty response from Groq".to_string())
}

/// Groq LLM backend implementing the unified `Provider` trait.
pub struct GroqProvider {
    api_key: String,
    model: String,
}

impl GroqProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: "openai/gpt-oss-120b".to_string(),
        }
    }

    pub fn with_model(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
        }
    }
}

impl Provider for GroqProvider {
    fn name(&self) -> &str {
        "Groq"
    }

    fn complete_stream(
        &self,
        messages: &[Message],
        tools: &[Value],
        on_event: &mut dyn FnMut(ProviderEvent),
    ) -> Result<(), ProviderError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
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

        let mut body = json!({
            "model": self.model,
            "messages": req_messages,
        });

        if !tools.is_empty() {
            body["tools"] = Value::Array(tools.to_vec());
        }

        let response = client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key.trim()))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let status = response.status();
        let resp_text = response
            .text()
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        if !status.is_success() {
            return Err(ProviderError::Malformed(format!(
                "Status {}: {}",
                status, resp_text
            )));
        }

        let parsed: Value = serde_json::from_str(&resp_text)
            .map_err(|e| ProviderError::Malformed(format!("JSON parse error: {e}")))?;

        if let Some(choices) = parsed.get("choices").and_then(|c| c.as_array()) {
            if let Some(choice) = choices.first() {
                if let Some(message) = choice.get("message") {
                    if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                        if !content.is_empty() {
                            on_event(ProviderEvent::TextDelta(content.to_string()));
                        }
                    }

                    if let Some(tool_calls) = message.get("tool_calls").and_then(|tc| tc.as_array()) {
                        for tc in tool_calls {
                            let id = tc
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let func = tc.get("function");
                            let name = func
                                .and_then(|f| f.get("name"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let args_raw = func
                                .and_then(|f| f.get("arguments"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("{}");
                            let args: Value =
                                serde_json::from_str(args_raw).unwrap_or(json!({}));

                            on_event(ProviderEvent::ToolCallComplete { id, name, args });
                        }
                    }
                }
            }
        }

        on_event(ProviderEvent::Done);
        Ok(())
    }
}
