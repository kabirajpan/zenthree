//! Agentic Loop Engine
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
use std::collections::HashSet;
use std::sync::Arc;

use crate::context::pruner::ContextManager;
use crate::skills::SkillRegistry;
use crate::tools::registry::ToolRegistry;
use crate::traits::provider::Provider;
use crate::types::approval::{ApprovalDecision, ApprovalRequest};
use crate::types::error::ProviderError;
use crate::types::event::{AgentEvent, ProviderEvent};
use crate::types::message::{Message, ToolCall};
use crate::verification::VerificationRunner;
use crate::orchestrator::{
    AgentConfig, AgentHook, AgentIntent, AgentSession, DispatchOutcome, HookRegistry,
    ProvisioningResult, RepetitionTracker, ToolDispatcher, ToolProvisioner,
};

/// The main autonomous agent orchestrator tying Provider, Tools, and Conversation together.
pub struct Agent {
    provider: Box<dyn Provider>,
    tools: ToolRegistry,
    conversation: Vec<Message>,
    config: AgentConfig,
    skills: Arc<SkillRegistry>,
    active_tools: HashSet<String>,
    ast: Arc<crate::ast::DynamicAstEngine>,
    explicit_intent: Option<AgentIntent>,
    explicit_tools: Option<HashSet<String>>,
    reflex: crate::orchestrator::BioReflexClient,
    hooks: HookRegistry,
    rules: Vec<crate::rules::Rule>,
    trust_status: crate::trust::TrustStatus,
    task_manager: crate::tasks::TaskManager,
}

impl Agent {
    /// Create a new agent with a provider and tool registry.
    /// Automatically discovers built-in and workspace skills.
    pub fn new(provider: Box<dyn Provider>, tools: ToolRegistry) -> Self {
        let cwd = std::env::current_dir().unwrap_or_default();
        Self::with_workspace(provider, tools, cwd)
    }

    /// Create an agent without a workspace root (General Chat mode).
    /// Safe: does not bind to any directory, does not register filesystem/terminal tools.
    pub fn without_workspace(provider: Box<dyn Provider>, tools: ToolRegistry) -> Self {
        let skills = Arc::new(crate::skills::SkillRegistry::default());
        let ast = Arc::new(crate::ast::DynamicAstEngine::new());
        Self {
            provider,
            tools,
            conversation: Vec::new(),
            config: AgentConfig::default(),
            skills,
            active_tools: HashSet::new(),
            ast,
            explicit_intent: None,
            explicit_tools: None,
            reflex: crate::orchestrator::BioReflexClient::default(),
            hooks: HookRegistry::default(),
            rules: Vec::new(),
            trust_status: crate::trust::TrustStatus::Trusted,
            task_manager: crate::tasks::TaskManager::default(),
        }
    }

    /// Create an agent initialized for a specific workspace root, discovering project skills and rules.
    pub fn with_workspace(
        provider: Box<dyn Provider>,
        mut tools: ToolRegistry,
        workspace: impl AsRef<std::path::Path>,
    ) -> Self {
        let ws_path = workspace.as_ref().to_path_buf();
        let trust_mgr = crate::trust::WorkspaceTrustManager::default();
        let trust_status = trust_mgr.check_trust(&ws_path);
        let mut hooks = HookRegistry::default();
        hooks.register(crate::trust::WorkspaceTrustHook::new(trust_status));

        let skills_report = crate::skills::discover(&ws_path).unwrap_or_default();
        let skills = Arc::new(skills_report.registry);
        let rules = crate::rules::discover_rules(&ws_path);
        let ast = Arc::new(crate::ast::DynamicAstEngine::new());
        tools.register(crate::tools::ActivateSkillTool::new(skills.clone()));
        let tools_list = tools.manifest_pairs();
        let tools_provider: crate::tools::search::ToolListProvider = Arc::new(move || tools_list.clone());
        if tools.find("search").is_some() {
            tools.register(
                crate::tools::SearchTool::new()
                    .with_ast(ast.clone())
                    .with_skills(skills.clone())
                    .with_tools_provider(tools_provider),
            );
        }
        if tools.find("read_file").is_some() {
            tools.register(crate::tools::ReadFileTool::new().with_ast(ast.clone()).with_workspace(ws_path.clone()));
        }
        if tools.find("write_file").is_some() {
            tools.register(crate::tools::WriteFileTool::new().with_workspace(ws_path.clone()));
        }
        if tools.find("edit_file").is_some() {
            tools.register(crate::tools::EditFileTool::new().with_workspace(ws_path.clone()));
        }
        if tools.find("multi_replace_file_content").is_some() {
            tools.register(crate::tools::MultiReplaceFileContentTool::new().with_workspace(ws_path.clone()));
        }
        if tools.find("delete_file").is_some() {
            tools.register(crate::tools::DeleteFileTool::new().with_workspace(ws_path.clone()));
        }
        if tools.find("rename_file").is_some() {
            tools.register(crate::tools::RenameFileTool::new().with_workspace(ws_path.clone()));
        }
        if tools.find("list_directory").is_some() {
            tools.register(crate::tools::ListDirectoryTool::new().with_workspace(ws_path.clone()));
        }
        let task_mgr = crate::tasks::TaskManager::with_log_dir(ws_path.join(".zene").join("tasks"));
        if tools.find("run_terminal").is_some() {
            tools.register(crate::tools::RunTerminalTool::new().with_workspace(ws_path.clone()).with_task_manager(task_mgr.clone()));
        }
        if tools.find("manage_task").is_some() {
            tools.register(crate::tools::ManageTaskTool::new(task_mgr.clone()));
        }
        Self {
            provider,
            tools,
            conversation: Vec::new(),
            config: AgentConfig::default(),
            skills,
            active_tools: HashSet::new(),
            ast,
            explicit_intent: None,
            explicit_tools: None,
            reflex: crate::orchestrator::BioReflexClient::default(),
            hooks,
            rules,
            trust_status,
            task_manager: task_mgr,
        }
    }

