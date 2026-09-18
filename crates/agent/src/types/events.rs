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
    FileAnalyzed {
        file_path: String,
        start_line: usize,
        end_line: usize,
    },
    SearchExecuted {
        query: String,
        result_count: usize,
    },
    ThoughtBlock {
        content: String,
        duration_secs: u64,
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
    StatusChanged {
        status: String,
    },
    PermissionPrompt {
        call_id: String,
        tool_name: String,
        reason: String,
        suggested_action: Option<String>,
    },
    DiffHunk {
        file_path: String,
        line: usize,
        old: String,
        new: String,
    },
    CheckpointCommitted {
        checkpoint_id: String,
        files: Vec<String>,
    },
    CheckpointReverted {
        checkpoint_id: String,
    },
    Error {
        error: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingPermissionRequest {
    pub call_id: String,
    pub tool_name: String,
    pub reason: String,
    pub suggested_action: Option<String>,
}
