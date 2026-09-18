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
