use crate::types::message::Message;

/// Manages context window pruning, safety capping, and history bounding.
pub struct ContextManager;

impl ContextManager {
    /// Prune older intermediate messages if history exceeds the configured cap,
    /// while preserving the initial user request and the latest tool/assistant context.
    pub fn prune_history(conversation: &mut Vec<Message>, max_messages: usize) {
        if conversation.len() > max_messages {
            let overflow = conversation.len() - max_messages;
            if overflow > 0 && conversation.len() > 2 {
                let remove_count = overflow.min(conversation.len() - 2);
                conversation.drain(1..=remove_count);
            }
        }
    }

    /// Enforce a hard cap on tool output to prevent massive dumps from blowing out LLM context limits.
    /// Preserves the head and tail of the output with an informative truncation marker, safely respecting UTF-8 boundaries.
    pub fn sanitize_tool_result(output: &str, max_chars: usize) -> String {
        if output.len() <= max_chars {
            return output.to_string();
        }

        let head_target = max_chars / 2;
        let head_end = output
            .char_indices()
            .map(|(idx, _)| idx)
            .take_while(|&idx| idx <= head_target)
            .last()
            .unwrap_or(0);

        let tail_target = output.len().saturating_sub(max_chars / 4);
        let tail_start = output
            .char_indices()
            .map(|(idx, _)| idx)
            .find(|&idx| idx >= tail_target)
            .unwrap_or(output.len());

        let head = &output[..head_end];
        let tail = &output[tail_start..];

        format!(
            "{}\n\n... [Output truncated: {} total characters. Showing head and tail to prevent context overflow] ...\n\n{}",
            head.trim_end(),
            output.len(),
            tail.trim_start()
        )
    }

    /// Compact historical tool outputs in older turns so they do not consume context window tokens
    /// on subsequent turns, while preserving the recent tool output intact.
    pub fn compact_prior_tool_results(conversation: &mut [Message]) {
        let len = conversation.len();
        if len <= 2 {
            return;
        }
        for msg in &mut conversation[..len.saturating_sub(2)] {
            if msg.role == crate::types::message::Role::Tool && msg.content.len() > 250 {
                let preview: String = msg.content.chars().take(80).collect();
                msg.content = format!(
                    "[Prior output ({} chars) compacted: {}...]",
                    msg.content.len(),
                    preview
                );
            }
        }
    }

    /// Emergency compaction when a 413 Payload Too Large or token rate limit is encountered.
    /// Retains only the initial setup and the immediate active turn.
    pub fn emergency_prune(conversation: &mut Vec<Message>) {
        if conversation.len() > 3 {
            let drain_end = conversation.len() - 2;
            conversation.drain(1..drain_end);
        }
        Self::compact_prior_tool_results(conversation);
    }
}
