use std::path::Path;
use std::sync::{Arc, Mutex};

use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::schema::v1::{
    AuthenticateRequest, ClientCapabilities, ContentBlock, FileSystemCapabilities,
    Implementation, InitializeRequest, NewSessionRequest, PromptRequest,
    ReadTextFileRequest, ReadTextFileResponse, RequestPermissionOutcome,
    RequestPermissionRequest, RequestPermissionResponse, SelectedPermissionOutcome,
    SessionConfigId, SessionConfigKind, SessionConfigSelectOptions, SessionConfigValueId,
    SessionNotification, SetSessionConfigOptionRequest, TextContent,
    WriteTextFileRequest, WriteTextFileResponse,
};
use agent_client_protocol::{AcpAgent, Agent, Client, ConnectionTo, LineDirection};
use tokio::sync::{broadcast, oneshot};

use super::config::AcpAgentConfig;
use super::types::{AcpEvent, AcpMode, AcpModel, AcpSlashCommand};

/// Generic ACP client capable of managing any ACP-compliant agent subprocess.
#[derive(Clone)]
pub struct AcpClient {
    config: AcpAgentConfig,
    connection: ConnectionTo<Agent>,
    session_id: Arc<Mutex<Option<String>>>,
    models: Arc<Mutex<Vec<AcpModel>>>,
    current_model: Arc<Mutex<Option<String>>>,
    modes: Arc<Mutex<Vec<AcpMode>>>,
    current_mode: Arc<Mutex<Option<String>>>,
    event_tx: broadcast::Sender<AcpEvent>,
}

impl AcpClient {
    /// Start the agent subprocess and complete initialization and authentication.
    pub async fn start(config: AcpAgentConfig) -> Result<Self, String> {
        let (event_tx, _) = broadcast::channel(128);
        let event_tx_clone = event_tx.clone();

        let sdk_config = config.to_sdk_config();

        let agent = AcpAgent::new(sdk_config).with_debug(|line, dir| {
            match dir {
                LineDirection::Stdin => log::trace!("[ACP STDIN] {}", line),
                LineDirection::Stdout => log::trace!("[ACP STDOUT] {}", line),
                LineDirection::Stderr => log::warn!("[ACP STDERR] {}", line),
            }
        });

        let client_builder = Client.builder();

        // 1. Notification dispatcher
        let notif_tx = event_tx_clone.clone();
        let client_builder = client_builder.on_receive_notification(
            async move |notification: SessionNotification, _cx| {
                match notification.update {
                    agent_client_protocol::schema::v1::SessionUpdate::AgentMessageChunk(chunk) => {
                        if let ContentBlock::Text(t) = chunk.content {
                            let _ = notif_tx.send(AcpEvent::TextDelta(t.text));
                        }
                    }
                    agent_client_protocol::schema::v1::SessionUpdate::AgentThoughtChunk(thought) => {
                        if let ContentBlock::Text(t) = thought.content {
                            let _ = notif_tx.send(AcpEvent::ThoughtDelta(t.text));
                        }
                    }
                    agent_client_protocol::schema::v1::SessionUpdate::AvailableCommandsUpdate(cmds) => {
                        let commands: Vec<AcpSlashCommand> = cmds
                            .available_commands
                            .into_iter()
                            .map(|c| AcpSlashCommand {
                                name: c.name,
                                description: c.description,
                            })
                            .collect();
                        let _ = notif_tx.send(AcpEvent::AvailableCommands(commands));
                    }
                    agent_client_protocol::schema::v1::SessionUpdate::ToolCall(call) => {
                        let _ = notif_tx.send(AcpEvent::ToolCall {
                            id: call.tool_call_id.0.to_string(),
                            title: call.title,
                            name: call.name,
                            raw_input: call.raw_input,
                        });
                    }
                    agent_client_protocol::schema::v1::SessionUpdate::ToolCallUpdate(update) => {
                        let _ = notif_tx.send(AcpEvent::ToolCallUpdate {
                            id: update.tool_call_id.0.to_string(),
                            status: format!("{:?}", update.fields.status),
                            raw_output: update.fields.raw_output.map(|v| match v {
                                serde_json::Value::String(s) => s,
                                serde_json::Value::Null => String::new(),
                                other => serde_json::to_string_pretty(&other).unwrap_or_default(),
                            }),
                        });
                    }
                    _ => {}
                }
                Ok(())
            },
            agent_client_protocol::on_receive_notification!(),
        );

        // 2. Permission handler (auto-approves with first option or cancels)
        let client_builder = client_builder.on_receive_request(
            async move |req: RequestPermissionRequest, responder, _cx| {
                let first_opt = req.options.first().map(|o| o.option_id.clone());
                if let Some(opt_id) = first_opt {
                    let _ = responder.respond(RequestPermissionResponse::new(
                        RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(opt_id)),
                    ));
                } else {
                    let _ = responder.respond(RequestPermissionResponse::new(
                        RequestPermissionOutcome::Cancelled,
                    ));
                }
                Ok(())
            },
            agent_client_protocol::on_receive_request!(),
        );

