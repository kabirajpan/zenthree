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
use crate::permissions::{PermissionEngine, ToolPermissionDecision};

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
    pub permissions: PermissionEngine,
}

impl AgentLoop {
    pub fn new(ctx: AgentLoopContext, event_sender: AgentEventHandler) -> Self {
        Self {
            ctx,
            event_sender,
            permissions: PermissionEngine::default(),
        }
    }

    pub fn with_channel(ctx: AgentLoopContext, sender: Sender<AgentEvent>) -> Self {
        Self {
            ctx,
            event_sender: Box::new(move |evt| {
                let _ = sender.send(evt);
            }),
            permissions: PermissionEngine::default(),
        }
    }

    pub fn with_permissions(mut self, permissions: PermissionEngine) -> Self {
        self.permissions = permissions;
        self
    }

    /// Runs the autonomous ReAct loop until task completion or max iterations
    pub fn run(&mut self, goal: &str) -> Result<String, String> {
        let max_steps = self.ctx.max_iterations.max(1);
        let chain = ProviderChain::new(
            self.ctx.gemini_key.clone(),
            self.ctx.groq_key.clone(),
            self.ctx.preferred_model.clone(),
        );

        let system_prompt = self.ctx.system_prompt.clone().unwrap_or_else(build_autonomous_agent_prompt);

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
                let clean_summary = sanitize_summary_tool_blocks(&response);
                let final_summary = if clean_summary.trim().is_empty() {
                    if let Some(thought) = extract_thought_content(&response) {
                        thought
                    } else if response.contains("<tool_call") || response.contains("{\"tool\":") {
                        "I attempted to execute an action, but the tool invocation could not be processed. Please check the command or rephrase.".to_string()
                    } else if response.trim().is_empty() {
                        "No response received from the model. Please check the model status indicator or switch models.".to_string()
                    } else {
                        response.trim().to_string()
                    }
                } else {
                    clean_summary
                };
                (self.event_sender)(AgentEvent::Completed {
                    summary: final_summary.clone(),
                });
                return Ok(final_summary);
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
                    AgentToolCall::WriteFile { path, .. } => (format!("write `{}`", path), "Working"),
                    AgentToolCall::DetectEnvironment { .. } => ("detect environment".to_string(), "Working"),
                    AgentToolCall::GitStatus => ("git status".to_string(), "Working"),
                    AgentToolCall::GitDiff { .. } => ("git diff".to_string(), "Working"),
                    AgentToolCall::Mcp { tool_name, .. } => (format!("mcp `{}`", tool_name), "Working"),
                };

                let input_str = match call {
                    AgentToolCall::RunCommand { cmd } => cmd.clone(),
                    AgentToolCall::ReadFile { path, .. } => path.clone(),
                    AgentToolCall::EditFile { path, .. } => path.clone(),
                    AgentToolCall::WriteFile { path, .. } => path.clone(),
                    _ => call.to_json_args().to_string(),
                };

                let decision = self.permissions.evaluate(call.tool_name(), &input_str);
                if let ToolPermissionDecision::Deny { reason } = decision {
                    let output = format!("Blocked by security policy: {}", reason);
                    (self.event_sender)(AgentEvent::ToolFinished {
                        tool: tool_name.clone(),
                        ok: false,
                        output_snippet: output.clone(),
                        duration_ms: 0,
                    });
                    tool_results.push(AgentToolResult {
                        tool_name: call.tool_name().to_string(),
                        ok: false,
                        output,
                    });
                    continue;
                }

                if let ToolPermissionDecision::Confirm { reason, suggested_action } = decision {
                    (self.event_sender)(AgentEvent::PermissionPrompt {
                        call_id: format!("call_{}_{}", iteration, call.tool_name()),
                        tool_name: call.tool_name().to_string(),
                        reason,
                        suggested_action: Some(suggested_action),
                    });
                }

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
                Next Step Instructions:\n\
                - If you discovered modular containers or subdirectories (e.g. `crates/`, `src/`, `packages/`) that have not been explored yet, DO NOT summarize prematurely! Execute `{{\"tool\": \"list_directory\", \"path\": \"<container>\"}}` and descend into child crates/folders to inspect leaf files.\n\
                - Once all required changes, DFS tree traversal to leaf files, and verification steps are complete, provide your final summary to the user.\n\
                - Otherwise, proceed directly with the next tool call or file block.",
                observation
            );
        }

        let clean_summary = sanitize_summary_tool_blocks(&last_response);
        let final_summary = if clean_summary.trim().is_empty() {
            if let Some(thought) = extract_thought_content(&last_response) {
                thought
            } else if !last_response.trim().is_empty() {
                last_response.trim().to_string()
            } else {
                "I completed the workspace tasks and applied the requested changes.".to_string()
            }
        } else {
            clean_summary
        };
        (self.event_sender)(AgentEvent::Completed {
            summary: final_summary.clone(),
        });
        Ok(final_summary)
    }
}

fn sanitize_summary_tool_blocks(text: &str) -> String {
    let mut out = text.to_string();
    while let Some(start) = out.find("{\"tool\":") {
        let slice = &out[start..];
        let mut depth = 0;
        let mut close = None;
        for (i, c) in slice.char_indices() {
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
                if depth == 0 {
                    close = Some(i);
                    break;
                }
            }
        }
        if let Some(end) = close {
            out.replace_range(start..start + end + 1, "");
        } else if let Some(simple_end) = slice.find('}') {
            out.replace_range(start..start + simple_end + 1, "");
        } else {
            break;
        }
    }
    // Also remove any <tool_call>...</tool_call> or < tool_call>...</ tool_call>
    if let Ok(re) = regex::Regex::new(r"(?is)<\s*(?:tool_call|function_call)\b[^>]*>.*?<\s*/\s*(?:tool_call|function_call)\s*>") {
        out = re.replace_all(&out, "").to_string();
    }
    if let Ok(re_unclosed) = regex::Regex::new(r"(?is)<\s*(?:tool_call|function_call)\b[^>]*>.*") {
        out = re_unclosed.replace_all(&out, "").to_string();
    }
    out.trim().to_string()
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
    let first_xml = text.find("<tool_call").or_else(|| text.find("< tool_call"));
    let mut cutoff = None;
    for pos in [first_file, first_tool, first_xml].into_iter().flatten() {
        cutoff = Some(match cutoff {
            Some(curr) => std::cmp::min(curr, pos),
            None => pos,
        });
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_loop_permissions_initialization() {
        let ctx = AgentLoopContext::new(PathBuf::from("."));
        let agent_loop = AgentLoop::new(ctx, Box::new(|_| {}));
        let decision = agent_loop.permissions.evaluate("terminal", "rm -rf /");
        assert!(decision.is_denied());

        let allow_decision = agent_loop.permissions.evaluate("read_file", "src/lib.rs");
        assert!(allow_decision.is_allowed());
    }
}
