use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Configuration options controlling the behavior of the orchestrator loop.
#[derive(Clone)]
pub struct AgentConfig {
    /// Maximum turns allowed before hard stopping (prevents infinite tool loops).
    pub max_iterations: usize,
    /// When true, mutating and destructive tools execute automatically without manual confirmation.
    /// Default is true during development phase; set to false for production approval gates.
    pub auto_approve_all: bool,
    /// Maximum number of network retries when calling provider.
    pub max_provider_retries: usize,
    /// Maximum messages to retain in active memory before trimming older turns.
    pub max_history_messages: usize,
    /// Maximum characters allowed per tool output before sanitization/truncation.
    pub max_tool_output_chars: usize,
    /// Optional cancellation token checked between execution steps.
    pub cancellation_token: Option<Arc<AtomicBool>>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            auto_approve_all: true,
            max_provider_retries: 2,
            max_history_messages: 50,
            max_tool_output_chars: 8000,
            cancellation_token: None,
        }
    }
}
