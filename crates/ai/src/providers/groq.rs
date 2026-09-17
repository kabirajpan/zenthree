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
        .timeout(std::time::Duration::from_secs(15))
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
                    if err_msg.contains("429") || err_msg.contains("Rate limit") || err_msg.contains("rate_limit") {
                        last_error = format!("Rate limit (429) on {}: {}", model, err_msg);
                        continue;
                    }
                    return Err(format!("Groq error: {}", err_msg));
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
                    return Err("Invalid choices format from Groq API".to_string());
                }
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("429") {
                    last_error = format!("HTTP 429 rate limit on {}: {}", model, err_str);
                    continue;
                }
                return Err(format!("Groq API request error: {}", err_str));
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
