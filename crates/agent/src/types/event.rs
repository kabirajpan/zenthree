use serde_json::Value;
use super::approval::ApprovalRequest;

/// Incremental events emitted during a provider stream turn.
#[derive(Debug, Clone)]
pub enum ProviderEvent {
    /// Text chunk (render immediately in UI).
    TextDelta(String),
    /// Thought / reasoning delta (from models with chain-of-thought).
    ThoughtDelta(String),
    /// Assistant started invoking a tool.
    ToolCallStart { id: String, name: String },
    /// Incremental JSON argument fragment.
    ToolCallArgsDelta { id: String, fragment: String },
    /// Tool call arguments assembled and ready to execute.
    ToolCallComplete {
        id: String,
        name: String,
        args: Value,
        thought_signature: Option<String>,
    },
    /// An external agent (e.g. via ACP) executed a tool internally.
    ExternalToolActivity {
        name: String,
        args: Value,
    },
    /// Provider finished this turn.
    Done,
    /// Provider error encountered mid-turn.
    Error(String),
}

/// Events broadcast to UI subscribers as the agent orchestrates the loop.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// Streaming text delta.
    Text(String),
    /// Streaming thought / reasoning delta.
    Thought(String),
    /// Agent started executing a tool.
    ToolStarted { name: String, args: Value },
    /// Agent needs user approval (when approval gate is active).
    NeedsApproval(ApprovalRequest),
    /// Tool completed execution with result.
    ToolFinished { name: String, result: Value },
    /// Entire turn or plan finished.
    Finished,
    /// Execution failed.
    Failed(String),
}
