use crate::state::types::*;
use crate::state::ZenthreeState;

#[derive(Default)]
struct InlineThoughtParser {
    in_thought: bool,
    buffer: String,
}

impl InlineThoughtParser {
    fn new() -> Self {
        Self::default()
    }

    fn feed(
        &mut self,
        chunk: &str,
        mut on_thought: impl FnMut(String),
        mut on_text: impl FnMut(String),
    ) {
        self.buffer.push_str(chunk);

        loop {
            if !self.in_thought {
                let thought_start = self
                    .buffer
                    .find("<thought>")
                    .map(|idx| (idx, 9))
                    .or_else(|| self.buffer.find("<thinking>").map(|idx| (idx, 10)));

                if let Some((start_idx, tag_len)) = thought_start {
                    if start_idx > 0 {
                        let text_part = self.buffer[..start_idx].to_string();
                        on_text(text_part);
                    }
                    self.buffer = self.buffer[start_idx + tag_len..].to_string();
                    self.in_thought = true;
                } else {
                    let mut partial = false;
                    for tag in &["<thought>", "<thinking>"] {
                        for len in 1..tag.len() {
                            if self.buffer.ends_with(&tag[..len]) {
                                partial = true;
                                break;
                            }
                        }
                        if partial {
                            break;
                        }
                    }

                    if partial {
                        break;
                    } else {
                        let text_part = std::mem::take(&mut self.buffer);
                        if !text_part.is_empty() {
                            on_text(text_part);
                        }
                        break;
                    }
                }
            } else {
                let thought_end = self
                    .buffer
                    .find("</thought>")
                    .map(|idx| (idx, 10))
                    .or_else(|| self.buffer.find("</thinking>").map(|idx| (idx, 11)));

                if let Some((end_idx, tag_len)) = thought_end {
                    if end_idx > 0 {
                        let thought_part = self.buffer[..end_idx].to_string();
                        on_thought(thought_part);
                    }
                    self.buffer = self.buffer[end_idx + tag_len..].to_string();
                    self.in_thought = false;
                } else {
                    let mut partial = false;
                    for tag in &["</thought>", "</thinking>"] {
                        for len in 1..tag.len() {
                            if self.buffer.ends_with(&tag[..len]) {
                                partial = true;
                                break;
                            }
                        }
                        if partial {
                            break;
                        }
                    }

                    if partial {
                        break;
                    } else {
                        let thought_part = std::mem::take(&mut self.buffer);
                        if !thought_part.is_empty() {
                            on_thought(thought_part);
                        }
                        break;
                    }
                }
            }
        }
    }

    fn finish(self, mut on_thought: impl FnMut(String), mut on_text: impl FnMut(String)) {
        if !self.buffer.is_empty() {
            if self.in_thought {
                on_thought(self.buffer);
            } else {
                on_text(self.buffer);
            }
        }
    }
}

impl ZenthreeState {
    pub fn cancel_active_ai(&mut self) {
        if let Some(ref token) = self.ai_cancel_token {
            token.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        self.ai_cancel_token = None;
        self.active_agent_terminal_tasks.clear();
        self.show_agent_terminal_popup = false;
        self.ai_update_signal.set(Vec::new());
        for m in &mut self.ai_messages {
            if !m.is_user && m.is_working {
                m.is_working = false;
            }
        }
    }

    pub fn send_ai_message(&mut self, text: &str) {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }

        // Cancel any currently running background AI task before starting a new one
        self.cancel_active_ai();

        let cancel_token = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.ai_cancel_token = Some(cancel_token.clone());

        // Keep maximum 60 messages to bound memory
        if self.ai_messages.len() >= 60 {
            let remove_count = self.ai_messages.len() - 58;
            self.ai_messages.drain(0..remove_count);
        }

        // Mark any previous assistant working message as finished so turns don't collide
        for m in &mut self.ai_messages {
            if !m.is_user && m.is_working {
                m.is_working = false;
            }
        }

        self.ai_messages.push(AiMessage {
            is_user: true,
            content: trimmed.to_string(),
            code_snippet: None,
            thoughts: None,
            thoughts_expanded: false,
            expanded: false,
            trajectory: Vec::new(),
            is_working: false,
        });

        self.ai_messages.push(AiMessage {
            is_user: false,
            content: String::new(),
            code_snippet: None,
            thoughts: None,
            thoughts_expanded: false,
            expanded: false,
            trajectory: Vec::new(),
            is_working: true,
        });

        self.ai_input.clear();
        self.save_current_ai_session();

        // Clone the signal handle so the background thread can push updates.
        // ArcSignal is Send + Sync, so this is safe to move into the thread.
        let update_signal = self.ai_update_signal.clone();
        // Clear any leftover updates from a previous session
        update_signal.set(Vec::new());

        let cancel_for_send = cancel_token.clone();
        let send = move |update: AiUiUpdate| {
            if cancel_for_send.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }
            update_signal.with_mut(|queue| queue.push(update));
            // ArcSignal::with_mut already calls trigger_redraw_hook() — no timer needed
        };
        let prompt = trimmed.to_string();
        // Pass workspace dir and workspace state so tools run only on open projects
        let current_dir = self.current_dir.clone();
        let has_workspace = self.has_workspace;
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
        let (ws_context_prompt, git_context_prompt) = if self.has_workspace {
            let ws = agent::context::WorkspaceContext::new(&current_dir)
                .with_open_files(open_file_infos)
                .format_for_prompt();
            let git = agent::context::GitContext::format_for_prompt(&current_dir);
            (ws, git)
        } else {
            (String::new(), None)
        };

