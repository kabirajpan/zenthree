use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl AcpRequest {
    pub fn new(id: u64, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC 2.0 response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AcpErrorObject>,
}

/// JSON-RPC 2.0 error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpErrorObject {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// JSON-RPC 2.0 notification message (no id)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// Generic incoming message from the ACP agent server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AcpIncomingMessage {
    Response(AcpResponse),
    Notification(AcpNotification),
}

/// Client capability and editor info sent during `initialize`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
    #[serde(rename = "workspaceRoots")]
    pub workspace_roots: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// Agent info returned in initialize response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion", default)]
    pub protocol_version: String,
    #[serde(rename = "agentInfo", default)]
    pub agent_info: Option<AgentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentInfo {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// Parameters for creating a new session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionNewParams {
    #[serde(rename = "workspacePath")]
    pub workspace_path: String,
    #[serde(rename = "systemPrompt", skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionNewResult {
    #[serde(rename = "sessionId")]
    pub session_id: String,
}

/// Parameters for sending a prompt to an active session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPromptParams {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<PromptAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptAttachment {
    pub r#type: String, // "file", "diff", "diagnostic"
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Streaming update notification payload from agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUpdateNotification {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "eventType")]
    pub event_type: String, // "thinking", "chunk", "tool_call", "tool_result", "diff", "status", "completed", "error"
    #[serde(default)]
    pub content: Option<String>,
    #[serde(rename = "toolName", default)]
    pub tool_name: Option<String>,
    #[serde(rename = "toolArgs", default)]
    pub tool_args: Option<Value>,
    #[serde(rename = "toolResult", default)]
    pub tool_result: Option<String>,
    #[serde(rename = "filePath", default)]
    pub file_path: Option<String>,
    #[serde(rename = "patch", default)]
    pub patch: Option<String>,
}

/// Cancel parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCancelParams {
    #[serde(rename = "sessionId")]
    pub session_id: String,
}
