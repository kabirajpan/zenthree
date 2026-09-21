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
            trajectory: Vec::new(),
            is_working: false,
        });

        self.ai_messages.push(AiMessage {
            is_user: false,
            content: String::new(),
            code_snippet: None,
            thoughts: None,
            expanded: false,
            trajectory: vec![TrajectoryStep::Exploration(ExplorationGroup {
                expanded: false,
                items: vec![ExplorationItem::Thought {
                    summary: "Planning approach...".to_string(),
                    duration_secs: None,
                }],
                is_active: true,
            })],
            is_working: true,
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

        // Gather workspace & active tab context
        let mut open_file_infos = Vec::new();
        let active_idx = self.active_tab;
        for (idx, tab) in self.tabs.iter().enumerate() {
            if let Some(ref p) = tab.path {
                let is_active = Some(idx) == active_idx;
                let mut info = agent::context::OpenFileInfo::new(p.clone(), is_active);
                if is_active {
                    let cursor = tab.editor.cursor();
                    let content = tab.editor.buffer.to_string();
                    info = info.with_cursor(cursor.row + 1, cursor.column + 1, &content, 10);
                }
                open_file_infos.push(info);
            }
        }
        let ws_context_prompt = agent::context::WorkspaceContext::new(&current_dir)
            .with_open_files(open_file_infos)
            .format_for_prompt();
        let git_context_prompt = agent::context::GitContext::format_for_prompt(&current_dir);

        std::thread::spawn(move || {
            // Set CWD so all tools (get_project_structure, search_codebase, etc.)
            // operate on the user's workspace, not the binary's launch directory.
            let _ = std::env::set_current_dir(&current_dir);

            if let Some(mut agent) = agent::create_agent_for_model_and_workspace(
                match selected_model.provider() {
                    crate::state::AiProvider::Gemini => "gemini",
                    crate::state::AiProvider::Groq => "groq",
                },
                selected_model.model_id(),
                &current_dir,
            ) {
                // Restore conversation history from previous messages
                if let Some(ref json) = session_json {
                    let _ = agent.import_session_json(json);
                }

                // On the first turn inject a system message with workspace context, git context, and ZENE skills
                if is_fresh_session {
                    let skills_summary = agent.skills().prompt_summary();
                    let mut sys_msg = format!(
                        "You are ZENE, an advanced AI coding assistant embedded in the Zenthree IDE.\n\
                        Workspace: {}\n\
                        Guidelines:\n\
                        - Use available tools to explore the codebase before editing.\n\
                        - Prefer surgical edits using `edit_file` rather than rewriting whole files.\n\
                        - Run `get_diagnostics` or test suites via `run_terminal` to verify your changes.\n\
                        - Activate workflow skills on demand via `activate_skill(name: \"...\")` for step-by-step guidance.\n\n\
                        Available Workflow Skills:\n{}\n\n{}",
                        current_dir.display(),
                        skills_summary,
                        ws_context_prompt
                    );
                    if let Some(ref git_ctx) = git_context_prompt {
                        sys_msg.push_str("\n\n");
                        sys_msg.push_str(git_ctx);
                    }
                    agent.push_system_message(sys_msg);
                }

                let mut final_text = String::new();
                let mut on_event = |ev: agent::types::AgentEvent| match ev {
                    agent::types::AgentEvent::ToolStarted { name, args } => {
                        let _ = tx.send(AiUiUpdate::ToolStart { name, args });
                    }
                    agent::types::AgentEvent::ToolFinished { name, .. } => {
                        let _ = tx.send(AiUiUpdate::ToolDone { name });
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
        let mut files_to_open: Vec<std::path::PathBuf> = Vec::new();
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
                                    last.content = status;
                                }
                            }
                            AiUiUpdate::ToolStart { name, args } => {
                                if let Some(last) =
                                    self.ai_messages.iter_mut().rev().find(|m| !m.is_user)
                                {
                                    match name.as_str() {
                                        "edit_file" => {
                                            if let Some(TrajectoryStep::Exploration(ref mut grp)) =
                                                last.trajectory.last_mut()
                                            {
                                                grp.is_active = false;
                                            }
                                            let path = args
                                                .get("path")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("file")
                                                .to_string();
                                            let target = args
                                                .get("target")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("");
                                            let replacement = args
                                                .get("replacement")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("");
                                            let diff = agent::context::GroupedDiff::from_replacement(target, replacement);
                                            let lines_removed = diff.lines_removed.max(1);
                                            let lines_added = diff.lines_added.max(1);
                                            last.trajectory.push(TrajectoryStep::FileEdit {
                                                path: path.clone(),
                                                lines_added,
                                                lines_removed,
                                            });

                                            let path_buf = std::path::PathBuf::from(&path);
                                            let mut orig_content = String::new();
                                            let mut new_content_str = String::new();
                                            if let Ok(new_content) = std::fs::read_to_string(&path_buf) {
                                                new_content_str = new_content.clone();
                                                orig_content = new_content.replacen(replacement, target, 1);
                                            }

                                            let start_line = if let Some(idx) = new_content_str.find(replacement) {
                                                new_content_str[..idx].lines().count()
                                            } else {
                                                0
                                            };
                                            let removed_lines: Vec<String> = target.lines().map(|s| s.to_string()).collect();
                                            let added_lines_count = replacement.lines().count().max(1);

                                            let review = FileDiffReview {
                                                path: path_buf.clone(),
                                                original_content: orig_content,
                                                is_new_file: false,
                                                hunks: vec![DiffHunk {
                                                    start_line,
                                                    removed_lines,
                                                    added_lines_count,
                                                }],
                                                active_hunk_idx: 0,
                                            };

                                            if let Some(existing_idx) = self.pending_reviews.iter().position(|r| r.path == path_buf) {
                                                self.pending_reviews[existing_idx] = review;
                                                self.active_review_idx = Some(existing_idx);
                                            } else {
                                                self.pending_reviews.push(review);
                                                self.active_review_idx = Some(self.pending_reviews.len() - 1);
                                            }

                                            for tab in &mut self.tabs {
                                                let matches = if let Some(ref tp) = tab.path {
                                                    tp == &path_buf || tp.ends_with(&path) || path_buf.ends_with(tp)
                                                } else {
                                                    tab.name == path
                                                };
                                                if matches {
                                                    tab.lines_added += lines_added;
                                                    tab.lines_removed += lines_removed;
                                                    if let Some(ref p) = tab.path {
                                                        if let Ok(new_content) = std::fs::read_to_string(p) {
                                                            tab.editor = ::editor::editor::Editor::new(::editor::buffer::Buffer::from_text(&new_content));
                                                            tab.shaping_cache.clear();
                                                            tab.version += 1;
                                                        }
                                                    }
                                                }
                                            }
                                            files_to_open.push(path_buf);
                                        }
                                        "write_file" => {
                                            if let Some(TrajectoryStep::Exploration(ref mut grp)) =
                                                last.trajectory.last_mut()
                                            {
                                                grp.is_active = false;
                                            }
                                            let path = args
                                                .get("path")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("file")
                                                .to_string();
                                            let content = args
                                                .get("content")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("");
                                            let lines_added = content.lines().count().max(1);
                                            last.trajectory.push(TrajectoryStep::FileEdit {
                                                path: path.clone(),
                                                lines_added,
                                                lines_removed: 0,
                                            });

                                            let path_buf = std::path::PathBuf::from(&path);
                                            let added_lines_count = content.lines().count().max(1);
                                            let review = FileDiffReview {
                                                path: path_buf.clone(),
                                                original_content: String::new(),
                                                is_new_file: true,
                                                hunks: vec![DiffHunk {
                                                    start_line: 0,
                                                    removed_lines: Vec::new(),
                                                    added_lines_count,
                                                }],
                                                active_hunk_idx: 0,
                                            };

                                            if let Some(existing_idx) = self.pending_reviews.iter().position(|r| r.path == path_buf) {
                                                self.pending_reviews[existing_idx] = review;
                                                self.active_review_idx = Some(existing_idx);
                                            } else {
                                                self.pending_reviews.push(review);
                                                self.active_review_idx = Some(self.pending_reviews.len() - 1);
                                            }

                                            for tab in &mut self.tabs {
                                                let matches = if let Some(ref tp) = tab.path {
                                                    tp == &path_buf || tp.ends_with(&path) || path_buf.ends_with(tp)
                                                } else {
                                                    tab.name == path
                                                };
                                                if matches {
                                                    tab.lines_added += lines_added;
                                                    if let Some(ref p) = tab.path {
                                                        if let Ok(new_content) = std::fs::read_to_string(p) {
                                                            tab.editor = ::editor::editor::Editor::new(::editor::buffer::Buffer::from_text(&new_content));
                                                            tab.shaping_cache.clear();
                                                            tab.version += 1;
                                                        }
                                                    }
                                                }
                                            }
                                            files_to_open.push(path_buf);
                                        }
                                        "run_terminal" => {
                                            if let Some(TrajectoryStep::Exploration(ref mut grp)) =
                                                last.trajectory.last_mut()
                                            {
                                                grp.is_active = false;
                                            }
                                            let command = args
                                                .get("command")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string();
                                            last.trajectory.push(TrajectoryStep::Command { command });
                                        }
                                        "create_plan" => {
                                            let title = args
                                                .get("title")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("Execution Plan")
                                                .to_string();
                                            let steps = args
                                                .get("steps")
                                                .and_then(|v| v.as_array())
                                                .map(|arr| {
                                                    arr.iter()
                                                        .filter_map(|s| s.as_str().map(|str_v| str_v.to_string()))
                                                        .collect()
                                                })
                                                .unwrap_or_default();
                                            last.trajectory.push(TrajectoryStep::Plan {
                                                title,
                                                steps,
                                                expanded: true,
                                            });
                                        }
                                        _ => {
                                            let item = match name.as_str() {
                                                "read_file" => {
                                                    let path = args
                                                        .get("path")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("")
                                                        .to_string();
                                                    let start_line = args
                                                        .get("start_line")
                                                        .and_then(|v| v.as_u64())
                                                        .map(|u| u as usize);
                                                    let end_line = args
                                                        .get("end_line")
                                                        .and_then(|v| v.as_u64())
                                                        .map(|u| u as usize);
                                                    ExplorationItem::AnalyzedFile {
                                                        path,
                                                        start_line,
                                                        end_line,
                                                    }
                                                }
                                                "search_codebase" | "find_files" => {
                                                    let query = args
                                                        .get("query")
                                                        .or_else(|| args.get("pattern"))
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("")
                                                        .to_string();
                                                    ExplorationItem::Searched { query }
                                                }
                                                "list_directory" | "get_project_structure" => {
                                                    let path = args
                                                        .get("path")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or(".")
                                                        .to_string();
                                                    ExplorationItem::ExploredDirectory { path }
                                                }
                                                "get_diagnostics" => {
                                                    ExplorationItem::AnalyzedFile {
                                                        path: "diagnostics".to_string(),
                                                        start_line: None,
                                                        end_line: None,
                                                    }
                                                }
                                                _ => ExplorationItem::Thought {
                                                    summary: format!("Calling {}...", name),
                                                    duration_secs: None,
                                                },
                                            };

                                            if let Some(TrajectoryStep::Exploration(ref mut grp)) =
                                                last.trajectory.last_mut()
                                            {
                                                grp.items.push(item);
                                                grp.is_active = true;
                                            } else {
                                                last.trajectory.push(TrajectoryStep::Exploration(
                                                    ExplorationGroup {
                                                        expanded: false,
                                                        items: vec![item],
                                                        is_active: true,
                                                    },
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                            AiUiUpdate::ToolDone { name: _ } => {
                                // Tool done
                            }
                            AiUiUpdate::Done {
                                reply,
                                session_json,
                            } => {
                                if let Some(last) =
                                    self.ai_messages.iter_mut().rev().find(|m| !m.is_user)
                                {
                                    // Deactivate all exploration groups
                                    for step in &mut last.trajectory {
                                        if let TrajectoryStep::Exploration(ref mut grp) = step {
                                            grp.is_active = false;
                                        }
                                    }
                                    last.is_working = false;

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
                                        trajectory: Vec::new(),
                                        is_working: false,
                                    });
                                }
                                // Persist updated conversation history for the next message
                                if session_json.is_some() {
                                    self.ai_session_json = session_json;
                                }
                                self.ai_response_rx = None;
                                for p in &files_to_open {
                                    self.open_file(p);
                                }
                                return true;
                            }
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        if let Some(last) = self.ai_messages.iter_mut().rev().find(|m| !m.is_user) {
                            last.is_working = false;
                            for step in &mut last.trajectory {
                                if let TrajectoryStep::Exploration(ref mut grp) = step {
                                    grp.is_active = false;
                                }
                            }
                            if last.content.is_empty() {
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
        for p in files_to_open {
            self.open_file(&p);
        }
        changed
    }
}
