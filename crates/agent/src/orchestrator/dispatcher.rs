use std::collections::HashSet;
use serde_json::Value;

use crate::context::pruner::ContextManager;
use crate::orchestrator::hooks::{HookDecision, HookRegistry};
use crate::tools::registry::ToolRegistry;
use crate::traits::tool::ToolRisk;
use crate::types::approval::{ApprovalDecision, ApprovalRequest};
use crate::types::event::AgentEvent;
use crate::types::message::ToolCall;

/// Tracks consecutive identical tool calls to prevent infinite loops.
#[derive(Debug, Default, Clone)]
pub struct RepetitionTracker {
    last_call_signature: String,
    consecutive_repeat_count: usize,
}

impl RepetitionTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if the tool call is an identical repeat. Returns an error message if repeated 3+ times.
    pub fn check_and_update(&mut self, tool_name: &str, tool_args: &Value) -> Result<(), String> {
        let call_sig = format!("{}:{}", tool_name, tool_args);
        if call_sig == self.last_call_signature {
            self.consecutive_repeat_count += 1;
        } else {
            self.consecutive_repeat_count = 1;
            self.last_call_signature = call_sig;
        }

        if self.consecutive_repeat_count >= 3 {
            Err(format!(
                "Repetitive call blocked: You have called tool '{}' with identical arguments {} times consecutively. \
                 You already have this output in your context. \
                 Do NOT call this tool again. Immediately formulate your final response or conclusion for the user.",
                tool_name, self.consecutive_repeat_count
            ))
        } else {
            Ok(())
        }
    }
}

/// Outcome of dispatching a single tool call.
#[derive(Debug)]
pub enum DispatchOutcome {
    /// Tool result string to be added to conversation history.
    Result(String),
    /// Execution was cancelled by user.
    Cancelled(String),
}

/// Modular dispatcher that executes tool calls with hook interception, approval gating, and repetition defense.
pub struct ToolDispatcher;

