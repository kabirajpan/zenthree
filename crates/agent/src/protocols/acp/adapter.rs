//! Agent Client Protocol (ACP) Adapter
//!
//! Bridges editor JSON-RPC communication with the Zenthree Agent system.
//! Supports two-way communication:
//! 1. Inbound ACP Requests (from Editor UI / CI harness -> Agent Orchestrator)
//! 2. Outbound Agent Events (from Agent Orchestrator -> ACP JSON-RPC Notifications/Responses)

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::types::AgentEvent;

/// Standard ACP JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpRpcRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// Standard ACP JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AcpRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Standard ACP JSON-RPC 2.0 Notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// Parsed ACP Inbound Command from the Editor
#[derive(Debug, Clone, PartialEq)]
pub enum AcpInboundCommand {
    Initialize {
        client_name: String,
        client_version: String,
        workspace_roots: Vec<String>,
    },
    NewSession {
        cwd: String,
    },
    Prompt {
        session_id: String,
        prompt: String,
    },
    Cancel {
        session_id: String,
    },
    PermissionResponse {
        session_id: String,
        call_id: String,
        decision: String, // "allow_once", "always_allow", "deny_once", "always_deny"
    },
    Unknown {
        method: String,
    },
}

pub struct AcpAdapter;

impl AcpAdapter {
    /// Parses an incoming JSON-RPC string into a typed inbound command.
    pub fn parse_inbound_request(raw_json: &str) -> Result<(Value, AcpInboundCommand), String> {
        let req: AcpRpcRequest = serde_json::from_str(raw_json)
            .map_err(|e| format!("Failed to parse JSON-RPC request: {}", e))?;

        let command = match req.method.as_str() {
            "initialize" => {
                let params = req.params.unwrap_or(Value::Null);
                let client_name = params
                    .get("clientInfo")
                    .and_then(|c| c.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let client_version = params
                    .get("clientInfo")
                    .and_then(|c| c.get("version"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("0.0.0")
                    .to_string();
                let workspace_roots = params
                    .get("workspaceRoots")
                    .and_then(|r| r.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                AcpInboundCommand::Initialize {
                    client_name,
                    client_version,
                    workspace_roots,
                }
            }
            "session/new" => {
                let params = req.params.unwrap_or(Value::Null);
                let cwd = params
                    .get("cwd")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".")
                    .to_string();
                AcpInboundCommand::NewSession { cwd }
            }
            "session/prompt" | "prompt" => {
                let params = req.params.unwrap_or(Value::Null);
                let session_id = params
                    .get("sessionId")
                    .or_else(|| params.get("session_id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let prompt = params
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                AcpInboundCommand::Prompt { session_id, prompt }
            }
            "session/cancel" | "cancel" => {
                let params = req.params.unwrap_or(Value::Null);
                let session_id = params
                    .get("sessionId")
                    .or_else(|| params.get("session_id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                AcpInboundCommand::Cancel { session_id }
            }
            "session/confirm_permission" | "permission/respond" => {
                let params = req.params.unwrap_or(Value::Null);
                let session_id = params
                    .get("sessionId")
                    .or_else(|| params.get("session_id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let call_id = params
                    .get("callId")
                    .or_else(|| params.get("call_id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let decision = params
                    .get("decision")
                    .and_then(|v| v.as_str())
                    .unwrap_or("deny_once")
                    .to_string();
                AcpInboundCommand::PermissionResponse {
                    session_id,
                    call_id,
                    decision,
                }
            }
            other => AcpInboundCommand::Unknown {
                method: other.to_string(),
            },
        };

        Ok((req.id, command))
    }

    /// Translates an internal `AgentEvent` into an outbound ACP JSON-RPC notification.
    pub fn format_outbound_event(session_id: &str, event: &AgentEvent) -> AcpRpcNotification {
        match event {
            AgentEvent::Thinking { delta } => AcpRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "session/update".to_string(),
                params: Some(json!({
                    "sessionId": session_id,
                    "type": "thinking",
                    "content": delta,
                })),
            },
            AgentEvent::StreamChunk { delta } => AcpRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "session/update".to_string(),
                params: Some(json!({
                    "sessionId": session_id,
                    "type": "agent_message_chunk",
                    "content": delta,
                })),
            },
            AgentEvent::ToolStarted { tool, summary } => AcpRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "session/update".to_string(),
                params: Some(json!({
                    "sessionId": session_id,
                    "type": "tool_call",
                    "tool": tool,
                    "summary": summary,
                })),
            },
            AgentEvent::ToolFinished { tool, ok, output_snippet, duration_ms } => AcpRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "session/update".to_string(),
                params: Some(json!({
                    "sessionId": session_id,
                    "type": "tool_call_update",
                    "tool": tool,
                    "success": ok,
                    "output": output_snippet,
                    "durationMs": duration_ms,
                })),
            },
            AgentEvent::Completed { summary } => AcpRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "session/completed".to_string(),
                params: Some(json!({
                    "sessionId": session_id,
                    "summary": summary,
                })),
            },
            AgentEvent::Error { error } => AcpRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "session/error".to_string(),
                params: Some(json!({
                    "sessionId": session_id,
                    "error": error,
                })),
            },
            AgentEvent::PendingModifications { summaries } => AcpRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "session/update".to_string(),
                params: Some(json!({
                    "sessionId": session_id,
                    "type": "files_edited",
                    "files": summaries.iter().map(|f| json!({
                        "path": f.file_name,
                        "linesAdded": f.lines_added,
                        "linesRemoved": f.lines_removed,
                    })).collect::<Vec<_>>(),
                })),
            },
            AgentEvent::SubAgentActivated { role, name } => AcpRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "session/update".to_string(),
                params: Some(json!({
                    "sessionId": session_id,
                    "type": "subagent_activated",
                    "role": role,
                    "name": name,
                })),
            },
            AgentEvent::StatusChanged { status } => AcpRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "session/status".to_string(),
                params: Some(json!({
                    "sessionId": session_id,
                    "status": status,
                })),
            },
            _ => AcpRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "session/update".to_string(),
                params: Some(json!({
                    "sessionId": session_id,
                    "type": "generic_event",
                })),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_inbound_prompt_request() {
        let raw = r#"{
            "jsonrpc": "2.0",
            "id": 42,
            "method": "session/prompt",
            "params": {
                "sessionId": "sess_abc",
                "prompt": "Fix bug in parser"
            }
        }"#;

        let (id, cmd) = AcpAdapter::parse_inbound_request(raw).unwrap();
        assert_eq!(id, json!(42));
        match cmd {
            AcpInboundCommand::Prompt { session_id, prompt } => {
                assert_eq!(session_id, "sess_abc");
                assert_eq!(prompt, "Fix bug in parser");
            }
            _ => panic!("Expected Prompt command"),
        }
    }

    #[test]
    fn test_format_outbound_stream_chunk() {
        let event = AgentEvent::StreamChunk {
            delta: "Hello world".to_string(),
        };
        let notif = AcpAdapter::format_outbound_event("sess_123", &event);
        assert_eq!(notif.method, "session/update");
        let params = notif.params.unwrap();
        assert_eq!(params["sessionId"], "sess_123");
        assert_eq!(params["type"], "agent_message_chunk");
        assert_eq!(params["content"], "Hello world");
    }

    #[test]
    fn test_format_outbound_tool_finished() {
        let event = AgentEvent::ToolFinished {
            tool: "read_file".to_string(),
            ok: true,
            output_snippet: "file content".to_string(),
            duration_ms: 12,
        };
        let notif = AcpAdapter::format_outbound_event("sess_123", &event);
        let params = notif.params.unwrap();
        assert_eq!(params["type"], "tool_call_update");
        assert_eq!(params["success"], true);
        assert_eq!(params["durationMs"], 12);
    }
}
