use crate::types::ChatMessage;

/// IPv4-preferred DNS resolver to prevent `Network is unreachable (os error 101)` on IPv4-only networks
fn ipv4_resolver(netloc: &str) -> std::io::Result<Vec<std::net::SocketAddr>> {
    use std::net::ToSocketAddrs;
    let mut addrs: Vec<std::net::SocketAddr> = netloc.to_socket_addrs()?.collect();
    addrs.sort_by_key(|a| if a.is_ipv4() { 0 } else { 1 });
    Ok(addrs)
}

pub fn call_gemini_api(
    api_key: &str,
    system_prompt: &str,
    user_prompt: &str,
    history: &[ChatMessage],
) -> Result<String, String> {
    call_gemini_api_with_model(api_key, system_prompt, user_prompt, history, None)
}

pub fn call_gemini_api_with_model(
    api_key: &str,
    system_prompt: &str,
    user_prompt: &str,
    history: &[ChatMessage],
    preferred_model: Option<&str>,
) -> Result<String, String> {
    let mut models = vec![
        "gemini-3.6-flash",
        "gemini-3.1-flash-lite-preview",
        "gemini-flash-latest",
        "gemini-flash-lite-latest",
        "gemini-2.5-pro",
        "gemini-3.5-flash",
        "gemma-4-31b-it",
    ];

    if let Some(pref) = preferred_model {
        let mut trimmed = pref.trim();
        if trimmed == "gemini-2.5-flash" || trimmed == "gemini-3.5-flash" {
            trimmed = "gemini-3.6-flash";
        }
        if !trimmed.is_empty() {
            if let Some(pos) = models.iter().position(|m| *m == trimmed) {
                let m = models.remove(pos);
                models.insert(0, m);
            } else {
                models.insert(0, trimmed);
            }
        }
    }

    let mut last_error = String::new();
    let agent = ureq::builder()
        .resolver(ipv4_resolver)
        .timeout(std::time::Duration::from_secs(12))
        .build();

    for model in &models {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, api_key
        );

        let mut contents = Vec::new();

        for turn in history {
            let role = if turn.role == "assistant" || turn.role == "model" {
                "model"
            } else {
                "user"
            };
            contents.push(serde_json::json!({
                "role": role,
                "parts": [{ "text": turn.content }]
            }));
        }

        contents.push(serde_json::json!({
            "role": "user",
            "parts": [{ "text": user_prompt }]
        }));

        let payload = serde_json::json!({
            "system_instruction": {
                "parts": [{ "text": system_prompt }]
            },
            "contents": contents,
            "generationConfig": {
                "temperature": 0.3,
                "maxOutputTokens": 4096
            }
        });

        let resp = agent.post(&url)
            .set("Content-Type", "application/json")
            .set("X-goog-api-key", api_key)
            .send_json(payload);

        match resp {
            Ok(r) => {
                let json: serde_json::Value = match r.into_json() {
                    Ok(j) => j,
                    Err(e) => {
                        last_error = format!("Failed to parse Gemini response JSON: {}", e);
                        continue;
                    }
                };

                if let Some(err) = json.get("error") {
                    let msg = err["message"].as_str().unwrap_or("Unknown Gemini error");
                    let code = err.get("code").and_then(|c| c.as_i64()).unwrap_or(0);
                    if code == 429 || msg.contains("429") || msg.contains("quota") || msg.contains("Quota") {
                        last_error = format!("Gemini quota limit (429) on {}: {}", model, msg);
                    } else {
                        last_error = format!("Gemini error on {}: {}", model, msg);
                    }
                    continue;
                }

                if let Some(candidates) = json.get("candidates").and_then(|c| c.as_array()) {
                    if let Some(first) = candidates.first() {
                        if let Some(text) = first["content"]["parts"][0]["text"].as_str() {
                            return Ok(text.to_string());
                        }
                    }
                }

                last_error = format!("Gemini {} returned unexpected response format", model);
            }
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                if code == 404 {
                    last_error = format!("Gemini model `{}` not found (404).", model);
                } else if code == 429 {
                    last_error = format!("Gemini quota limit reached (429) on {}: {}", model, body);
                } else {
                    last_error = format!("Gemini API error ({}): {}", code, body);
                }
            }
            Err(e) => {
                last_error = format!("Gemini connection error on {}: {}", model, e);
            }
        }
    }

    Err(last_error)
}

