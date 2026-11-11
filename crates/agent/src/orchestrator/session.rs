use serde::{Deserialize, Serialize};
use crate::types::message::Message;

/// Serializable session state enabling session resume after IDE restarts.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentSession {
    pub session_id: String,
    pub conversation: Vec<Message>,
}

impl AgentSession {
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            conversation: Vec::new(),
        }
    }

    /// Wipes opaque thought signatures from all assistant tool calls in conversation history.
    /// This prevents HTTP 400 INVALID_ARGUMENT errors when replaying sessions across IDE restarts to providers like Gemini.
    pub fn sanitize_thought_signatures(&mut self) {
        for msg in &mut self.conversation {
            if let Some(tool_calls) = &mut msg.tool_calls {
                for tc in tool_calls {
                    tc.thought_signature = None;
                }
            }
        }
    }

    /// Serialize session to pretty JSON string for disk storage.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Restore session from JSON string and sanitize thought signatures for safe replay.
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        let mut session: Self = serde_json::from_str(json_str)?;
        session.sanitize_thought_signatures();
        Ok(session)
    }
}
