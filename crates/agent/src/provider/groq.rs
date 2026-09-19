use serde::{Deserialize, Serialize};

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