use super::types::{ModelRequest, ModelResponse, ModelToolCall, ProviderRole, StreamChunk, TokenUsage};
use std::io::{BufRead, BufReader};

pub struct GeminiProvider {
    api_key: String,
    preferred_model: String,
}

impl GeminiProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            preferred_model: "gemini-2.5-flash".to_string(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.preferred_model = model.into();
        self
    }

    /// Converts unified ModelRequest into Gemini REST format.
    pub fn build_payload(&self, req: &ModelRequest) -> serde_json::Value {
        let mut contents = Vec::new();
        for msg in &req.messages {
            let role = match msg.role {
                ProviderRole::Assistant => "model",
                _ => "user",
            };
            contents.push(serde_json::json!({
                "role": role,
                "parts": [{ "text": msg.content }]
            }));
        }

        let mut payload = serde_json::json!({
            "contents": contents,
            "generationConfig": {
                "temperature": req.temperature.unwrap_or(0.3),
                "maxOutputTokens": req.max_tokens.unwrap_or(8192)
            }
        });

        if let Some(ref sys) = req.system {
            payload["system_instruction"] = serde_json::json!({
                "parts": [{ "text": sys }]
            });
        }

        if !req.tools.is_empty() {
            let mut function_declarations = Vec::new();
            for t in &req.tools {
                if let Some(f) = t.get("function") {
                    function_declarations.push(f.clone());
                } else {
                    function_declarations.push(t.clone());
                }
            }
            payload["tools"] = serde_json::json!([{
                "function_declarations": function_declarations
            }]);
        }

        payload
    }

    /// Streams completion using Google Gemini streamGenerateContent SSE endpoint.
    pub fn stream_completion(
        &self,
        req: &ModelRequest,
        mut on_chunk: impl FnMut(StreamChunk),
    ) -> Result<ModelResponse, String> {
        let model = if req.model.is_empty() { &self.preferred_model } else { &req.model };
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
            model, self.api_key
        );

        let payload = self.build_payload(req);

        let agent = ureq::builder()
            .resolver(ipv4_resolver)
            .timeout(std::time::Duration::from_secs(60))
            .build();

        let resp = agent.post(&url)
            .set("Content-Type", "application/json")
            .send_json(payload)
            .map_err(|e| format!("Gemini streaming connection error: {}", e))?;

        let reader = BufReader::new(resp.into_reader());
        let mut final_content = String::new();
        let mut tool_calls = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Error reading Gemini SSE line: {}", e))?;
            let trimmed = line.trim();
            if !trimmed.starts_with("data:") {
                continue;
            }
            let data_str = trimmed["data:".len()..].trim();
            if data_str.is_empty() || data_str == "[DONE]" {
                continue;
            }

            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data_str) {
                if let Some(candidates) = val.get("candidates").and_then(|c| c.as_array()) {
                    if let Some(candidate) = candidates.first() {
                        if let Some(parts) = candidate.get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array()) {
                            for part in parts {
                                if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                    final_content.push_str(text);
                                    on_chunk(StreamChunk::Text { delta: text.to_string() });
                                }
                                if let Some(func_call) = part.get("functionCall") {
                                    let name = func_call["name"].as_str().unwrap_or("").to_string();
                                    let args = func_call.get("args").cloned().unwrap_or(serde_json::Value::Null);
                                    tool_calls.push(ModelToolCall {
                                        id: format!("call_{}", name),
                                        name: name.clone(),
                                        arguments: args.clone(),
                                    });
                                    on_chunk(StreamChunk::ToolCallChunk {
                                        id: format!("call_{}", name),
                                        name,
                                        arguments_delta: serde_json::to_string(&args).unwrap_or_default(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        on_chunk(StreamChunk::Done);

        Ok(ModelResponse {
            content: final_content,
            thinking: None,
            tool_calls,
            usage: TokenUsage::default(),
        })
    }
}
