use serde::{Deserialize, Serialize};

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