        let cancel_for_thread = cancel_token.clone();
        std::thread::spawn(move || {
            if has_workspace {
                // Set CWD so all tools (get_project_structure, search_codebase, etc.)
                // operate on the user's workspace, not the binary's launch directory.
                let _ = std::env::set_current_dir(&current_dir);
            }

            if cancel_for_thread.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }

            let provider_name = match selected_model.provider() {
                crate::state::AiProvider::Antigravity => "antigravity",
                crate::state::AiProvider::Gemini => "gemini",
                crate::state::AiProvider::Groq => "groq",
            };

            let maybe_agent = if has_workspace {
                agent::create_agent_for_model_and_workspace(
                    provider_name,
                    selected_model.model_id(),
                    &current_dir,
                )
            } else {
                agent::create_agent_without_workspace(
                    provider_name,
                    selected_model.model_id(),
                )
            };

            if let Some(agent) = maybe_agent {
                let mut agent = agent.with_cancellation_token(cancel_for_thread.clone());

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
                    if has_workspace {
                        let skills_summary = agent.skills().prompt_summary();
                        let tools_manifest = agent.tools().manifest();
                        let mut sys_msg = format!(
                            "You are a helpful, accurate, and concise AI coding assistant.\n\
                            Workspace: {}\n\
                            Platform: {} ({})\n\
                            Environment: Zenthree IDE (Assistant: ZENE)\n\n\
                            Available Tools (Level 1 Manifest):\n{}\n\n\
                            Available Workflow Skills:\n{}\n\n\
                            Guidelines:\n\
                            - CONDITIONAL IDENTITY: You operate inside Zenthree IDE as the AI assistant ZENE. In normal coding and project tasks, stay strictly focused on the user's project and code—never bring up Zenthree IDE, ZENE, or your internal assistant platform unprompted. ONLY disclose or explain this identity if the user explicitly asks about it (e.g. 'What IDE is this?', 'Who are you?', 'What assistant is this?').\n\
                            - CASUAL CHAT & GREETINGS: When the user simply says 'hi', 'hello', or greets you, reply warmly, concisely, and conversationally. Do NOT call file reading or directory listing tools unless the user specifically asks a question about the workspace or requests a task.\n\
                            - STRICT PROJECT BOUNDARY: You must operate EXCLUSIVELY inside the current workspace project root ('./'). You are strictly forbidden from navigating to parent directories ('..') or inspecting files outside this project.\n\
                            - STRICT .zenthree USAGE & NOTES BOUNDARY: The `.zenthree/` directory is reserved for IDE metadata and the user's private notes vault (`.zenthree/notes/`). For coding, debugging, refactoring, or codebase architecture tasks (e.g. 'explain me this project'), do NOT enter, read, search, or list `.zenthree/`. Focus exclusively on the project's actual source code (`src/`, `crates/`, build configs, docs). ONLY inspect or edit files inside `.zenthree/notes/` when the user explicitly asks about notes or requests note creation/editing. NEVER inspect `.zenthree_sessions/` or `.zenthree_state.json`.\n\
                            - DIRECT ACTION & IMMEDIATE EXECUTION: When the user asks you to create, scaffold, implement, fix, or run something, take immediate action using your available tools. Do NOT pause execution or create unnecessary markdown planning files for straightforward user tasks. Execute commands and edits directly.\n\
                            - PROJECT ROOT & SCAFFOLDING: The current workspace directory is already the project root. All terminal commands run natively with this directory as their working directory (no need to prefix commands with `cd /path/to/project &&`). You are strictly forbidden from creating projects in /tmp or outside the workspace. When scaffolding or initializing a new application (e.g. using `bun create vite`, `npm create`, `cargo new`), always scaffold directly into `./` with non-interactive and overwrite flags (e.g. `bun create vite ./ --template react --no-interactive --overwrite`) so files are created directly at the workspace root without failing if existing files (like plans or configs) already exist. Never create a redundant nested subfolder unless the user explicitly requested a subfolder name.\n\
                            - ARCHITECTURAL PLANNING: Reserve `create_plan` strictly for large, complex multi-file architectural refactors or when the user explicitly asks for a plan.\n\
                            - Use `search`, `read_file`, and `list_directory` to explore files, symbols, and directory trees.\n\
                            - EFFICIENT FILE READING: Call `read_file(path: \"...\")` directly to read complete files (it automatically provides up to 500 lines).\n\
                            - Prefer surgical edits using `edit_file` rather than rewriting whole files.\n\
                            - Run `get_diagnostics` or test/build suites via `run_terminal` to verify your changes.\n\
                            - Activate workflow skills on demand via `activate_skill(name: \"...\")` for step-by-step guidance.\n\n{}",
                            current_dir.display(),
                            std::env::consts::OS,
                            std::env::consts::ARCH,
                            tools_manifest,
                            skills_summary,
                            ws_context_prompt
                        );
                        if let Some(ref git_ctx) = git_context_prompt {
                            sys_msg.push_str("\n\n");
                            sys_msg.push_str(git_ctx);
                        }
                        agent.push_system_message(sys_msg);
                    } else {
                        let sys_msg = format!(
                            "You are a helpful, accurate, and concise AI coding assistant.\n\
                            Mode: General Chat (No workspace folder open)\n\
                            Platform: {} ({})\n\
                            Environment: Zenthree IDE (Assistant: ZENE)\n\n\
                            Guidelines:\n\
                            - CONDITIONAL IDENTITY: You operate inside Zenthree IDE as the AI assistant ZENE. In normal coding and project tasks, stay strictly focused on the user's project and code—never bring up Zenthree IDE, ZENE, or your internal assistant platform unprompted. ONLY disclose or explain this identity if the user explicitly asks about it (e.g. 'What IDE is this?', 'Who are you?', 'What assistant is this?').\n\
                            - NO WORKSPACE OPEN: There is currently NO workspace or project folder open.\n\
                            - You do NOT have a workspace directory and must NOT execute commands or create/edit/delete files on the local filesystem.\n\
                            - Provide helpful, clear explanations and code blocks in markdown for general programming questions, design patterns, algorithms, or snippets.\n\
                            - If the user asks you to create a project, scaffold an application, or edit files, politely inform them: 'No project folder is currently open. Please open a project folder before scaffolding or modifying project files.'",
                            std::env::consts::OS,
                            std::env::consts::ARCH,
                        );
                        agent.push_system_message(sys_msg);
                    }
                } else if has_workspace {
                    let skills_summary = agent.skills().prompt_summary();
                    let tools_manifest = agent.tools().manifest();
                    let mut sys_msg = format!(
                        "You are a helpful, accurate, and concise AI coding assistant.\n\
                        Workspace: {}\n\
                        Platform: {} ({})\n\
                        Environment: Zenthree IDE (Assistant: ZENE)\n\n\
                        Available Tools (Level 1 Manifest):\n{}\n\n\
                        Available Workflow Skills:\n{}\n\n\
                        Guidelines:\n\
                        - CONDITIONAL IDENTITY: You operate inside Zenthree IDE as the AI assistant ZENE. In normal coding and project tasks, stay strictly focused on the user's project and code—never bring up Zenthree IDE, ZENE, or your internal assistant platform unprompted. ONLY disclose or explain this identity if the user explicitly asks about it (e.g. 'What IDE is this?', 'Who are you?', 'What assistant is this?').\n\
                        - CASUAL CHAT & GREETINGS: When the user simply says 'hi', 'hello', or greets you, reply warmly, concisely, and conversationally. Do NOT call file reading or directory listing tools unless the user specifically asks a question about the workspace or requests a task.\n\
                        - STRICT PROJECT BOUNDARY: You must operate EXCLUSIVELY inside the current workspace project root ('./'). You are strictly forbidden from navigating to parent directories ('..') or inspecting files outside this project.\n\
                        - STRICT .zenthree USAGE & NOTES BOUNDARY: The `.zenthree/` directory is reserved for IDE metadata and the user's private notes vault (`.zenthree/notes/`). For coding, debugging, refactoring, or codebase architecture tasks (e.g. 'explain me this project'), do NOT enter, read, search, or list `.zenthree/`. Focus exclusively on the project's actual source code (`src/`, `crates/`, build configs, docs). ONLY inspect or edit files inside `.zenthree/notes/` when the user explicitly asks about notes or requests note creation/editing. NEVER inspect `.zenthree_sessions/` or `.zenthree_state.json`.\n\
                        - DIRECT ACTION & IMMEDIATE EXECUTION: When the user asks you to create, scaffold, implement, fix, or run something, take immediate action using your available tools. Do NOT pause execution or create unnecessary markdown planning files for straightforward user tasks. Execute commands and edits directly.\n\
                        - PROJECT ROOT & SCAFFOLDING: The current workspace directory is already the project root. All terminal commands run natively with this directory as their working directory (no need to prefix commands with `cd /path/to/project &&`). You are strictly forbidden from creating projects in /tmp or outside the workspace. When scaffolding or initializing a new application (e.g. using `bun create vite`, `npm create`, `cargo new`), always scaffold directly into `./` with non-interactive and overwrite flags (e.g. `bun create vite ./ --template react --no-interactive --overwrite`) so files are created directly at the workspace root without failing if existing files (like plans or configs) already exist. Never create a redundant nested subfolder unless the user explicitly requested a subfolder name.\n\
                        - ARCHITECTURAL PLANNING: Reserve `create_plan` strictly for large, complex multi-file architectural refactors or when the user explicitly asks for a plan.\n\
                        - Use `search`, `read_file`, and `list_directory` to explore files, symbols, and directory trees.\n\
                        - EFFICIENT FILE READING: Call `read_file(path: \"...\")` directly to read complete files (it automatically provides up to 500 lines).\n\
                        - Prefer surgical edits using `edit_file` rather than rewriting whole files.\n\
                        - Run `get_diagnostics` or test/build suites via `run_terminal` to verify your changes.\n\
                        - Activate workflow skills on demand via `activate_skill(name: \"...\")` for step-by-step guidance.\n\n{}",
                        current_dir.display(),
                        std::env::consts::OS,
                        std::env::consts::ARCH,
                        tools_manifest,
                        skills_summary,
                        ws_context_prompt
                    );
                    if let Some(ref git_ctx) = git_context_prompt {
                        sys_msg.push_str("\n\n");
                        sys_msg.push_str(git_ctx);
                    }
                    if let Some(first) = agent.conversation_mut().first_mut() {
                        if first.role == agent::types::Role::System {
                            first.content = sys_msg;
                        }
                    }
                }

                let mut final_text = String::new();
                let cancel_for_event = cancel_for_thread.clone();
                let mut thought_parser = InlineThoughtParser::new();
                let send_fn = send.clone();

                let mut on_event = |ev: agent::types::AgentEvent| {
                    if cancel_for_event.load(std::sync::atomic::Ordering::Relaxed) {
                        return;
                    }
                    match ev {
                        agent::types::AgentEvent::ToolStarted { name, args } => {
                            send_fn(AiUiUpdate::ToolStart { name, args });
                        }
                        agent::types::AgentEvent::ToolFinished { name, result } => {
                            send_fn(AiUiUpdate::ToolDone {
                                name,
                                result: Some(result),
                            });
                        }
                        agent::types::AgentEvent::Thought(thought) => {
                            send_fn(AiUiUpdate::Thought(thought));
                        }
                        agent::types::AgentEvent::Text(delta) => {
                            thought_parser.feed(
                                &delta,
                                |th| send_fn(AiUiUpdate::Thought(th)),
                                |tx| {
                                    final_text.push_str(&tx);
                                    send_fn(AiUiUpdate::TextDelta(tx));
                                },
                            );
                        }
                        _ => {}
                    }
                };
                let mut ask_approval =
                    |_: agent::types::ApprovalRequest| agent::types::ApprovalDecision::Approve;

                match agent.run(&prompt, &mut on_event, &mut ask_approval) {
                    Ok(text) => {
                        thought_parser.finish(
                            |th| send(AiUiUpdate::Thought(th)),
                            |tx| {
                                final_text.push_str(&tx);
                                send(AiUiUpdate::TextDelta(tx));
                            },
                        );

                        if cancel_for_thread.load(std::sync::atomic::Ordering::Relaxed) {
                            return;
                        }
                        let reply = if !text.trim().is_empty() {
                            text
                        } else if !final_text.trim().is_empty() {
                            final_text
                        } else {
                            "I received your request, but no response was generated. Please check your selected model or API quota.".to_string()
                        };
                        // Export the updated conversation so the next message has full memory
                        let new_session = agent.export_session_json().ok();
                        send(AiUiUpdate::Done {
                            reply,
                            session_json: new_session,
                        });
                    }
                    Err(e) => {
                        thought_parser.finish(
                            |th| send(AiUiUpdate::Thought(th)),
                            |tx| {
                                final_text.push_str(&tx);
                                send(AiUiUpdate::TextDelta(tx));
                            },
                        );

                        if cancel_for_thread.load(std::sync::atomic::Ordering::Relaxed) {
                            return;
                        }
                        send(AiUiUpdate::Done {
                            reply: format!("Agent error: {e}"),
                            session_json,
                        });
                    }
                }
            } else {
                // Fallback: no provider key found
                if cancel_for_thread.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }
                match agent::provider::complete(&prompt) {
                    Ok(content) => {
                        if cancel_for_thread.load(std::sync::atomic::Ordering::Relaxed) {
                            return;
                        }
                        send(AiUiUpdate::Done {
                            reply: content,
                            session_json: None,
                        });
                    }
                    Err(err) => {
                        if cancel_for_thread.load(std::sync::atomic::Ordering::Relaxed) {
                            return;
                        }
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
        // Fast path: if empty, return immediately without acquiring write-lock or triggering redraw hook
        if self.ai_update_signal.with(|queue| queue.is_empty()) {
            return false;
        }
        // Atomically take the full update queue from the signal without triggering a spurious redraw loop
        let updates = self.ai_update_signal.with_silent_mut(|queue| std::mem::take(queue));
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
                            AiUiUpdate::TextDelta(chunk) => {
                                if let Some(last) =
                                    self.ai_messages.iter_mut().rev().find(|m| !m.is_user)
                                {
                                    let old_hash = {
                                        use std::hash::{Hash, Hasher};
                                        let mut h =
                                            std::collections::hash_map::DefaultHasher::new();
                                        last.content.hash(&mut h);
                                        h.finish()
                                    };
                                    self.ai_block_cache.remove(&old_hash);
                                    last.content.push_str(&chunk);
                                }
                            }
                            AiUiUpdate::Thought(chunk) => {
                                if let Some(last) =
                                    self.ai_messages.iter_mut().rev().find(|m| !m.is_user)
                                {
                                    let thoughts = last.thoughts.get_or_insert_with(String::new);
                                    thoughts.push_str(&chunk);
                                    last.thoughts_expanded = true;
                                }
                            }
                            AiUiUpdate::ToolStart { name, args } => {
                                if let Some(last) =
                                    self.ai_messages.iter_mut().rev().find(|m| !m.is_user)
                                {
                                    let clean_tool_name = clean_tool_str(&name);

                                    match clean_tool_name {
                                        "edit_file" | "client_edit_file" | "replace_file_content" | "multi_replace_file_content" | "client_modify_file" => {
                                            if let Some(TrajectoryStep::Exploration(ref mut grp)) =
                                                last.trajectory.last_mut()
                                            {
                                                grp.is_active = false;
                                            }
                                            let path = args
                                                .get("path")
                                                .or_else(|| args.get("file_path"))
                                                .or_else(|| args.get("filePath"))
                                                .or_else(|| args.get("TargetFile"))
                                                .or_else(|| args.get("target_file"))
                                                .or_else(|| args.get("file"))
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("file")
                                                .to_string();
                                            let full_content = args
                                                .get("content")
                                                .or_else(|| args.get("code_content"))
                                                .or_else(|| args.get("CodeContent"))
                                                .and_then(|v| v.as_str());

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

                                            let existing_rev = self.pending_reviews.iter().find(|r| path_matches(&r.path, &effective_path));
                                            let mut orig_content = String::new();
                                            let mut new_content_str = String::new();

                                            if let Some(full) = full_content {
                                                new_content_str = full.to_string();
                                                if let Some(rev) = existing_rev {
                                                    orig_content = rev.original_content.clone();
                                                } else if let Ok(disk_content) = std::fs::read_to_string(&effective_path) {
                                                    orig_content = disk_content;
                                                } else if let Some(idx) = tab_match_idx {
                                                    orig_content = self.tabs[idx].editor.buffer.to_string();
                                                }
                                            } else {
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
                                            }

                                            let (diff_lines, preview_text) = crate::state::compute_line_diff(&orig_content, &new_content_str);
                                            let lines_added = diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Added).count();
                                            let lines_removed = diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Removed).count();
                                            last.trajectory.push(TrajectoryStep::FileEdit {
                                                path: path.clone(),
                                                lines_added,
                                                lines_removed,
                                                is_new: false,
                                                expanded: false,
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
                                                let is_md = name.ends_with(".md") || name.ends_with(".markdown");
                                                self.tabs.push(EditorTab {
                                                    name,
                                                    path: Some(effective_path.clone()),
                                                    editor: ::editor::editor::Editor::new(::editor::buffer::Buffer::from_text(&preview_text)),
                                                    is_dirty: true,
                                                    scroll_y: 0.0,
                                                    target_scroll_y: 0.0,
                                                    shaping_cache: std::collections::HashMap::new(),
                                                    version: 0,
                                                    lines_added,
                                                    lines_removed,
                                                    is_markdown_preview: is_md,
                                                });
                                                self.tabs.len() - 1
                                            };
                                            self.set_active_tab(Some(tab_idx));
                                        }
                                        "write_file" | "client_write_file" | "write_to_file" | "create_file" | "client_create_file" | "write_text_file" | "fs/write_text_file" => {
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
                                                .or_else(|| args.get("code_content"))
                                                .or_else(|| args.get("CodeContent"))
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
                                            let existing_rev = self.pending_reviews.iter().find(|r| path_matches(&r.path, &effective_path));
                                            let (orig_content, is_new_file) = if let Some(rev) = existing_rev {
                                                (rev.original_content.clone(), rev.is_new_file)
                                            } else if file_existed {
                                                let prev = std::fs::read_to_string(&effective_path).unwrap_or_default();
                                                (prev, false)
                                            } else if let Some(idx) = tab_match_idx {
                                                let prev = self.tabs[idx].editor.buffer.to_string();
                                                let is_new = !file_existed && prev.is_empty();
                                                (prev, is_new)
                                            } else {
                                                (String::new(), true)
                                            };

                                            let (mut diff_lines, preview_text) = crate::state::compute_line_diff(&orig_content, content);
                                            if is_new_file && !diff_lines.iter().any(|(_, s)| *s == DiffLineState::Added) {
                                                diff_lines = content
                                                    .lines()
                                                    .map(|l| (l.to_string(), DiffLineState::Added))
                                                    .collect();
                                            }
                                            let lines_added = if is_new_file {
                                                content.lines().count().max(1)
                                            } else {
                                                diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Added).count()
                                            };
                                            let lines_removed = if is_new_file {
                                                0
                                            } else {
                                                diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Removed).count()
                                            };
                                            last.trajectory.push(TrajectoryStep::FileEdit {
                                                path: path.clone(),
                                                lines_added,
                                                lines_removed,
                                                is_new: is_new_file,
                                                expanded: false,
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
                                                let is_md = name.ends_with(".md") || name.ends_with(".markdown");
                                                self.tabs.push(EditorTab {
                                                    name,
                                                    path: Some(effective_path.clone()),
                                                    editor: ::editor::editor::Editor::new(::editor::buffer::Buffer::from_text(&preview_text)),
                                                    is_dirty: true,
                                                    scroll_y: 0.0,
                                                    target_scroll_y: 0.0,
                                                    shaping_cache: std::collections::HashMap::new(),
                                                    version: 0,
                                                    lines_added,
                                                    lines_removed,
                                                    is_markdown_preview: is_md,
                                                });
                                                self.tabs.len() - 1
                                            };
                                            self.set_active_tab(Some(tab_idx));
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
                                        "update_plan_step" => {
                                            let plan_path = self.current_dir.join("implementation_plan.md");
                                            for tab in &mut self.tabs {
                                                if let Some(ref p) = tab.path {
                                                    if path_matches(p, &plan_path) || p.file_name().and_then(|n| n.to_str()) == Some("implementation_plan.md") {
                                                        if let Ok(fresh) = std::fs::read_to_string(p) {
                                                            tab.editor = ::editor::editor::Editor::new(::editor::buffer::Buffer::from_text(&fresh));
                                                            tab.version += 1;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        _ => {
                                            let stripped_name = clean_tool_str(&name);
                                            let clean_name = stripped_name
                                                .split(':')
                                                .last()
                                                .unwrap_or(stripped_name)
                                                .split('.')
                                                .last()
                                                .unwrap_or(stripped_name);

                                            let item = match clean_name {
                                                "read_file" | "view_file" | "client_view_file" | "client_read_file" | "read_text_file" | "fs/read_text_file" => {
                                                    let symbol = args.get("symbol").and_then(|v| v.as_str()).map(String::from);
                                                    let is_peek = args.get("peek").and_then(|v| v.as_bool()).unwrap_or(false);
                                                    let mut start_line = args.get("start_line").and_then(|v| v.as_u64()).map(|u| u as usize);
                                                    let mut end_line = args.get("end_line").and_then(|v| v.as_u64()).map(|u| u as usize);

                                                    if let Some(paths) = args.get("paths").and_then(|v| v.as_array()) {
                                                        let path_strings = paths.iter().filter_map(|p| p.as_str().map(String::from)).collect();
                                                        ExplorationItem::BatchRead {
                                                            paths: path_strings,
                                                            output: None,
                                                            expanded: false,
                                                        }
                                                    } else {
                                                        let path = args.get("path")
                                                            .or_else(|| args.get("file_path"))
                                                            .or_else(|| args.get("filePath"))
                                                            .or_else(|| args.get("TargetFile"))
                                                            .or_else(|| args.get("target_file"))
                                                            .or_else(|| args.get("file"))
                                                            .or_else(|| args.get("absolute_path"))
                                                            .and_then(|v| v.as_str())
                                                            .unwrap_or("")
                                                            .to_string();
                                                        let path_obj = std::path::Path::new(&path);
                                                        let resolved = if path_obj.is_absolute() {
                                                            path_obj.to_path_buf()
                                                        } else {
                                                            self.current_dir.join(path_obj)
                                                        };
                                                        if resolved.is_dir() {
                                                            ExplorationItem::ExploredDirectory {
                                                                path,
                                                                output: None,
                                                                expanded: false,
                                                            }
                                                        } else {
                                                            if let Ok(content) = std::fs::read_to_string(&resolved) {
                                                                let total_lines = content.lines().count();
                                                                if let (Some(s), Some(e)) = (start_line, end_line) {
                                                                    if s <= 1 && e >= total_lines {
                                                                        start_line = None;
                                                                        end_line = None;
                                                                    } else if e > total_lines {
                                                                        end_line = Some(total_lines);
                                                                    }
                                                                }
                                                            }
                                                            ExplorationItem::AnalyzedFile {
                                                                path,
                                                                start_line,
                                                                end_line,
                                                                symbol,
                                                                is_peek,
                                                                output: None,
                                                                expanded: false,
                                                            }
                                                        }
                                                    }
                                                }
                                                "search" | "search_codebase" | "find_files" | "find_file" | "search_directory" | "grep_search" | "grep" | "file_search" | "search_files" | "find_in_files" => {
                                                    let query = args
                                                        .get("query")
                                                        .or_else(|| args.get("pattern"))
                                                        .or_else(|| args.get("search_term"))
                                                        .or_else(|| args.get("term"))
                                                        .or_else(|| args.get("text"))
                                                        .or_else(|| args.get("glob"))
                                                        .or_else(|| args.get("filter"))
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("")
                                                        .to_string();
                                                    let scope = args
                                                        .get("in")
                                                        .or_else(|| args.get("path"))
                                                        .or_else(|| args.get("search_path"))
                                                        .or_else(|| args.get("directory"))
                                                        .or_else(|| args.get("directory_path"))
                                                        .and_then(|v| v.as_str())
                                                        .map(String::from);
                                                    let final_query = if !query.is_empty() {
                                                        query
                                                    } else if let Some(ref sc) = scope {
                                                        sc.clone()
                                                    } else {
                                                        "files".to_string()
                                                    };
                                                    ExplorationItem::Searched {
                                                        query: final_query,
                                                        scope,
                                                        output: None,
                                                        expanded: false,
                                                    }
                                                }
                                                "list_directory" | "get_project_structure" | "list_dir" | "client_list_dir" | "client_list_directory" | "client_show_directory" | "show_directory" => {
                                                    let path = args
                                                        .get("path")
                                                        .or_else(|| args.get("directory_path"))
                                                        .or_else(|| args.get("dir"))
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or(".")
                                                        .to_string();
                                                    ExplorationItem::ExploredDirectory {
                                                        path,
                                                        output: None,
                                                        expanded: false,
                                                    }
                                                }
                                                "activate_skill" => {
                                                    let name = args
                                                        .get("name")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("")
                                                        .to_string();
                                                    ExplorationItem::ActivatedSkill {
                                                        name,
                                                        output: None,
                                                        expanded: false,
                                                    }
                                                }
                                                "get_diagnostics" => {
                                                    ExplorationItem::CheckedDiagnostics {
                                                        output: None,
                                                        expanded: false,
                                                    }
                                                }
                                                "run_terminal" | "run_command" | "execute_command" | "bash" | "terminal" | "sh" | "exec" | "client_run_command" | "client_run_terminal" => {
                                                    let command = args
                                                        .get("command")
                                                        .or_else(|| args.get("CommandLine"))
                                                        .or_else(|| args.get("cmd"))
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or(stripped_name)
                                                        .to_string();
                                                    let is_daemon = args.get("IsDaemon").or_else(|| args.get("is_daemon")).and_then(|v| v.as_bool()).unwrap_or(false);
                                                    let is_background = is_daemon
                                                        || command.contains("run dev")
                                                        || command.contains("dev")
                                                        || command.contains("start")
                                                        || command.contains("serve")
                                                        || command.contains("watch")
                                                        || command.contains("daemon")
                                                        || command.trim_end().ends_with('&');

                                                    self.active_agent_terminal_tasks.retain(|t| t.command != command);
                                                    self.active_agent_terminal_tasks.push(ActiveAgentTerminalTask {
                                                        id: format!("term_task_{}", self.active_agent_terminal_tasks.len() + 1),
                                                        command: command.clone(),
                                                        started_at: std::time::Instant::now(),
                                                        latest_status: if is_background { "Running in background...".to_string() } else { "Running...".to_string() },
                                                        is_background,
                                                    });
                                                    ExplorationItem::ExecutedCommand {
                                                        command,
                                                        output: None,
                                                        expanded: false,
                                                    }
                                                }
                                                cmd if cmd.starts_with("ls ") || cmd == "ls" || cmd.starts_with("env ") || cmd.starts_with("git ") || cmd.starts_with("cargo ") || cmd.starts_with("npm ") || cmd.starts_with("npx ") || cmd.starts_with("bun ") || cmd.starts_with("yarn ") || cmd.starts_with("pnpm ") || cmd.starts_with("node ") || cmd.starts_with("python") || cmd.starts_with("pytest") || cmd.starts_with("make") || cmd.starts_with("docker") || cmd.starts_with("deno") || cmd.starts_with("echo ") || cmd.starts_with("cat ") || cmd.contains('|') || args.get("command").is_some() || args.get("CommandLine").is_some() || args.get("cmd").is_some() => {
                                                    let command = args
                                                        .get("command")
                                                        .or_else(|| args.get("CommandLine"))
                                                        .or_else(|| args.get("cmd"))
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or(cmd)
                                                        .to_string();
                                                    let is_daemon = args.get("IsDaemon").or_else(|| args.get("is_daemon")).and_then(|v| v.as_bool()).unwrap_or(false);
                                                    let is_background = is_daemon
                                                        || command.contains("run dev")
                                                        || command.contains("dev")
                                                        || command.contains("start")
                                                        || command.contains("serve")
                                                        || command.contains("watch")
                                                        || command.contains("daemon")
                                                        || command.trim_end().ends_with('&');

                                                    self.active_agent_terminal_tasks.retain(|t| t.command != command);
                                                    self.active_agent_terminal_tasks.push(ActiveAgentTerminalTask {
                                                        id: format!("term_task_{}", self.active_agent_terminal_tasks.len() + 1),
                                                        command: command.clone(),
                                                        started_at: std::time::Instant::now(),
                                                        latest_status: if is_background { "Running in background...".to_string() } else { "Running...".to_string() },
                                                        is_background,
                                                    });
                                                    ExplorationItem::ExecutedCommand {
                                                        command,
                                                        output: None,
                                                        expanded: false,
                                                    }
                                                }
                                                "write_to_file" | "write_file" | "create_file" | "client_create_file" | "client_write_file" | "write_text_file" | "fs/write_text_file"
                                                | "replace_file_content" | "multi_replace_file_content" | "edit_file" | "client_edit_file" | "client_modify_file" | "modify_file" | "apply_diff" | "patch" => {
                                                    let path = args.get("path")
                                                        .or_else(|| args.get("TargetFile"))
                                                        .or_else(|| args.get("file_path"))
                                                        .or_else(|| args.get("filePath"))
                                                        .or_else(|| args.get("file"))
                                                        .or_else(|| args.get("target_file"))
                                                        .or_else(|| args.get("absolute_path"))
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("")
                                                        .to_string();
                                                    let content = args.get("content")
                                                        .or_else(|| args.get("code_content"))
                                                        .or_else(|| args.get("CodeContent"))
                                                        .or_else(|| args.get("contents"))
                                                        .or_else(|| args.get("text"))
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("");

                                                    let full_path = if !path.is_empty() {
                                                        let p = std::path::Path::new(&path);
                                                        if p.is_absolute() {
                                                            p.to_path_buf()
                                                        } else {
                                                            self.current_dir.join(p)
                                                        }
                                                    } else {
                                                        std::path::PathBuf::new()
                                                    };
                                                    let file_exists = !path.is_empty() && full_path.exists();
                                                    let is_new = !file_exists && (clean_name.contains("write") || clean_name.contains("create"));

                                                    let (lines_added, lines_removed) = if is_new {
                                                        let count = if !content.is_empty() { content.lines().count().max(1) } else { 0 };
                                                        (count, 0)
                                                    } else if file_exists {
                                                        let old_content = std::fs::read_to_string(&full_path).unwrap_or_default();
                                                        compute_line_diff(&old_content, content)
                                                    } else {
                                                        let count = if !content.is_empty() { content.lines().count().max(1) } else { 0 };
                                                        (count, 0)
                                                    };

                                                    ExplorationItem::WrittenFile {
                                                        path,
                                                        lines_added,
                                                        lines_removed,
                                                        is_new,
                                                        output: if !content.is_empty() { Some(content.to_string()) } else { None },
                                                        expanded: false,
                                                    }
                                                }
                                                _ => {
                                                    let clean_verb = clean_name.replace('_', " ");
                                                    let display = if clean_verb.is_empty() {
                                                        "Running tool".to_string()
                                                    } else {
                                                        clean_verb
                                                    };
                                                    ExplorationItem::ExecutedCommand {
                                                        command: display,
                                                        output: None,
                                                        expanded: false,
                                                    }
                                                }
                                            };

                                            let is_duplicate = if let Some(TrajectoryStep::Exploration(ref grp)) = last.trajectory.last() {
                                                grp.items.iter().any(|existing| match (existing, &item) {
                                                    (ExplorationItem::AnalyzedFile { path: p1, .. }, ExplorationItem::AnalyzedFile { path: p2, .. }) => p1 == p2,
                                                    (ExplorationItem::ExploredDirectory { path: p1, .. }, ExplorationItem::ExploredDirectory { path: p2, .. }) => p1 == p2,
                                                    (ExplorationItem::Searched { query: q1, .. }, ExplorationItem::Searched { query: q2, .. }) => q1 == q2,
                                                    (ExplorationItem::ExecutedCommand { command: c1, .. }, ExplorationItem::ExecutedCommand { command: c2, .. }) => c1 == c2,
                                                    (ExplorationItem::WrittenFile { path: p1, .. }, ExplorationItem::WrittenFile { path: p2, .. }) => p1 == p2,
                                                    _ => false,
                                                })
                                            } else {
                                                false
                                            };

                                            if !is_duplicate {
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
                            }
                            AiUiUpdate::ToolDone { name, result } => {
                                // Tool done: update file tree in case file was created/modified
                                self.rebuild_tree();
                                let stripped_done = clean_tool_str(&name);
                                let clean_done = stripped_done
                                    .split(':')
                                    .last()
                                    .unwrap_or(stripped_done)
                                    .split('.')
                                    .last()
                                    .unwrap_or(stripped_done);
                                let is_term_done = matches!(
                                    clean_done,
                                    "run_terminal" | "run_command" | "execute_command" | "bash" | "terminal" | "sh" | "exec" | "client_run_command" | "client_run_terminal"
                                ) || clean_done.starts_with("ls ")
                                  || clean_done == "ls"
                                  || clean_done.starts_with("env ")
                                  || clean_done.starts_with("git ")
                                  || clean_done.starts_with("cargo ")
                                  || clean_done.starts_with("npm ")
                                  || clean_done.starts_with("npx ")
                                  || clean_done.starts_with("bun ")
                                  || clean_done.starts_with("yarn ")
                                  || clean_done.starts_with("pnpm ")
                                  || clean_done.starts_with("node ")
                                  || clean_done.starts_with("python")
                                  || clean_done.starts_with("pytest")
                                  || clean_done.starts_with("make")
                                  || clean_done.starts_with("docker")
                                  || clean_done.starts_with("deno")
                                  || clean_done.starts_with("echo ")
                                  || clean_done.starts_with("cat ")
                                  || clean_done.contains('|');

                                let is_background_res = if let Some(ref res) = result {
                                    let s = match res {
                                        serde_json::Value::String(s) => s.as_str(),
                                        _other => "",
                                    };
                                    s.contains("background") || s.contains("ready on http") || s.contains("Local: http") || s.contains("task-") || s.contains("daemon") || s.contains("Server running")
                                } else {
                                    false
                                };

                                if is_term_done && !self.active_agent_terminal_tasks.is_empty() {
                                    if let Some(pos) = self.active_agent_terminal_tasks.iter().position(|t| {
                                        t.command.contains(clean_done) || clean_done.contains(&t.command)
                                    }) {
                                        let is_bg = self.active_agent_terminal_tasks[pos].is_background || is_background_res;
                                        if is_bg {
                                            self.active_agent_terminal_tasks[pos].is_background = true;
                                            self.active_agent_terminal_tasks[pos].latest_status = "Running in background".to_string();
                                        } else {
                                            self.active_agent_terminal_tasks.remove(pos);
                                        }
                                    } else {
                                        let is_bg = self.active_agent_terminal_tasks[0].is_background || is_background_res;
                                        if is_bg {
                                            self.active_agent_terminal_tasks[0].is_background = true;
                                            self.active_agent_terminal_tasks[0].latest_status = "Running in background".to_string();
                                        } else {
                                            self.active_agent_terminal_tasks.remove(0);
                                        }
                                    }
                                    if self.active_agent_terminal_tasks.is_empty() {
                                        self.show_agent_terminal_popup = false;
                                    }
                                }

                                if let Some(res) = result {
                                    if let Some(last) = self.ai_messages.iter_mut().rev().find(|m| !m.is_user) {
                                        if let Some(TrajectoryStep::Exploration(ref mut grp)) = last.trajectory.last_mut() {
                                            let formatted_output = match res {
                                                serde_json::Value::String(s) => s,
                                                serde_json::Value::Null => String::new(),
                                                other => serde_json::to_string_pretty(&other).unwrap_or_default(),
                                            };
                                            if !formatted_output.trim().is_empty() {
                                                if let Some(last_item) = grp.items.last_mut() {
                                                    last_item.set_output(formatted_output);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            AiUiUpdate::Done {
                                reply,
                                session_json,
                            } => {
                                self.rebuild_tree();
                                // Retain persistent background processes started by the agent
                                self.active_agent_terminal_tasks.retain(|t| t.is_background);
                                if self.active_agent_terminal_tasks.is_empty() {
                                    self.show_agent_terminal_popup = false;
                                }
                                // Auto-open modified files list once the task/editing is completed
                                if !self.pending_reviews.is_empty() {
                                    self.show_modified_files_popup = true;
                                }
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
                                    if reply == "Request stopped." {
                                        if last.content.trim().is_empty() {
                                            last.content = "*Request stopped by user.*".to_string();
                                        } else if !last.content.contains("stopped") {
                                            last.content.push_str("\n\n*Request stopped by user.*");
                                        }
                                    } else if !last.content.trim().is_empty() {
                                        if last.content.trim().len() < reply.trim().len() {
                                            last.content = reply;
                                        }
                                    } else {
                                        last.content = reply;
                                    }
                                } else {
                                    self.ai_messages.push(AiMessage {
                                        is_user: false,
                                        content: reply,
                                        code_snippet: None,
                                        thoughts: None,
                                        thoughts_expanded: false,
                                        expanded: false,
                                        trajectory: Vec::new(),
                                        is_working: false,
                                    });
                                }
                                // Persist updated conversation history for the next message
                                if session_json.is_some() {
                                    self.ai_session_json = session_json;
                                }
                                self.save_current_ai_session();
                                // Done: is_working was already set to false above
                            }
                        }
        }
        true
    }
}

fn project_dir_key(path: &std::path::Path) -> String {
    use std::hash::{Hash, Hasher};
    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    canon.to_string_lossy().hash(&mut hasher);
    let hash = hasher.finish();
    let name = canon.file_name().and_then(|n| n.to_str()).unwrap_or("project");
    let safe_name: String = name.chars().filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
    format!("{safe_name}_{hash:x}")
}

impl ZenthreeState {
    pub fn ai_sessions_base_dir(&self) -> std::path::PathBuf {
        if let Some(ref dir) = self.custom_sessions_dir {
            return dir.clone();
        }
        if let Ok(dir) = std::env::var("ZENTHREE_SESSIONS_DIR") {
            return std::path::PathBuf::from(dir);
        }
        if cfg!(test) || std::env::var("ZENTHREE_TEST_MODE").is_ok() {
            return std::env::temp_dir().join(format!("zenthree_test_sessions_default_{}", std::process::id()));
        }
        if let Ok(home) = std::env::var("HOME") {
            std::path::PathBuf::from(home).join(".config").join("zenthree").join("ai_sessions")
        } else {
            std::path::PathBuf::from(".zenthree_sessions")
        }
    }

    pub fn refresh_ai_sessions(&mut self) {
        let base = self.ai_sessions_base_dir();
        let is_test = cfg!(test) || std::env::var("ZENTHREE_TEST_MODE").is_ok() || self.custom_sessions_dir.is_some();
        let mut summaries = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&base) {
            for entry in entries.flatten() {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                // Never load test project directories into the user's UI
                if !is_test && dir_name.starts_with("zenthree_test_proj_") {
                    continue;
                }
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        if let Ok(sess_entries) = std::fs::read_dir(entry.path()) {
                            for s_entry in sess_entries.flatten() {
                                let p = s_entry.path();
                                if p.extension().and_then(|ext| ext.to_str()) == Some("json") {
                                    if let Ok(content) = std::fs::read_to_string(&p) {
                                        if let Ok(saved) = serde_json::from_str::<SavedAiSession>(&content) {
                                            // In normal UI mode, ignore any leftover test sessions
                                            if is_test || (!saved.title.starts_with("Hello AI session test")
                                                && !saved.title.starts_with("General chat query")
                                                && !saved.title.starts_with("Project chat query")
                                                && !saved.title.starts_with("What is 2 + 2?")
                                                && !saved.title.starts_with("Explain the workspace"))
                                            {
                                                summaries.push(SessionSummary {
                                                    id: saved.id,
                                                    title: saved.title,
                                                    project_path: saved.project_path,
                                                    project_name: saved.project_name,
                                                    created_at: saved.created_at,
                                                    updated_at: saved.updated_at,
                                                    message_count: saved.messages.len(),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        self.ai_sessions = summaries;

        let cur_str = if self.has_workspace {
            self.current_dir.canonicalize().unwrap_or_else(|_| self.current_dir.clone()).to_string_lossy().to_string()
        } else {
            "general".to_string()
        };
        self.ai_expanded_projects.insert(cur_str.clone());
    }

    pub fn save_current_ai_session(&mut self) {
        // Only create or save a session if the user has actually sent a message
        let has_user_message = self.ai_messages.iter().any(|m| m.is_user && !m.content.trim().is_empty());
        if !has_user_message {
            return;
        }

        let session_id = if let Some(ref id) = self.ai_session_id {
            id.clone()
        } else {
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis();
            let new_id = format!("sess_{}", now);
            self.ai_session_id = Some(new_id.clone());
            new_id
        };

        // Determine title: first user message
        let title = self.ai_messages.iter()
            .find(|m| m.is_user)
            .map(|m| {
                let first_line = m.content.lines().next().unwrap_or("").trim();
                if first_line.len() > 60 {
                    let mut end = 60;
                    while !first_line.is_char_boundary(end) {
                        end -= 1;
                    }
                    format!("{}...", &first_line[..end])
                } else if !first_line.is_empty() {
                    first_line.to_string()
                } else {
                    "Untitled Session".to_string()
                }
            })
            .unwrap_or_else(|| "New Session".to_string());

        let (project_path, project_name, base_dir) = if self.has_workspace {
            let canon = self.current_dir.canonicalize().unwrap_or_else(|_| self.current_dir.clone());
            let p_path = canon.to_string_lossy().to_string();
            let p_name = canon.file_name().and_then(|n| n.to_str()).unwrap_or("project").to_string();
            let b_dir = self.ai_sessions_base_dir().join(project_dir_key(&canon));
            (p_path, p_name, b_dir)
        } else {
            ("general".to_string(), "General".to_string(), self.ai_sessions_base_dir().join("general"))
        };
        let now_sec = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();

        let created_at = self.ai_sessions.iter().find(|s| s.id == session_id).map(|s| s.created_at).unwrap_or(now_sec);

        let saved = SavedAiSession {
            id: session_id.clone(),
            title: title.clone(),
            project_path: project_path.clone(),
            project_name: project_name.clone(),
            created_at,
            updated_at: now_sec,
            messages: self.ai_messages.clone(),
            agent_mode: self.agent_mode,
            model: self.selected_ai_model,
            session_json: self.ai_session_json.clone(),
        };

        let _ = std::fs::create_dir_all(&base_dir);
        let file_path = base_dir.join(format!("{}.json", session_id));
        if let Ok(json_str) = serde_json::to_string_pretty(&saved) {
            let _ = std::fs::write(file_path, json_str);
        }

        // Update summaries in memory
        if let Some(existing) = self.ai_sessions.iter_mut().find(|s| s.id == session_id) {
            existing.title = title;
            existing.updated_at = now_sec;
            existing.message_count = self.ai_messages.len();
        } else {
            self.ai_sessions.insert(0, SessionSummary {
                id: session_id,
                title,
                project_path,
                project_name,
                created_at,
                updated_at: now_sec,
                message_count: self.ai_messages.len(),
            });
        }
        self.ai_sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    }

    pub fn load_ai_session(&mut self, session_id: &str) {
        self.cancel_active_ai();
        let base = self.ai_sessions_base_dir();
        if let Ok(entries) = std::fs::read_dir(&base) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        let candidate = entry.path().join(format!("{}.json", session_id));
                        if candidate.exists() {
                            if let Ok(content) = std::fs::read_to_string(&candidate) {
                                if let Ok(saved) = serde_json::from_str::<SavedAiSession>(&content) {
                                    self.ai_session_id = Some(saved.id);
                                    self.ai_messages = saved.messages;
                                    self.ai_session_json = saved.session_json;
                                    self.agent_mode = saved.agent_mode;
                                    self.selected_ai_model = saved.model;
                                    self.ai_block_cache.clear();
                                    self.ai_sessions_visible = false;
                                    return;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn delete_ai_session(&mut self, session_id: &str) {
        if self.ai_session_id.as_deref() == Some(session_id) {
            self.cancel_active_ai();
        }
        let base = self.ai_sessions_base_dir();
        if let Ok(entries) = std::fs::read_dir(&base) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        let candidate = entry.path().join(format!("{}.json", session_id));
                        if candidate.exists() {
                            let _ = std::fs::remove_file(candidate);
                        }
                    }
                }
            }
        }
        self.ai_sessions.retain(|s| s.id != session_id);
        if self.ai_session_id.as_deref() == Some(session_id) {
            self.ai_messages.clear();
            self.ai_session_id = None;
            self.ai_session_json = None;
        }
    }

    pub fn start_new_ai_session(&mut self) {
        self.cancel_active_ai();
        self.ai_messages.clear();
        self.ai_session_id = None;
        self.ai_session_json = None;
        self.ai_sessions_visible = false;
    }
}

pub(crate) fn clean_tool_str(raw: &str) -> &str {
    let mut s = raw.trim();
    loop {
        let prev = s;
        for prefix in &["Running ", "Run ", "Calling ", "Call "] {
            if let Some(rest) = s.strip_prefix(prefix) {
                s = rest.trim();
            }
        }
        s = s.trim_end_matches(['?', '.', '…', ' ']);
        if s == prev {
            break;
        }
    }
    s
}

pub(crate) fn compute_line_diff(old_text: &str, new_text: &str) -> (usize, usize) {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();
    let n = old_lines.len();
    let m = new_lines.len();

    if n > 2500 || m > 2500 {
        return if m >= n { (m - n, 0) } else { (0, n - m) };
    }

    let mut dp = vec![0usize; m + 1];
    for &o_line in &old_lines {
        let mut prev = 0;
        for (j, &n_line) in new_lines.iter().enumerate() {
            let temp = dp[j + 1];
            if o_line == n_line {
                dp[j + 1] = prev + 1;
            } else if dp[j] > dp[j + 1] {
                dp[j + 1] = dp[j];
            }
            prev = temp;
        }
    }
    let lcs_len = dp[m];
    let lines_removed = n.saturating_sub(lcs_len);
    let lines_added = m.saturating_sub(lcs_len);
    (lines_added, lines_removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_line_diff_multi_area() {
        let old = "Line 1\nLine 2: Red\nLine 3\nLine 4: Blue\nLine 5";
        let new = "Line 1\nLine 2: Crimson\nLine 3\nLine 4: Azure\nLine 5";
        let (added, removed) = compute_line_diff(old, new);
        assert_eq!(added, 2);
        assert_eq!(removed, 2);
    }

    #[test]
    fn test_compute_line_diff_additions_only() {
        let old = "Line 1\nLine 2";
        let new = "Line 1\nLine 1.5\nLine 2\nLine 3";
        let (added, removed) = compute_line_diff(old, new);
        assert_eq!(added, 2);
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_compute_line_diff_identical() {
        let old = "Same line 1\nSame line 2";
        let new = "Same line 1\nSame line 2";
        let (added, removed) = compute_line_diff(old, new);
        assert_eq!(added, 0);
        assert_eq!(removed, 0);
    }
}



