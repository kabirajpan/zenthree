use serde::{Deserialize, Serialize};
use crate::types::ChatMessage;

/// Default character limit for historical tool results. If an older tool result
/// exceeds this length, its body is pruned to save context budget.
pub const DEFAULT_HISTORICAL_TOOL_RESULT_BUDGET: usize = 500;

/// Statistics on the pruning operation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PruningReport {
    /// Number of tool result messages pruned.
    pub pruned_count: usize,
    /// Total characters removed across all pruned tool results.
    pub characters_saved: usize,
    /// Estimated tokens saved (approx. 4 chars per token).
    pub estimated_tokens_saved: usize,
}

/// Prunes historical tool results from past completed turns.
#[derive(Debug, Clone)]
pub struct ToolPairPruner {
    max_historical_char_budget: usize,
}

impl Default for ToolPairPruner {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolPairPruner {
    /// Creates a new pruner with the default historical tool result budget (500 chars).
    pub fn new() -> Self {
        Self {
            max_historical_char_budget: DEFAULT_HISTORICAL_TOOL_RESULT_BUDGET,
        }
    }

    /// Creates a new pruner with a customized character budget for historical tool results.
    pub fn with_char_budget(budget: usize) -> Self {
        Self {
            max_historical_char_budget: budget,
        }
    }

    /// Prunes completed historical tool results from a sequence of chat messages.
    ///
    /// `active_turn_message_count`: The number of most recent messages that belong
    /// to the active (current) turn. These messages will NOT be pruned.
    pub fn prune_history(
        &self,
        messages: &[ChatMessage],
        active_turn_message_count: usize,
    ) -> (Vec<ChatMessage>, PruningReport) {
        if messages.is_empty() {
            return (Vec::new(), PruningReport::default());
        }

        let total_len = messages.len();
        let cutoff_index = total_len.saturating_sub(active_turn_message_count);

        let mut pruned_messages = Vec::with_capacity(total_len);
        let mut report = PruningReport::default();

        for (idx, msg) in messages.iter().enumerate() {
            // If message is in the active turn window, keep it untouched
            if idx >= cutoff_index {
                pruned_messages.push(msg.clone());
                continue;
            }

            // Check if this is a historical tool message or tool result
            if msg.role.to_lowercase() == "tool" || msg.role.to_lowercase() == "function" {
                let char_count = msg.content.chars().count();
                if char_count > self.max_historical_char_budget {
                    // Extract first line or prefix for context
                    let head: String = msg
                        .content
                        .lines()
                        .next()
                        .unwrap_or("")
                        .chars()
                        .take(120)
                        .collect();

                    let pruned_content = format!(
                        "[Historical tool output pruned: returned {char_count} chars. Summary: \"{head}\"... Context preserved in assistant response.]"
                    );

                    let characters_saved = char_count.saturating_sub(pruned_content.chars().count());
                    report.pruned_count += 1;
                    report.characters_saved += characters_saved;

                    let mut new_msg = msg.clone();
                    new_msg.content = pruned_content;
                    pruned_messages.push(new_msg);
                    continue;
                }
            }

            pruned_messages.push(msg.clone());
        }

        report.estimated_tokens_saved = report.characters_saved / 4;
        (pruned_messages, report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_turn_not_pruned() {
        let pruner = ToolPairPruner::with_char_budget(100);
        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Run test".to_string(),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "Running cargo test...".to_string(),
            },
            ChatMessage {
                role: "tool".to_string(),
                content: "X".repeat(500),
            },
        ];

        // If active turn message count is 1, the last tool message is considered active
        let (pruned, report) = pruner.prune_history(&messages, 1);
        assert_eq!(report.pruned_count, 0);
        assert_eq!(pruned[2].content.len(), 500);
    }

    #[test]
    fn test_historical_tool_result_pruned() {
        let pruner = ToolPairPruner::with_char_budget(100);
        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Check files".to_string(),
            },
            ChatMessage {
                role: "tool".to_string(),
                content: "test_output_first_line\n".to_string() + &"Y".repeat(1000),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "The tests passed successfully.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Next step please.".to_string(),
            },
        ];

        // Active turn is the last user message (active_turn_message_count = 1)
        let (pruned, report) = pruner.prune_history(&messages, 1);
        assert_eq!(report.pruned_count, 1);
        assert!(report.characters_saved > 800);
        assert!(pruned[1].content.contains("[Historical tool output pruned"));
        assert!(pruned[1].content.contains("test_output_first_line"));
    }
}