    /// Read-only access to background task supervisor.
    pub fn task_manager(&self) -> &crate::tasks::TaskManager {
        &self.task_manager
    }

    /// Read-only trust status of the workspace.
    pub fn trust_status(&self) -> crate::trust::TrustStatus {
        self.trust_status
    }

    /// Set trust status on this agent, updating security enforcement hooks.
    pub fn with_trust_status(mut self, status: crate::trust::TrustStatus) -> Self {
        self.set_trust_status(status);
        self
    }

    /// Update trust status dynamically in-place, registering updated security enforcement hooks.
    pub fn set_trust_status(&mut self, status: crate::trust::TrustStatus) {
        self.trust_status = status;
        self.hooks.register(crate::trust::WorkspaceTrustHook::new(status));
    }


    /// Discovered rules and custom instructions active for this agent.
    pub fn rules(&self) -> &[crate::rules::Rule] {
        &self.rules
    }

    /// Formatted `<custom_instructions>` XML block for injection into system prompts.
    pub fn rules_prompt(&self) -> String {
        crate::rules::format_rules_prompt(&self.rules)
    }

    /// Register a lifecycle hook into the agent.
    pub fn with_hook<H: AgentHook + 'static>(mut self, hook: H) -> Self {
        self.hooks.register(hook);
        self
    }

    /// Set or replace the entire hook registry.
    pub fn with_hooks(mut self, hooks: HookRegistry) -> Self {
        self.hooks = hooks;
        self
    }

    /// Read-only reference to the active hook registry.
    pub fn hooks(&self) -> &HookRegistry {
        &self.hooks
    }

    /// Mutable reference to the hook registry.
    pub fn hooks_mut(&mut self) -> &mut HookRegistry {
        &mut self.hooks
    }

    /// Set a custom bio-reflex client (e.g. for custom endpoint or mock tests).
    pub fn with_reflex(mut self, reflex: crate::orchestrator::BioReflexClient) -> Self {
        self.reflex = reflex;
        self
    }

    /// Read-only reference to the bio-reflex client.
    pub fn reflex(&self) -> &crate::orchestrator::BioReflexClient {
        &self.reflex
    }

    /// Read-only reference to the dynamic AST engine.
    pub fn ast(&self) -> &Arc<crate::ast::DynamicAstEngine> {
        &self.ast
    }

    /// Dynamically register a Tree-sitter language grammar for an extension.
    pub fn register_language(&self, ext: &str, language: tree_sitter::Language) {
        self.ast.register_language(ext, language);
    }

    /// Dynamically register a Tree-sitter language grammar for multiple extensions.
    pub fn register_extensions(&self, exts: &[&str], language: tree_sitter::Language) {
        self.ast.register_extensions(exts, language);
    }

    /// Builder method to set or override the active intent.
    pub fn with_intent(mut self, intent: AgentIntent) -> Self {
        self.set_intent(intent);
        self
    }

    /// Set or override the active intent for the next run.
    pub fn set_intent(&mut self, intent: AgentIntent) {
        self.explicit_intent = Some(intent.clone());
        self.explicit_tools = None;
        self.active_tools.clear();
        match intent {
            AgentIntent::Discussion | AgentIntent::Exploration => {
                for name in self.tools.safe_names() {
                    self.active_tools.insert(name.to_string());
                }
            }
            AgentIntent::Action(_) => {
                for name in self.tools.list_names() {
                    self.active_tools.insert(name.to_string());
                }
            }
        }


    }

    /// Add a tool to the session cache dynamically (e.g. via discovery).
    pub fn enable_tool(&mut self, tool_name: &str) {
        self.active_tools.insert(tool_name.to_string());
        self.explicit_tools.get_or_insert_with(HashSet::new).insert(tool_name.to_string());
    }

    /// Enable all tools (fallback mode).
    pub fn enable_all_tools(&mut self) {
        self.active_tools.clear();
        for name in self.tools.list_names() {
            self.active_tools.insert(name.to_string());
        }
        self.explicit_tools = Some(self.active_tools.clone());
    }

    /// Read-only slice of active tool names in the session cache.
    pub fn active_tools(&self) -> &HashSet<String> {
        &self.active_tools
    }

    /// Read-only reference to the discovered skill registry.
    pub fn skills(&self) -> &SkillRegistry {
        &self.skills
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

    /// Mutable reference to conversation history.
    pub fn conversation_mut(&mut self) -> &mut Vec<Message> {
        &mut self.conversation
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

    /// Compact historical conversation context and tool results.
    /// Returns (total_messages, initial_chars, compacted_chars).
    pub fn compact_memory(&mut self) -> (usize, usize, usize) {
        let initial_chars: usize = self.conversation.iter().map(|m| m.content.len()).sum();
        let total_msgs = self.conversation.len();
        ContextManager::compact_prior_tool_results(&mut self.conversation);
        let compacted_chars: usize = self.conversation.iter().map(|m| m.content.len()).sum();
        (total_msgs, initial_chars, compacted_chars)
    }

    /// Execute a user request through the agent loop.
    pub fn run(
        &mut self,
        user_input: &str,
        on_event: &mut dyn FnMut(AgentEvent),
        ask_approval: &mut dyn FnMut(ApprovalRequest) -> ApprovalDecision,
    ) -> Result<String, ProviderError> {
        // Ingest any pending reactive background notifications from completed tasks
        for notif in self.task_manager.drain_notifications() {
            self.conversation.push(Message::user(format!("<SYSTEM_NOTIFICATION>\n{}\n</SYSTEM_NOTIFICATION>", notif)));
        }

        self.conversation.push(Message::user(user_input));
        self.hooks.trigger_turn_start(user_input);

        // Modular Tool Provisioning: evaluate overrides, biological reflex, and active tool set
        match ToolProvisioner::provision(
            user_input,
            &self.tools,
            self.explicit_tools.take(),
            self.explicit_intent.take(),
            &self.reflex,
        ) {
            ProvisioningResult::ReflexLockout(decision) => {
                let lock_msg = format!(
                    "🚨 PHYSICAL SAFETY REFLEX LOCK ENGAGED ({} µs latency, {:.1}% confidence)\n{}\nExecution blocked to prevent workspace damage.",
                    decision.reaction_time_us, decision.confidence, decision.explanation
                );
                on_event(AgentEvent::Failed(lock_msg.clone()));
                self.hooks.trigger_turn_finish(&lock_msg);
                return Ok(lock_msg);
            }
            ProvisioningResult::ActiveTools(tools) => {
                self.active_tools = tools;
            }
        }

        let mut repetition_tracker = RepetitionTracker::new();

        for iteration in 0..self.config.max_iterations {
            // Adaptive Throttling: Inject a 3-second delay between consecutive iterations
            // to prevent bursting and hitting API rate limits (e.g., Gemini's 15 RPM).
            if iteration > 0 {
                std::thread::sleep(std::time::Duration::from_secs(3));
            }

            // Situation J: Check for user cancellation at safe checkpoint
            if self.is_cancelled() {
                let cancel_msg = "Agent execution was cancelled by the user.".to_string();
                on_event(AgentEvent::Failed(cancel_msg.clone()));
                return Ok(cancel_msg);
            }

            // Situation L: Context window overflow management
            ContextManager::prune_history(&mut self.conversation, self.config.max_history_messages);
            ContextManager::compact_prior_tool_results(&mut self.conversation);

            let mut assistant_text = String::new();
            let mut pending_tool_calls: Vec<ToolCall> = Vec::new();

            // On-demand tool schemas: 0 in discussion mode, 3-5 in action/explore modes
            let tool_schemas = if self.active_tools.is_empty() {
                Vec::new()
            } else {
                let active_refs: Vec<&str> = self.active_tools.iter().map(|s| s.as_str()).collect();
                self.tools.openai_schemas_for(&active_refs)
            };

            // Situation G: Stream response with bounded network retries and 413 auto-recovery
            let mut retries = 0;
            loop {
                assistant_text.clear();
                pending_tool_calls.clear();
                let mut provider_error: Option<ProviderError> = None;

                let stream_res = self.provider.complete_stream(
                    &self.conversation,
                    &tool_schemas,
                    &mut |event| {
                        if self.is_cancelled() {
                            return;
                        }
                        match event {
                            ProviderEvent::TextDelta(delta) => {
                                assistant_text.push_str(&delta);
                                on_event(AgentEvent::Text(delta));
                            }
                            ProviderEvent::ThoughtDelta(thought) => {
                                on_event(AgentEvent::Thought(thought));
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
                            ProviderEvent::ExternalToolActivity { name, args } => {
                                on_event(AgentEvent::ToolStarted {
                                    name,
                                    args,
                                });
                            }
                            ProviderEvent::ExternalToolFinished { name, output } => {
                                on_event(AgentEvent::ToolFinished {
                                    name,
                                    result: output.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null),
                                });
                            }
                            ProviderEvent::Error(err) => {
                                provider_error = Some(ProviderError::Malformed(err));
                            }
                            _ => {}
                        }
                    },
                );

                if self.is_cancelled() {
                    let cancel_msg = "Agent execution was cancelled by the user.".to_string();
                    on_event(AgentEvent::Failed(cancel_msg.clone()));
                    return Ok(cancel_msg);
                }

                let active_err = provider_error.clone().or_else(|| stream_res.as_ref().err().cloned());
                if let Some(err) = active_err {
                    let err_str = err.to_string();
                    let is_payload_too_large = err_str.contains("413")
                        || err_str.contains("Payload Too Large")
                        || err_str.contains("Request too large")
                        || err_str.contains("rate_limit_exceeded");

                    if is_payload_too_large && retries < self.config.max_provider_retries + 1 {
                        retries += 1;
                        // Emergency compact conversation down to essentials and retry immediately
                        ContextManager::emergency_prune(&mut self.conversation);
                        continue;
                    }

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

                // ── Verification step ────────────────────────────────────────
                // If a verification command is configured, run it now.
                // On failure: inject the error back as a new user turn so the
                // agent can fix the issue and the loop continues.
                if let Some(ref cmd) = self.config.verification_command.clone() {
                    let cwd = std::env::current_dir()
                        .ok()
                        .and_then(|p| p.to_str().map(|s| s.to_string()));

                    on_event(AgentEvent::Text(format!("\n🔍 Verifying: `{}`\n", cmd)));

                    let result = VerificationRunner::run(cmd, cwd.as_deref());
                    if result.is_pass() {
                        on_event(AgentEvent::Finished);
                        self.hooks.trigger_turn_finish(&assistant_text);
                        return Ok(assistant_text);
                    } else {
                        // Feed failure back into the loop as a new user message
                        let feedback = result.as_feedback(cmd);
                        on_event(AgentEvent::Text(format!("\n⚠️ Verification failed, asking agent to fix...\n")));
                        self.conversation.push(Message::user(&feedback));
                        // Continue the outer for loop — agent gets another turn
                        continue;
                    }
                }
                // ── End verification ─────────────────────────────────────────

                on_event(AgentEvent::Finished);
                self.hooks.trigger_turn_finish(&assistant_text);
                return Ok(assistant_text);
            }

            // Record assistant turn with tool calls in history
            self.conversation.push(Message::assistant_with_tools(
                &assistant_text,
                pending_tool_calls.clone(),
            ));

            let cancel_token = self.config.cancellation_token.clone();
            let is_cancelled = move || {
                cancel_token.as_ref().map(|t| t.load(Ordering::Relaxed)).unwrap_or(false)
            };

            // Situation B & C: Process tool calls through modular ToolDispatcher
            for tool_call in pending_tool_calls {
                let tool_id = tool_call.id.clone();
                let outcome = ToolDispatcher::dispatch(
                    tool_call,
                    &self.tools,
                    &self.hooks,
                    &mut self.active_tools,
                    &mut repetition_tracker,
                    self.config.max_tool_output_chars,
                    self.config.auto_approve_all,
                    &is_cancelled,
                    on_event,
                    ask_approval,
                );

                match outcome {
                    DispatchOutcome::Result(sanitized_result) => {
                        self.conversation.push(Message::tool_result(tool_id, sanitized_result));
                    }
                    DispatchOutcome::Cancelled(cancel_msg) => {
                        self.conversation.push(Message::tool_result(tool_id, &cancel_msg));
                        self.hooks.trigger_turn_finish(&cancel_msg);
                        return Ok(cancel_msg);
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
        self.hooks.trigger_turn_finish(&cap_msg);
        Ok(cap_msg)
    }
}
