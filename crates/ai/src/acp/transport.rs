use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::acp::protocol::{AcpIncomingMessage, AcpNotification, AcpRequest, AcpResponse};

pub struct AcpTransport {
    child: Option<Child>,
    stdin_tx: Sender<String>,
    next_id: AtomicU64,
    pending_requests: Arc<Mutex<HashMap<u64, SyncSender<AcpResponse>>>>,
    notification_rx: Receiver<AcpNotification>,
}

impl AcpTransport {
    /// Spawns a child process with piped stdin/stdout and establishes the JSON-RPC IPC.
    pub fn spawn(command: &str, args: &[&str], current_dir: Option<&std::path::Path>) -> Result<Self, String> {
        Self::spawn_with_env(command, args, current_dir, &[])
    }

    /// Spawns a child process with explicit environment variables, piped stdin/stdout, and JSON-RPC IPC.
    pub fn spawn_with_env(
        command: &str,
        args: &[&str],
        current_dir: Option<&std::path::Path>,
        env_vars: &[(String, String)],
    ) -> Result<Self, String> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(dir) = current_dir {
            cmd.current_dir(dir);
        }

        for (k, v) in env_vars {
            cmd.env(k, v);
        }

        let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn ACP agent process '{}': {}", command, e))?;

        let stdin = child.stdin.take().ok_or("Failed to open child process stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to open child process stdout")?;
        let stderr = child.stderr.take();

        // Stdin writer thread
        let (stdin_tx, stdin_rx) = channel::<String>();
        thread::Builder::new()
            .name("acp-stdin-writer".to_string())
            .spawn(move || {
                let mut writer = stdin;
                while let Ok(msg) = stdin_rx.recv() {
                    if writeln!(writer, "{}", msg).is_err() || writer.flush().is_err() {
                        break;
                    }
                }
            })
            .map_err(|e| format!("Failed to spawn stdin thread: {}", e))?;

        // Stderr logger thread
        if let Some(stderr) = stderr {
            thread::Builder::new()
                .name("acp-stderr-logger".to_string())
                .spawn(move || {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines().map_while(Result::ok) {
                        log::debug!("[Goose/ACP Stderr] {}", line);
                    }
                })
                .ok();
        }

        let pending_requests: Arc<Mutex<HashMap<u64, SyncSender<AcpResponse>>>> = Arc::new(Mutex::new(HashMap::new()));
        let pending_clone = Arc::clone(&pending_requests);

        let (notify_tx, notify_rx) = channel::<AcpNotification>();

        // Stdout reader thread
        thread::Builder::new()
            .name("acp-stdout-reader".to_string())
            .spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    if let Ok(incoming) = serde_json::from_str::<AcpIncomingMessage>(trimmed) {
                        match incoming {
                            AcpIncomingMessage::Response(resp) => {
                                if let Ok(mut pending) = pending_clone.lock() {
                                    if let Some(tx) = pending.remove(&resp.id) {
                                        let _ = tx.send(resp);
                                    }
                                }
                            }
                            AcpIncomingMessage::Notification(notify) => {
                                let _ = notify_tx.send(notify);
                            }
                        }
                    } else {
                        log::warn!("[Goose/ACP] Non-JSON-RPC line received: {}", trimmed);
                    }
                }
            })
            .map_err(|e| format!("Failed to spawn stdout thread: {}", e))?;

        Ok(Self {
            child: Some(child),
            stdin_tx,
            next_id: AtomicU64::new(1),
            pending_requests,
            notification_rx: notify_rx,
        })
    }

    /// Sends a JSON-RPC request and synchronously waits for the response (with timeout)
    pub fn send_request(&self, method: &str, params: Option<serde_json::Value>, timeout: Duration) -> Result<AcpResponse, String> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let req = AcpRequest::new(id, method, params);

        let (res_tx, res_rx) = std::sync::mpsc::sync_channel::<AcpResponse>(1);
        if let Ok(mut pending) = self.pending_requests.lock() {
            pending.insert(id, res_tx);
        } else {
            return Err("Failed to acquire pending requests lock".to_string());
        }

        let json_str = serde_json::to_string(&req).map_err(|e| format!("Failed to serialize ACP request: {}", e))?;
        self.stdin_tx.send(json_str).map_err(|e| format!("Failed to send ACP request: {}", e))?;

        res_rx.recv_timeout(timeout).map_err(|e| {
            if let Ok(mut pending) = self.pending_requests.lock() {
                pending.remove(&id);
            }
            format!("ACP request '{}' (id {}) timed out or channel closed: {}", method, id, e)
        })
    }

    /// Sends a JSON-RPC request and returns the request ID and receiver channel for asynchronous response handling.
    pub fn send_request_channel(&self, method: &str, params: Option<serde_json::Value>) -> Result<(u64, Receiver<AcpResponse>), String> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let req = AcpRequest::new(id, method, params);

        let (res_tx, res_rx) = std::sync::mpsc::sync_channel::<AcpResponse>(1);
        if let Ok(mut pending) = self.pending_requests.lock() {
            pending.insert(id, res_tx);
        } else {
            return Err("Failed to acquire pending requests lock".to_string());
        }

        let json_str = serde_json::to_string(&req).map_err(|e| format!("Failed to serialize ACP request: {}", e))?;
        self.stdin_tx.send(json_str).map_err(|e| format!("Failed to send ACP request: {}", e))?;
        let (res_tx_unbounded, res_rx_unbounded) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            if let Ok(resp) = res_rx.recv() {
                let _ = res_tx_unbounded.send(resp);
            }
        });
        Ok((id, res_rx_unbounded))
    }

    /// Sends a fire-and-forget notification
    pub fn send_notification(&self, method: &str, params: Option<serde_json::Value>) -> Result<(), String> {
        let notif = AcpNotification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
        };
        let json_str = serde_json::to_string(&notif).map_err(|e| format!("Failed to serialize ACP notification: {}", e))?;
        self.stdin_tx.send(json_str).map_err(|e| format!("Failed to send ACP notification: {}", e))
    }

    /// Receives any incoming notification without blocking
    pub fn try_recv_notification(&self) -> Option<AcpNotification> {
        self.notification_rx.try_recv().ok()
    }

    /// Returns a reference to the notification receiver
    pub fn notification_receiver(&self) -> &Receiver<AcpNotification> {
        &self.notification_rx
    }

    /// Checks if the child process is still alive
    pub fn is_alive(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(None) => true,
                Ok(Some(status)) => {
                    log::warn!("[Goose/ACP] Process exited with status: {}", status);
                    false
                }
                Err(e) => {
                    log::error!("[Goose/ACP] Error checking process status: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }
}

impl Drop for AcpTransport {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
