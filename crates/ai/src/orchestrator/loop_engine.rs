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
        initial_user_prompt.push_str("Grounding Rule: Only refer to files that actually exist in the workspace tree above. Never assume or hallucinate project names or files that are not present.\n\n");

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

            // Query LLM
            let response = match chain.complete(&system_prompt, &current_user_prompt, &history) {
                Ok(res) => res,
                Err(err) => {
                    let err_msg = format!("LLM provider failure at step {}: {}", iteration + 1, err);
                    (self.event_sender)(AgentEvent::Error { error: err_msg.clone() });
                    return Err(err_msg);
                }
            };

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
                (self.event_sender)(AgentEvent::Completed {
                    summary: response.clone(),
                });
                return Ok(response);
            }

            // Execute all requested tools
            let mut tool_results: Vec<AgentToolResult> = Vec::new();

            for call in &tool_calls {
                let tool_name = match call {
                    AgentToolCall::ReadFile { path, .. } => format!("read `{}`", path),
                    AgentToolCall::SearchCode { query } => format!("search `{}`", query),
                    AgentToolCall::ListDirectory { path } => format!("list `{}`", path.as_deref().unwrap_or(".")),
                    AgentToolCall::RunCommand { cmd } => format!("run `{}`", cmd),
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
                let res = execute_agent_tool(&self.ctx.workspace_root, call);
                let duration_ms = start_time.elapsed().as_millis() as u64;

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
