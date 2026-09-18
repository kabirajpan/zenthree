pub mod protocol;
pub mod transport;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use log::{warn, error};
use protocol::*;
use transport::{LspIncomingMessage, LspTransport};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerStatus {
    NotStarted,
    Starting,
    Ready,
    Failed,
}

#[derive(Debug, Clone)]
pub struct LspServerInfo {
    pub language_id: &'static str,
    pub binary_name: &'static str,
    pub display_name: &'static str,
    pub install_hint: &'static str,
}

impl LspServerInfo {
    pub fn for_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "rs" => Some(Self {
                language_id: "rust",
                binary_name: "rust-analyzer",
                display_name: "rust-analyzer",
                install_hint: "rustup component add rust-analyzer",
            }),
            "go" => Some(Self {
                language_id: "go",
                binary_name: "gopls",
                display_name: "gopls",
                install_hint: "go install golang.org/x/tools/gopls@latest",
            }),
            "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" => Some(Self {
                language_id: "cpp",
                binary_name: "clangd",
                display_name: "clangd",
                install_hint: "sudo apt install clangd",
            }),
            "py" => Some(Self {
                language_id: "python",
                binary_name: "pyright-langserver",
                display_name: "pyright",
                install_hint: "npm install -g pyright",
            }),
            "ts" | "tsx" | "js" | "jsx" => Some(Self {
                language_id: "typescript",
                binary_name: "typescript-language-server",
                display_name: "tsserver",
                install_hint: "npm install -g typescript typescript-language-server",
            }),
            _ => None,
        }
    }
}

pub struct ActiveSession {
    pub info: LspServerInfo,
    pub transport: LspTransport,
    pub status: ServerStatus,
    pub next_request_id: u64,
    pub open_documents: HashMap<PathBuf, u64>, // path -> version
}

pub struct LspManager {
    pub sessions: HashMap<&'static str, ActiveSession>,
    pub diagnostics: HashMap<PathBuf, Vec<Diagnostic>>,
    pub latest_completions: Option<Vec<CompletionItem>>,
    pub pending_completion_id: Option<u64>,
    pub latest_definition: Option<Location>,
    pub pending_definition_id: Option<u64>,
    pub latest_hover: Option<Hover>,
    pub pending_hover_id: Option<u64>,
}

