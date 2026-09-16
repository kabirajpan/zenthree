use std::path::PathBuf;

/// Attempts to load the Groq API key from environment variable or local .env files
pub fn load_groq_api_key() -> Option<String> {
    if let Ok(val) = std::env::var("GROQ_API_KEY") {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    let candidates = [
        PathBuf::from(".env"),
        PathBuf::from("apps/zenthree/.env"),
        PathBuf::from("../.env"),
        PathBuf::from("../../.env"),
    ];

    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some((k, v)) = trimmed.split_once('=') {
                    if k.trim() == "GROQ_API_KEY" {
                        let key = v.trim().trim_matches('"').trim_matches('\'').trim();
                        if !key.is_empty() {
                            return Some(key.to_string());
                        }
                    }
                }
            }
        }
    }

    // Also check standard workspace absolute path if relative path doesn't resolve
    let fixed_path = PathBuf::from("/home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/.env");
    if let Ok(content) = std::fs::read_to_string(&fixed_path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some((k, v)) = trimmed.split_once('=') {
                if k.trim() == "GROQ_API_KEY" {
                    let key = v.trim().trim_matches('"').trim_matches('\'').trim();
                    if !key.is_empty() {
                        return Some(key.to_string());
                    }
                }
            }
        }
    }

    None
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

const PRIMARY_MODEL: &str = "openai/gpt-oss-120b";
const FALLBACK_MODEL: &str = "openai/gpt-oss-20b";
const TERTIARY_MODEL: &str = "qwen/qwen3.8-27b";

/// IPv4-preferred DNS resolver to prevent `Network is unreachable (os error 101)` on IPv4-only networks
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
        .timeout(std::time::Duration::from_secs(45))
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

    Err(format!("Groq API rate limit exceeded across models: {}", last_error))
}

/// Calls Groq chat completions API with system and user prompts
pub fn call_groq_api(
    api_key: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, String> {
    call_groq_api_with_model(api_key, system_prompt, user_prompt, None)
}

pub fn call_groq_api_with_model(
    api_key: &str,
    system_prompt: &str,
    user_prompt: &str,
    preferred_model: Option<&str>,
) -> Result<String, String> {
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt.to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: user_prompt.to_string(),
        },
    ];
    call_groq_api_messages_with_model(api_key, &messages, preferred_model)
}
