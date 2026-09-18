//! MCP Stdio Transport
//!
//! Spawns an external child process and communicates via JSON-RPC 2.0 over
//! standard input/output using newline-delimited JSON.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;

use super::types::{JsonRpcNotification, JsonRpcRequest, McpMessage};

pub struct McpStdioTransport {
    child: Child,
    stdin: ChildStdin,
    next_id: AtomicU64,
    pending_requests: Arc<Mutex<HashMap<u64, Sender<Result<Value, String>>>>>,
    notification_tx: Option<Sender<JsonRpcNotification>>,
    is_running: Arc<AtomicBool>,
}

impl McpStdioTransport {
    /// Spawns an external command as an MCP server.
    pub fn spawn(
        command: &str,
        args: &[&str],
        cwd: Option<&Path>,
        env_vars: &[(&str, &str)],
    ) -> Result<Self, String> {
        let mut cmd = Command::new(command);
        cmd.args(args);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit());

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        for (k, v) in env_vars {
            cmd.env(k, v);
        }

        let mut child = cmd.spawn().map_err(|e| {
            format!("Failed to spawn MCP process '{}': {}", command, e)
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Failed to capture MCP process stdin".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture MCP process stdout".to_string())?;

        let pending_requests: Arc<Mutex<HashMap<u64, Sender<Result<Value, String>>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let is_running = Arc::new(AtomicBool::new(true));

        let pending_clone = Arc::clone(&pending_requests);
        let running_clone = Arc::clone(&is_running);

        // Spawn background reader thread for stdout
        thread::Builder::new()
            .name(format!("mcp-stdio-reader-{}", command))
            .spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if !running_clone.load(Ordering::SeqCst) {
                        break;
                    }
                    match line {
                        Ok(text) => {
                            let trimmed = text.trim();
                            if trimmed.is_empty() {
                                continue;
                            }
                            if let Ok(msg) = serde_json::from_str::<McpMessage>(trimmed) {
                                match msg {
                                    McpMessage::Response(resp) => {
                                        if let Some(id_num) = resp.id.as_u64() {
                                            if let Some(tx) = pending_clone.lock().unwrap().remove(&id_num) {
                                                if let Some(err) = resp.error {
                                                    let _ = tx.send(Err(format!(
                                                        "JSON-RPC Error ({}): {}",
                                                        err.code, err.message
                                                    )));
                                                } else {
                                                    let _ = tx.send(Ok(resp.result.unwrap_or(Value::Null)));
                                                }
                                            }
                                        }
                                    }
                                    McpMessage::Notification(_notif) => {
                                        // Notifications can be forwarded if listener configured
                                    }
                                    McpMessage::Request(_req) => {
                                        // Server-to-client request (e.g. roots/list or sampling)
                                    }
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
                running_clone.store(false, Ordering::SeqCst);
            })
            .map_err(|e| format!("Failed to spawn MCP reader thread: {}", e))?;

        Ok(Self {
            child,
            stdin,
            next_id: AtomicU64::new(1),
            pending_requests,
            notification_tx: None,
            is_running,
        })
    }

    /// Sets an optional channel to receive notifications emitted by the server.
    pub fn set_notification_receiver(&mut self, tx: Sender<JsonRpcNotification>) {
        self.notification_tx = Some(tx);
    }

    /// Sends a JSON-RPC request and synchronously blocks until a response is received or timeout expires.
    pub fn send_request(
        &mut self,
        method: &str,
        params: Option<Value>,
        timeout: Duration,
    ) -> Result<Value, String> {
        if !self.is_alive() {
            return Err("MCP child process is not running".to_string());
        }

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let request = JsonRpcRequest::new(id, method, params);
        let serialized = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        let (tx, rx) = channel();
        self.pending_requests.lock().unwrap().insert(id, tx);

        // Write line to stdin
        if let Err(e) = writeln!(self.stdin, "{}", serialized) {
            self.pending_requests.lock().unwrap().remove(&id);
            return Err(format!("Failed to write to MCP stdin: {}", e));
        }
        let _ = self.stdin.flush();

        let start = Instant::now();
        loop {
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(res) => return res,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if start.elapsed() > timeout {
                        self.pending_requests.lock().unwrap().remove(&id);
                        return Err(format!(
                            "MCP request '{}' (id: {}) timed out after {:?}",
                            method, id, timeout
                        ));
                    }
                    if !self.is_alive() {
                        self.pending_requests.lock().unwrap().remove(&id);
                        return Err("MCP server terminated while waiting for response".to_string());
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    self.pending_requests.lock().unwrap().remove(&id);
                    return Err("MCP transport receiver channel disconnected".to_string());
                }
            }
        }
    }

    /// Sends a notification message (no response expected).
    pub fn send_notification(&mut self, method: &str, params: Option<Value>) -> Result<(), String> {
        let notif = JsonRpcNotification::new(method, params);
        let serialized = serde_json::to_string(&notif)
            .map_err(|e| format!("Failed to serialize notification: {}", e))?;

        writeln!(self.stdin, "{}", serialized)
            .map_err(|e| format!("Failed to write notification to MCP stdin: {}", e))?;
        let _ = self.stdin.flush();
        Ok(())
    }

    /// Checks if the child process is still running.
    pub fn is_alive(&mut self) -> bool {
        if !self.is_running.load(Ordering::SeqCst) {
            return false;
        }
        match self.child.try_wait() {
            Ok(None) => true,
            _ => {
                self.is_running.store(false, Ordering::SeqCst);
                false
            }
        }
    }

    /// Terminates the child process.
    pub fn terminate(&mut self) -> Result<(), String> {
        self.is_running.store(false, Ordering::SeqCst);
        let _ = self.child.kill();
        let _ = self.child.wait();
        Ok(())
    }
}

impl Drop for McpStdioTransport {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}
