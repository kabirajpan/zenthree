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
            trajectory: Vec::new(),
            is_working: true,
        });

        self.ai_input.clear();

        // Clone the signal handle so the background thread can push updates.
        // ArcSignal is Send + Sync, so this is safe to move into the thread.
        let update_signal = self.ai_update_signal.clone();
        // Clear any leftover updates from a previous session
        update_signal.set(Vec::new());

        let send = move |update: AiUiUpdate| {
            update_signal.with_mut(|queue| queue.push(update));
            // ArcSignal::with_mut already calls trigger_redraw_hook() — no timer needed
        };
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
                // Dynamically register all IDE languages into the agent's AST engine
                let lang_reg = syntax::LanguageRegistry::new();
                for lang in lang_reg.all() {
                    agent.register_extensions(lang.extensions, lang.language.clone());
                }

                // Restore conversation history from previous messages
                if let Some(ref json) = session_json {
                    let _ = agent.import_session_json(json);
                }

                // On the first turn inject a system message with workspace context, git context, ZENE skills, and Level 1 Tool Manifest
                if is_fresh_session {
                    let skills_summary = agent.skills().prompt_summary();
                    let tools_manifest = agent.tools().manifest();
                    let mut sys_msg = format!(
                        "You are ZENE, an advanced AI coding assistant embedded in the Zenthree IDE.\n\
                        Workspace: {}\n\n\
                        Available Tools (Level 1 Manifest):\n{}\n\n\
                        Available Workflow Skills:\n{}\n\n\
                        Guidelines:\n\
                        - Tools follow lazy loading: use available tools when needed.\n\
                        - Use `search` and `read_file` to explore before modifying files.\n\
                        - Prefer surgical edits using `edit_file` rather than rewriting whole files.\n\
                        - Run `get_diagnostics` or test suites via `run_terminal` to verify your changes.\n\
                        - Activate workflow skills on demand via `activate_skill(name: \"...\")` for step-by-step guidance.\n\n{}",
                        current_dir.display(),
                        tools_manifest,
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
                        send(AiUiUpdate::ToolStart { name, args });
                    }
                    agent::types::AgentEvent::ToolFinished { name, .. } => {
                        send(AiUiUpdate::ToolDone { name });
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
                        send(AiUiUpdate::Done {
                            reply,
                            session_json: new_session,
                        });
                    }
                    Err(e) => {
                        send(AiUiUpdate::Done {
                            reply: format!("Agent error: {e}"),
                            session_json,
                        });
                    }
                }
            } else {
                // Fallback: no provider key found
                match agent::provider::complete(&prompt) {
                    Ok(content) => {
                        send(AiUiUpdate::Done {
                            reply: content,
                            session_json: None,
                        });
                    }
                    Err(err) => {
                        send(AiUiUpdate::Done {
                            reply: format!("Error: {err}"),
                            session_json: None,
                        });
                    }
                }
            };
        });
    }

    /// Drain all queued AI updates from the signal and apply them to state.
    /// Returns `true` if any update was processed (indicating the UI should redraw).
    /// The signal's `trigger_redraw_hook()` already wakes the event loop, so this
    /// only needs to be called once per frame — no 50ms timer required.
    pub fn drain_ai_updates(&mut self) -> bool {
        // Atomically take the full update queue from the signal.
        let updates = self.ai_update_signal.with_mut(|queue| std::mem::take(queue));
        if updates.is_empty() {
            return false;
        }
        for update in updates {
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

                                            let raw_path = std::path::PathBuf::from(&path);
                                            let path_buf = if raw_path.is_absolute() {
                                                raw_path.clone()
                                            } else {
                                                self.current_dir.join(&raw_path)
                                            };
                                            let path_buf = if let Ok(canon) = path_buf.canonicalize() {
                                                canon
                                            } else {
                                                path_buf
                                            };

                                            let tab_match_idx = self.tabs.iter().position(|t| {
                                                if let Some(ref tp) = t.path {
                                                    path_matches(tp, &path_buf) || path_matches(tp, &raw_path)
                                                } else {
                                                    t.name == path
                                                }
                                            });

                                            let effective_path = if let Some(idx) = tab_match_idx {
                                                self.tabs[idx].path.clone().unwrap_or_else(|| path_buf.clone())
                                            } else {
                                                path_buf.clone()
                                            };

                                            let mut orig_content = String::new();
                                            let mut new_content_str = String::new();

                                            if let Some(idx) = tab_match_idx {
                                                let current_text = self.tabs[idx].editor.buffer.to_string();
                                                if current_text.contains(target) {
                                                    orig_content = current_text;
                                                    new_content_str = orig_content.replacen(target, replacement, 1);
                                                } else {
                                                    let norm_current = current_text.replace("\r\n", "\n");
                                                    let norm_target = target.replace("\r\n", "\n");
                                                    if norm_current.contains(&norm_target) {
                                                        orig_content = current_text;
                                                        let norm_replacement = replacement.replace("\r\n", "\n");
                                                        new_content_str = norm_current.replacen(&norm_target, &norm_replacement, 1);
                                                    } else if current_text.contains(replacement) && !replacement.is_empty() {
                                                        new_content_str = current_text.clone();
                                                        orig_content = current_text.replacen(replacement, target, 1);
                                                    }
                                                }
                                            }

                                            if orig_content.is_empty() {
                                                if let Ok(disk_content) = std::fs::read_to_string(&effective_path) {
                                                    if disk_content.contains(target) {
                                                        orig_content = disk_content;
                                                        new_content_str = orig_content.replacen(target, replacement, 1);
                                                    } else {
                                                        let norm_disk = disk_content.replace("\r\n", "\n");
                                                        let norm_target = target.replace("\r\n", "\n");
                                                        if norm_disk.contains(&norm_target) {
                                                            orig_content = disk_content;
                                                            let norm_replacement = replacement.replace("\r\n", "\n");
                                                            new_content_str = norm_disk.replacen(&norm_target, &norm_replacement, 1);
                                                        } else if disk_content.contains(replacement) && !replacement.is_empty() {
                                                            new_content_str = disk_content.clone();
                                                            orig_content = disk_content.replacen(replacement, target, 1);
                                                        } else {
                                                            orig_content = disk_content.clone();
                                                            new_content_str = disk_content;
                                                        }
                                                    }
                                                }
                                            }

                                            let (diff_lines, preview_text) = crate::state::compute_line_diff(&orig_content, &new_content_str);
                                            let lines_added = diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Added).count();
                                            let lines_removed = diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Removed).count();
                                            last.trajectory.push(TrajectoryStep::FileEdit {
                                                path: path.clone(),
                                                lines_added,
                                                lines_removed,
                                            });

                                            let review = FileDiffReview {
                                                path: effective_path.clone(),
                                                original_content: orig_content,
                                                proposed_content: new_content_str,
                                                diff_lines,
                                                is_new_file: false,
                                                active_hunk_idx: 0,
                                            };

                                            if let Some(existing_idx) = self.pending_reviews.iter().position(|r| {
                                                path_matches(&r.path, &effective_path) || path_matches(&r.path, &path_buf)
                                            }) {
                                                self.pending_reviews[existing_idx] = review;
                                                self.active_review_idx = Some(existing_idx);
                                            } else {
                                                self.pending_reviews.push(review);
                                                self.active_review_idx = Some(self.pending_reviews.len() - 1);
                                            }

                                            let tab_idx = if let Some(idx) = tab_match_idx {
                                                let tab = &mut self.tabs[idx];
                                                tab.path = Some(effective_path.clone());
                                                tab.lines_added = lines_added;
                                                tab.lines_removed = lines_removed;
                                                tab.is_dirty = true;
                                                tab.editor = ::editor::editor::Editor::new(::editor::buffer::Buffer::from_text(&preview_text));
                                                tab.shaping_cache.clear();
                                                tab.version += 1;
                                                idx
                                            } else {
                                                let name = effective_path.file_name().and_then(|n| n.to_str()).unwrap_or("file").to_string();
                                                self.tabs.push(EditorTab {
                                                    name,
                                                    path: Some(effective_path.clone()),
                                                    editor: ::editor::editor::Editor::new(::editor::buffer::Buffer::from_text(&preview_text)),
                                                    is_dirty: true,
                                                    scroll_y: 0.0,
                                                    shaping_cache: std::collections::HashMap::new(),
                                                    version: 0,
                                                    lines_added,
                                                    lines_removed,
                                                });
                                                self.tabs.len() - 1
                                            };
                                            self.set_active_tab(Some(tab_idx));
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

                                            let raw_path = std::path::PathBuf::from(&path);
                                            let path_buf = if raw_path.is_absolute() {
                                                raw_path.clone()
                                            } else {
                                                self.current_dir.join(&raw_path)
                                            };
                                            let path_buf = if let Ok(canon) = path_buf.canonicalize() {
                                                canon
                                            } else {
                                                path_buf
                                            };

                                            let tab_match_idx = self.tabs.iter().position(|t| {
                                                if let Some(ref tp) = t.path {
                                                    path_matches(tp, &path_buf) || path_matches(tp, &raw_path)
                                                } else {
                                                    t.name == path
                                                }
                                            });

                                            let effective_path = if let Some(idx) = tab_match_idx {
                                                self.tabs[idx].path.clone().unwrap_or_else(|| path_buf.clone())
                                            } else {
                                                path_buf.clone()
                                            };

                                            let file_existed = effective_path.exists();
                                            let (orig_content, is_new_file) = if let Some(idx) = tab_match_idx {
                                                let prev = self.tabs[idx].editor.buffer.to_string();
                                                (prev, false)
                                            } else if file_existed {
                                                let prev = std::fs::read_to_string(&effective_path).unwrap_or_default();
                                                (prev, false)
                                            } else {
                                                (String::new(), true)
                                            };

                                            let (diff_lines, preview_text) = crate::state::compute_line_diff(&orig_content, content);
                                            let lines_added = diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Added).count();
                                            let lines_removed = diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Removed).count();
                                            last.trajectory.push(TrajectoryStep::FileEdit {
                                                path: path.clone(),
                                                lines_added,
                                                lines_removed,
                                            });

                                            let review = FileDiffReview {
                                                path: effective_path.clone(),
                                                original_content: orig_content,
                                                proposed_content: content.to_string(),
                                                diff_lines,
                                                is_new_file,
                                                active_hunk_idx: 0,
                                            };

                                            if let Some(existing_idx) = self.pending_reviews.iter().position(|r| {
                                                path_matches(&r.path, &effective_path) || path_matches(&r.path, &path_buf)
                                            }) {
                                                self.pending_reviews[existing_idx] = review;
                                                self.active_review_idx = Some(existing_idx);
                                            } else {
                                                self.pending_reviews.push(review);
                                                self.active_review_idx = Some(self.pending_reviews.len() - 1);
                                            }

                                            let tab_idx = if let Some(idx) = tab_match_idx {
                                                let tab = &mut self.tabs[idx];
                                                tab.path = Some(effective_path.clone());
                                                tab.lines_added = lines_added;
                                                tab.lines_removed = lines_removed;
                                                tab.is_dirty = true;
                                                tab.editor = ::editor::editor::Editor::new(::editor::buffer::Buffer::from_text(&preview_text));
                                                tab.shaping_cache.clear();
                                                tab.version += 1;
                                                idx
                                            } else {
                                                let name = effective_path.file_name().and_then(|n| n.to_str()).unwrap_or("file").to_string();
                                                self.tabs.push(EditorTab {
                                                    name,
                                                    path: Some(effective_path.clone()),
                                                    editor: ::editor::editor::Editor::new(::editor::buffer::Buffer::from_text(&preview_text)),
                                                    is_dirty: true,
                                                    scroll_y: 0.0,
                                                    shaping_cache: std::collections::HashMap::new(),
                                                    version: 0,
                                                    lines_added,
                                                    lines_removed,
                                                });
                                                self.tabs.len() - 1
                                            };
                                            self.set_active_tab(Some(tab_idx));
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
                                                    let symbol = args.get("symbol").and_then(|v| v.as_str()).map(String::from);
                                                    let is_peek = args.get("peek").and_then(|v| v.as_bool()).unwrap_or(false);
                                                    let start_line = args.get("start_line").and_then(|v| v.as_u64()).map(|u| u as usize);
                                                    let end_line = args.get("end_line").and_then(|v| v.as_u64()).map(|u| u as usize);

                                                    if let Some(paths) = args.get("paths").and_then(|v| v.as_array()) {
                                                        let path_strings = paths.iter().filter_map(|p| p.as_str().map(String::from)).collect();
                                                        ExplorationItem::BatchRead { paths: path_strings }
                                                    } else {
                                                        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                        ExplorationItem::AnalyzedFile {
                                                            path,
                                                            start_line,
                                                            end_line,
                                                            symbol,
                                                            is_peek,
                                                        }
                                                    }
                                                }
                                                "search" => {
                                                    let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                    let scope = args.get("in").and_then(|v| v.as_str()).map(String::from);
                                                    ExplorationItem::Searched { query, scope }
                                                }
                                                "search_codebase" | "find_files" => {
                                                    let query = args
                                                        .get("query")
                                                        .or_else(|| args.get("pattern"))
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("")
                                                        .to_string();
                                                    ExplorationItem::Searched { query, scope: None }
                                                }
                                                "list_directory" | "get_project_structure" => {
                                                    let path = args
                                                        .get("path")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or(".")
                                                        .to_string();
                                                    ExplorationItem::ExploredDirectory { path }
                                                }
                                                "activate_skill" => {
                                                    let name = args
                                                        .get("name")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("")
                                                        .to_string();
                                                    ExplorationItem::ActivatedSkill { name }
                                                }
                                                "get_diagnostics" => {
                                                    ExplorationItem::CheckedDiagnostics
                                                }
                                                "run_terminal" => {
                                                    let command = args
                                                        .get("command")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("")
                                                        .to_string();
                                                    ExplorationItem::ExecutedCommand { command }
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
                                // Done: is_working was already set to false above
                            }
                        }
        }
        true
    }
}
