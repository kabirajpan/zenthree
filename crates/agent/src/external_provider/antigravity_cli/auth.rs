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
            Some(OAuthToken {
                account_email: None,
            })
        } else {
            None
        }
    }

    /// Check if the Antigravity ACP agent binary is installed on this machine.
    pub fn is_agent_installed(&self) -> bool {
        AcpAgentConfig::suggest_antigravity().is_some()
    }

    /// Clear stored credentials (sign out).
    pub fn logout(&self) -> Result<(), std::io::Error> {
        let p = Self::token_path();
        if p.exists() {
            std::fs::remove_file(p)?;
        }
        Ok(())
    }

    /// Alias for logout.
    pub fn clear_token(&self) {
        let _ = self.logout();
    }

    /// Launch interactive authentication via `agy login` or ACP browser flow.
    pub fn login_interactive(&self) -> Result<(), String> {
        // If `agy` CLI is on PATH, run `agy login`
        let mut cmd = std::process::Command::new("agy");
        cmd.arg("login");
        match cmd.spawn() {
            Ok(mut child) => {
                let status = child.wait().map_err(|e| e.to_string())?;
                if status.success() {
                    Ok(())
                } else {
                    Err(format!("`agy login` exited with status: {}", status))
                }
            }
            Err(e) => {
                log::warn!("Could not spawn `agy login`: {}. Spawning ACP server for auth...", e);
                // Alternatively, starting ACP client with auth_method_id="oauth-personal" will trigger ACP auth
                if let Some(config) = AcpAgentConfig::suggest_antigravity() {
                    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
                    rt.block_on(async {
                        let client = super::super::acp::AcpClient::start(config).await?;
                        let _ = client.create_session(std::env::current_dir().unwrap_or_default()).await;
                        Ok::<(), String>(())
                    })?;
                    Ok(())
                } else {
                    Err("Neither `agy` CLI nor `agy_acp_server.par` could be found".to_string())
                }
            }
        }
    }
}
