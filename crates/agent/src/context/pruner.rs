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
}
