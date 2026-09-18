//! Model Context Protocol (MCP 2.0) Module
//!
//! Provides client connectivity, transport implementations (Stdio and SSE),
//! and typed schemas for the Model Context Protocol 2.0.

pub mod types;
pub mod transport_stdio;
pub mod transport_sse;
pub mod client;

pub use types::*;
pub use transport_stdio::McpStdioTransport;
pub use transport_sse::McpSseTransport;
pub use client::{McpClient, McpTransport};