        // 3. File System: Read text file handler
        let client_builder = client_builder.on_receive_request(
            async move |req: ReadTextFileRequest, responder, _cx| {
                match std::fs::read_to_string(&req.path) {
                    Ok(content) => {
                        let _ = responder.respond(ReadTextFileResponse::new(content));
                    }
                    Err(e) => {
                        log::error!("ACP read_text_file error: {:?}", e);
                        let _ = responder.respond_with_error(agent_client_protocol::Error::internal_error());
                    }
                }
                Ok(())
            },
            agent_client_protocol::on_receive_request!(),
        );

        // 4. File System: Write text file handler
        let client_builder = client_builder.on_receive_request(
            async move |req: WriteTextFileRequest, responder, _cx| {
                if let Some(parent) = req.path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                match std::fs::write(&req.path, &req.content) {
                    Ok(_) => {
                        let _ = responder.respond(WriteTextFileResponse::new());
                    }
                    Err(e) => {
                        log::error!("ACP write_text_file error: {:?}", e);
                        let _ = responder.respond_with_error(agent_client_protocol::Error::internal_error());
                    }
                }
                Ok(())
            },
            agent_client_protocol::on_receive_request!(),
        );

        // 5. Connect and initialize in background task
        let (cx_tx, cx_rx) = oneshot::channel();

        tokio::spawn(async move {
            let mut cx_sender = Some(cx_tx);
            let connect_res = client_builder
                .connect_with(agent, |cx: ConnectionTo<Agent>| {
                    let sender = cx_sender.take();
                    async move {
                        if let Some(tx) = sender {
                            let _ = tx.send(cx.clone());
                        }
                        cx.incoming_closed().await;
                        Ok(())
                    }
                })
                .await;

            if let Err(e) = connect_res {
                log::error!("ACP connection ended with error: {:?}", e);
            }
        });

        let connection = cx_rx
            .await
            .map_err(|_| "Failed to establish ACP connection with agent subprocess".to_string())?;

        // 6. Protocol Handshake: InitializeRequest
        let mut fs_caps = FileSystemCapabilities::new();
        fs_caps.read_text_file = true;
        fs_caps.write_text_file = true;

        let client_caps = ClientCapabilities::new().fs(fs_caps);

        let init_req = InitializeRequest::new(ProtocolVersion::V1)
            .client_info(Implementation::new("zenthree", "0.1.0"))
            .client_capabilities(client_caps);

        let init_resp = connection
            .send_request(init_req)
            .block_task()
            .await
            .map_err(|e| format!("ACP Initialize failed: {:?}", e))?;

        log::info!(
            "ACP Agent initialized: name={:?} version={:?}",
            init_resp.agent_info.as_ref().map(|i| &i.name),
            init_resp.agent_info.as_ref().map(|i| &i.version)
        );

        // 7. Authenticate if configured
        if let Some(ref auth_method) = config.auth_method_id {
            log::info!("Authenticating ACP agent with method: {}", auth_method);
            let auth_id = agent_client_protocol::schema::v1::AuthMethodId::new(std::sync::Arc::from(auth_method.as_str()));
            let auth_req = AuthenticateRequest::new(auth_id);
            let _ = connection.send_request(auth_req).block_task().await;
        }

