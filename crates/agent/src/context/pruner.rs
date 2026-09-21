use crate::types::message::Message;

/// Manages context window pruning, safety capping, and history bounding.
pub struct ContextManager;

impl ContextManager {
    /// Prune older intermediate messages if history exceeds the configured cap,
    /// while preserving the initial user request, system prompt (if present),
    /// and the latest tool/assistant context.
    pub fn prune_history(conversation: &mut Vec<Message>, max_messages: usize) {
        if conversation.len() > max_messages {
            let overflow = conversation.len() - max_messages;
            let has_system = conversation
                .first()
                .map(|m| m.role == crate::types::message::Role::System)
                .unwrap_or(false);
            let start_idx = if has_system { 2 } else { 1 };
            if conversation.len() > start_idx + 2 {
                let max_drainable = conversation.len().saturating_sub(start_idx + 2);
                let remove_count = overflow.min(max_drainable);
                if remove_count > 0 {
                    conversation.drain(start_idx..start_idx + remove_count);
                }
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
    /// Retains the initial system & user setup and the immediate active turn, avoiding
    /// removing user queries which causes Jinja chat template exceptions.
    pub fn emergency_prune(conversation: &mut Vec<Message>) {
        let has_system = conversation
            .first()
            .map(|m| m.role == crate::types::message::Role::System)
            .unwrap_or(false);
        let start_idx = if has_system { 2 } else { 1 };
        if conversation.len() > start_idx + 2 {
            let drain_end = conversation.len() - 2;
            conversation.drain(start_idx..drain_end);
        }
        Self::compact_prior_tool_results(conversation);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::message::Role;

    #[test]
    fn test_emergency_prune_preserves_system_and_user() {
        let mut conv = vec![
            Message::system("system instructions"),
            Message::user("initial user prompt"),
            Message::assistant("old assistant"),
            Message::tool_result("call_1", "old tool output"),
            Message::assistant("current assistant"),
            Message::tool_result("call_2", "current tool output"),
        ];

        ContextManager::emergency_prune(&mut conv);

        // Must still have system AND user messages
        assert_eq!(conv[0].role, Role::System);
        assert_eq!(conv[1].role, Role::User);
        assert_eq!(conv[1].content, "initial user prompt");
        // Must have current assistant & tool
        assert_eq!(conv[2].role, Role::Assistant);
        assert_eq!(conv[2].content, "current assistant");
        assert_eq!(conv[3].role, Role::Tool);
        assert_eq!(conv[3].content, "current tool output");
        assert_eq!(conv.len(), 4);
    }

    #[test]
    fn test_prune_history_preserves_user_message() {
        let mut conv = vec![
            Message::system("sys"),
            Message::user("user query"),
            Message::assistant("a1"),
            Message::tool_result("t1", "res1"),
            Message::assistant("a2"),
            Message::tool_result("t2", "res2"),
        ];

        ContextManager::prune_history(&mut conv, 4);

        assert_eq!(conv[0].role, Role::System);
        assert_eq!(conv[1].role, Role::User);
        assert_eq!(conv.len(), 4);
        assert!(conv.iter().any(|m| m.role == Role::User));
    }
}
