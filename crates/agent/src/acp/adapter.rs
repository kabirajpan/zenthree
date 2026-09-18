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
                    if let Some(update_val) = params.get("update") {
                        if let Ok(update) = serde_json::from_value::<SessionUpdateNotification>(update_val.clone()) {
                            events.extend(Self::convert_session_update(update));
                        }
                    } else if let Ok(update) = serde_json::from_value::<SessionUpdateNotification>(params.clone()) {
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
            "agent_message_chunk" | "chunk" | "text" | "content" => {
                if let Some(ref val) = update.content {
                    let delta = extract_text_from_value(val);
                    if !delta.is_empty() {
                        events.push(AgentEvent::StreamChunk { delta });
                    }
                }
            }
            "thinking" => {
                if let Some(ref val) = update.content {
                    let delta = extract_text_from_value(val);
                    if !delta.is_empty() {
                        events.push(AgentEvent::Thinking { delta });
                    }
                }
            }
            "subagent" | "agent_switch" => {
                let name = update.content.map(|v| extract_text_from_value(&v)).unwrap_or_else(|| "Goose Subagent".to_string());
                events.push(AgentEvent::SubAgentActivated {
                    role: "subagent".to_string(),
                    name,
                });
            }
            "tool_call" | "tool_start" => {
                let tool = update.title
                    .or(update.tool_name)
                    .unwrap_or_else(|| "tool".to_string());
                let summary = if let Some(ref args) = update.raw_input.or(update.tool_args) {
                    format!("{}({})", tool, args)
                } else {
                    format!("Running {}", tool)
                };
                events.push(AgentEvent::ToolStarted { tool, summary });
            }
            "tool_call_update" | "tool_result" | "tool_end" => {
                let tool = update.title
                    .or(update.tool_name)
                    .unwrap_or_else(|| "tool".to_string());
                let output = update.raw_output
                    .or(update.tool_result)
                    .map(|v| extract_text_from_value(&v))
                    .or_else(|| update.content.map(|v| extract_text_from_value(&v)))
                    .unwrap_or_default();
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
                if let Some(ref val) = update.content {
                    let msg = extract_text_from_value(val);
                    events.push(AgentEvent::TaskUpdated {
                        task_id: "goose-step".to_string(),
                        status: "in_progress".to_string(),
                        title: msg,
                    });
                }
            }
            "completed" => {
                let summary = update.content.map(|v| extract_text_from_value(&v)).unwrap_or_else(|| "Finished".to_string());
                events.push(AgentEvent::Completed { summary });
            }
            "error" => {
                let error = update.content.map(|v| extract_text_from_value(&v)).unwrap_or_else(|| "Goose execution failed".to_string());
                events.push(AgentEvent::Error { error });
            }
            other => {
                log::debug!("[Goose/ACP] Unknown event type: {}", other);
                if let Some(ref val) = update.content {
                    let delta = extract_text_from_value(val);
                    if !delta.is_empty() {
                        events.push(AgentEvent::StreamChunk { delta });
                    }
                }
            }
        }

        events
    }
}

fn extract_text_from_value(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Object(map) => {
            if let Some(t) = map.get("text").and_then(|v| v.as_str()) {
                t.to_string()
            } else if let Some(c) = map.get("content") {
                extract_text_from_value(c)
            } else {
                String::new()
            }
        }
        serde_json::Value::Array(arr) => {
            let mut out = String::new();
            for item in arr {
                let sub = extract_text_from_value(item);
                if !sub.is_empty() {
                    if !out.is_empty() {
                        out.push('\n');
                    }
                    out.push_str(&sub);
                }
            }
            out
        }
        _ => String::new(),
    }
}
