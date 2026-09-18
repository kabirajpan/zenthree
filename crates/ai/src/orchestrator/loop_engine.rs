use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::Instant;

use crate::types::{AgentEvent, CancellationToken, ChatMessage, EditedFileSummary};
use crate::tools::{
    execute_agent_tool, parse_agent_tool_calls, extract_all_file_modifications,
    AgentToolCall, AgentToolResult,
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

        // Only the user's message is passed first; LLM deduces intent and explores tools dynamically
        let mut initial_user_prompt = String::new();
        if let Some(ref active) = self.ctx.active_file {
            let rel = active.strip_prefix(&self.ctx.workspace_root).unwrap_or(active).to_string_lossy();
            let cursor_str = self.ctx.cursor_line.map(|l| format!(" (cursor line {})", l)).unwrap_or_default();
            initial_user_prompt.push_str(&format!("[Active open file in editor: `{}`{}]\n\n", rel, cursor_str));
        }
        initial_user_prompt.push_str(goal);

        let mut history: Vec<ChatMessage> = self.ctx.prior_history.clone();
        let mut current_user_prompt = initial_user_prompt;
        let mut last_response = String::new();

        for iteration in 0..max_steps {
            if self.ctx.cancellation_token.is_cancelled() {
                (self.event_sender)(AgentEvent::Error {
                    error: "Task cancelled by user".to_string(),
                });
                return Err("Task cancelled by user".to_string());
            }

            // If conversation history has grown large, compact context
            if history.len() >= 6 || history.iter().map(|m| m.content.len()).sum::<usize>() > 14000 {
                (self.event_sender)(AgentEvent::StatusChanged {
                    status: "Compacting".to_string(),
                });
                let keep_count = 4;
                let start_idx = history.len().saturating_sub(keep_count);
                let recent = history[start_idx..].to_vec();
                let summary_item = ChatMessage::new(
                    "user",
                    format!("[Context Compacted: Prior {} interaction turns retained in working memory]", start_idx),
                );
                history = vec![summary_item];
                history.extend(recent);
            }

            (self.event_sender)(AgentEvent::StatusChanged {
                status: "Working".to_string(),
            });

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

            for modif in &file_mods {
                let target = &modif.file_path;
                if let Some(parent) = target.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }

                let original_code = std::fs::read_to_string(target).unwrap_or_default();
                if let Ok(new_code) = modif.apply_to(&original_code) {
                    if let Ok(_) = std::fs::write(target, &new_code) {
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
                }
            }

            if !file_summaries.is_empty() {
                (self.event_sender)(AgentEvent::PendingModifications {
                    summaries: file_summaries.clone(),
                });
            }

            // Check for tool calls
            let tool_calls = parse_agent_tool_calls(&response);

            // If neither tool calls nor file modifications were produced, the agent has finished its goal!
            if tool_calls.is_empty() && file_mods.is_empty() {
                let non_thought = strip_thought_tags(&response);
                let trimmed = non_thought.trim();

                // 1. Check if the LLM output was purely <thought> with NO response and NO tool call
                if trimmed.is_empty() && iteration < max_steps - 1 {
                    current_user_prompt = "You provided reasoning inside <thought>, but did not execute a tool call or answer the user. Use tools (e.g. {\"tool\": \"list_directory\", \"path\": \".\"}, {\"tool\": \"search_code\", ...}, or {\"tool\": \"read_file\", ...}) to explore the files down to leaf code, or provide your complete final response.".to_string();
                    continue;
                }

                // 2. Protect against premature truncated fragments like "Let's list the"
                let is_accidental_fragment = trimmed.len() < 35
                    && (trimmed.starts_with("Let's") || trimmed.starts_with("I will") || trimmed.starts_with("Next,") || trimmed.ends_with("the"));

                if is_accidental_fragment && iteration < max_steps - 1 {
                    current_user_prompt = "Please output your next tool call as a JSON object (e.g. {\"tool\": \"list_directory\", \"path\": \"...\"}) or provide your complete final response.".to_string();
                    continue;
                }

                // 3. For exploratory or architectural user queries on the very first turn:
                // If the user asks to explain the project/code or how a component works, do not allow lazy answers without inspecting code!
                let is_codebase_query = {
                    let q = goal.to_lowercase();
                    q.contains("explain") || q.contains("how ") || q.contains("architecture") || q.contains("structure") || q.contains("where ") || q.contains("what is this")
                };
                if iteration == 0 && is_codebase_query && max_steps > 1 {
                    current_user_prompt = "To provide an accurate and in-depth explanation, you must inspect the actual codebase files first. Follow BFS: begin by listing the workspace root directory ({\"tool\": \"list_directory\", \"path\": \".\"}) or searching code for relevant symbols.".to_string();
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
                let (tool_name, tool_status) = match call {
                    AgentToolCall::ReadFile { path, .. } => (format!("read `{}`", path), "Reading"),
                    AgentToolCall::SearchCode { query } => (format!("search `{}`", query), "Searching"),
                    AgentToolCall::ListDirectory { path } => (format!("list `{}`", path.as_deref().unwrap_or(".")), "Exploring"),
                    AgentToolCall::RunCommand { cmd } => (format!("run `{}`", cmd), "Working"),
                    AgentToolCall::EditFile { path, .. } => (format!("patch `{}`", path), "Working"),
                    AgentToolCall::DetectEnvironment { .. } => ("detect environment".to_string(), "Working"),
                    AgentToolCall::GitStatus => ("git status".to_string(), "Working"),
                    AgentToolCall::GitDiff { .. } => ("git diff".to_string(), "Working"),
                };

                (self.event_sender)(AgentEvent::StatusChanged {
                    status: tool_status.to_string(),
                });

                (self.event_sender)(AgentEvent::ToolStarted {
                    tool: tool_name.clone(),
                    summary: format!("Executing {}", tool_name),
                });

                let start_time = Instant::now();
                let res = execute_agent_tool(&self.ctx.workspace_root, call);
                let duration_ms = start_time.elapsed().as_millis() as u64;

                // Emit granular cognitive loop events for UI exploration
                match call {
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
                    AgentToolCall::RunCommand { cmd } => {
                        (self.event_sender)(AgentEvent::SearchExecuted {
                            query: format!("run `{}`", cmd),
                            result_count: if res.ok { 1 } else { 0 },
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

                // Immediately return to Working decision state for next step
                (self.event_sender)(AgentEvent::StatusChanged {
                    status: "Working".to_string(),
                });

                tool_results.push(res);
            }

            // Build observation feedback for the next iteration
            let mut observation = String::new();
            if !file_summaries.is_empty() {
                observation.push_str("### File Updates Applied to Disk:\n");
                for s in &file_summaries {
                    observation.push_str(&format!("- Wrote `{}` (+{} / -{} lines)\n", s.file_name, s.lines_added, s.lines_removed));
                }
                observation.push('\n');
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

            current_user_prompt = format!(
                "Observation Feedback:\n{}\n\n\
                Next Step Instructions (BFS Codebase Exploration):\n\
                - Follow Breadth-First Search (BFS) to inspect the files before concluding:\n\
                  * If you discovered relevant directories or modules, inspect them next with `{{\"tool\": \"list_directory\", \"path\": \"<dir>\"}}`.\n\
                  * When you identify key source files, descend to the leaf files and READ them with `{{\"tool\": \"read_file\", \"path\": \"<file>\", \"start_line\": 1, \"end_line\": 60}}`.\n\
                - Only once you have inspected the real leaf code files, provide your comprehensive, accurate final Markdown explanation.",
                observation
            );
        }

        // If the loop finished and the last response was a tool call rather than a final summary,
        // request a final synthesis so the user always gets a clean, comprehensive answer.
        let final_summary = if !parse_agent_tool_calls(&last_response).is_empty() {
            let synthesis_prompt = "You have completed your workspace exploration. Please provide your comprehensive final response, explanation, or summary to the user now based on all the information gathered above.";
            match chain.complete(&system_prompt, synthesis_prompt, &history) {
                Ok(res) => res,
                Err(_) => last_response,
            }
        } else {
            last_response
        };

        (self.event_sender)(AgentEvent::Completed {
            summary: final_summary.clone(),
        });
        Ok(final_summary)
    }
}

fn strip_thought_tags(text: &str) -> String {
    let mut clean = text.to_string();
    while let Some(start) = clean.find("<thought>") {
        if let Some(end) = clean[start..].find("</thought>") {
            clean.replace_range(start..start + end + 10, "");
        } else {
            clean.truncate(start);
            break;
        }
    }
    while let Some(start) = clean.find("<thinking>") {
        if let Some(end) = clean[start..].find("</thinking>") {
            clean.replace_range(start..start + end + 11, "");
        } else {
            clean.truncate(start);
            break;
        }
    }
    clean
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
