use std::path::PathBuf;

/// Attempts to load the Google Gemini API key from environment variable or local .env files
pub fn load_gemini_api_key() -> Option<String> {
    if let Ok(val) = std::env::var("GEMINI_API_KEY") {
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
                    if k.trim() == "GEMINI_API_KEY" {
                        let key = v.trim().trim_matches('"').trim_matches('\'').trim();
                        if !key.is_empty() {
                            return Some(key.to_string());
                        }
                    }
                }
            }
        }
    }

    let fixed_paths = [
        PathBuf::from("/home/kabir/Desktop/projects/personal/framworks/zenthra-v2/.env"),
        PathBuf::from("/home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/.env"),
    ];
    for fixed_path in &fixed_paths {
        if let Ok(content) = std::fs::read_to_string(fixed_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some((k, v)) = trimmed.split_once('=') {
                    if k.trim() == "GEMINI_API_KEY" {
                        let key = v.trim().trim_matches('"').trim_matches('\'').trim();
                        if !key.is_empty() {
                            return Some(key.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

/// IPv4-preferred DNS resolver to prevent `Network is unreachable (os error 101)` on IPv4-only networks
fn ipv4_resolver(netloc: &str) -> std::io::Result<Vec<std::net::SocketAddr>> {
    use std::net::ToSocketAddrs;
    let mut addrs: Vec<std::net::SocketAddr> = netloc.to_socket_addrs()?.collect();
    addrs.sort_by_key(|a| if a.is_ipv4() { 0 } else { 1 });
    Ok(addrs)
}

/// Calls Google Gemini API with system instructions, user prompt, and multi-turn history
pub fn call_gemini_api(
    api_key: &str,
    system_prompt: &str,
    user_prompt: &str,
    history: &[crate::groq::ChatMessage],
) -> Result<String, String> {
    call_gemini_api_with_model(api_key, system_prompt, user_prompt, history, None)
}

pub fn call_gemini_api_with_model(
    api_key: &str,
    system_prompt: &str,
    user_prompt: &str,
    history: &[crate::groq::ChatMessage],
    preferred_model: Option<&str>,
) -> Result<String, String> {
    // Model preference: Google Gemini models verified live for this API key:
    let mut models = vec![
        "gemini-2.5-flash",
        "gemini-2.5-pro",
        "gemma-4-26b-a4b-it",
    ];
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
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, api_key
        );

        let mut contents = Vec::new();

        // Add history turns (Gemini requires "user" and "model" roles)
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

        // Add current turn
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

                last_error = format!("Unexpected response format from Gemini API on {}", model);
            }
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                if code == 429 || body.contains("429") || body.contains("quota") || body.contains("RESOURCE_EXHAUSTED") {
                    last_error = format!("Gemini quota limit (429) on {}: {}", model, body);
                } else if code == 503 || body.contains("503") || body.contains("UNAVAILABLE") || body.contains("No capacity") {
                    last_error = format!("Gemini server unavailable / no capacity (503) on {}: {}", model, body);
                } else {
                    last_error = format!("Gemini API error (HTTP {}) on {}: {}", code, model, body);
                }
            }
            Err(e) => {
                last_error = format!("Gemini API request error on {}: {}", model, e);
            }
        }
    }

    Err(format!("Gemini API error: {}", last_error))
}
