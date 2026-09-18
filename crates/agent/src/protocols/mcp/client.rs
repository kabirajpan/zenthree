//! MCP Client Manager
//!
//! Provides a unified, high-level client for interacting with external MCP 2.0 servers
//! over Stdio or SSE transports. Manages protocol handshakes, capability negotiation,
//! tool discovery, and tool execution.

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};

use super::transport_sse::McpSseTransport;
use super::transport_stdio::McpStdioTransport;
use super::types::{
    CallToolParams, CallToolResult, GetPromptParams, GetPromptResult, InitializeParams,
    InitializeResult, ListPromptsResult, ListResourcesResult, ListToolsResult, McpPrompt,
    McpResource, McpTool, ReadResourceParams, ReadResourceResult, ServerCapabilities,
};

pub enum McpTransport {
    Stdio(McpStdioTransport),
    Sse(McpSseTransport),
}

impl McpTransport {
    pub fn send_request(
        &mut self,
        method: &str,
        params: Option<Value>,
        timeout: Duration,
    ) -> Result<Value, String> {
        match self {
            McpTransport::Stdio(t) => t.send_request(method, params, timeout),
            McpTransport::Sse(t) => t.send_request(method, params, timeout),
        }
    }

    pub fn send_notification(&mut self, method: &str, params: Option<Value>) -> Result<(), String> {
        match self {
            McpTransport::Stdio(t) => t.send_notification(method, params),
            McpTransport::Sse(_) => {
                // SSE HTTP notifications are typically no-op or post to endpoint
                Ok(())
            }
        }
    }

    pub fn is_alive(&mut self) -> bool {
        match self {
            McpTransport::Stdio(t) => t.is_alive(),
            McpTransport::Sse(t) => t.is_connected(),
        }
    }
}

pub struct McpClient {
    transport: McpTransport,
    server_capabilities: Option<ServerCapabilities>,
    server_instructions: Option<String>,
    cached_tools: Vec<McpTool>,
}

impl McpClient {
    /// Connects to a local MCP server by launching a child process over standard I/O.
    pub fn connect_stdio(
        command: &str,
        args: &[&str],
        cwd: Option<&Path>,
        env_vars: &[(&str, &str)],
    ) -> Result<Self, String> {
        let transport = McpStdioTransport::spawn(command, args, cwd, env_vars)?;
        Ok(Self {
            transport: McpTransport::Stdio(transport),
            server_capabilities: None,
            server_instructions: None,
            cached_tools: Vec::new(),
        })
    }

    /// Connects to a remote MCP server over HTTP Server-Sent Events.
    pub fn connect_sse(sse_url: &str, auth_header: Option<&str>) -> Result<Self, String> {
        let mut transport = McpSseTransport::new(sse_url);
        if let Some(auth) = auth_header {
            transport = transport.with_auth(auth);
        }
        transport.connect(Duration::from_secs(10))?;
        Ok(Self {
            transport: McpTransport::Sse(transport),
            server_capabilities: None,
            server_instructions: None,
            cached_tools: Vec::new(),
        })
    }

    /// Performs the MCP 2.0 initialization handshake.
    pub fn initialize(&mut self, timeout: Duration) -> Result<InitializeResult, String> {
        let params = InitializeParams::default();
        let value_params = serde_json::to_value(&params)
            .map_err(|e| format!("Failed to serialize initialize params: {}", e))?;

        let resp_value = self.transport.send_request("initialize", Some(value_params), timeout)?;
        let init_result: InitializeResult = serde_json::from_value(resp_value)
            .map_err(|e| format!("Failed to deserialize initialize result: {}", e))?;

        // Send initialized notification as required by MCP spec
        let _ = self.transport.send_notification("notifications/initialized", None);

        self.server_capabilities = Some(init_result.capabilities.clone());
        self.server_instructions = init_result.instructions.clone();

        Ok(init_result)
    }

