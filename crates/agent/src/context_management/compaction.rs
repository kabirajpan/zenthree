use serde::{Deserialize, Serialize};
use crate::types::ChatMessage;

/// Default threshold ratio of context window capacity that triggers auto-compaction (75%).
pub const DEFAULT_COMPACTION_THRESHOLD_RATIO: f64 = 0.75;

/// Fallback maximum context window size in tokens if unspecified by provider (128,000 tokens).
pub const DEFAULT_MAX_CONTEXT_TOKENS: usize = 128_000;

/// Number of recent message turns preserved completely uncompacted at the tail.
pub const DEFAULT_PRESERVED_TAIL_TURNS: usize = 4;

/// Compaction prompt following the Zed agent handoff standard.
pub const COMPACTION_PROMPT_TEMPLATE: &str = "\
You are compacting this conversation into a handoff for another agent that will resume the work.\n\n\
Include:\n\
- Goal: what the user is ultimately trying to achieve\n\
- State: progress so far, current blockers, and decisions made\n\
- Context: constraints, preferences, and critical data/examples/references needed to continue\n\
- Next: the specific steps that remain\n\
- Pitfalls: anything tried that didn't work\n\n\
Write it so the next agent can act without re-asking the user. Be concise and well-structured.";

/// Report on the compaction operation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionReport {
    pub was_compacted: bool,
    pub original_message_count: usize,
    pub compacted_message_count: usize,
    pub original_estimated_tokens: usize,
    pub compacted_estimated_tokens: usize,
    pub tokens_saved: usize,
    pub handoff_summary: Option<String>,
}

/// Manages sliding-window conversation compaction and handoff generation.
#[derive(Debug, Clone)]
pub struct ConversationCompactor {
    threshold_ratio: f64,
    max_context_tokens: usize,
    preserved_tail_turns: usize,
}

impl Default for ConversationCompactor {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_CONTEXT_TOKENS)
    }
}

impl ConversationCompactor {
    /// Creates a new compactor with specified context token capacity and default 75% threshold.
    pub fn new(max_context_tokens: usize) -> Self {
        Self {
            threshold_ratio: DEFAULT_COMPACTION_THRESHOLD_RATIO,
            max_context_tokens,
            preserved_tail_turns: DEFAULT_PRESERVED_TAIL_TURNS,
        }
    }

    /// Creates a customized compactor with specific threshold ratio and tail turn budget.
    pub fn with_settings(
        threshold_ratio: f64,
        max_context_tokens: usize,
        preserved_tail_turns: usize,
    ) -> Self {
        Self {
            threshold_ratio,
            max_context_tokens,
            preserved_tail_turns,
        }
    }

    /// Estimates token count of a string using the standard 4-chars-per-token heuristic.
    pub fn estimate_tokens(text: &str) -> usize {
        (text.chars().count() + 3) / 4
    }

    /// Estimates total token count across a message list.
    pub fn estimate_total_tokens(messages: &[ChatMessage]) -> usize {
        messages.iter().map(|m| Self::estimate_tokens(&m.content)).sum()
    }

    /// Returns true if active tokens exceed the compaction threshold (e.g. 75% of context window).
    pub fn should_compact(&self, messages: &[ChatMessage]) -> bool {
        let total_tokens = Self::estimate_total_tokens(messages);
        let threshold = (self.max_context_tokens as f64 * self.threshold_ratio) as usize;
        total_tokens >= threshold
    }

    /// Compacts the message list using deterministic heuristic summarization.
    ///
    /// Preserves:
    /// 1. System prompt (if message 0 has role "system")
    /// 2. The primary original user prompt
    /// 3. Injects `<conversation_summary>` representing collapsed intermediate turns
    /// 4. Preserves the last `preserved_tail_turns` turns intact
    pub fn compact(&self, messages: &[ChatMessage]) -> (Vec<ChatMessage>, CompactionReport) {
        let original_tokens = Self::estimate_total_tokens(messages);
        let original_count = messages.len();

        if messages.len() <= self.preserved_tail_turns + 2 || !self.should_compact(messages) {
            return (
                messages.to_vec(),
                CompactionReport {
                    was_compacted: false,
                    original_message_count: original_count,
                    compacted_message_count: original_count,
                    original_estimated_tokens: original_tokens,
                    compacted_estimated_tokens: original_tokens,
                    tokens_saved: 0,
                    handoff_summary: None,
                },
            );
        }

        let mut output = Vec::new();
        let mut start_idx = 0;

        // 1. Preserve system prompt if present
        if let Some(first) = messages.first() {
            if first.role.to_lowercase() == "system" {
                output.push(first.clone());
                start_idx = 1;
            }
        }

        // 2. Preserve initial user goal/prompt
        let first_user_idx = messages[start_idx..]
            .iter()
            .position(|m| m.role.to_lowercase() == "user")
            .map(|pos| start_idx + pos);

        if let Some(user_idx) = first_user_idx {
            output.push(messages[user_idx].clone());
            start_idx = user_idx + 1;
        }

        // Determine tail cutoff
        let tail_start = messages.len().saturating_sub(self.preserved_tail_turns);
        if start_idx >= tail_start {
            // Nothing to compact between start and tail
            return (
                messages.to_vec(),
                CompactionReport {
                    was_compacted: false,
                    original_message_count: original_count,
                    compacted_message_count: original_count,
                    original_estimated_tokens: original_tokens,
                    compacted_estimated_tokens: original_tokens,
                    tokens_saved: 0,
                    handoff_summary: None,
                },
            );
        }

        let intermediate_messages = &messages[start_idx..tail_start];
        let summary_text = self.generate_heuristic_summary(intermediate_messages);

        // 3. Inject summary block as a system/assistant handoff context
        output.push(ChatMessage {
            role: "system".to_string(),
            content: format!(
                "<conversation_summary>\n\
                ## Compacted Conversation Handoff\n\
                {}\n\
                </conversation_summary>",
                summary_text
            ),
        });

        // 4. Append preserved tail turns
        for tail_msg in &messages[tail_start..] {
            output.push(tail_msg.clone());
        }

        let compacted_tokens = Self::estimate_total_tokens(&output);
        let tokens_saved = original_tokens.saturating_sub(compacted_tokens);

        let report = CompactionReport {
            was_compacted: true,
            original_message_count: original_count,
            compacted_message_count: output.len(),
            original_estimated_tokens: original_tokens,
            compacted_estimated_tokens: compacted_tokens,
            tokens_saved,
            handoff_summary: Some(summary_text),
        };

        (output, report)
    }