impl ToolDispatcher {
    /// Dispatch and execute a single tool call through the complete lifecycle.
    pub fn dispatch(
        tool_call: ToolCall,
        tools: &ToolRegistry,
        hooks: &HookRegistry,
        active_tools: &mut HashSet<String>,
        repetition_tracker: &mut RepetitionTracker,
        max_output_chars: usize,
        auto_approve_all: bool,
        is_cancelled: &dyn Fn() -> bool,
        on_event: &mut dyn FnMut(AgentEvent),
        ask_approval: &mut dyn FnMut(ApprovalRequest) -> ApprovalDecision,
    ) -> DispatchOutcome {
        if is_cancelled() {
            let cancel_msg = "Task interrupted by user. Stopping cleanly.".to_string();
            on_event(AgentEvent::Failed(cancel_msg.clone()));
            return DispatchOutcome::Cancelled(cancel_msg);
        }

        let tool_name = &tool_call.name;
        let tool_id = &tool_call.id;
        let mut tool_args = tool_call.args;

        // 1. Repetitive call defense
        if let Err(loop_msg) = repetition_tracker.check_and_update(tool_name, &tool_args) {
            return DispatchOutcome::Result(loop_msg);
        }

        // 2. Resolve tool in registry
        let tool = match tools.find(tool_name) {
            Some(t) => {
                active_tools.insert(tool_name.clone());
                t
            }
            None => {
                let valid = tools.list_names().join(", ");
                let err_msg = format!(
                    "Error: Unknown tool '{}'. Available tools are: [{}]. Please call a valid tool.",
                    tool_name, valid
                );
                return DispatchOutcome::Result(err_msg);
            }
        };

        // 3. Lifecycle Phase 2: Decide Hooks (Security & Policy evaluation)
        let hook_decision = hooks.evaluate_decide(tool_name, &tool_args, tool.risk());
        match hook_decision {
            HookDecision::Deny(reason) => {
                let deny_msg = format!("Policy Violation: Tool '{}' execution denied: {}", tool_name, reason);
                return DispatchOutcome::Result(deny_msg);
            }
            HookDecision::RequireApproval(reason) => {
                let req = ApprovalRequest {
                    id: tool_id.clone(),
                    tool_name: tool_name.clone(),
                    args: tool_args.clone(),
                    preview: format!("🔒 POLICY CHECK ({}): Tool '{}' requires approval", reason, tool_name),
                };
                on_event(AgentEvent::NeedsApproval(req.clone()));
                match ask_approval(req) {
                    ApprovalDecision::Approve => {}
                    ApprovalDecision::ApproveWithEdits(new_args) => {
                        tool_args = new_args;
                    }
                    ApprovalDecision::Reject => {
                        let reject_msg = format!("User rejected execution of tool '{}'. Please try an alternative approach.", tool_name);
                        return DispatchOutcome::Result(reject_msg);
                    }
                }
            }
            HookDecision::Allow => {}
        }

        // 4. Approval Gate for Destructive / Mutating Actions (if not auto-approved)
        if !auto_approve_all && tool.risk() != ToolRisk::ReadOnly {
            let preview = match tool.risk() {
                ToolRisk::Destructive => {
                    format!("⚠️ DESTRUCTIVE ACTION: '{}'\nArguments: {}", tool_name, tool_args)
                }
                _ => format!("Tool '{}': {}", tool_name, tool_args),
            };

            let req = ApprovalRequest {
                id: tool_id.clone(),
                tool_name: tool_name.clone(),
                args: tool_args.clone(),
                preview,
            };
            on_event(AgentEvent::NeedsApproval(req.clone()));

            match ask_approval(req) {
                ApprovalDecision::Approve => {}
                ApprovalDecision::ApproveWithEdits(new_args) => {
                    tool_args = new_args;
                }
                ApprovalDecision::Reject => {
                    let reject_msg = format!("User rejected execution of tool '{}'. Please try an alternative approach.", tool_name);
                    return DispatchOutcome::Result(reject_msg);
                }
            }
        }

        // 5. Lifecycle Phase 3: Transform Hooks (Sanitization & Secret Redaction)
        tool_args = hooks.apply_transforms(tool_name, tool_args);

        // 6. Lifecycle Phase 1: Inspect Hook (Tool Start)
        hooks.trigger_tool_start(tool_name, &tool_args);
        on_event(AgentEvent::ToolStarted {
            name: tool_name.clone(),
            args: tool_args.clone(),
        });

        // 7. Execute Tool
        let exec_result = tool.execute(tool_args);
        let result_for_hook = exec_result.as_ref().map_err(|e| e.to_string()).cloned();
        hooks.trigger_tool_finish(tool_name, &result_for_hook);

        match exec_result {
            Ok(result) => {
                on_event(AgentEvent::ToolFinished {
                    name: tool_name.clone(),
                    result: result.clone(),
                });
                let result_str = match &result {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                let sanitized_result = ContextManager::sanitize_tool_result(&result_str, max_output_chars);

                // Dynamic mid-task tool elevation: if search discovered tools, elevate into active set
                if tool_name == "search" && result_str.contains("[tools]") {
                    let mut in_tools_section = false;
                    for line in result_str.lines() {
                        let trimmed = line.trim();
                        if trimmed == "[tools]" {
                            in_tools_section = true;
                            continue;
                        } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
                            in_tools_section = false;
                        }
                        if in_tools_section {
                            if let Some((cand, _)) = trimmed.split_once(':') {
                                let cand = cand.trim();
                                if tools.find(cand).is_some() {
                                    active_tools.insert(cand.to_string());
                                }
                            }
                        }
                    }
                }

                DispatchOutcome::Result(sanitized_result)
            }
            Err(e) => {
                let err_str = format!("Tool '{}' failed: {}", tool_name, e);
                let sanitized_err = ContextManager::sanitize_tool_result(&err_str, 2000);
                DispatchOutcome::Result(sanitized_err)
            }
        }
    }
}
