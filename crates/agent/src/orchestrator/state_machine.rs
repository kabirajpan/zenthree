use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::context_management::ContextManager;
use crate::diff::CheckpointTimeline;
use crate::environment::AgentEnvironment;
use crate::orchestrator::prompt_builder::SystemPromptBuilder;
use crate::orchestrator::session::AgentSession;
use crate::orchestrator::steer::{SteeringController, SteeringSignal};
use crate::permissions::PermissionEngine;
use crate::tools::{AgentToolCall, AgentToolResult, execute_agent_tool};

/// States in the deterministic agent turn execution lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentTurnState {
    /// Initial idle state.
    Init,
    /// Assembling context: rules, skills catalog, host env, active files.
    PrepareContext,
    /// Invoking model / streaming token chunks & tool calls.
    StreamLlm,
    /// Evaluating security rules & user permission confirmation for mutating tools.
    PermissionGate {
        pending_tools: Vec<AgentToolCall>,
    },
    /// Executing approved tools in the sandboxed workspace jail.
    ExecuteTools {
        approved_tools: Vec<AgentToolCall>,
    },
    /// Spilling oversized outputs (>150KB) and pruning completed historical turns.
    SpillAndPrune,
    /// Compacting conversation if token usage exceeds threshold.
    CheckCompaction,
    /// Turn completed with final assistant response.
    Completed {
        final_message: String,
    },
    /// Turn cancelled mid-flight by user steering.
    Cancelled {
        reason: String,
    },
    /// Turn failed due to unrecoverable error.
    Failed {
        error: String,
    },
}

/// Events emitted during state machine transitions for UI telemetry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FsmEvent {
    Transition { from: String, to: String },
    TokenChunk(String),
    ToolCallRequested(AgentToolCall),
    ToolCallSpilled { tool_name: String, path: PathBuf },
    TurnFinished { turn_id: u64, is_success: bool },
    Steered { signal: SteeringSignal },
}

/// The deterministic Finite State Machine orchestrator for Zenthree.
#[derive(Debug)]
pub struct AgentStateMachine {
    pub state: AgentTurnState,
    pub session: AgentSession,
    pub environment: AgentEnvironment,
    pub permissions: PermissionEngine,
    pub context_manager: ContextManager,
    pub steering: SteeringController,
    pub checkpoints: CheckpointTimeline,
    pub events: Vec<FsmEvent>,
}

impl AgentStateMachine {
    /// Initializes a new agent state machine for the given workspace root.
    pub fn new(workspace_root: Option<PathBuf>) -> Self {
        let roots: Vec<&std::path::Path> = workspace_root.iter().map(|p| p.as_path()).collect();
        let env = AgentEnvironment::new(roots);
        let cm = ContextManager::new(workspace_root.clone(), 128_000);

        Self {
            state: AgentTurnState::Init,
            session: AgentSession::new(workspace_root),
            environment: env,
            permissions: PermissionEngine::default(),
            context_manager: cm,
            steering: SteeringController::new(),
            checkpoints: CheckpointTimeline::new(),
            events: Vec::new(),
        }
    }

    fn transition(&mut self, next: AgentTurnState) {
        let from_desc = format!("{:?}", self.state);
        let to_desc = format!("{:?}", next);
        self.events.push(FsmEvent::Transition {
            from: from_desc,
            to: to_desc,
        });
        self.state = next;
    }

    /// Step 1: Prepares context and initializes the session prompt.
    pub fn prepare_context(&mut self) -> String {
        self.transition(AgentTurnState::PrepareContext);
        let builder = SystemPromptBuilder::new(self.session.workspace_root.clone());
        let master_prompt = builder.build();
        self.session.set_system_prompt(&master_prompt);
        master_prompt
    }

    /// Checks mid-flight user steering signals. Returns true if execution was redirected or aborted.
    pub fn check_steering(&mut self, turn_id: u64) -> bool {
        match self.steering.check() {
            SteeringSignal::Continue => false,
            SteeringSignal::Cancel { reason } => {
                self.events.push(FsmEvent::Steered {
                    signal: SteeringSignal::Cancel {
                        reason: reason.clone(),
                    },
                });
                self.session.cancel_turn(turn_id, &reason);
                self.transition(AgentTurnState::Cancelled { reason });
                true
            }
            SteeringSignal::Redirect { user_feedback } => {
                self.events.push(FsmEvent::Steered {
                    signal: SteeringSignal::Redirect {
                        user_feedback: user_feedback.clone(),
                    },
                });
                self.session.messages.push(crate::types::ChatMessage {
                    role: "user".to_string(),
                    content: format!("[User Mid-Flight Correction]: {}", user_feedback),
                });
                self.steering.reset();
                false
            }
        }
    }