    /// Generates structured handoff summary from intermediate messages.
    fn generate_heuristic_summary(&self, messages: &[ChatMessage]) -> String {
        let mut key_actions = Vec::new();
        let mut errors_encountered = Vec::new();
        let mut assistant_decisions = Vec::new();

        for msg in messages {
            let lower_role = msg.role.to_lowercase();
            if lower_role == "tool" || lower_role == "function" {
                if msg.content.contains("error") || msg.content.contains("failed") {
                    let first_line = msg.content.lines().next().unwrap_or("").trim();
                    if !first_line.is_empty() {
                        errors_encountered.push(first_line.to_string());
                    }
                } else {
                    let first_line = msg.content.lines().next().unwrap_or("").trim();
                    if !first_line.is_empty() {
                        key_actions.push(first_line.to_string());
                    }
                }
            } else if lower_role == "assistant" {
                // Collect first sentence/line of assistant reasoning
                if let Some(line) = msg.content.lines().find(|l| !l.trim().is_empty()) {
                    let truncated: String = line.chars().take(120).collect();
                    assistant_decisions.push(truncated);
                }
            }
        }

        let mut summary = String::new();
        summary.push_str("### State & Progress So Far\n");
        if assistant_decisions.is_empty() {
            summary.push_str("- Completed intermediate exploration and execution steps.\n");
        } else {
            for dec in assistant_decisions.iter().take(5) {
                summary.push_str(&format!("- {}\n", dec));
            }
        }

        summary.push_str("\n### Operations & Tool Executions\n");
        if key_actions.is_empty() {
            summary.push_str("- Tools executed without fatal errors.\n");
        } else {
            for act in key_actions.iter().take(5) {
                summary.push_str(&format!("- Action: {}\n", act));
            }
        }

        if !errors_encountered.is_empty() {
            summary.push_str("\n### Pitfalls & Resolved Errors\n");
            for err in errors_encountered.iter().take(3) {
                summary.push_str(&format!("- Encountered: {}\n", err));
            }
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_compaction_under_threshold() {
        let compactor = ConversationCompactor::new(10_000); // 7,500 token threshold
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "System prompt".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Implement feature X".to_string(),
            },
        ];

        let (out, report) = compactor.compact(&messages);
        assert!(!report.was_compacted);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn test_compaction_triggered_over_threshold() {
        // Set small context token limit to force compaction trigger
        let compactor = ConversationCompactor::with_settings(0.5, 100, 2);

        let mut messages = Vec::new();
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: "System prompt instructions".to_string(),
        });
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: "Primary user goal".to_string(),
        });

        // 6 intermediate bulky messages (realistically verbose tool payloads)
        for i in 0..6 {
            messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: format!("Step {} decision: modified buffer.", i),
            });
            messages.push(ChatMessage {
                role: "tool".to_string(),
                content: format!("Tool result output {} with verbose log lines.\n{}", i, "LOG ENTRY TRACE LINE DATA\n".repeat(40)),
            });
        }

        // Tail messages (last 2)
        messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: "Active turn assistant message.".to_string(),
        });
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: "Active turn user follow up.".to_string(),
        });

        let (compacted, report) = compactor.compact(&messages);
        assert!(report.was_compacted);
        assert!(report.compacted_message_count < report.original_message_count);
        assert!(report.tokens_saved > 0);

        // System prompt preserved at index 0
        assert_eq!(compacted[0].role, "system");
        assert_eq!(compacted[0].content, "System prompt instructions");

        // Primary user prompt preserved at index 1
        assert_eq!(compacted[1].role, "user");
        assert_eq!(compacted[1].content, "Primary user goal");

        // Injected handoff summary block present
        assert_eq!(compacted[2].role, "system");
        assert!(compacted[2].content.contains("<conversation_summary>"));
        assert!(compacted[2].content.contains("Compacted Conversation Handoff"));

        // Preserved tail messages
        let len = compacted.len();
        assert_eq!(compacted[len - 2].content, "Active turn assistant message.");
        assert_eq!(compacted[len - 1].content, "Active turn user follow up.");
    }
}
