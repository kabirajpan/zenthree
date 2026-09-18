use crate::types::ChatMessage;
use super::errors::{classify_provider_error, ProviderError};
use super::gemini;
use super::groq;

pub struct ProviderChain {
    pub gemini_key: Option<String>,
    pub groq_key: Option<String>,
    pub preferred_model: Option<String>,
}

impl ProviderChain {
    pub fn new(
        gemini_key: Option<String>,
        groq_key: Option<String>,
        preferred_model: Option<String>,
    ) -> Self {
        Self {
            gemini_key,
            groq_key,
            preferred_model,
        }
    }

    /// Creates a ProviderChain by checking environment variables GEMINI_API_KEY / GROQ_API_KEY and .env
    pub fn from_env() -> Self {
        let gemini_key = super::keys::load_gemini_api_key();
        let groq_key = super::keys::load_groq_api_key();
        Self {
            gemini_key,
            groq_key,
            preferred_model: None,
        }
    }

    /// Completes the prompt with retries on transient errors and automatic fallback between Gemini and Groq
    pub fn complete(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        history: &[ChatMessage],
    ) -> Result<String, String> {
        let is_gemini_target = self
            .preferred_model
            .as_deref()
            .map(|m| m.contains("gemini") || m.contains("gemma"))
            .unwrap_or(false);
        let is_groq_target = self
            .preferred_model
            .as_deref()
            .map(|m| m.contains("gpt") || m.contains("qwen") || m.contains("llama"))
            .unwrap_or(false);

        let mut try_gemini_first = (is_gemini_target || !is_groq_target) && self.gemini_key.is_some();
        if !try_gemini_first && self.groq_key.is_none() && self.gemini_key.is_some() {
            try_gemini_first = true;
        }

        let mut errors = Vec::new();
        let mut gemini_quota_error: Option<String> = None;

        if try_gemini_first {
            if let Some(ref gkey) = self.gemini_key {
                let pref = self.preferred_model.as_deref();
                for attempt in 0..2 {
                    if attempt > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                    match gemini::call_gemini_api_with_model(gkey, system_prompt, user_prompt, history, pref) {
                        Ok(res) => return Ok(res),
                        Err(e) => {
                            let is_quota = e.contains("429")
                                || e.contains("quota")
                                || e.contains("Quota")
                                || e.contains("RESOURCE_EXHAUSTED");
                            let is_unavailable = e.contains("503")
                                || e.contains("UNAVAILABLE")
                                || e.contains("No capacity");
                            if is_quota || is_unavailable {
                                let reason = if is_quota {
                                    "Gemini quota limit reached (429)"
                                } else {
                                    "Gemini server unavailable (503 - No capacity available)"
                                };
                                gemini_quota_error = Some(reason.to_string());
                                errors.push(format!("{}: {}", reason, e));
                                break; // Instantly failover to Groq
                            }
                            let classified = classify_provider_error(&e);
                            match classified {
                                ProviderError::Transient(_) if attempt == 0 => {
                                    log::warn!("Gemini transient error on attempt 1, retrying: {}", e);
                                    continue;
                                }
                                _ => {
                                    errors.push(format!("Gemini failed: {}", e));
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            // Automatic fallback to Groq
            if let Some(ref qkey) = self.groq_key {
                log::info!("Falling back from Gemini to Groq provider...");
                let pref = if is_groq_target { self.preferred_model.as_deref() } else { None };
                match groq::call_groq_api_with_model(qkey, system_prompt, user_prompt, history, pref) {
                    Ok(res) => {
                        if let Some(ref q_err) = gemini_quota_error {
                            let note = if q_err.contains("503") || q_err.contains("capacity") {
                                "> [!WARNING]\n> **AI Provider Notice**: Google Gemini server unavailable (code 503: No capacity available). Auto-routed request to **Groq Cloud (Active)**.\n\n"
                            } else {
                                "> [!NOTE]\n> **AI Provider Notice**: Google Gemini quota limit reached (429). Auto-routed request to **Groq Cloud (Active)**.\n\n"
                            };
                            return Ok(format!("{}{}", note, res));
                        }
                        return Ok(res);
                    }
                    Err(e) => errors.push(format!("Groq fallback failed: {}", e)),
                }
            }
        } else {
            // Groq first
            if let Some(ref qkey) = self.groq_key {
                let pref = self.preferred_model.as_deref();
                for attempt in 0..2 {
                    if attempt > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                    match groq::call_groq_api_with_model(qkey, system_prompt, user_prompt, history, pref) {
                        Ok(res) => return Ok(res),
                        Err(e) => {
                            let classified = classify_provider_error(&e);
                            match classified {
                                ProviderError::Transient(_) if attempt == 0 => {
                                    log::warn!("Groq transient error on attempt 1, retrying: {}", e);
                                    continue;
                                }
                                _ => {
                                    errors.push(format!("Groq failed: {}", e));
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            // Automatic fallback to Gemini
            if let Some(ref gkey) = self.gemini_key {
                log::info!("Falling back from Groq to Gemini provider...");
                let pref = if is_gemini_target { self.preferred_model.as_deref() } else { None };
                match gemini::call_gemini_api_with_model(gkey, system_prompt, user_prompt, history, pref) {
                    Ok(res) => return Ok(res),
                    Err(e) => errors.push(format!("Gemini fallback failed: {}", e)),
                }
            }
        }

        if errors.is_empty() {
            Err("No AI provider configured (GEMINI_API_KEY or GROQ_API_KEY required)".to_string())
        } else {
            let mut summary = String::from("⚠️ **AI Provider Service Notice**:\n\n");
            for err in &errors {
                summary.push_str(&format!("- {}\n", err));
            }
            summary.push_str("\n*Tip: Check your API key quotas or switch models via the bottom model selector.*");
            Err(summary)
        }
    }
}
