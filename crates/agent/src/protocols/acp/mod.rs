//! Agent Client Protocol (ACP) Module
//!
//! Standardized JSON-RPC interface for headless testing, CI evaluation,
//! and IDE / editor integration harnesses.

pub mod adapter;

pub use adapter::{
    AcpAdapter, AcpInboundCommand, AcpRpcError, AcpRpcNotification, AcpRpcRequest, AcpRpcResponse,
};