impl LspManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            diagnostics: HashMap::new(),
            latest_completions: None,
            pending_completion_id: None,
            latest_definition: None,
            pending_definition_id: None,
            latest_hover: None,
            pending_hover_id: None,
        }
    }

    /// Check if a binary exists in system PATH
    pub fn is_server_installed(binary_name: &str) -> bool {
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path_var) {
                let full = dir.join(binary_name);
                if full.is_file() {
                    return true;
                }
                #[cfg(windows)]
                {
                    let full_exe = dir.join(format!("{}.exe", binary_name));
                    let full_cmd = dir.join(format!("{}.cmd", binary_name));
                    if full_exe.is_file() || full_cmd.is_file() {
                        return true;
                    }
                }
            }
        }
        std::process::Command::new("which")
            .arg(binary_name)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
            || std::process::Command::new("where")
                .arg(binary_name)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
    }

    /// Ensure a session exists for the given file's language
    pub fn ensure_session_for_file(&mut self, path: &Path, workspace_root: Option<&Path>) -> Option<&mut ActiveSession> {
        let info = LspServerInfo::for_path(path)?;
        let lang = info.language_id;

        if !self.sessions.contains_key(lang) {
            // Attempt to start
            if !Self::is_server_installed(info.binary_name) {
                return None;
            }

            let args: &[&str] = match info.binary_name {
                "typescript-language-server" => &["--stdio"],
                "pyright-langserver" => &["--stdio"],
                _ => &[],
            };

            match LspTransport::start(info.binary_name, args) {
                Ok(transport) => {
                    let mut session = ActiveSession {
                        info: info.clone(),
                        transport,
                        status: ServerStatus::Starting,
                        next_request_id: 1,
                        open_documents: HashMap::new(),
                    };

                    // Send initialize request
                    let root_uri = workspace_root.map(|p| format!("file://{}", p.display()));
                    let init_params = InitializeParams {
                        process_id: Some(std::process::id()),
                        client_info: Some(ClientInfo {
                            name: "Zenthree".to_string(),
                            version: Some("0.1.0".to_string()),
                        }),
                        root_uri,
                        capabilities: serde_json::json!({
                            "textDocument": {
                                "synchronization": {
                                    "openClose": true,
                                    "change": 1 // Full sync
                                },
                                "completion": {
                                    "completionItem": {
                                        "snippetSupport": false,
                                        "documentationFormat": ["plaintext"]
                                    }
                                }
                            }
                        }),
                    };

                    let req = JsonRpcRequest::new(session.next_request_id, "initialize", init_params);
                    session.next_request_id += 1;
                    if let Ok(bytes) = encode_lsp_message(&req) {
                        let _ = session.transport.send_raw(&bytes);
                    }

                    // Notification: initialized
                    let notif = JsonRpcNotification::new("initialized", serde_json::json!({}));
                    if let Ok(bytes) = encode_lsp_message(&notif) {
                        let _ = session.transport.send_raw(&bytes);
                    }

                    session.status = ServerStatus::Ready;
                    self.sessions.insert(lang, session);
                }
                Err(e) => {
                    error!("Failed to start LSP {}: {:?}", info.binary_name, e);
                    return None;
                }
            }
        }

        self.sessions.get_mut(lang)
    }

    /// Notify LSP that a file was opened
    pub fn on_document_open(&mut self, path: &Path, content: &str, workspace_root: Option<&Path>) {
        let uri = format!("file://{}", path.display());
        let info = if let Some(i) = LspServerInfo::for_path(path) { i } else { return; };
        let lang_id = info.language_id.to_string();

        if let Some(session) = self.ensure_session_for_file(path, workspace_root) {
            session.open_documents.insert(path.to_path_buf(), 1);
            let params = DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri,
                    language_id: lang_id,
                    version: 1,
                    text: content.to_string(),
                },
            };
            let notif = JsonRpcNotification::new("textDocument/didOpen", params);
            if let Ok(bytes) = encode_lsp_message(&notif) {
                let _ = session.transport.send_raw(&bytes);
            }
        }
    }

    /// Notify LSP that document content changed
    pub fn on_document_change(&mut self, path: &Path, content: &str) {
        let uri = format!("file://{}", path.display());
        let info = if let Some(i) = LspServerInfo::for_path(path) { i } else { return; };

        if let Some(session) = self.sessions.get_mut(info.language_id) {
            let ver = session.open_documents.entry(path.to_path_buf()).or_insert(0);
            *ver += 1;
            let current_ver = *ver;

            let params = DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier {
                    uri,
                    version: current_ver,
                },
                content_changes: vec![TextDocumentContentChangeEvent {
                    text: content.to_string(),
                }],
            };
            let notif = JsonRpcNotification::new("textDocument/didChange", params);
            if let Ok(bytes) = encode_lsp_message(&notif) {
                let _ = session.transport.send_raw(&bytes);
            }
        }
    }

    /// Notify LSP that document was closed
    pub fn on_document_close(&mut self, path: &Path) {
        let uri = format!("file://{}", path.display());
        let info = if let Some(i) = LspServerInfo::for_path(path) { i } else { return; };

        if let Some(session) = self.sessions.get_mut(info.language_id) {
            session.open_documents.remove(path);
            let params = DidCloseTextDocumentParams {
                text_document: TextDocumentIdentifier { uri },
            };
            let notif = JsonRpcNotification::new("textDocument/didClose", params);
            if let Ok(bytes) = encode_lsp_message(&notif) {
                let _ = session.transport.send_raw(&bytes);
            }
        }
        self.diagnostics.remove(path);
    }

    /// Request completions at the given line and character
    pub fn request_completion(&mut self, path: &Path, line: u32, character: u32) {
        let uri = format!("file://{}", path.display());
        let info = if let Some(i) = LspServerInfo::for_path(path) { i } else { return; };

        if let Some(session) = self.sessions.get_mut(info.language_id) {
            let req_id = session.next_request_id;
            session.next_request_id += 1;
            self.pending_completion_id = Some(req_id);

            let params = CompletionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character },
            };
            let req = JsonRpcRequest::new(req_id, "textDocument/completion", params);
            if let Ok(bytes) = encode_lsp_message(&req) {
                let _ = session.transport.send_raw(&bytes);
            }
        }
    }

    /// Request definition for symbol at line and character
    pub fn request_definition(&mut self, path: &Path, line: u32, character: u32) {
        let uri = format!("file://{}", path.display());
        let info = if let Some(i) = LspServerInfo::for_path(path) { i } else { return; };

        if let Some(session) = self.sessions.get_mut(info.language_id) {
            let req_id = session.next_request_id;
            session.next_request_id += 1;
            self.pending_definition_id = Some(req_id);

            let params = GotoDefinitionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character },
            };
            let req = JsonRpcRequest::new(req_id, "textDocument/definition", params);
            if let Ok(bytes) = encode_lsp_message(&req) {
                let _ = session.transport.send_raw(&bytes);
            }
        }
    }

    /// Request hover documentation for symbol at line and character
    pub fn request_hover(&mut self, path: &Path, line: u32, character: u32) {
        let uri = format!("file://{}", path.display());
        let info = if let Some(i) = LspServerInfo::for_path(path) { i } else { return; };

        if let Some(session) = self.sessions.get_mut(info.language_id) {
            let req_id = session.next_request_id;
            session.next_request_id += 1;
            self.pending_hover_id = Some(req_id);

            let params = HoverParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character },
            };
            let req = JsonRpcRequest::new(req_id, "textDocument/hover", params);
            if let Ok(bytes) = encode_lsp_message(&req) {
                let _ = session.transport.send_raw(&bytes);
            }
        }
    }

    /// Poll incoming LSP messages across all sessions (called once per frame)
    pub fn poll(&mut self) -> bool {
        let mut had_updates = false;

        let langs: Vec<&'static str> = self.sessions.keys().cloned().collect();
        for lang in langs {
            let mut incoming = Vec::new();
            if let Some(session) = self.sessions.get_mut(lang) {
                while let Some(msg) = session.transport.try_recv() {
                    incoming.push(msg);
                }
            }

            for msg in incoming {
                had_updates = true;
                match msg {
                    LspIncomingMessage::Notification { method, params } => {
                        if method == "textDocument/publishDiagnostics" {
                            if let Some(p) = params {
                                if let Ok(diag_params) = serde_json::from_value::<PublishDiagnosticsParams>(p) {
                                    let path_str = diag_params.uri.trim_start_matches("file://");
                                    let path = PathBuf::from(path_str);
                                    self.diagnostics.insert(path, diag_params.diagnostics);
                                }
                            }
                        }
                    }
                    LspIncomingMessage::Response { id, result, .. } => {
                        if Some(id) == self.pending_completion_id {
                            self.pending_completion_id = None;
                            if let Some(res) = result.clone() {
                                if let Ok(comp_res) = serde_json::from_value::<CompletionResponse>(res) {
                                    match comp_res {
                                        CompletionResponse::List(l) => self.latest_completions = Some(l.items),
                                        CompletionResponse::Array(a) => self.latest_completions = Some(a),
                                    }
                                }
                            }
                        }
                        if Some(id) == self.pending_definition_id {
                            self.pending_definition_id = None;
                            if let Some(res) = result.clone() {
                                if let Ok(def_res) = serde_json::from_value::<GotoDefinitionResponse>(res) {
                                    match def_res {
                                        GotoDefinitionResponse::Scalar(loc) => self.latest_definition = Some(loc),
                                        GotoDefinitionResponse::Array(mut locs) => {
                                            self.latest_definition = locs.pop();
                                        }
                                    }
                                }
                            }
                        }
                        if Some(id) == self.pending_hover_id {
                            self.pending_hover_id = None;
                            if let Some(res) = result {
                                if let Ok(hover) = serde_json::from_value::<Hover>(res) {
                                    self.latest_hover = Some(hover);
                                }
                            }
                        }
                    }
                    LspIncomingMessage::ServerDied => {
                        warn!("LSP server for {} terminated", lang);
                        if let Some(session) = self.sessions.get_mut(lang) {
                            session.status = ServerStatus::Failed;
                        }
                    }
                }
            }
        }

        had_updates
    }

    /// Get total error and warning count for a path
    pub fn get_diagnostic_counts(&self, path: Option<&Path>) -> (usize, usize) {
        if let Some(p) = path {
            if let Some(diags) = self.diagnostics.get(p) {
                let mut errors = 0;
                let mut warnings = 0;
                for d in diags {
                    match d.severity {
                        Some(DiagnosticSeverity::Error) => errors += 1,
                        Some(DiagnosticSeverity::Warning) => warnings += 1,
                        _ => {}
                    }
                }
                return (errors, warnings);
            }
        }
        (0, 0)
    }
}
