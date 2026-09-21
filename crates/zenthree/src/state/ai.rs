use crate::state::types::*;
use crate::state::ZenthreeState;

impl ZenthreeState {
    pub fn send_ai_message(&mut self, text: &str) {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }

        // Keep maximum 60 messages to bound memory
        if self.ai_messages.len() >= 60 {
            let remove_count = self.ai_messages.len() - 58;
            self.ai_messages.drain(0..remove_count);
        }

        self.ai_messages.push(AiMessage {
            is_user: true,
            content: trimmed.to_string(),
            code_snippet: None,
            thoughts: None,
            expanded: false,
        });

        self.ai_messages.push(AiMessage {
            is_user: false,
            content: "Thinking...".to_string(),
            code_snippet: None,
            thoughts: None,
            expanded: false,
        });

        self.ai_input.clear();

        let (tx, rx) = std::sync::mpsc::channel::<AiUiUpdate>();
        self.ai_response_rx = Some(rx);

        let prompt = trimmed.to_string();
        // Pass workspace dir so tools run on the right project
        let current_dir = self.current_dir.clone();
        // Pass existing session JSON so the agent remembers prior conversation
        let session_json = self.ai_session_json.clone();
        let is_fresh_session = session_json.is_none();
        // Pass selected model so the thread uses the right provider
        let selected_model = self.selected_ai_model;

        std::thread::spawn(move || {
            // Set CWD so all tools (get_project_structure, search_codebase, etc.)
            // operate on the user's workspace, not the binary's launch directory.
            let _ = std::env::set_current_dir(&current_dir);

            if let Some(mut agent) = agent::create_agent_for_model(
                match selected_model.provider() {
                    crate::state::AiProvider::Gemini => "gemini",
                    crate::state::AiProvider::Groq => "groq",
                },
                selected_model.model_id(),
            ) {
                // Restore conversation history from previous messages
                if let Some(ref json) = session_json {
                    let _ = agent.import_session_json(json);
                }

                // On the first turn inject a system message with workspace context
                if is_fresh_session {
                    agent.push_system_message(format!(
                        "You are an AI coding assistant embedded in the Zenthra IDE. \
                        The user's current workspace is: {}. \
                        Use the available tools to explore the project before answering code questions. \
                        Always prefer targeted reads (start_line/end_line) over dumping entire files.",
                        current_dir.display()
                    ));
                }

                let mut final_text = String::new();
                let mut on_event = |ev: agent::types::AgentEvent| match ev {
                    agent::types::AgentEvent::ToolStarted { name } => {
                        let _ = tx.send(AiUiUpdate::Status(format!("🔧 Calling {name}...")));
                    }
                    agent::types::AgentEvent::ToolFinished { name, .. } => {
                        let _ = tx.send(AiUiUpdate::Status(format!(
                            "🔧 Finished {name}, analyzing..."
                        )));
                    }
                    agent::types::AgentEvent::Text(delta) => {
                        final_text.push_str(&delta);
                    }
                    _ => {}
                };
                let mut ask_approval =
                    |_: agent::types::ApprovalRequest| agent::types::ApprovalDecision::Approve;

                match agent.run(&prompt, &mut on_event, &mut ask_approval) {
                    Ok(text) => {
                        let reply = if text.is_empty() { final_text } else { text };
                        // Export the updated conversation so the next message has full memory
                        let new_session = agent.export_session_json().ok();
                        let _ = tx.send(AiUiUpdate::Done {
                            reply,
                            session_json: new_session,
                        });
                    }
                    Err(e) => {
                        let _ = tx.send(AiUiUpdate::Done {
                            reply: format!("Agent error: {e}"),
                            session_json,
                        });
                    }
                }
            } else {
                // Fallback: no provider key found
                match agent::provider::complete(&prompt) {
                    Ok(content) => {
                        let _ = tx.send(AiUiUpdate::Done {
                            reply: content,
                            session_json: None,
                        });
                    }
                    Err(err) => {
                        let _ = tx.send(AiUiUpdate::Done {
                            reply: format!("Error: {err}"),
                            session_json: None,
                        });
                    }
                }
            };
        });
    }

    pub fn poll_ai_response(&mut self) -> bool {
        let mut changed = false;
        if let Some(ref rx) = self.ai_response_rx {
            loop {
                match rx.try_recv() {
                    Ok(update) => {
                        changed = true;
                        match update {
                            AiUiUpdate::Status(status) => {
                                if let Some(last) =
                                    self.ai_messages.iter_mut().rev().find(|m| !m.is_user)
                                {
                                    // Evict old cached blocks before overwriting content
                                    let old_hash = {
                                        use std::hash::{Hash, Hasher};
                                        let mut h =
                                            std::collections::hash_map::DefaultHasher::new();
                                        last.content.hash(&mut h);
                                        h.finish()
                                    };
                                    self.ai_block_cache.remove(&old_hash);
                                    last.content = status;
                                }
                            }
                            AiUiUpdate::Done {
                                reply,
                                session_json,
                            } => {
                                if let Some(last) =
                                    self.ai_messages.iter_mut().rev().find(|m| !m.is_user)
                                {
                                    // Evict stale cache entry
                                    let old_hash = {
                                        use std::hash::{Hash, Hasher};
                                        let mut h =
                                            std::collections::hash_map::DefaultHasher::new();
                                        last.content.hash(&mut h);
                                        h.finish()
                                    };
                                    self.ai_block_cache.remove(&old_hash);
                                    last.content = reply;
                                } else {
                                    self.ai_messages.push(AiMessage {
                                        is_user: false,
                                        content: reply,
                                        code_snippet: None,
                                        thoughts: None,
                                        expanded: false,
                                    });
                                }
                                // Persist updated conversation history for the next message
                                if session_json.is_some() {
                                    self.ai_session_json = session_json;
                                }
                                self.ai_response_rx = None;
                                return true;
                            }
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        if let Some(last) = self.ai_messages.iter_mut().rev().find(|m| !m.is_user) {
                            if last.content == "Thinking..." || last.content.starts_with("🔧") {
                                last.content = "Connection closed without response.".to_string();
                            }
                        }
                        self.ai_response_rx = None;
                        changed = true;
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                }
            }
        }
        changed
    }
}
