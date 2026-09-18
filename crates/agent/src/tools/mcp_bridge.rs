//! MCP Tool Bridge
//!
//! Bridges discovered Model Context Protocol (MCP 2.0) tools directly into
//! the LLM tool calling suite, function declaration schemas, and execution dispatcher.
//!
//! Follows the Zed `ContextServerRegistry` and Goose `rmcp` reference patterns:
//! - Normalizes MCP `inputSchema` into standard LLM JSON schemas.
//! - Injects external tool capabilities into LLM system prompts (`<mcp_tools>`).
//! - Dispatches model tool calls to the appropriate external MCP server.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::protocols::mcp::{CallToolResult, McpClient, McpTool};
use crate::tools::AgentToolResult;

/// Metadata for an MCP tool registered for LLM usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredMcpTool {
    pub server_id: String,
    pub tool: McpTool,
    pub full_tool_name: String,
}

/// Dynamic registry that bridges external MCP servers into the LLM runtime
#[derive(Default)]
pub struct McpToolBridge {
    servers: HashMap<String, Arc<Mutex<McpClient>>>,
    tools: HashMap<String, RegisteredMcpTool>,
}

impl McpToolBridge {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an MCP client under a unique server ID and indexes its tools for the LLM.
    pub fn register_server(
        &mut self,
        server_id: impl Into<String>,
        mut client: McpClient,
        timeout: Duration,
    ) -> Result<usize, String> {
        let server_id = server_id.into();

        // Discover external tools from server
        let discovered_tools = client.list_tools(timeout)?;
        let count = discovered_tools.len();

        let client_arc = Arc::new(Mutex::new(client));
        self.servers.insert(server_id.clone(), client_arc);

        for tool in discovered_tools {
            let tool_name = tool.name.clone();
            let full_name = format!("{}__{}", server_id, tool_name);

            let reg = RegisteredMcpTool {
                server_id: server_id.clone(),
                tool: tool.clone(),
                full_tool_name: full_name.clone(),
            };

            // Index by both exact tool name and namespaced name
            self.tools.insert(tool_name.clone(), reg.clone());
            self.tools.insert(full_name, reg);
        }

        Ok(count)
    }

    /// Returns all registered MCP tools converted into standard LLM function calling schemas.
    pub fn to_llm_tool_schemas(&self) -> Vec<Value> {
        let mut schemas = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for reg in self.tools.values() {
            if seen.insert(&reg.tool.name) {
                schemas.push(Self::convert_to_function_schema(&reg.tool));
            }
        }

        schemas.sort_by(|a, b| {
            let name_a = a["function"]["name"].as_str().unwrap_or("");
            let name_b = b["function"]["name"].as_str().unwrap_or("");
            name_a.cmp(name_b)
        });

        schemas
    }

    /// Normalizes an MCP tool schema into standard OpenAI / Anthropic / Gemini function calling format.
    pub fn convert_to_function_schema(mcp_tool: &McpTool) -> Value {
        let mut params = mcp_tool.input_schema.clone();
        if params.is_null() || !params.is_object() {
            params = json!({
                "type": "object",
                "properties": {}
            });
        }

        json!({
            "type": "function",
            "function": {
                "name": mcp_tool.name,
                "description": mcp_tool.description.clone().unwrap_or_else(|| format!("External tool {}", mcp_tool.name)),
                "parameters": params
            }
        })
    }

    /// Formats an `<mcp_tools>` prompt block explaining available external tools to the LLM.
    pub fn format_mcp_prompt_block(&self) -> String {
        if self.tools.is_empty() {
            return String::new();
        }

        let mut out = String::new();
        out.push_str("<mcp_tools>\n");
        out.push_str("The following external tools are available via Model Context Protocol (MCP):\n\n");

        let mut seen = std::collections::HashSet::new();
        for reg in self.tools.values() {
            if seen.insert(&reg.tool.name) {
                out.push_str(&format!(
                    "- `{}` (Server: `{}`): {}\n",
                    reg.tool.name,
                    reg.server_id,
                    reg.tool.description.as_deref().unwrap_or("No description provided.")
                ));
            }
        }
        out.push_str("</mcp_tools>");
        out
    }

    /// Checks if a tool name belongs to a registered MCP server.
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Dispatches an LLM tool call to the appropriate external MCP server.
    pub fn execute_mcp_tool(
        &self,
        name: &str,
        arguments: Option<Value>,
        timeout: Duration,
    ) -> AgentToolResult {
        let reg = match self.tools.get(name) {
            Some(r) => r,
            None => {
                return AgentToolResult {
                    tool_name: name.to_string(),
                    ok: false,
                    output: format!("Error: MCP tool '{}' is not registered", name),
                };
            }
        };

        let server = match self.servers.get(&reg.server_id) {
            Some(s) => s,
            None => {
                return AgentToolResult {
                    tool_name: name.to_string(),
                    ok: false,
                    output: format!("Error: MCP server '{}' not found", reg.server_id),
                };
            }
        };

        let mut client = match server.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return AgentToolResult {
                    tool_name: name.to_string(),
                    ok: false,
                    output: format!("Error: Failed to acquire lock on MCP server '{}'", reg.server_id),
                };
            }
        };

        match client.call_tool(&reg.tool.name, arguments, timeout) {
            Ok(CallToolResult { content, is_error }) => {
                let text = content
                    .iter()
                    .filter_map(|c| match c {
                        crate::protocols::mcp::McpContent::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                let ok = !is_error.unwrap_or(false);
                AgentToolResult {
                    tool_name: name.to_string(),
                    ok,
                    output: text,
                }
            }
            Err(e) => AgentToolResult {
                tool_name: name.to_string(),
                ok: false,
                output: format!("MCP execution error: {}", e),
            },
        }
    }

    /// Returns the number of distinct registered MCP tools.
    pub fn tool_count(&self) -> usize {
        let mut seen = std::collections::HashSet::new();
        self.tools.values().filter(|r| seen.insert(&r.tool.name)).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_tool_schema_conversion() {
        let tool = McpTool {
            name: "fetch_repo".to_string(),
            description: Some("Fetches git repository details".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "repo_url": { "type": "string" }
                },
                "required": ["repo_url"]
            }),
        };

        let function_schema = McpToolBridge::convert_to_function_schema(&tool);
        assert_eq!(function_schema["type"], "function");
        assert_eq!(function_schema["function"]["name"], "fetch_repo");
        assert_eq!(function_schema["function"]["description"], "Fetches git repository details");
        assert_eq!(function_schema["function"]["parameters"]["properties"]["repo_url"]["type"], "string");
    }

    #[test]
    fn test_empty_schema_normalization() {
        let tool = McpTool {
            name: "ping_server".to_string(),
            description: None,
            input_schema: Value::Null,
        };

        let function_schema = McpToolBridge::convert_to_function_schema(&tool);
        assert_eq!(function_schema["function"]["parameters"]["type"], "object");
    }
}
