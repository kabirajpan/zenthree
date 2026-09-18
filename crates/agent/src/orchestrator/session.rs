use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use crate::tools::{AgentToolCall, AgentToolResult};
use crate::types::ChatMessage;

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Execution status of an agent turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
    Cancelled,
}

/// Record of an individual turn in the conversation DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnRecord {
    pub turn_id: u64,
    pub user_prompt: String,
    pub assistant_response: Option<String>,
    pub tool_calls: Vec<AgentToolCall>,
    pub tool_results: Vec<AgentToolResult>,
    pub status: TurnStatus,
    pub created_at: SystemTime,
}

/// High-level conversation session managing thread history, turns, and message linearized log.
#[derive(Debug, Clone)]
pub struct AgentSession {
    pub session_id: String,
    pub workspace_root: Option<PathBuf>,
    pub turns: Vec<TurnRecord>,
    pub messages: Vec<ChatMessage>,
    turn_counter: u64,
}

impl AgentSession {
    /// Creates a new empty agent session.
    pub fn new(workspace_root: Option<PathBuf>) -> Self {
        let id = format!(
            "session_{}_{}",
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0),
            SESSION_COUNTER.fetch_add(1, Ordering::Relaxed)
        );

        Self {
            session_id: id,
            workspace_root,
            turns: Vec::new(),
            messages: Vec::new(),
            turn_counter: 0,
        }
    }

    /// Appends the system prompt at the beginning of the conversation.
    pub fn set_system_prompt(&mut self, prompt: &str) {
        if let Some(first) = self.messages.first_mut() {
            if first.role == "system" {
                first.content = prompt.to_string();
                return;
            }
        }
        self.messages.insert(
            0,
            ChatMessage {
                role: "system".to_string(),
                content: prompt.to_string(),
            },
        );
    }

    /// Initiates a new user turn.
    pub fn start_user_turn(&mut self, prompt: &str) -> u64 {
        self.turn_counter += 1;
        let turn_id = self.turn_counter;

        self.turns.push(TurnRecord {
            turn_id,
            user_prompt: prompt.to_string(),
            assistant_response: None,
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
            status: TurnStatus::InProgress,
            created_at: SystemTime::now(),
        });

        self.messages.push(ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        });

        turn_id
    }

    /// Records a tool execution request within a turn.
    pub fn record_tool_call(&mut self, turn_id: u64, call: AgentToolCall) {
        if let Some(turn) = self.turns.iter_mut().find(|t| t.turn_id == turn_id) {
            turn.tool_calls.push(call.clone());
        }
    }

    /// Records a completed tool result within a turn.
    pub fn record_tool_result(&mut self, turn_id: u64, result: AgentToolResult) {
        if let Some(turn) = self.turns.iter_mut().find(|t| t.turn_id == turn_id) {
            turn.tool_results.push(result.clone());
        }

        self.messages.push(ChatMessage {
            role: "tool".to_string(),
            content: result.output,
        });
    }

    /// Concludes a turn with the final assistant response.
    pub fn complete_turn(&mut self, turn_id: u64, assistant_response: &str) {
        if let Some(turn) = self.turns.iter_mut().find(|t| t.turn_id == turn_id) {
            turn.assistant_response = Some(assistant_response.to_string());
            turn.status = TurnStatus::Completed;
        }

        self.messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: assistant_response.to_string(),
        });
    }

    /// Marks a turn as cancelled (e.g. by user steering).
    pub fn cancel_turn(&mut self, turn_id: u64, reason: &str) {
        if let Some(turn) = self.turns.iter_mut().find(|t| t.turn_id == turn_id) {
            turn.status = TurnStatus::Cancelled;
            turn.assistant_response = Some(format!("[Turn interrupted by user: {}]", reason));
        }

        self.messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: format!("[Turn interrupted by user: {}]", reason),
        });
    }

    /// Returns the active turn count.
    pub fn turn_count(&self) -> usize {
        self.turns.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_turn_lifecycle() {
        let mut session = AgentSession::new(None);
        session.set_system_prompt("System rules");

        let turn_id = session.start_user_turn("Run check");
        assert_eq!(turn_id, 1);
        assert_eq!(session.turns.len(), 1);
        assert_eq!(session.messages.len(), 2); // system + user

        session.record_tool_result(
            turn_id,
            AgentToolResult {
                tool_name: "terminal".to_string(),
                ok: true,
                output: "Finished dev".to_string(),
            },
        );
        assert_eq!(session.messages.len(), 3); // system + user + tool

        session.complete_turn(turn_id, "Check passed successfully.");
        assert_eq!(session.turns[0].status, TurnStatus::Completed);
        assert_eq!(session.messages.len(), 4); // system + user + tool + assistant
    }
}