    /// Fetches all tools exposed by the MCP server, following cursor pagination.
    pub fn list_tools(&mut self, timeout: Duration) -> Result<Vec<McpTool>, String> {
        let mut all_tools = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let params = cursor.as_ref().map(|c| json!({ "cursor": c }));
            let resp_value = self.transport.send_request("tools/list", params, timeout)?;
            let list_res: ListToolsResult = serde_json::from_value(resp_value)
                .map_err(|e| format!("Failed to parse tools/list response: {}", e))?;

            all_tools.extend(list_res.tools);

            if let Some(next) = list_res.next_cursor {
                if !next.is_empty() {
                    cursor = Some(next);
                    continue;
                }
            }
            break;
        }

        self.cached_tools = all_tools.clone();
        Ok(all_tools)
    }

    /// Invokes a tool on the external MCP server.
    pub fn call_tool(
        &mut self,
        name: &str,
        arguments: Option<Value>,
        timeout: Duration,
    ) -> Result<CallToolResult, String> {
        let params = CallToolParams {
            name: name.to_string(),
            arguments,
        };
        let value_params = serde_json::to_value(&params)
            .map_err(|e| format!("Failed to serialize call_tool params: {}", e))?;

        let resp_value = self.transport.send_request("tools/call", Some(value_params), timeout)?;
        let result: CallToolResult = serde_json::from_value(resp_value)
            .map_err(|e| format!("Failed to parse tools/call response: {}", e))?;

        Ok(result)
    }

    /// Fetches available resources from the MCP server.
    pub fn list_resources(&mut self, timeout: Duration) -> Result<Vec<McpResource>, String> {
        let resp_value = self.transport.send_request("resources/list", None, timeout)?;
        let list_res: ListResourcesResult = serde_json::from_value(resp_value)
            .map_err(|e| format!("Failed to parse resources/list response: {}", e))?;
        Ok(list_res.resources)
    }

    /// Reads resource content from the given URI.
    pub fn read_resource(&mut self, uri: &str, timeout: Duration) -> Result<ReadResourceResult, String> {
        let params = ReadResourceParams { uri: uri.to_string() };
        let value_params = serde_json::to_value(&params)
            .map_err(|e| format!("Failed to serialize read_resource params: {}", e))?;

        let resp_value = self.transport.send_request("resources/read", Some(value_params), timeout)?;
        let result: ReadResourceResult = serde_json::from_value(resp_value)
            .map_err(|e| format!("Failed to parse resources/read response: {}", e))?;
        Ok(result)
    }

    /// Lists prompt templates exposed by the MCP server.
    pub fn list_prompts(&mut self, timeout: Duration) -> Result<Vec<McpPrompt>, String> {
        let resp_value = self.transport.send_request("prompts/list", None, timeout)?;
        let list_res: ListPromptsResult = serde_json::from_value(resp_value)
            .map_err(|e| format!("Failed to parse prompts/list response: {}", e))?;
        Ok(list_res.prompts)
    }

    /// Gets a rendered prompt template with arguments.
    pub fn get_prompt(
        &mut self,
        name: &str,
        arguments: Option<Value>,
        timeout: Duration,
    ) -> Result<GetPromptResult, String> {
        let params = GetPromptParams {
            name: name.to_string(),
            arguments,
        };
        let value_params = serde_json::to_value(&params)
            .map_err(|e| format!("Failed to serialize get_prompt params: {}", e))?;

        let resp_value = self.transport.send_request("prompts/get", Some(value_params), timeout)?;
        let result: GetPromptResult = serde_json::from_value(resp_value)
            .map_err(|e| format!("Failed to parse prompts/get response: {}", e))?;
        Ok(result)
    }

    /// Pings the MCP server to verify liveness.
    pub fn ping(&mut self, timeout: Duration) -> Result<(), String> {
        self.transport.send_request("ping", None, timeout)?;
        Ok(())
    }

    /// Returns the cached tool list discovered during the last `list_tools` call.
    pub fn cached_tools(&self) -> &[McpTool] {
        &self.cached_tools
    }

    /// Returns server instructions received during handshake if any.
    pub fn server_instructions(&self) -> Option<&str> {
        self.server_instructions.as_deref()
    }
}
