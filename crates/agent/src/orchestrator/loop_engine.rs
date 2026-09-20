//! Agent Orchestrator Loop Engine
//!
//! Implements the complete execution specification from `docs/agent/Orchestrator.md`:
//! - Situations A & B: Normal completion & single tool execution.
//! - Situation C: Multi-tool batching.
//! - Situation D: Non-crashing failure feedback (tools feed errors back as data).
//! - Situation E: Unknown tool handling with registered tool names feedback.
//! - Situation G: Bounded retries for provider network errors.
//! - Situation H & I: Destructive action warning previews & rejection feedback.
//! - Situation J: Mid-execution clean interruption checks.
//! - Situation K: Loop iteration capping.
//! - Situation L: Context window overflow pruning.
//! - Situation O: Session export/import for persistence across restarts.

use std::sync::atomic::Ordering;
use serde_json::Value;

use super::config::AgentConfig;
use super::context::ContextManager;
use super::state::AgentSession;
use crate::tools::registry::ToolRegistry;
use crate::traits::provider::Provider;
use crate::traits::tool::ToolRisk;
use crate::types::approval::{ApprovalDecision, ApprovalRequest};
use crate::types::error::ProviderError;
use crate::types::event::{AgentEvent, ProviderEvent};
use crate::types::message::{Message, ToolCall};

/// The main autonomous agent orchestrator tying Provider, Tools, and Conversation together.
pub struct Agent {
    provider: Box<dyn Provider>,
    tools: ToolRegistry,
    conversation: Vec<Message>,
    config: AgentConfig,
}

impl Agent {
    /// Create a new agent with a provider and tool registry.
    pub fn new(provider: Box<dyn Provider>, tools: ToolRegistry) -> Self {
        Self {
            provider,
            tools,
            conversation: Vec::new(),
            config: AgentConfig::default(),
        }
    }

    /// Set a custom configuration for this agent.
    pub fn with_config(mut self, config: AgentConfig) -> Self {
        self.config = config;
        self
    }

    /// Attach an atomic cancellation token to enable clean user interrupts mid-task.
    pub fn with_cancellation_token(mut self, token: std::sync::Arc<std::sync::atomic::AtomicBool>) -> Self {
        self.config.cancellation_token = Some(token);
        self
    }

    /// Check if a cancellation request has been triggered.
    pub fn is_cancelled(&self) -> bool {
        self.config
            .cancellation_token
            .as_ref()
            .map(|t| t.load(Ordering::Relaxed))
            .unwrap_or(false)
    }

    /// Read-only slice of conversation history.
    pub fn conversation(&self) -> &[Message] {
        &self.conversation
    }

    /// Clear conversation history.
    pub fn clear_conversation(&mut self) {
        self.conversation.clear();
    }

    /// Prepend a system instruction message to the conversation.
    /// Call this before the first `run()` on a fresh session to inject
    /// workspace context (e.g. current directory, IDE info, available tools hint).
    pub fn push_system_message(&mut self, content: impl Into<String>) {
        use crate::types::message::Message;
        self.conversation.insert(0, Message::system(content));
    }

    /// Reference to tool registry.
    pub fn tools(&self) -> &ToolRegistry {
        &self.tools
    }

    /// Mutable reference to tool registry.
    pub fn tools_mut(&mut self) -> &mut ToolRegistry {
        &mut self.tools
    }

    /// Export session history to JSON string for persistence across IDE restarts.
    pub fn export_session_json(&self) -> Result<String, serde_json::Error> {
        let session = AgentSession {
            session_id: "default".to_string(),
            conversation: self.conversation.clone(),
        };
        session.to_json()
    }

    /// Import session history from JSON string to resume an existing session.
    pub fn import_session_json(&mut self, json_str: &str) -> Result<(), serde_json::Error> {
        let session = AgentSession::from_json(json_str)?;
        self.conversation = session.conversation;
        Ok(())
    }

