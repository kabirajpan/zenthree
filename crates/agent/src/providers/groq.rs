use crate::types::ChatMessage;

pub const PRIMARY_MODEL: &str = "openai/gpt-oss-120b";
pub const FALLBACK_MODEL: &str = "openai/gpt-oss-20b";
pub const TERTIARY_MODEL: &str = "qwen/qwen3.8-27b";

fn ipv4_resolver(netloc: &str) -> std::io::Result<Vec<std::net::SocketAddr>> {
    use std::net::ToSocketAddrs;
    let mut addrs: Vec<std::net::SocketAddr> = netloc.to_socket_addrs()?.collect();
    addrs.sort_by_key(|a| if a.is_ipv4() { 0 } else { 1 });
    Ok(addrs)
}

pub fn call_groq_api_messages(
    api_key: &str,
    messages: &[ChatMessage],
) -> Result<String, String> {
    call_groq_api_messages_with_model(api_key, messages, None)
}

pub fn call_groq_api_messages_with_model(
    api_key: &str,
    messages: &[ChatMessage],
    preferred_model: Option<&str>,
) -> Result<String, String> {
    let url = "https://api.groq.com/openai/v1/chat/completions";

    let mut models = vec![PRIMARY_MODEL, FALLBACK_MODEL, TERTIARY_MODEL];
    if let Some(pref) = preferred_model {
        let trimmed = pref.trim();
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
        .timeout(std::time::Duration::from_secs(30))
        .build();

    for model in &models {
        let payload = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": 0.3,
            "max_tokens": 4096
        });

        let resp = agent.post(url)
            .set("Authorization", &format!("Bearer {}", api_key))
            .set("Content-Type", "application/json")
            .send_json(payload);

        match resp {
            Ok(r) => {
                let json: serde_json::Value = match r.into_json() {
                    Ok(j) => j,
                    Err(e) => {
                        last_error = format!("Failed to parse Groq response JSON: {}", e);
                        continue;
                    }
                };

                if let Some(err) = json.get("error") {
                    let err_msg = err["message"].as_str().unwrap_or("Unknown error").to_string();
                    last_error = format!("Groq error on {}: {}", model, err_msg);
                    continue;
                }

                if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                    if !content.trim().is_empty() {
                        return Ok(content.to_string());
                    }
                }
                if let Some(reasoning) = json["choices"][0]["message"]["reasoning"].as_str() {
                    if !reasoning.trim().is_empty() {
                        return Ok(reasoning.to_string());
                    }
                }
                if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                    return Ok(content.to_string());
                } else {
                    last_error = format!("Invalid choices format from Groq API on {}", model);
                    continue;
                }
            }
            Err(e) => {
                let err_str = e.to_string();
                last_error = format!("Groq API error on {}: {}", model, err_str);
                continue;
            }
        }
    }

    Err(format!("All Groq models exhausted. Last error: {}", last_error))
}

pub fn call_groq_api(
    api_key: &str,
    system_prompt: &str,
    user_prompt: &str,
    history: &[ChatMessage],
) -> Result<String, String> {
    call_groq_api_with_model(api_key, system_prompt, user_prompt, history, None)
}

pub fn call_groq_api_with_model(
    api_key: &str,
    system_prompt: &str,
    user_prompt: &str,
    history: &[ChatMessage],
    preferred_model: Option<&str>,
) -> Result<String, String> {
    let mut messages = Vec::new();
    if !system_prompt.trim().is_empty() {
        messages.push(ChatMessage::system(system_prompt));
    }
    for turn in history {
        messages.push(turn.clone());
    }
    messages.push(ChatMessage::user(user_prompt));
    call_groq_api_messages_with_model(api_key, &messages, preferred_model)
}

use super::openai::OpenAiProvider;
use super::types::{ModelRequest, ModelResponse, StreamChunk};

pub const GROQ_API_URL: &str = "https://api.groq.com/openai/v1/chat/completions";
pub const DEFAULT_GROQ_MODEL: &str = "llama-3.3-70b-versatile";

pub struct GroqProvider {
    inner: OpenAiProvider,
    default_model: String,
}

impl GroqProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        let inner = OpenAiProvider::new(api_key).with_custom_url(GROQ_API_URL);
        Self {
            inner,
            default_model: DEFAULT_GROQ_MODEL.to_string(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }

    /// Streams completion from Groq high-speed inference.
    pub fn stream_completion(
        &self,
        req: &ModelRequest,
        on_chunk: impl FnMut(StreamChunk),
    ) -> Result<ModelResponse, String> {
        let mut request = req.clone();
        if request.model.is_empty() {
            request.model = self.default_model.clone();
        }
        self.inner.stream_completion(&request, on_chunk)
    }
}