    /// Step 2 & 3: Evaluates pending tool calls through the PermissionEngine.
    pub fn evaluate_permissions(
        &mut self,
        tool_calls: Vec<AgentToolCall>,
    ) -> (Vec<AgentToolCall>, Vec<AgentToolCall>) {
        self.transition(AgentTurnState::PermissionGate {
            pending_tools: tool_calls.clone(),
        });

        let mut approved = Vec::new();
        let mut denied = Vec::new();

        for call in tool_calls {
            let input_str = match &call {
                AgentToolCall::RunCommand { cmd } => cmd.clone(),
                AgentToolCall::ReadFile { path, .. } => path.clone(),
                AgentToolCall::EditFile { path, .. } => path.clone(),
                AgentToolCall::WriteFile { path, .. } => path.clone(),
                _ => call.to_json_args().to_string(),
            };
            let decision = self.permissions.evaluate(call.tool_name(), &input_str);
            if decision.is_allowed() {
                approved.push(call);
            } else {
                denied.push(call);
            }
        }

        (approved, denied)
    }

    /// Step 4 & 5: Executes approved tools and spills oversized outputs to disk.
    pub fn execute_and_spill_tools(
        &mut self,
        turn_id: u64,
        tools_to_run: Vec<AgentToolCall>,
    ) -> Vec<AgentToolResult> {
        self.transition(AgentTurnState::ExecuteTools {
            approved_tools: tools_to_run.clone(),
        });

        let mut results = Vec::with_capacity(tools_to_run.len());
        let ws = self.session.workspace_root.clone().unwrap_or_else(|| std::path::PathBuf::from("."));

        for call in tools_to_run {
            self.events.push(FsmEvent::ToolCallRequested(call.clone()));
            self.session.record_tool_call(turn_id, call.clone());

            // Execute tool using unified executor
            let mut result = execute_agent_tool(&ws, &call);

            // Step 5: Large output spiller check
            let spill_res = self
                .context_manager
                .process_tool_output(call.tool_name(), &result.output);

            if spill_res.is_spilled {
                if let Some(ref path) = spill_res.spill_path {
                    self.events.push(FsmEvent::ToolCallSpilled {
                        tool_name: call.tool_name().to_string(),
                        path: path.clone(),
                    });
                }
                result.output = spill_res.rendered_content;
            }

            self.session.record_tool_result(turn_id, result.clone());
            results.push(result);
        }

        self.transition(AgentTurnState::SpillAndPrune);
        results
    }

    /// Step 6: Checks compaction threshold and finalizes turn.
    pub fn finalize_turn(&mut self, turn_id: u64, final_response: &str) {
        self.transition(AgentTurnState::CheckCompaction);

        // Pruning and compaction check
        let (compacted_messages, report) = self
            .context_manager
            .prepare_messages(&self.session.messages, 2);

        self.session.messages = compacted_messages;

        self.session.complete_turn(turn_id, final_response);
        self.transition(AgentTurnState::Completed {
            final_message: final_response.to_string(),
        });

        self.events.push(FsmEvent::TurnFinished {
            turn_id,
            is_success: true,
        });

        let _ = report;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_fsm_turn_execution_flow() {
        let temp = TempDir::new().unwrap();
        let ws = temp.path().to_path_buf();

        let mut fsm = AgentStateMachine::new(Some(ws));

        // 1. Prepare Context
        let prompt = fsm.prepare_context();
        assert!(prompt.contains("Zenthree autonomous software engineering agent"));

        // 2. Start Turn
        let turn_id = fsm.session.start_user_turn("Verify codebase");

        // 3. Evaluate Permissions for benign tool
        let tool_call = AgentToolCall::ReadFile {
            path: "Cargo.toml".to_string(),
            start_line: None,
            end_line: None,
        };

        let (approved, denied) = fsm.evaluate_permissions(vec![tool_call]);
        assert_eq!(approved.len(), 1);
        assert_eq!(denied.len(), 0);

        // 4. Execute Tools
        let results = fsm.execute_and_spill_tools(turn_id, approved);
        assert_eq!(results.len(), 1);

        // 5. Finalize Turn
        fsm.finalize_turn(turn_id, "Verification complete.");
        assert!(matches!(fsm.state, AgentTurnState::Completed { .. }));
        assert_eq!(fsm.events.len() >= 4, true);
    }

    #[test]
    fn test_fsm_mid_flight_cancellation() {
        let temp = TempDir::new().unwrap();
        let ws = temp.path().to_path_buf();

        let mut fsm = AgentStateMachine::new(Some(ws));
        fsm.prepare_context();
        let turn_id = fsm.session.start_user_turn("Long running task");

        // Emit cancel signal
        fsm.steering.emit(SteeringSignal::Cancel {
            reason: "Stop requested".to_string(),
        });

        let interrupted = fsm.check_steering(turn_id);
        assert!(interrupted);
        assert!(matches!(fsm.state, AgentTurnState::Cancelled { .. }));
    }
}
