use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::Instant;

use crate::types::{AgentEvent, CancellationToken, ChatMessage, EditedFileSummary};
use crate::tools::{
    execute_agent_tool, parse_agent_tool_calls, extract_all_file_modifications,
    AgentToolCall, AgentToolResult, find_nearest_manifest_dir, parse_cd_statement,
    is_scaffolding_command, detect_scaffolded_project_files,
};
use crate::providers::ProviderChain;
use crate::context::build_autonomous_agent_prompt;

#[derive(Debug, Clone)]
pub struct AgentLoopContext {
    pub workspace_root: PathBuf,
    pub gemini_key: Option<String>,
    pub groq_key: Option<String>,
    pub preferred_model: Option<String>,
    pub active_file: Option<PathBuf>,
    pub active_buffer: Option<String>,
    pub cursor_line: Option<usize>,
    pub open_tabs: Vec<PathBuf>,
    pub recent_terminal_output: Option<String>,
    pub workspace_tree: Option<String>,
    pub project_context: Option<String>,
    pub max_iterations: usize,
    pub cancellation_token: CancellationToken,
    pub system_prompt: Option<String>,
    pub prior_history: Vec<ChatMessage>,
}

impl AgentLoopContext {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            gemini_key: None,
            groq_key: None,
            preferred_model: None,
            active_file: None,
            active_buffer: None,
            cursor_line: None,
            open_tabs: Vec::new(),
            recent_terminal_output: None,
            workspace_tree: None,
            project_context: None,
            max_iterations: 10,
            cancellation_token: CancellationToken::new(),
            system_prompt: None,
            prior_history: Vec::new(),
        }
    }
}

pub type AgentEventHandler = Box<dyn Fn(AgentEvent) + Send + Sync>;

pub struct AgentLoop {
    ctx: AgentLoopContext,
    event_sender: AgentEventHandler,
}

impl AgentLoop {
    pub fn new(ctx: AgentLoopContext, event_sender: AgentEventHandler) -> Self {
        Self { ctx, event_sender }
    }

    pub fn with_channel(ctx: AgentLoopContext, sender: Sender<AgentEvent>) -> Self {
        Self {
            ctx,
            event_sender: Box::new(move |evt| {
                let _ = sender.send(evt);
            }),
        }
    }

