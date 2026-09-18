//! Protocols Module
//!
//! Provides external protocol integrations including:
//! - Model Context Protocol (MCP 2.0): Client implementation supporting child process Stdio and HTTP SSE transports.
//! - Agent Client Protocol (ACP): Bidirectional JSON-RPC communication bridge for editor harnesses and CI.

pub mod mcp;
pub mod acp;

pub use mcp::{
    CallToolParams, CallToolResult, InitializeParams, InitializeResult, JsonRpcError,
    JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, ListPromptsResult, ListResourcesResult,
    ListToolsResult, McpClient, McpContent, McpMessage, McpPrompt, McpPromptArgument,
    McpPromptContent, McpPromptMessage, McpResource, McpRole, McpSseTransport, McpStdioTransport,
    McpTool, McpTransport, ReadResourceParams, ReadResourceResult, ResourceContent,
    ServerCapabilities,
};

pub use acp::{
    AcpAdapter, AcpInboundCommand, AcpRpcError, AcpRpcNotification, AcpRpcRequest, AcpRpcResponse,
};
