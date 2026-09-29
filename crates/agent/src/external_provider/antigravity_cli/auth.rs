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
            return Some(OAuthToken {
                account_email: None,
            });
        }

        // Secondary candidate paths across Linux, macOS, and Windows
        if let Some(home) = dirs::home_dir() {
            let candidates = [
                home.join(".gemini/antigravity-acp/settings.json"),
                home.join(".gemini/antigravity/acp_token.json"),
                home.join(".gemini/antigravity/settings.json"),
            ];
            for c in &candidates {
                if c.exists() {
                    return Some(OAuthToken {
                        account_email: None,
                    });
                }
            }
        }

        if let Some(local_data) = dirs::data_local_dir() {
            let win_token = local_data.join("gemini/antigravity-acp/acp_token.json");
            if win_token.exists() {
                return Some(OAuthToken {
                    account_email: None,
                });
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
            let _ = std::fs::remove_file(p);
        }
        if let Some(home) = dirs::home_dir() {
            let alt = home.join(".gemini/antigravity/acp_token.json");
            if alt.exists() {
                let _ = std::fs::remove_file(alt);
            }
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
            Ok(())
        } else {
            Err("Neither `agy` CLI nor `agy_acp_server` binary could be found on this system. Please check your installation.".to_string())
        }
    }
}
