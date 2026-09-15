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
    // Model preference: Google Gemini models verified live for this API key:
    let models = [
        "gemini-3.6-flash",
        "gemini-3.7-flash",
        "gemini-flash-latest",
        "gemini-3.8-flash",
        "gemini-2.5-flash",
    ];
    let mut last_error = String::new();

    let agent = ureq::builder()
        .resolver(ipv4_resolver)
        .timeout(std::time::Duration::from_secs(45))
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
                    last_error = format!("Gemini error on {}: {}", model, msg);
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
            Err(e) => {
                last_error = format!("Gemini API request error on {}: {}", model, e);
            }
        }
    }

    Err(format!("Gemini API error: {}", last_error))
}
