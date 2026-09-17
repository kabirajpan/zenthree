use crate::acp::protocol::{AcpNotification, SessionUpdateNotification};
use crate::types::{AgentEvent, EditedFileSummary};

pub struct AcpAdapter;

impl AcpAdapter {
    /// Translates an incoming ACP JSON-RPC notification into one or more Zenthree `AgentEvent`s.
    pub fn convert_notification(notification: &AcpNotification) -> Vec<AgentEvent> {
        let mut events = Vec::new();

        match notification.method.as_str() {
            "session/update" => {
                if let Some(params) = &notification.params {
                    if let Ok(update) = serde_json::from_value::<SessionUpdateNotification>(params.clone()) {
                        events.extend(Self::convert_session_update(update));
                    }
                }
            }
            "session/completed" | "session/done" => {
                let summary = notification
                    .params
                    .as_ref()
                    .and_then(|p| p.get("summary").or_else(|| p.get("message")))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Session completed successfully")
                    .to_string();
                events.push(AgentEvent::Completed { summary });
            }
            "session/error" => {
                let error = notification
                    .params
                    .as_ref()
                    .and_then(|p| p.get("error").or_else(|| p.get("message")))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown agent error")
                    .to_string();
                events.push(AgentEvent::Error { error });
            }
            _ => {
                log::debug!("[Goose/ACP] Unhandled notification method: {}", notification.method);
            }
        }

        events
    }

    /// Converts a typed `SessionUpdateNotification` into `AgentEvent`s.
    pub fn convert_session_update(update: SessionUpdateNotification) -> Vec<AgentEvent> {
        let mut events = Vec::new();

        match update.event_type.as_str() {
            "thinking" => {
                if let Some(delta) = update.content {
                    events.push(AgentEvent::Thinking { delta });
                }
            }
            "chunk" | "text" | "content" => {
                if let Some(delta) = update.content {
                    events.push(AgentEvent::StreamChunk { delta });
                }
            }
            "subagent" | "agent_switch" => {
                let name = update.content.unwrap_or_else(|| "Goose Subagent".to_string());
                events.push(AgentEvent::SubAgentActivated {
                    role: "subagent".to_string(),
                    name,
                });
            }
            "tool_call" | "tool_start" => {
                let tool = update.tool_name.unwrap_or_else(|| "tool".to_string());
                let summary = if let Some(args) = update.tool_args {
                    format!("{}({})", tool, args)
                } else {
                    format!("Running {}", tool)
                };
                events.push(AgentEvent::ToolStarted { tool, summary });
            }
            "tool_result" | "tool_end" => {
                let tool = update.tool_name.unwrap_or_else(|| "tool".to_string());
                let output = update.tool_result.or(update.content).unwrap_or_default();
                let output_snippet = if output.len() > 300 {
                    format!("{}...", &output[..300])
                } else {
                    output
                };
                events.push(AgentEvent::ToolFinished {
                    tool,
                    ok: true,
                    output_snippet,
                    duration_ms: 0,
                });
            }
            "diff" | "file_change" => {
                if let Some(path) = update.file_path {
                    events.push(AgentEvent::PendingModifications {
                        summaries: vec![EditedFileSummary {
                            file_name: path,
                            lines_added: 0,
                            lines_removed: 0,
                            original_code: None,
                            proposed_code: None,
                        }],
                    });
                }
            }
            "status" => {
                if let Some(msg) = update.content {
                    events.push(AgentEvent::TaskUpdated {
                        task_id: "goose-step".to_string(),
                        status: "in_progress".to_string(),
                        title: msg,
                    });
                }
            }
            "completed" => {
                let summary = update.content.unwrap_or_else(|| "Finished".to_string());
                events.push(AgentEvent::Completed { summary });
            }
            "error" => {
                let error = update.content.unwrap_or_else(|| "Goose execution failed".to_string());
                events.push(AgentEvent::Error { error });
            }
            other => {
                log::debug!("[Goose/ACP] Unknown event type: {}", other);
                if let Some(content) = update.content {
                    events.push(AgentEvent::StreamChunk { delta: content });
                }
            }
        }

        events
    }
}
