use serde::{Deserialize, Serialize};
use super::message::EditedFileSummary;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    SubAgentActivated {
        role: String,
        name: String,
    },
    Thinking {
        delta: String,
    },
    StreamChunk {
        delta: String,
    },
    ToolStarted {
        tool: String,
        summary: String,
    },
    ToolFinished {
        tool: String,
        ok: bool,
        output_snippet: String,
        duration_ms: u64,
    },
    PendingModifications {
        summaries: Vec<EditedFileSummary>,
    },
    TaskUpdated {
        task_id: String,
        status: String,
        title: String,
    },
    Completed {
        summary: String,
    },
    Error {
        error: String,
    },
}