        Ok(Self {
            config,
            connection,
            session_id: Arc::new(Mutex::new(None)),
            models: Arc::new(Mutex::new(Vec::new())),
            current_model: Arc::new(Mutex::new(None)),
            modes: Arc::new(Mutex::new(Vec::new())),
            current_mode: Arc::new(Mutex::new(None)),
            event_tx,
        })
    }

    /// Create a new session with working directory.
    pub async fn create_session(&self, cwd: impl AsRef<Path>) -> Result<String, String> {
        let new_session_req = NewSessionRequest::new(cwd.as_ref().to_path_buf());

        let resp = self
            .connection
            .send_request(new_session_req)
            .block_task()
            .await
            .map_err(|e| format!("ACP session/new failed: {:?}", e))?;

        let session_id_str = resp.session_id.0.to_string();
        *self.session_id.lock().unwrap() = Some(session_id_str.clone());

        // Parse modes if returned
        if let Some(modes_state) = resp.modes {
            let mut parsed_modes = Vec::new();
            for m in modes_state.available_modes {
                parsed_modes.push(AcpMode {
                    id: m.id.0.to_string(),
                    name: m.name,
                    description: m.description,
                });
            }
            let cur_mode = modes_state.current_mode_id.0.to_string();
            *self.current_mode.lock().unwrap() = Some(cur_mode);
            *self.modes.lock().unwrap() = parsed_modes;
        }

        // Parse config options for models & modes
        if let Some(config_options) = resp.config_options {
            for opt in config_options {
                if let SessionConfigKind::Select(select_opt) = opt.kind {
                    if opt.id.0.as_ref() == "model" {
                        let mut parsed_models = Vec::new();
                        match select_opt.options {
                            SessionConfigSelectOptions::Ungrouped(options) => {
                                for o in options {
                                    parsed_models.push(AcpModel {
                                        id: o.value.0.to_string(),
                                        name: o.name,
                                        description: o.description,
                                    });
                                }
                            }
                            SessionConfigSelectOptions::Grouped(groups) => {
                                for g in groups {
                                    for o in g.options {
                                        parsed_models.push(AcpModel {
                                            id: o.value.0.to_string(),
                                            name: o.name,
                                            description: o.description,
                                        });
                                    }
                                }
                            }
                            _ => {}
                        }
                        let dynamic_list: Vec<super::super::antigravity_cli::ModelInfo> = parsed_models.iter().map(|m| {
                            super::super::antigravity_cli::ModelInfo {
                                id: m.id.clone(),
                                display_name: m.name.clone(),
                                max_tokens: 1_048_576,
                                supports_tools: true,
                                supports_thinking: true,
                                description: m.description.clone().unwrap_or_default(),
                            }
                        }).collect();
                        super::super::antigravity_cli::models::set_dynamic_models(dynamic_list);
                        *self.models.lock().unwrap() = parsed_models;
                        *self.current_model.lock().unwrap() = Some(select_opt.current_value.0.to_string());
                    }
                }
            }
        }

        if let Some(ref target_model) = self.config.default_model {
            let req = SetSessionConfigOptionRequest::new(
                resp.session_id.clone(),
                SessionConfigId::new("model"),
                SessionConfigValueId::new(target_model.clone()),
            );
            if let Ok(_) = self.connection.send_request(req).block_task().await {
                log::info!("ACP session model switched to: {}", target_model);
                *self.current_model.lock().unwrap() = Some(target_model.clone());
            }
        }

        Ok(session_id_str)
    }

    /// Send a prompt and stream responses through a callback until the turn finishes.
    pub async fn prompt(
        &self,
        prompt_text: &str,
        mut on_event: impl FnMut(AcpEvent) + Send + 'static,
    ) -> Result<String, String> {
        let session_id = self
            .session_id()
            .ok_or_else(|| "No active ACP session. Call create_session first.".to_string())?;

        let mut event_rx = self.event_tx.subscribe();

        let prompt_req = PromptRequest::new(
            session_id,
            vec![ContentBlock::Text(TextContent::new(prompt_text))],
        );

        let prompt_fut = self.connection.send_request(prompt_req).block_task();

        tokio::pin!(prompt_fut);

        let mut final_text = String::new();

        loop {
            tokio::select! {
                prompt_res = &mut prompt_fut => {
                    let resp = prompt_res.map_err(|e| format!("ACP Prompt failed: {:?}", e))?;
                    on_event(AcpEvent::TurnCompleted {
                        stop_reason: format!("{:?}", resp.stop_reason),
                    });
                    break;
                }
                event_res = event_rx.recv() => {
                    if let Ok(event) = event_res {
                        if let AcpEvent::TextDelta(ref text) = event {
                            final_text.push_str(text);
                        }
                        on_event(event);
                    }
                }
            }
        }

        Ok(final_text)
    }

    /// Get list of models reported by the agent.
    pub fn models(&self) -> Vec<AcpModel> {
        self.models.lock().unwrap().clone()
    }

    /// Get list of modes reported by the agent.
    pub fn modes(&self) -> Vec<AcpMode> {
        self.modes.lock().unwrap().clone()
    }

    /// Current model ID.
    pub fn current_model(&self) -> Option<String> {
        self.current_model.lock().unwrap().clone()
    }

    /// Current mode ID.
    pub fn current_mode(&self) -> Option<String> {
        self.current_mode.lock().unwrap().clone()
    }

    /// Active session ID.
    pub fn session_id(&self) -> Option<String> {
        self.session_id.lock().unwrap().clone()
    }

    /// Configuration used for this agent.
    pub fn config(&self) -> &AcpAgentConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_acp_client_antigravity() {
        let Some(config) = AcpAgentConfig::suggest_antigravity() else {
            eprintln!("Skipping test: Antigravity ACP binary not found on this system");
            return;
        };

        let client = AcpClient::start(config)
            .await
            .expect("Failed to start ACP client");

        let cwd = std::env::current_dir().unwrap();
        let session_id = client
            .create_session(&cwd)
            .await
            .expect("Failed to create ACP session");

        assert!(!session_id.is_empty());

        let models = client.models();
        println!("Reported models: {}", models.len());
        for m in &models {
            println!("  Model: {} ({})", m.name, m.id);
        }

        let modes = client.modes();
        println!("Reported modes: {}", modes.len());
        for m in &modes {
            println!("  Mode: {} ({})", m.name, m.id);
        }

        let mut received_chunks = Vec::new();
        let prompt_result = client
            .prompt("hi", move |event| {
                if let AcpEvent::TextDelta(t) = event {
                    received_chunks.push(t);
                }
            })
            .await;

        assert!(prompt_result.is_ok());
    }
}
