pub mod protocol;
pub mod transport;
pub mod adapter;

pub use protocol::*;
pub use transport::AcpTransport;
pub use adapter::AcpAdapter;

use std::path::Path;
use std::sync::mpsc::Sender;
use std::time::Duration;
use serde_json::json;

use crate::types::{AgentEvent, CancellationToken};

pub struct AcpClient {
    transport: AcpTransport,
    session_id: Option<String>,
    workspace_path: String,
    agent_info: Option<AgentInfo>,
}

impl AcpClient {
    /// Launches a Goose or any ACP-compliant agent child process and initializes the connection.
    pub fn connect(command: &str, args: &[&str], workspace_root: &Path) -> Result<Self, String> {
        let ws_str = workspace_root.to_string_lossy().to_string();

        // Collect environment variables from system and .env files
        let mut env_vars = Vec::new();
        let candidates = [
            workspace_root.join(".env"),
            workspace_root.join("apps/zenthree/.env"),
            workspace_root.join("../.env"),
            workspace_root.join("../../.env"),
        ];
        for env_path in &candidates {
            if let Ok(content) = std::fs::read_to_string(env_path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    if let Some((k, v)) = trimmed.split_once('=') {
                        let k = k.trim().to_string();
                        let v = v.trim().trim_matches('"').trim_matches('\'').to_string();
                        if k == "GEMINI_API_KEY" {
                            env_vars.push(("GOOGLE_API_KEY".to_string(), v.clone()));
                        }
                        env_vars.push((k, v));
                    }
                }
                break;
            }
        }
        if let Ok(val) = std::env::var("GEMINI_API_KEY") {
            env_vars.push(("GOOGLE_API_KEY".to_string(), val.clone()));
            env_vars.push(("GEMINI_API_KEY".to_string(), val));
        }
        if let Ok(val) = std::env::var("GROQ_API_KEY") {
            env_vars.push(("GROQ_API_KEY".to_string(), val));
        }

        let transport = AcpTransport::spawn_with_env(command, args, Some(workspace_root), &env_vars)?;

        let init_params = InitializeParams {
            protocol_version: 1,
            client_info: ClientInfo {
                name: "zenthree".to_string(),
                version: "0.1.0".to_string(),
            },
            workspace_roots: vec![ws_str.clone()],
        };

        // Handshake: initialize
        let resp = transport.send_request(
            "initialize",
            Some(serde_json::to_value(init_params).map_err(|e| e.to_string())?),
            Duration::from_secs(15),
        )?;

        let agent_info = resp.result.and_then(|res| {
            serde_json::from_value::<InitializeResult>(res).ok()
        }).and_then(|r| r.agent_info);

        // Tell server client is initialized
        let _ = transport.send_notification("initialized", None);

        let mut client = Self {
            transport,
            session_id: None,
            workspace_path: ws_str,
            agent_info,
        };

        // Create initial session
        client.new_session(None, None)?;

        Ok(client)
    }

    /// Creates a new session on the ACP agent
    pub fn new_session(&mut self, system_prompt: Option<String>, mode: Option<String>) -> Result<String, String> {
        let params = SessionNewParams {
            cwd: self.workspace_path.clone(),
            mcp_servers: vec![],
            workspace_path: Some(self.workspace_path.clone()),
            system_prompt,
            mode: mode.or_else(|| Some("auto".to_string())),
        };

        let resp = self.transport.send_request(
            "session/new",
            Some(serde_json::to_value(params).map_err(|e| e.to_string())?),
            Duration::from_secs(15),
        )?;

        let result = resp.result.ok_or_else(|| {
            resp.error.map(|e| e.message).unwrap_or_else(|| "Failed to create ACP session".to_string())
        })?;

        let new_res = serde_json::from_value::<SessionNewResult>(result)
            .map_err(|e| format!("Failed to parse session/new response: {}", e))?;

        let session_id = new_res.session_id;
        self.session_id = Some(session_id.clone());
        Ok(session_id)
    }

    /// Dispatches a prompt to the agent session and streams resulting events to `event_tx`.
    pub fn send_prompt(
        &mut self,
        prompt: &str,
        _attachments: Vec<PromptAttachment>,
        event_tx: &Sender<AgentEvent>,
        cancel_token: &CancellationToken,
    ) -> Result<String, String> {
        let session_id = self.session_id.clone().ok_or("No active ACP session")?;

        let prompt_blocks = json!([{"type": "text", "text": prompt}]);
        let params = json!({
            "sessionId": session_id,
            "prompt": prompt_blocks,
        });

        let (_req_id, res_rx) = self.transport.send_request_channel("session/prompt", Some(params))?;

        let mut full_response = String::new();

        loop {
            // Check cancellation
            if cancel_token.is_cancelled() {
                let cancel_params = SessionCancelParams {
                    session_id: session_id.clone(),
                };
                let _ = self.transport.send_notification(
                    "session/cancel",
                    Some(json!(cancel_params)),
                );
                let _ = event_tx.send(AgentEvent::Completed {
                    summary: "Operation cancelled by user".to_string(),
                });
                return Ok("Cancelled".to_string());
            }

            // Check if final response arrived
            if let Ok(resp) = res_rx.try_recv() {
                if let Some(err) = resp.error {
                    let _ = event_tx.send(AgentEvent::Error { error: err.message.clone() });
                    return Err(err.message);
                }
                let summary = if !full_response.trim().is_empty() {
                    full_response.clone()
                } else {
                    "Session completed successfully".to_string()
                };
                let _ = event_tx.send(AgentEvent::Completed { summary: summary.clone() });
                return Ok(summary);
            }

            // Read notifications
            if let Some(notif) = self.transport.try_recv_notification() {
                let events = AcpAdapter::convert_notification(&notif);
                for ev in events {
                    if let AgentEvent::StreamChunk { ref delta } = ev {
                        full_response.push_str(delta);
                    }
                    let _ = event_tx.send(ev);
                }
            } else {
                std::thread::sleep(Duration::from_millis(15));
            }

            // Check if process crashed
            if !self.transport.is_alive() {
                let err_msg = "Goose/ACP process terminated unexpectedly".to_string();
                let _ = event_tx.send(AgentEvent::Error {
                    error: err_msg.clone(),
                });
                return Err(err_msg);
            }
        }
    }

    pub fn agent_info(&self) -> Option<&AgentInfo> {
        self.agent_info.as_ref()
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }
}
