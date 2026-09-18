//! MCP Server-Sent Events (SSE) Transport
//!
//! Connects to an external MCP server over HTTP SSE and JSON-RPC 2.0.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use serde_json::Value;

use super::types::{
    JsonRpcRequest, JsonRpcResponse, MCP_VERSION_LATEST,
};

pub struct McpSseTransport {
    sse_url: String,
    post_endpoint: Option<String>,
    next_id: AtomicU64,
    protocol_version: String,
    auth_header: Option<String>,
    is_connected: AtomicBool,
}

impl McpSseTransport {
    /// Initializes a new SSE transport configuration.
    pub fn new(sse_url: impl Into<String>) -> Self {
        Self {
            sse_url: sse_url.into(),
            post_endpoint: None,
            next_id: AtomicU64::new(1),
            protocol_version: MCP_VERSION_LATEST.to_string(),
            auth_header: None,
            is_connected: AtomicBool::new(false),
        }
    }

    /// Sets an authorization header (e.g. `Bearer <token>`).
    pub fn with_auth(mut self, auth_header: impl Into<String>) -> Self {
        self.auth_header = Some(auth_header.into());
        self
    }

    /// Sets explicit post message endpoint if already known.
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.post_endpoint = Some(endpoint.into());
        self.is_connected.store(true, Ordering::SeqCst);
        self
    }

    /// Connects to the SSE endpoint to discover the message posting URL.
    pub fn connect(&mut self, timeout: Duration) -> Result<(), String> {
        let mut req = ureq::get(&self.sse_url)
            .timeout(timeout)
            .set("Accept", "text/event-stream");

        if let Some(auth) = &self.auth_header {
            req = req.set("Authorization", auth);
        }

        let resp = req.call().map_err(|e| {
            format!("Failed to connect to MCP SSE endpoint '{}': {}", self.sse_url, e)
        })?;

        // Read initial SSE stream events to locate `endpoint` event
        let reader = resp.into_reader();
        let buf_reader = std::io::BufReader::new(reader);
        use std::io::BufRead;

        let mut current_event = String::new();
        for line in buf_reader.lines().take(20) {
            let line = line.map_err(|e| format!("Error reading SSE line: {}", e))?;
            let trimmed = line.trim();
            if trimmed.starts_with("event:") {
                current_event = trimmed["event:".len()..].trim().to_string();
            } else if trimmed.starts_with("data:") {
                let data = trimmed["data:".len()..].trim();
                if current_event == "endpoint" || self.post_endpoint.is_none() {
                    // Normalize relative or absolute endpoint URL
                    let resolved_url = if data.starts_with("http://") || data.starts_with("https://") {
                        data.to_string()
                    } else if data.starts_with('/') {
                        let base = self.extract_origin(&self.sse_url);
                        format!("{}{}", base, data)
                    } else {
                        format!("{}/{}", self.sse_url.trim_end_matches('/'), data)
                    };
                    self.post_endpoint = Some(resolved_url);
                    self.is_connected.store(true, Ordering::SeqCst);
                    return Ok(());
                }
            }
        }

        // Default to `/messages` sibling endpoint if not explicitly provided
        if self.post_endpoint.is_none() {
            let base = self.extract_origin(&self.sse_url);
            self.post_endpoint = Some(format!("{}/messages", base));
            self.is_connected.store(true, Ordering::SeqCst);
        }

        Ok(())
    }

    /// Helper to extract protocol + host from URL
    fn extract_origin(&self, url_str: &str) -> String {
        if let Some(idx) = url_str.find("://") {
            let rest = &url_str[idx + 3..];
            if let Some(slash) = rest.find('/') {
                return url_str[..idx + 3 + slash].to_string();
            }
        }
        url_str.trim_end_matches('/').to_string()
    }

    /// Sends a JSON-RPC request to the discovered message endpoint.
    pub fn send_request(
        &mut self,
        method: &str,
        params: Option<Value>,
        timeout: Duration,
    ) -> Result<Value, String> {
        let endpoint = self.post_endpoint.as_ref().ok_or_else(|| {
            "SSE transport is not connected to a message endpoint".to_string()
        })?;

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let request = JsonRpcRequest::new(id, method, params);

        let mut req = ureq::post(endpoint)
            .timeout(timeout)
            .set("Content-Type", "application/json")
            .set("MCP-Protocol-Version", &self.protocol_version);

        if let Some(auth) = &self.auth_header {
            req = req.set("Authorization", auth);
        }

        let resp = req.send_json(&request).map_err(|e| {
            format!("HTTP request failed to MCP endpoint '{}': {}", endpoint, e)
        })?;

        let json_resp: JsonRpcResponse = resp.into_json().map_err(|e| {
            format!("Failed to parse MCP JSON-RPC response: {}", e)
        })?;

        if let Some(err) = json_resp.error {
            return Err(format!("MCP JSON-RPC Error ({}): {}", err.code, err.message));
        }

        Ok(json_resp.result.unwrap_or(Value::Null))
    }

    /// Checks if the transport has an active endpoint configuration.
    pub fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::SeqCst) && self.post_endpoint.is_some()
    }
}