    /// Runs the autonomous ReAct loop until task completion or max iterations
    pub fn run(&mut self, goal: &str) -> Result<String, String> {
        let max_steps = self.ctx.max_iterations.max(1);
        let chain = ProviderChain::new(
            self.ctx.gemini_key.clone(),
            self.ctx.groq_key.clone(),
            self.ctx.preferred_model.clone(),
        );

        let system_prompt = self.ctx.system_prompt.clone().unwrap_or_else(|| build_autonomous_agent_prompt(95.0));

        // Build implicit context header
        let mut initial_user_prompt = String::new();

        if let Some(ref tree) = self.ctx.workspace_tree {
            if !tree.trim().is_empty() {
                initial_user_prompt.push_str(&format!(
                    "### Project Workspace Tree:\n```\n{}\n```\n\n",
                    tree.trim()
                ));
            } else {
                initial_user_prompt.push_str("### Project Workspace Tree:\n(Workspace directory is currently empty - no project files exist yet)\n\n");
            }
        } else {
            initial_user_prompt.push_str("### Project Workspace Tree:\n(Workspace directory is currently empty - no project files exist yet)\n\n");
        }
        initial_user_prompt.push_str("Workspace Grounding & Anti-Root-Dumping Protocol (When creating or modifying files upon user request):\n\
- EXACT PATHS: Always specify the full workspace-relative path in `### File: <path>` (e.g. `src/components/Button.tsx` or `crates/ai/src/tools/read.rs`). Never emit bare filenames.\n\
- NO ROOT DUMPING: Never create new source code, components, or services on the workspace root directory. Follow the project folder hierarchy shown in the tree (`src/`, `lib/`, `app/`, `crates/...`).\n\
- TARGET VERIFICATION: When modifying existing files, inspect the tree above or use `{\"tool\": \"search_code\", \"query\": \"<pattern>\"}` to verify the real relative path.\n\
- SUBDIRECTORY NAVIGATION: If a project or subcrate is inside a folder, navigate with `cd <folder>` or reference paths relative to that folder.\n\
- CONVERSATION & EXPLANATIONS: If the user is asking a conversational question or asking for an explanation, answer cleanly in Markdown without emitting `### File:` blocks.\n\n");

        if let Some(ref pctx) = self.ctx.project_context {
            if !pctx.trim().is_empty() {
                initial_user_prompt.push_str(pctx.trim());
                initial_user_prompt.push_str("\n\n");
            }
        }

        if let Some(ref active) = self.ctx.active_file {
            let rel = active.strip_prefix(&self.ctx.workspace_root).unwrap_or(active).to_string_lossy();
            let cursor_str = self.ctx.cursor_line.map(|l| format!(" (cursor at line {})", l)).unwrap_or_default();
            initial_user_prompt.push_str(&format!("Active Editor Document: `{}`{}\n", rel, cursor_str));
        }

        if !self.ctx.open_tabs.is_empty() {
            let tabs: Vec<String> = self.ctx.open_tabs.iter().map(|p| {
                p.strip_prefix(&self.ctx.workspace_root).unwrap_or(p).to_string_lossy().to_string()
            }).collect();
            initial_user_prompt.push_str(&format!("Open Editor Tabs: [{}]\n", tabs.join(", ")));
        }

        if let Some(ref buf) = self.ctx.active_buffer {
            if !buf.trim().is_empty() {
                let sample: String = buf.chars().take(3000).collect();
                initial_user_prompt.push_str(&format!("Current Active Buffer Content:\n```\n{}\n```\n\n", sample));
            }
        }

        if let Some(ref term_out) = self.ctx.recent_terminal_output {
            if !term_out.trim().is_empty() {
                let sample: String = term_out.chars().take(2000).collect();
                initial_user_prompt.push_str(&format!("Recent Terminal Output / Status:\n```\n{}\n```\n\n", sample));
            }
        }

        initial_user_prompt.push_str(&format!("User Goal / Request:\n{}", goal));

        let mut history: Vec<ChatMessage> = self.ctx.prior_history.clone();
        let mut current_user_prompt = initial_user_prompt;
        let mut last_response = String::new();
        let mut has_scaffolded_project = false;
        let mut files_written_count = 0usize;
        let mut prevented_premature_exit = 0usize;
        let mut active_cwd: Option<PathBuf> = None;

        for iteration in 0..max_steps {
            if self.ctx.cancellation_token.is_cancelled() {
                (self.event_sender)(AgentEvent::Error {
                    error: "Task cancelled by user".to_string(),
                });
                return Err("Task cancelled by user".to_string());
            }

            (self.event_sender)(AgentEvent::Thinking {
                delta: format!("Step {} of {}: Analyzing actions...", iteration + 1, max_steps),
            });

            let step_start = Instant::now();

            // Query LLM
            let response = match chain.complete(&system_prompt, &current_user_prompt, &history) {
                Ok(res) => res,
                Err(err) => {
                    let err_msg = format!("LLM provider failure at step {}: {}", iteration + 1, err);
                    (self.event_sender)(AgentEvent::Error { error: err_msg.clone() });
                    return Err(err_msg);
                }
            };

            let step_duration_secs = (step_start.elapsed().as_secs()).max(1);
            if let Some(thought_content) = extract_thought_content(&response) {
                (self.event_sender)(AgentEvent::ThoughtBlock {
                    content: thought_content,
                    duration_secs: step_duration_secs,
                });
            }

            last_response = response.clone();

            // Check for surgical file modifications or new file blocks
            let active_path = self.ctx.active_file.as_deref();
            let file_mods = extract_all_file_modifications(&response, &self.ctx.workspace_root, active_path);
            let mut file_summaries = Vec::new();
            let mut file_errors = Vec::new();

            for modif in &file_mods {
                let target = &modif.file_path;
                if let Some(parent) = target.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }

                let original_code = std::fs::read_to_string(target).unwrap_or_default();
                match modif.apply_to(&original_code) {
                    Ok(new_code) => {
                        match std::fs::write(target, &new_code) {
                            Ok(_) => {
                                // Track active working directory from written files
                                if let Some(dir) = find_nearest_manifest_dir(target, &self.ctx.workspace_root) {
                                    if dir != self.ctx.workspace_root {
                                        active_cwd = Some(dir);
                                    }
                                }
                                let rel_name = target
                                    .strip_prefix(&self.ctx.workspace_root)
                                    .unwrap_or(target)
                                    .to_string_lossy()
                                    .to_string();
                                let lines_added = new_code.lines().count().saturating_sub(original_code.lines().count());
                                let lines_removed = original_code.lines().count().saturating_sub(new_code.lines().count());
                                file_summaries.push(EditedFileSummary {
                                    file_name: rel_name,
                                    lines_added,
                                    lines_removed,
                                    original_code: Some(original_code),
                                    proposed_code: Some(new_code),
                                });
                            }
                            Err(e) => {
                                let rel_name = target
                                    .strip_prefix(&self.ctx.workspace_root)
                                    .unwrap_or(target)
                                    .to_string_lossy();
                                file_errors.push(format!("Failed to write to `{}`: {}", rel_name, e));
                            }
                        }
                    }
                    Err(e) => {
                        let rel_name = target
                            .strip_prefix(&self.ctx.workspace_root)
                            .unwrap_or(target)
                            .to_string_lossy();
                        file_errors.push(format!("Failed to apply patch to `{}`: {}", rel_name, e));
                    }
                }
            }

            if !file_summaries.is_empty() {
                files_written_count += file_summaries.len();
                (self.event_sender)(AgentEvent::PendingModifications {
                    summaries: file_summaries.clone(),
                });
            }

            // Check for tool calls
            let tool_calls = parse_agent_tool_calls(&response);

            // If neither tool calls nor file modifications were produced, the agent wants to finish:
            if tool_calls.is_empty() && file_mods.is_empty() {
                // PREMATURE COMPLETION GUARD:
                // If a project was scaffolded, but the agent hasn't written any application files yet,
                // do NOT allow it to exit with just the empty starter boilerplate!
                if has_scaffolded_project && files_written_count == 0 && prevented_premature_exit < 2 {
                    prevented_premature_exit += 1;
                    let warning = "PREMATURE COMPLETION BLOCKED: You initialized or scaffolded the project template, but you have NOT yet implemented the requested application logic, components, or UI (e.g. calculator, app features, state, styling)! Do not leave the project as an empty default template. Proceed now by creating/editing the main source files (e.g. `### File: <relative_path>`) to write the complete application code.";
                    history.push(ChatMessage {
                        role: "user".to_string(),
                        content: current_user_prompt,
                    });
                    history.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: response,
                    });
                    compact_conversation_history(&mut history);
                    current_user_prompt = format!(
                        "Observation Feedback:\n{}\n\n\
                        Next Step Instructions:\n\
                        - You MUST now implement the application code in the source files using `### File: <relative_path>`.\n\
                        - Do not stop until the full requested application is created!",
                        warning
                    );
                    continue;
                }

