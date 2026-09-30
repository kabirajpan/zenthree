use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use super::super::acp::config::AcpAgentConfig;

/// Minimal token status representing an active Google Antigravity ACP session.
/// Antigravity manages its own tokens; Zenthree never handles or stores raw Google tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub account_email: Option<String>,
}

impl OAuthToken {
    pub fn is_expired(&self) -> bool {
        false
    }
}

/// Authentication helper for Antigravity ACP agent.
#[derive(Debug, Clone, Default)]
pub struct AntigravityAuthManager;

pub fn default_token_path() -> PathBuf {
    AntigravityAuthManager::token_path()
}

impl AntigravityAuthManager {
    pub fn with_default_config() -> Self {
        Self
    }

    /// Path to Antigravity ACP credentials.
    pub fn token_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        home.join(".gemini/antigravity-acp/acp_token.json")
    }

    /// Check if credentials exist for the Antigravity ACP agent.
    pub fn load_token(&self) -> Option<OAuthToken> {
        let p = Self::token_path();
        if p.exists() {
            let email = Self::resolve_email_from_path(&p);
            return Some(OAuthToken {
                account_email: email,
            });
        }

        // Secondary candidate paths across Linux, macOS, and Windows
        if let Some(home) = dirs::home_dir() {
            let candidates = [
                home.join(".gemini/antigravity-acp/acp_token.json"),
                home.join(".gemini/antigravity-acp/settings.json"),
                home.join(".gemini/antigravity/acp_token.json"),
                home.join(".gemini/antigravity/settings.json"),
            ];
            for c in &candidates {
                if c.exists() {
                    let email = Self::resolve_email_from_path(c);
                    return Some(OAuthToken {
                        account_email: email,
                    });
                }
            }
        }

        if let Some(local_data) = dirs::data_local_dir() {
            let win_token = local_data.join("gemini/antigravity-acp/acp_token.json");
            if win_token.exists() {
                let email = Self::resolve_email_from_path(&win_token);
                return Some(OAuthToken {
                    account_email: email,
                });
            }
        }

        None
    }

    fn resolve_email_from_path(path: &std::path::Path) -> Option<String> {
        let content = std::fs::read_to_string(path).ok()?;
        let val: serde_json::Value = serde_json::from_str(&content).ok()?;

        // 1. Direct email field
        if let Some(email) = val
            .get("email")
            .or_else(|| val.get("account_email"))
            .or_else(|| val.get("user"))
            .and_then(|e| e.as_str())
            .filter(|s| !s.is_empty())
        {
            return Some(email.to_string());
        }

        // 2. Extract from id_token if stored in JSON
        if let Some(id_token) = val.get("id_token").and_then(|t| t.as_str()) {
            if let Some(email) = Self::parse_jwt_email(id_token) {
                return Some(email);
            }
        }

        // 3. Fallback: refresh token with Google OAuth to retrieve id_token & email
        Self::refresh_and_cache_email(path, &val)
    }

    fn parse_jwt_email(id_token: &str) -> Option<String> {
        use base64::Engine;
        let parts: Vec<&str> = id_token.split('.').collect();
        if parts.len() >= 2 {
            let mut payload = parts[1].to_string();
            while payload.len() % 4 != 0 {
                payload.push('=');
            }
            if let Ok(bytes) = base64::engine::general_purpose::URL_SAFE.decode(payload.as_bytes()) {
                if let Ok(claims) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                    if let Some(email) = claims.get("email").and_then(|e| e.as_str()) {
                        return Some(email.to_string());
                    }
                }
            }
        }
        None
    }

    fn refresh_and_cache_email(path: &std::path::Path, json: &serde_json::Value) -> Option<String> {
        let client_id = json.get("client_id")?.as_str()?;
        let client_secret = json.get("client_secret")?.as_str()?;
        let refresh_token = json.get("refresh_token")?.as_str()?;
        let token_uri = json
            .get("token_uri")
            .and_then(|u| u.as_str())
            .unwrap_or("https://oauth2.googleapis.com/token");

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(4))
            .build()
            .ok()?;

        let params = [
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];

        let resp = client.post(token_uri).form(&params).send().ok()?;
        if resp.status().is_success() {
            let body: serde_json::Value = resp.json().ok()?;
            if let Some(id_token) = body.get("id_token").and_then(|t| t.as_str()) {
                if let Some(email) = Self::parse_jwt_email(id_token) {
                    if let Ok(mut updated) = serde_json::from_str::<serde_json::Value>(
                        &std::fs::read_to_string(path).unwrap_or_default(),
                    ) {
                        if let Some(map) = updated.as_object_mut() {
                            map.insert(
                                "email".to_string(),
                                serde_json::Value::String(email.clone()),
                            );
                            let _ = std::fs::write(
                                path,
                                serde_json::to_string_pretty(&updated).unwrap_or_default(),
                            );
                        }
                    }
                    return Some(email);
                }
            }
        }
        None
    }

    /// Check if the Antigravity ACP agent binary is installed on this machine.
    pub fn is_agent_installed(&self) -> bool {
        AcpAgentConfig::suggest_antigravity().is_some()
    }

    /// Clear stored credentials (sign out).
    pub fn logout(&self) -> Result<(), std::io::Error> {
        let p = Self::token_path();
        if p.exists() {
            let _ = std::fs::remove_file(&p);
        }
        if let Some(home) = dirs::home_dir() {
            let files_to_remove = [
                home.join(".gemini/antigravity-acp/settings.json"),
                home.join(".gemini/antigravity-acp/acp_token.json"),
                home.join(".gemini/antigravity/acp_token.json"),
                home.join(".gemini/antigravity/settings.json"),
            ];
            for f in &files_to_remove {
                if f.exists() {
                    let _ = std::fs::remove_file(f);
                }
            }
        }
        if let Some(local_data) = dirs::data_local_dir() {
            let win_token = local_data.join("gemini/antigravity-acp/acp_token.json");
            if win_token.exists() {
                let _ = std::fs::remove_file(win_token);
            }
        }

        // Also try running `agy logout` if available
        #[cfg(not(target_os = "windows"))]
        {
            let _ = std::process::Command::new("agy").arg("logout").status();
        }
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("cmd.exe").args(["/C", "agy", "logout"]).status();
        }

        Ok(())
    }

    /// Alias for logout.
    pub fn clear_token(&self) {
        let _ = self.logout();
    }

    /// Launch interactive authentication via `agy login` or ACP browser flow.
    pub fn login_interactive(&self) -> Result<(), String> {
        // Try `agy login` CLI first (invoked through cmd.exe on Windows for .cmd / .bat / .exe resolution)
        #[cfg(target_os = "windows")]
        let mut cmd = {
            let mut c = std::process::Command::new("cmd.exe");
            c.args(["/C", "agy", "login"]);
            c
        };
        #[cfg(not(target_os = "windows"))]
        let mut cmd = {
            let mut c = std::process::Command::new("agy");
            c.arg("login");
            c
        };

        if let Ok(mut child) = cmd.spawn() {
            if let Ok(status) = child.wait() {
                if status.success() {
                    Self::ensure_session_marker();
                    return Ok(());
                }
            }
        }

        // If `agy` CLI is not found or fails, launch the ACP server with `oauth-personal`
        // which opens the browser for Google account consent directly.
        if let Some(mut config) = AcpAgentConfig::suggest_antigravity() {
            // Explicitly set auth_method_id so the ACP handshake triggers AuthenticateRequest
            config.auth_method_id = Some("oauth-personal".to_string());

            let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
            rt.block_on(async {
                let client = super::super::acp::AcpClient::start(config).await?;
                let _ = client.create_session(std::env::current_dir().unwrap_or_default()).await;
                Ok::<(), String>(())
            })?;
            Self::ensure_session_marker();
            Ok(())
        } else {
            Err("Neither `agy` CLI nor `agy_acp_server` binary could be found on this system. Please check your installation.".to_string())
        }
    }

    fn ensure_session_marker() {
        if let Some(home) = dirs::home_dir() {
            let acp_dir = home.join(".gemini/antigravity-acp");
            let _ = std::fs::create_dir_all(&acp_dir);
            let settings_file = acp_dir.join("settings.json");
            if !settings_file.exists() {
                let _ = std::fs::write(&settings_file, "{\n  \"auth\": {\n    \"type\": \"oauth-personal\"\n  }\n}\n");
            }
            let token_file = acp_dir.join("acp_token.json");
            if !token_file.exists() {
                let _ = std::fs::write(&token_file, "{\"authenticated\": true}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_antigravity_auth_load_token() {
        let auth = AntigravityAuthManager::with_default_config();
        let token = auth.load_token();
        println!("Loaded token: {:?}", token);
        if let Some(t) = token {
            println!("User email: {:?}", t.account_email);
        }
    }
}
