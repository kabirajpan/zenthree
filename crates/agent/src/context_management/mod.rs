pub mod large_output_spiller;
pub mod tool_pair_pruner;
pub mod compaction;

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub use large_output_spiller::{
    LargeOutputSpiller, SpilledOutputResult, DEFAULT_LARGE_OUTPUT_THRESHOLD,
};
pub use tool_pair_pruner::{
    ToolPairPruner, PruningReport, DEFAULT_HISTORICAL_TOOL_RESULT_BUDGET,
};
pub use compaction::{
    ConversationCompactor, CompactionReport, COMPACTION_PROMPT_TEMPLATE,
    DEFAULT_COMPACTION_THRESHOLD_RATIO, DEFAULT_MAX_CONTEXT_TOKENS,
};

use crate::types::ChatMessage;

/// Tracks token usage and context capacity for an active agent session.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenAccountant {
    pub total_input_tokens: usize,
    pub total_output_tokens: usize,
    pub total_cached_tokens: usize,
    pub last_turn_tokens: usize,
    pub max_context_tokens: usize,
}

impl TokenAccountant {
    /// Creates a new token accountant with specified maximum context limit.
    pub fn new(max_context_tokens: usize) -> Self {
        Self {
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_cached_tokens: 0,
            last_turn_tokens: 0,
            max_context_tokens,
        }
    }

    /// Records usage reported by the model provider.
    pub fn record_usage(&mut self, input: usize, output: usize, cached: usize) {
        self.total_input_tokens += input;
        self.total_output_tokens += output;
        self.total_cached_tokens += cached;
        self.last_turn_tokens = input + output;
    }

    /// Returns the fraction of context window consumed in the most recent turn.
    pub fn current_capacity_ratio(&self) -> f64 {
        if self.max_context_tokens == 0 {
            0.0
        } else {
            self.last_turn_tokens as f64 / self.max_context_tokens as f64
        }
    }

    /// Returns whether the last turn exceeded the given capacity ratio (e.g. 0.75).
    pub fn is_approaching_limit(&self, threshold_ratio: f64) -> bool {
        self.current_capacity_ratio() >= threshold_ratio
    }
}

/// Comprehensive report after running context preparation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextPreparationReport {
    pub pruning: PruningReport,
    pub compaction: CompactionReport,
    pub final_estimated_tokens: usize,
}

/// Unified context resilience coordinator managing spilling, pruning, and compaction.
#[derive(Debug, Clone)]
pub struct ContextManager {
    spiller: LargeOutputSpiller,
    pruner: ToolPairPruner,
    compactor: ConversationCompactor,
    accountant: TokenAccountant,
}

impl ContextManager {
    /// Creates a new `ContextManager` for the given workspace and context capacity.
    pub fn new(workspace_root: Option<PathBuf>, max_context_tokens: usize) -> Self {
        Self {
            spiller: LargeOutputSpiller::new(workspace_root),
            pruner: ToolPairPruner::new(),
            compactor: ConversationCompactor::new(max_context_tokens),
            accountant: TokenAccountant::new(max_context_tokens),
        }
    }

    /// Creates a `ContextManager` with custom configurations.
    pub fn with_custom_config(
        workspace_root: Option<PathBuf>,
        spill_threshold: usize,
        prune_budget: usize,
        compaction_ratio: f64,
        max_context_tokens: usize,
    ) -> Self {
        Self {
            spiller: LargeOutputSpiller::with_threshold(spill_threshold, workspace_root),
            pruner: ToolPairPruner::with_char_budget(prune_budget),
            compactor: ConversationCompactor::with_settings(compaction_ratio, max_context_tokens, 4),
            accountant: TokenAccountant::new(max_context_tokens),
        }
    }

    pub fn spiller(&self) -> &LargeOutputSpiller {
        &self.spiller
    }

    pub fn pruner(&self) -> &ToolPairPruner {
        &self.pruner
    }

    pub fn compactor(&self) -> &ConversationCompactor {
        &self.compactor
    }

    pub fn accountant(&self) -> &TokenAccountant {
        &self.accountant
    }

    pub fn accountant_mut(&mut self) -> &mut TokenAccountant {
        &mut self.accountant
    }

    /// Evaluates tool execution output. Spills to disk if exceeding the threshold.
    pub fn process_tool_output(&self, tool_name: &str, output: &str) -> SpilledOutputResult {
        self.spiller.process_output(tool_name, output)
    }

    /// Prepares messages for the next LLM call by running pruning and compaction.
    pub fn prepare_messages(
        &self,
        messages: &[ChatMessage],
        active_turn_message_count: usize,
    ) -> (Vec<ChatMessage>, ContextPreparationReport) {
        // Step 1: Prune historical tool results
        let (pruned_messages, pruning_report) =
            self.pruner.prune_history(messages, active_turn_message_count);

        // Step 2: Apply compaction if threshold is exceeded
        let (compacted_messages, compaction_report) =
            self.compactor.compact(&pruned_messages);

        let final_estimated_tokens = ConversationCompactor::estimate_total_tokens(&compacted_messages);

        let report = ContextPreparationReport {
            pruning: pruning_report,
            compaction: compaction_report,
            final_estimated_tokens,
        };

        (compacted_messages, report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_context_manager_end_to_end() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ContextManager::with_custom_config(
            Some(temp_dir.path().to_path_buf()),
            500,  // spill threshold: 500 chars
            100,  // prune budget: 100 chars
            0.5,  // compact ratio: 50%
            200,  // max tokens: 200
        );

        // 1. Tool spilling test
        let large_tool_output = "X".repeat(1000);
        let spill_res = manager.process_tool_output("terminal", &large_tool_output);
        assert!(spill_res.is_spilled);
        assert!(spill_res.rendered_content.contains("[TOOL OUTPUT EXCEEDED INLINE BUDGET]"));

        // 2. Message preparation (pruning + compaction)
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "System prompt".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Task request".to_string(),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "Running tool...".to_string(),
            },
            ChatMessage {
                role: "tool".to_string(),
                content: "Historical tool verbose output that exceeds budget: ".to_string() + &"Z".repeat(200),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "Tool completed successfully.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Active user follow-up.".to_string(),
            },
        ];

        let (prepared, report) = manager.prepare_messages(&messages, 1);
        assert!(report.pruning.pruned_count >= 1);
        assert!(!prepared.is_empty());

        // 3. Accountant test
        manager.accountant_mut().record_usage(100, 50, 20);
        assert_eq!(manager.accountant().total_input_tokens, 100);
        assert_eq!(manager.accountant().total_output_tokens, 50);
        assert_eq!(manager.accountant().last_turn_tokens, 150);
    }
}
