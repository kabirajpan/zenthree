use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use log::{error, info};

pub enum LspIncomingMessage {
    Response { id: u64, result: Option<serde_json::Value>, error: Option<serde_json::Value> },
    Notification { method: String, params: Option<serde_json::Value> },
    ServerDied,
}

pub struct LspTransport {
    _child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    receiver: Receiver<LspIncomingMessage>,
}

impl LspTransport {
    pub fn start(binary_name: &str, args: &[&str]) -> Result<Self, std::io::Error> {
        info!("Spawning LSP process: {} {:?}", binary_name, args);
        let mut child = Command::new(binary_name)
            .args(args)
            .env("PATH", crate::lsp::enriched_path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdin = child.stdin.take().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to capture child stdin")
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to capture child stdout")
        })?;

        let (tx, rx) = channel::<LspIncomingMessage>();

        // Background reader thread
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut header_buf = String::new();

            loop {
                header_buf.clear();
                let mut content_length: Option<usize> = None;

                // Read headers until empty line "\r\n"
                loop {
                    let mut line = String::new();
                    match reader.read_line(&mut line) {
                        Ok(0) => {
                            // EOF, server exited
                            let _ = tx.send(LspIncomingMessage::ServerDied);
                            return;
                        }
                        Ok(_) => {
                            let trimmed = line.trim();
                            if trimmed.is_empty() {
                                break; // Header end
                            }
                            if let Some(rest) = trimmed.strip_prefix("Content-Length:") {
                                if let Ok(len) = rest.trim().parse::<usize>() {
                                    content_length = Some(len);
                                }
                            }
                        }
                        Err(e) => {
                            error!("LSP read error: {:?}", e);
                            let _ = tx.send(LspIncomingMessage::ServerDied);
                            return;
                        }
                    }
                }

                if let Some(len) = content_length {
                    let mut body_buf = vec![0u8; len];
                    if let Err(e) = reader.read_exact(&mut body_buf) {
                        error!("LSP body read error: {:?}", e);
                        let _ = tx.send(LspIncomingMessage::ServerDied);
                        return;
                    }

                    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&body_buf) {
                        if let Some(id) = value.get("id").and_then(|v| v.as_u64()) {
                            // Response
                            let result = value.get("result").cloned();
                            let error = value.get("error").cloned();
                            let _ = tx.send(LspIncomingMessage::Response { id, result, error });
                        } else if let Some(method) = value.get("method").and_then(|m| m.as_str()) {
                            // Notification
                            let params = value.get("params").cloned();
                            let _ = tx.send(LspIncomingMessage::Notification {
                                method: method.to_string(),
                                params,
                            });
                        }
                    }
                }
            }
        });

        Ok(Self {
            _child: child,
            stdin: Arc::new(Mutex::new(stdin)),
            receiver: rx,
        })
    }

    pub fn send_raw(&self, bytes: &[u8]) -> Result<(), std::io::Error> {
        let mut stdin = self.stdin.lock().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "LSP stdin mutex poisoned")
        })?;
        stdin.write_all(bytes)?;
        stdin.flush()?;
        Ok(())
    }

    pub fn try_recv(&self) -> Option<LspIncomingMessage> {
        self.receiver.try_recv().ok()
    }
}