    /// Execute a user request through the agent loop.
    pub fn run(
        &mut self,
        user_input: &str,
        on_event: &mut dyn FnMut(AgentEvent),
        ask_approval: &mut dyn FnMut(ApprovalRequest) -> ApprovalDecision,
    ) -> Result<String, ProviderError> {
        self.conversation.push(Message::user(user_input));

        for _iteration in 0..self.config.max_iterations {
            // Situation J: Check for user cancellation at safe checkpoint
            if self.is_cancelled() {
                let cancel_msg = "Agent execution was cancelled by the user.".to_string();
                on_event(AgentEvent::Failed(cancel_msg.clone()));
                return Ok(cancel_msg);
            }

            // Situation L: Context window overflow management
            ContextManager::prune_history(&mut self.conversation, self.config.max_history_messages);

            let mut assistant_text = String::new();
            let mut pending_tool_calls: Vec<ToolCall> = Vec::new();

            let tool_schemas = self.tools.openai_schemas();

            // Situation G: Stream response with bounded network retries
            let mut retries = 0;
            loop {
                assistant_text.clear();
                pending_tool_calls.clear();
                let mut provider_error: Option<ProviderError> = None;

                let stream_res = self.provider.complete_stream(
                    &self.conversation,
                    &tool_schemas,
                    &mut |event| match event {
                        ProviderEvent::TextDelta(delta) => {
                            assistant_text.push_str(&delta);
                            on_event(AgentEvent::Text(delta));
                        }
                        ProviderEvent::ToolCallComplete {
                            id,
                            name,
                            args,
                            thought_signature,
                        } => {
                            pending_tool_calls.push(ToolCall {
                                id,
                                name,
                                args,
                                thought_signature,
                            });
                        }
                        ProviderEvent::Error(err) => {
                            provider_error = Some(ProviderError::Malformed(err));
                        }
                        _ => {}
                    },
                );

                if let Some(err) = provider_error.clone() {
                    if retries < self.config.max_provider_retries {
                        retries += 1;
                        std::thread::sleep(std::time::Duration::from_millis(200 * retries as u64));
                        continue;
                    }
                    on_event(AgentEvent::Failed(err.to_string()));
                    return Err(err);
                }

                if let Err(err) = stream_res {
                    if retries < self.config.max_provider_retries {
                        retries += 1;
                        std::thread::sleep(std::time::Duration::from_millis(200 * retries as u64));
                        continue;
                    }
                    on_event(AgentEvent::Failed(err.to_string()));
                    return Err(err);
                }

                break;
            }

            // Situation A: Plain text output with no tool calls -> Done!
            if pending_tool_calls.is_empty() {
                self.conversation.push(Message::assistant(&assistant_text));
                on_event(AgentEvent::Finished);
                return Ok(assistant_text);
            }

            // Record assistant turn with tool calls in history
            self.conversation.push(Message::assistant_with_tools(
                &assistant_text,
                pending_tool_calls.clone(),
            ));

            // Situation B & C: Process tool calls (batch gathered for next LLM turn)
            for tool_call in pending_tool_calls {
                // Check cancellation between tool steps
                if self.is_cancelled() {
                    let cancel_msg = "Task interrupted by user. Stopping cleanly.";
                    on_event(AgentEvent::Failed(cancel_msg.to_string()));
                    self.conversation.push(Message::tool_result(&tool_call.id, cancel_msg));
                    return Ok(cancel_msg.to_string());
                }

                let tool_name = &tool_call.name;
                let tool_id = &tool_call.id;
                let mut tool_args = tool_call.args;

                // Situation E: Model requested unknown / hallucinated tool
                let tool = match self.tools.find(tool_name) {
                    Some(t) => t,
                    None => {
                        let valid = self.tools.list_names().join(", ");
                        let err_msg = format!(
                            "Error: Unknown tool '{}'. Available tools are: [{}]. Please call a valid tool.",
                            tool_name, valid
                        );
                        self.conversation.push(Message::tool_result(tool_id, &err_msg));
                        continue;
                    }
                };

                // Situation H: Destructive action gating & preview rendering
                if !self.config.auto_approve_all && tool.risk() != ToolRisk::ReadOnly {
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
                        // Situation I: User rejects a proposed action
                        ApprovalDecision::Reject => {
                            let reject_msg = format!("User rejected execution of tool '{}'. Please try an alternative approach.", tool_name);
                            self.conversation.push(Message::tool_result(tool_id, &reject_msg));
                            continue;
                        }
                    }
                }

                on_event(AgentEvent::ToolStarted {
                    name: tool_name.clone(),
                });

                // Situation D: Tool execution (failures feed back to model as data, never crash loop)
                match tool.execute(tool_args) {
                    Ok(result) => {
                        on_event(AgentEvent::ToolFinished {
                            name: tool_name.clone(),
                            result: result.clone(),
                        });
                        let result_str = match &result {
                            Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        let sanitized_result = ContextManager::sanitize_tool_result(&result_str, self.config.max_tool_output_chars);
                        self.conversation.push(Message::tool_result(tool_id, sanitized_result));
                    }
                    Err(e) => {
                        let err_str = format!("Tool '{}' failed: {}", tool_name, e);
                        let sanitized_err = ContextManager::sanitize_tool_result(&err_str, 2000);
                        self.conversation.push(Message::tool_result(tool_id, sanitized_err));
                    }
                }
            }

            // Loop continues: all tool results are now in conversation; sent back in next turn
        }

        // Situation K: Loop reached maximum iterations cap
        let cap_msg = format!(
            "Agent reached maximum iterations ({}) without concluding.",
            self.config.max_iterations
        );
        on_event(AgentEvent::Failed(cap_msg.clone()));
        Ok(cap_msg)
    }
}
