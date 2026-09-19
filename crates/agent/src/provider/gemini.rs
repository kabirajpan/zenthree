use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::traits::provider::Provider;
use crate::types::error::ProviderError;
use crate::types::event::ProviderEvent;
use crate::types::message::{Message, Role};

// ─── Simple one-shot helper (kept for provider::complete_gemini) ─────────────

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Deserialize)]
struct GeminiPart {
    text: Option<String>,
}

#[derive(Deserialize)]
struct GeminiContent {
    parts: Option<Vec<GeminiPart>>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiContent>,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    error: Option<serde_json::Value>,
}

pub fn generate_content(api_key: &str, prompt: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key.trim()
    );

    let req_body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part {
                text: prompt.to_string(),
            }],
        }],
    };

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()
        .map_err(|e| format!("Gemini request failed: {e}"))?;

    let status = response.status();
    let resp_text = response
        .text()
        .map_err(|e| format!("Failed to read Gemini response: {e}"))?;

    if !status.is_success() {
        return Err(format!("Gemini API error (status {status}): {resp_text}"));
    }

    let parsed: GeminiResponse = serde_json::from_str(&resp_text)
        .map_err(|e| format!("Failed to parse Gemini response: {e}. Raw: {resp_text}"))?;

    if let Some(candidates) = parsed.candidates {
        if let Some(first) = candidates.into_iter().next() {
            if let Some(content) = first.content {
                if let Some(parts) = content.parts {
                    let text = parts
                        .into_iter()
                        .filter_map(|p| p.text)
                        .collect::<Vec<_>>()
                        .join("");
                    if !text.is_empty() {
                        return Ok(text.trim().to_string());
                    }
                }
            }
        }
    }

    if let Some(err) = parsed.error {
        return Err(format!("Gemini error response: {err}"));
    }

    Err("Empty response from Gemini".to_string())
}

// ─── Full Provider trait implementation with tool-calling ────────────────────

/// Default model for GeminiProvider.
/// `gemini-2.5-flash` has a 1,000,000 TPM free-tier limit and full tool-calling support.
const DEFAULT_GEMINI_MODEL: &str = "gemini-2.5-flash";

/// Gemini LLM backend implementing the unified `Provider` trait.
/// Uses the Gemini `generateContent` endpoint with `tools` + `functionDeclarations`
/// so the orchestrator can call any registered tool.
pub struct GeminiProvider {
    api_key: String,
    model: String,
}

impl GeminiProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: DEFAULT_GEMINI_MODEL.to_string(),
        }
    }

    pub fn with_model(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
        }
    }
}

impl Provider for GeminiProvider {
    fn name(&self) -> &str {
        "Gemini"
    }

    fn complete_stream(
        &self,
        messages: &[Message],
        tools: &[Value],
        on_event: &mut dyn FnMut(ProviderEvent),
    ) -> Result<(), ProviderError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model,
            self.api_key.trim()
        );

        // ── Convert messages → Gemini `contents` format ──────────────────────
        // Gemini uses role "user" / "model" (not "assistant"), and tool calls go
        // in a `functionCall` part while tool results go in a `functionResponse` part.
        let mut contents: Vec<Value> = Vec::new();

        for m in messages {
            match m.role {
                Role::System => {
                    // Gemini handles system instructions separately; prepend as a user turn
                    // so it ends up in context even without the systemInstruction field.
                    contents.push(json!({
                        "role": "user",
                        "parts": [{ "text": format!("[System]: {}", m.content) }]
                    }));
                }
                Role::User => {
                    contents.push(json!({
                        "role": "user",
                        "parts": [{ "text": m.content }]
                    }));
                }
                Role::Assistant => {
                    let mut parts: Vec<Value> = Vec::new();
                    if !m.content.is_empty() {
                        parts.push(json!({ "text": m.content }));
                    }
                    if let Some(tool_calls) = &m.tool_calls {
                        for tc in tool_calls {
                            parts.push(json!({
                                "functionCall": {
                                    "name": tc.name,
                                    "args": tc.args
                                }
                            }));
                        }
                    }
                    if !parts.is_empty() {
                        contents.push(json!({ "role": "model", "parts": parts }));
                    }
                }
                Role::Tool => {
                    // Gemini expects tool results as a "user" turn with functionResponse parts
                    contents.push(json!({
                        "role": "user",
                        "parts": [{
                            "functionResponse": {
                                "name": m.tool_call_id.as_deref().unwrap_or("tool"),
                                "response": { "output": m.content }
                            }
                        }]
                    }));
                }
            }
        }

        // ── Build request body ────────────────────────────────────────────────
        let mut body = json!({ "contents": contents });

        // Convert OpenAI-style tool schemas → Gemini functionDeclarations
        if !tools.is_empty() {
            let function_declarations: Vec<Value> = tools
                .iter()
                .filter_map(|t| {
                    // OpenAI schema: { "type": "function", "function": { "name", "description", "parameters" } }
                    let func = t.get("function")?;
                    Some(json!({
                        "name": func.get("name")?,
                        "description": func.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                        "parameters": func.get("parameters").cloned().unwrap_or(json!({}))
                    }))
                })
                .collect();

            if !function_declarations.is_empty() {
                body["tools"] = json!([{ "functionDeclarations": function_declarations }]);
            }
        }

        // ── Send request with 429 retry ───────────────────────────────────────
        const MAX_RETRIES: u32 = 3;
        let mut attempts = 0;
        let (status, resp_text) = loop {
            let response = client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .map_err(|e| ProviderError::Network(e.to_string()))?;

            let status = response.status();
            let retry_secs: u64 = if status.as_u16() == 429 {
                response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(|f| (f.ceil() as u64).max(1))
                    .unwrap_or(8)
            } else {
                0
            };

            let resp_text = response
                .text()
                .map_err(|e| ProviderError::Network(e.to_string()))?;

            if status.as_u16() == 429 && attempts < MAX_RETRIES {
                attempts += 1;
                std::thread::sleep(std::time::Duration::from_secs(retry_secs));
                continue;
            }

            break (status, resp_text);
        };

        if !status.is_success() {
            return Err(ProviderError::Malformed(format!(
                "Gemini API error (status {}): {}",
                status, resp_text
            )));
        }

        // ── Parse response ────────────────────────────────────────────────────
        let parsed: Value = serde_json::from_str(&resp_text)
            .map_err(|e| ProviderError::Malformed(format!("JSON parse error: {e}")))?;

        if let Some(candidates) = parsed.get("candidates").and_then(|c| c.as_array()) {
            if let Some(candidate) = candidates.first() {
                if let Some(parts) = candidate
                    .get("content")
                    .and_then(|c| c.get("parts"))
                    .and_then(|p| p.as_array())
                {
                    for part in parts {
                        // Plain text delta
                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                            if !text.is_empty() {
                                on_event(ProviderEvent::TextDelta(text.to_string()));
                            }
                        }

                        // Function/tool call
                        if let Some(fc) = part.get("functionCall") {
                            let name = fc
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let args = fc.get("args").cloned().unwrap_or(json!({}));
                            // Use the function name as the call ID (Gemini doesn't give one)
                            let id = name.clone();
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