                (self.event_sender)(AgentEvent::Completed {
                    summary: response.clone(),
                });
                return Ok(response);
            }

            // Execute all requested tools
            let mut tool_results: Vec<AgentToolResult> = Vec::new();

            for call in &tool_calls {
                let (effective_call, is_standalone_cd) = match call {
                    AgentToolCall::RunCommand { cmd, cwd } => {
                        if let Some((target_dir, remainder_cmd)) = parse_cd_statement(cmd) {
                            let base = active_cwd.as_deref().unwrap_or(&self.ctx.workspace_root);
                            let resolved = if target_dir == ".." {
                                base.parent().unwrap_or(&self.ctx.workspace_root).to_path_buf()
                            } else if target_dir == "." {
                                base.to_path_buf()
                            } else {
                                let p = std::path::Path::new(&target_dir);
                                if p.is_absolute() {
                                    p.to_path_buf()
                                } else {
                                    base.join(p)
                                }
                            };

                            if resolved.is_dir() && resolved.starts_with(&self.ctx.workspace_root) {
                                active_cwd = Some(resolved);
                            }

                            if let Some(rem) = remainder_cmd {
                                let rel = active_cwd.as_ref()
                                    .and_then(|ac| ac.strip_prefix(&self.ctx.workspace_root).ok())
                                    .map(|p| p.to_string_lossy().to_string());
                                (AgentToolCall::RunCommand { cmd: rem, cwd: rel.or_else(|| cwd.clone()) }, false)
                            } else {
                                (call.clone(), true)
                            }
                        } else {
                            let effective_cwd = cwd.clone().or_else(|| {
                                active_cwd.as_ref()
                                    .and_then(|ac| ac.strip_prefix(&self.ctx.workspace_root).ok())
                                    .map(|p| p.to_string_lossy().to_string())
                            });
                            (AgentToolCall::RunCommand { cmd: cmd.clone(), cwd: effective_cwd }, false)
                        }
                    }
                    _ => (call.clone(), false),
                };

                let tool_name = match &effective_call {
                    AgentToolCall::ReadFile { path, .. } => format!("read `{}`", path),
                    AgentToolCall::SearchCode { query } => format!("search `{}`", query),
                    AgentToolCall::ListDirectory { path } => format!("list `{}`", path.as_deref().unwrap_or(".")),
                    AgentToolCall::RunCommand { cmd, .. } => format!("run `{}`", cmd),
                    AgentToolCall::EditFile { path, .. } => format!("patch `{}`", path),
                    AgentToolCall::DetectEnvironment { .. } => "detect environment".to_string(),
                    AgentToolCall::GitStatus => "git status".to_string(),
                    AgentToolCall::GitDiff { .. } => "git diff".to_string(),
                };

                (self.event_sender)(AgentEvent::ToolStarted {
                    tool: tool_name.clone(),
                    summary: format!("Executing {}", tool_name),
                });

                let start_time = Instant::now();

                let res = if is_standalone_cd {
                    let rel = active_cwd.as_ref()
                        .and_then(|ac| ac.strip_prefix(&self.ctx.workspace_root).ok())
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| ".".to_string());
                    AgentToolResult {
                        tool_name: "run_command".to_string(),
                        ok: true,
                        output: format!("[Working Directory: {}]\nWorking directory changed to `{}`.", rel, rel),
                    }
                } else {
                    let r = execute_agent_tool(&self.ctx.workspace_root, &effective_call);
                    if r.ok {
                        if let AgentToolCall::RunCommand { ref cmd, ref cwd } = effective_call {
                            if let Some(ref c) = cwd {
                                let p = std::path::Path::new(c);
                                let resolved = if p.is_absolute() { p.to_path_buf() } else { self.ctx.workspace_root.join(p) };
                                if resolved.is_dir() && resolved.starts_with(&self.ctx.workspace_root) {
                                    active_cwd = Some(resolved);
                                }
                            }
                            if is_scaffolding_command(cmd) {
                                has_scaffolded_project = true;
                                let files = detect_scaffolded_project_files(&self.ctx.workspace_root);
                                if let Some(first_file) = files.first() {
                                    let p = self.ctx.workspace_root.join(first_file);
                                    if let Some(dir) = find_nearest_manifest_dir(&p, &self.ctx.workspace_root) {
                                        if dir != self.ctx.workspace_root {
                                            active_cwd = Some(dir);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    r
                };

                let duration_ms = start_time.elapsed().as_millis() as u64;

                // Emit granular cognitive loop events for UI exploration
                match &effective_call {
                    AgentToolCall::ReadFile { path, start_line, end_line } => {
                        let s = start_line.unwrap_or(1);
                        let e = end_line.unwrap_or(s.saturating_add(40));
                        (self.event_sender)(AgentEvent::FileAnalyzed {
                            file_path: path.clone(),
                            start_line: s,
                            end_line: e,
                        });
                    }
                    AgentToolCall::SearchCode { query } => {
                        let count = if res.ok {
                            res.output.lines().filter(|l| !l.trim().is_empty()).count()
                        } else {
                            0
                        };
                        (self.event_sender)(AgentEvent::SearchExecuted {
                            query: query.clone(),
                            result_count: count,
                        });
                    }
                    AgentToolCall::RunCommand { cmd, .. } => {
                        (self.event_sender)(AgentEvent::SearchExecuted {
                            query: format!("run `{}`", cmd),
                            result_count: if res.ok { 1 } else { 0 },
                        });
                    }
                    AgentToolCall::ListDirectory { path } => {
                        let count = if res.ok {
                            res.output.lines().filter(|l| !l.trim().is_empty()).count()
                        } else {
                            0
                        };
                        (self.event_sender)(AgentEvent::SearchExecuted {
                            query: format!("list `{}`", path.as_deref().unwrap_or(".")),
                            result_count: count,
                        });
                    }
                    _ => {}
                }

                let output_snippet = if res.output.chars().count() > 300 {
                    format!("{}...", res.output.chars().take(300).collect::<String>())
                } else {
                    res.output.clone()
                };

                (self.event_sender)(AgentEvent::ToolFinished {
                    tool: tool_name.clone(),
                    ok: res.ok,
                    output_snippet,
                    duration_ms,
                });

                tool_results.push(res);
            }

            // Build observation feedback for the next iteration
            let mut observation = String::new();
            if let Some(ref ac) = active_cwd {
                let rel_ac = ac.strip_prefix(&self.ctx.workspace_root).unwrap_or(ac).to_string_lossy();
                if !rel_ac.is_empty() && rel_ac != "." {
                    observation.push_str(&format!("Active Working Directory: `{}`\n\n", rel_ac));
                }
            }

            if !file_summaries.is_empty() {
                observation.push_str("### File Updates Applied to Disk:\n");
                for s in &file_summaries {
                    observation.push_str(&format!("- Wrote `{}` (+{} / -{} lines)\n", s.file_name, s.lines_added, s.lines_removed));
                }
                observation.push('\n');
            }

            if !file_errors.is_empty() {
                observation.push_str("### File Patch Failures:\n");
                for err in &file_errors {
                    observation.push_str(&format!("- {}\n", err));
                }
                observation.push_str("Recommendation: Call `read_file` on the target file to inspect its exact current content before re-attempting the SEARCH/REPLACE edit.\n\n");
            }

            if !tool_results.is_empty() {
                observation.push_str("### Tool Execution Observations:\n");
                for tr in &tool_results {
                    observation.push_str(&format!(
                        "- Tool `{}` (Status: {}):\n```\n{}\n```\n\n",
                        tr.tool_name,
                        if tr.ok { "SUCCESS" } else { "FAILED" },
                        tr.output.trim()
                    ));
                }
            }

            // Append assistant response and environment observation to conversation turns
            history.push(ChatMessage {
                role: "user".to_string(),
                content: current_user_prompt,
            });
            history.push(ChatMessage {
                role: "assistant".to_string(),
                content: response,
            });

            compact_conversation_history(&mut history);

            current_user_prompt = format!(
                "Observation Feedback:\n{}\n\n\
                Next Step Instructions:\n\
                - If all required changes and verification steps are complete, provide your final summary to the user.\n\
                - If more files need to be created/edited or build/test commands need to run, proceed directly with the next tool call or file block.",
                observation
            );
        }

        (self.event_sender)(AgentEvent::Completed {
            summary: last_response.clone(),
        });
        Ok(last_response)
    }
}

fn extract_thought_content(text: &str) -> Option<String> {
    if let Some(start) = text.find("<thought>") {
        if let Some(end) = text[start..].find("</thought>") {
            let inner = &text[start + 9..start + end];
            if !inner.trim().is_empty() {
                return Some(inner.trim().to_string());
            }
        }
    }
    if let Some(start) = text.find("<thinking>") {
        if let Some(end) = text[start..].find("</thinking>") {
            let inner = &text[start + 10..start + end];
            if !inner.trim().is_empty() {
                return Some(inner.trim().to_string());
            }
        }
    }
    // Fallback: Check for preamble reasoning before first tool call or file block
    let first_file = text.find("### File:");
    let first_tool = text.find("{\"tool\":");
    let cutoff = match (first_file, first_tool) {
        (Some(f), Some(t)) => Some(f.min(t)),
        (Some(f), None) => Some(f),
        (None, Some(t)) => Some(t),
        (None, None) => None,
    };
    if let Some(c) = cutoff {
        if c > 15 {
            let preamble = text[..c].trim();
            if !preamble.is_empty() && !preamble.starts_with("```") {
                return Some(preamble.to_string());
            }
        }
    }
    None
}

/// Compacts conversation history when it grows beyond 6 messages (3 full turns).
/// Preserves:
/// 1. Initial turn (user goal & first plan)
/// 2. Compact high-density summary of intermediate file writes & executed tools
/// 3. The most recent active turn with full observations
pub fn compact_conversation_history(history: &mut Vec<ChatMessage>) {
    if history.len() <= 6 {
        return;
    }
    let total = history.len();
    let to_compact_end = total.saturating_sub(2);
    if to_compact_end <= 2 {
        return;
    }

    let mut actions_summary = Vec::new();
    for msg in &history[2..to_compact_end] {
        if msg.role == "assistant" {
            for line in msg.content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("### File:") {
                    let f = trimmed.strip_prefix("### File:").unwrap_or(trimmed).trim();
                    actions_summary.push(format!("- Created/updated `{}`", f));
                } else if trimmed.starts_with("{\"tool\":") {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        if let Some(t) = v.get("tool").and_then(|t| t.as_str()) {
                            if t == "run_command" {
                                if let Some(cmd) = v.get("cmd").and_then(|c| c.as_str()) {
                                    actions_summary.push(format!("- Executed: `{}`", cmd));
                                }
                            }
                        }
                    }
                }
            }
        } else if msg.role == "user" && msg.content.contains("### File Updates Applied to Disk:") {
            for line in msg.content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("- Wrote `") {
                    actions_summary.push(trimmed.to_string());
                }
            }
        }
    }

    actions_summary.dedup();
    let summary_content = if actions_summary.is_empty() {
        "Previous Steps Summary: Completed intermediate exploration and execution commands.".to_string()
    } else {
        format!("Previous Steps Summary (Compacted Context):\n{}", actions_summary.join("\n"))
    };

    let compacted_turn = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "[System: Older context compacted to preserve optimal reasoning and token budget]".to_string(),
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: summary_content,
        },
    ];

    let mut new_history = Vec::new();
    new_history.extend_from_slice(&history[..2]);
    new_history.extend(compacted_turn);
    new_history.extend_from_slice(&history[to_compact_end..]);
    *history = new_history;
}
