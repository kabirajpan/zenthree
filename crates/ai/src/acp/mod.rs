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
        let transport = AcpTransport::spawn(command, args, Some(workspace_root))?;

        let init_params = InitializeParams {
            protocol_version: "1.0".to_string(),
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
            workspace_path: self.workspace_path.clone(),
            system_prompt,
            mode: mode.or_else(|| Some("code".to_string())),
        };

        let resp = self.transport.send_request(
            "session/new",
            Some(serde_json::to_value(params).map_err(|e| e.to_string())?),
            Duration::from_secs(10),
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
        attachments: Vec<PromptAttachment>,
        event_tx: &Sender<AgentEvent>,
        cancel_token: &CancellationToken,
    ) -> Result<String, String> {
        let session_id = self.session_id.clone().ok_or("No active ACP session")?;

        let params = SessionPromptParams {
            session_id: session_id.clone(),
            prompt: prompt.to_string(),
            attachments,
        };

        // Send prompt asynchronously or as a request
        // In ACP, session/prompt initiates the turn and notifications stream back.
        let prompt_val = serde_json::to_value(params).map_err(|e| e.to_string())?;
        
        // Spawn prompt dispatch
        let prompt_tx = self.transport.send_notification("session/prompt", Some(prompt_val));
        if let Err(e) = prompt_tx {
            return Err(format!("Failed to dispatch prompt to agent: {}", e));
        }

        let mut completed_summary = String::new();
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

            // Read notifications
            if let Some(notif) = self.transport.try_recv_notification() {
                let events = AcpAdapter::convert_notification(&notif);
                let mut finished = false;

                for ev in events {
                    let mut is_err = None;
                    match &ev {
                        AgentEvent::StreamChunk { delta } => {
                            full_response.push_str(delta);
                        }
                        AgentEvent::Completed { summary } => {
                            completed_summary = summary.clone();
                            finished = true;
                        }
                        AgentEvent::Error { error } => {
                            is_err = Some(error.clone());
                        }
                        _ => {}
                    }
                    let _ = event_tx.send(ev);
                    if let Some(err) = is_err {
                        return Err(err);
                    }
                }

                if finished {
                    break;
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

        if completed_summary.is_empty() {
            completed_summary = if full_response.is_empty() {
                "Finished task".to_string()
            } else {
                full_response
            };
        }

        Ok(completed_summary)
    }

    pub fn agent_info(&self) -> Option<&AgentInfo> {
        self.agent_info.as_ref()
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }
}
