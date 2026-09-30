use std::sync::Arc;
use serde_json::Value;

use crate::traits::provider::Provider;
use crate::types::error::ProviderError;
use crate::types::event::ProviderEvent;
use crate::types::message::{Message, Role};

use super::super::acp::{AcpAgentConfig, AcpClient, AcpEvent};
use super::auth::AntigravityAuthManager;
use super::models::{get_model_info, ModelInfo, DEFAULT_MODEL_ID};

/// Antigravity ACP Provider.
/// Connects to Google Antigravity backend through the official ACP agent protocol.
pub struct AntigravityCliProvider {
    config: AcpAgentConfig,
    model_info: ModelInfo,
    display_name: String,
}

impl AntigravityCliProvider {
    pub fn new(_auth_manager: Arc<AntigravityAuthManager>) -> Self {
        Self::with_model(_auth_manager, DEFAULT_MODEL_ID)
    }

    pub fn with_model(_auth_manager: Arc<AntigravityAuthManager>, model_id: &str) -> Self {
        let model_info = get_model_info(model_id);
        let display_name = format!("Google Antigravity ({})", model_info.display_name);

        let mut config = AcpAgentConfig::suggest_antigravity().unwrap_or_else(|| {
            AcpAgentConfig::new(
                "antigravity-acp",
                "Google Antigravity",
                "agy_acp_server.par",
            )
        });
        config.default_model = Some(model_id.to_string());

        Self {
            config,
            model_info,
            display_name,
        }
    }

    pub fn model_info(&self) -> &ModelInfo {
        &self.model_info
    }
}

impl Provider for AntigravityCliProvider {
    fn name(&self) -> &str {
        &self.display_name
    }

    fn complete_stream(
        &self,
        messages: &[Message],
        _tools: &[Value],
        on_event: &mut dyn FnMut(ProviderEvent),
    ) -> Result<(), ProviderError> {
        let last_user_msg = messages
            .iter()
            .rev()
            .find(|m| m.role == Role::User)
            .map(|m| m.content.clone())
            .unwrap_or_else(|| "Hello".to_string());

        let system_ctx = messages
            .iter()
            .find(|m| m.role == Role::System)
            .map(|m| m.content.as_str());

        let prompt = if let Some(sys) = system_ctx {
            format!("[Workspace Context]\n{}\n\n[User Instruction]\n{}", sys, last_user_msg)
        } else {
            last_user_msg
        };

        let config = self.config.clone();

        // Run ACP prompt synchronously for Provider trait
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let prompt_thread = std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(r) => r,
                Err(e) => return Err(format!("Failed to build tokio runtime: {}", e)),
            };

            rt.block_on(async {
                let client = AcpClient::start(config)
                    .await
                    .map_err(|e| format!("Failed to start ACP client: {}", e))?;

                let cwd = std::env::current_dir().unwrap_or_default();
                let _ = client
                    .create_session(&cwd)
                    .await
                    .map_err(|e| format!("Failed to create ACP session: {}", e))?;

                let _ = client
                    .prompt(&prompt, move |event| {
                        let _ = tx.send(event);
                    })
                    .await
                    .map_err(|e| format!("Prompt failed: {}", e))?;

                Ok::<(), String>(())
            })
        });

        while let Some(event) = rx.blocking_recv() {
            match event {
                AcpEvent::TextDelta(chunk) => {
                    on_event(ProviderEvent::TextDelta(chunk));
                }
                AcpEvent::ThoughtDelta(chunk) => {
                    on_event(ProviderEvent::ThoughtDelta(chunk));
                }
                AcpEvent::ToolCall { title, name, raw_input, .. } => {
                    let name_candidate = name.unwrap_or_else(|| {
                        title
                            .strip_prefix("Running ")
                            .unwrap_or(&title)
                            .trim()
                            .to_string()
                    });

                    let mapped_name = match name_candidate.as_str() {
                        "view_file" | "client_view_file" => "read_file",
                        "find_file" | "search_directory" | "grep_search" | "search_files" | "find_in_files" => "search_codebase",
                        "run_command" | "execute_command" | "bash" | "terminal" | "run_terminal" | "sh" | "exec" => "run_terminal",
                        other => other,
                    };

                    let mut args = raw_input.unwrap_or_else(|| serde_json::json!({}));
                    if let serde_json::Value::Object(ref mut map) = args {
                        if let Some(abs) = map.remove("absolute_path") {
                            map.insert("path".to_string(), abs);
                        } else if let Some(dir) = map.remove("directory_path") {
                            map.insert("path".to_string(), dir);
                        } else if let Some(sp) = map.remove("search_path") {
                            map.insert("path".to_string(), sp);
                        }
                        if let Some(pat) = map.get("pattern").and_then(|v| v.as_str()).map(String::from) {
                            if !map.contains_key("query") {
                                map.insert("query".to_string(), serde_json::Value::String(pat));
                            }
                        }
                        if let Some(cmd) = map.remove("CommandLine").or_else(|| map.remove("cmd")) {
                            if !map.contains_key("command") {
                                map.insert("command".to_string(), cmd);
                            }
                        }
                    }

                    let is_explicit_cmd = mapped_name == "run_terminal"
                        || name_candidate.starts_with("ls ")
                        || name_candidate == "ls"
                        || name_candidate.starts_with("env ")
                        || name_candidate.starts_with("git ")
                        || name_candidate.starts_with("cargo ")
                        || name_candidate.starts_with("npm ")
                        || name_candidate.starts_with("cat ")
                        || name_candidate.starts_with("grep ")
                        || name_candidate.contains('|');

                    let final_mapped_name = if is_explicit_cmd {
                        if let serde_json::Value::Object(ref mut map) = args {
                            if !map.contains_key("command") {
                                let cmd_str = if name_candidate != "run_terminal" && name_candidate != "run_command" {
                                    name_candidate.clone()
                                } else {
                                    title.strip_prefix("Running ").unwrap_or(&title).trim().to_string()
                                };
                                map.insert("command".to_string(), serde_json::Value::String(cmd_str));
                            }
                        }
                        "run_terminal"
                    } else {
                        mapped_name
                    };

                    on_event(ProviderEvent::ExternalToolActivity {
                        name: final_mapped_name.to_string(),
                        args,
                    });
                }
                AcpEvent::TurnCompleted { .. } => {
                    on_event(ProviderEvent::Done);
                }
                AcpEvent::Error(err) => {
                    on_event(ProviderEvent::Error(err));
                }
                _ => {}
            }
        }

        match prompt_thread.join() {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(ProviderError::Network(e)),
            Err(_) => Err(ProviderError::Network("ACP prompt thread panicked".to_string())),
        }
    }
}
