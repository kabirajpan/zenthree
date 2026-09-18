//! Unified Provider Types & Stream Chunk Specifications
//!
//! Strongly typed data structures representing normalized requests, streaming events,
//! reasoning / extended thinking deltas, tool calling payloads, and token consumption.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelRequest {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub messages: Vec<ProviderMessage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_budget: Option<u32>,
}

impl ModelRequest {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            system: None,
            messages: Vec::new(),
            tools: Vec::new(),
            temperature: None,
            max_tokens: None,
            thinking_budget: None,
        }
    }

    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    pub fn with_message(mut self, role: ProviderRole, content: impl Into<String>) -> Self {
        self.messages.push(ProviderMessage {
            role,
            content: content.into(),
            tool_call_id: None,
            tool_calls: None,
        });
        self
    }

    pub fn with_tools(mut self, tools: Vec<Value>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_thinking(mut self, budget: u32) -> Self {
        self.thinking_budget = Some(budget);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderMessage {
    pub role: ProviderRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ModelToolCall>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StreamChunk {
    Thinking { delta: String },
    Text { delta: String },
    ToolCallChunk {
        id: String,
        name: String,
        arguments_delta: String,
    },
    Usage {
        prompt_tokens: usize,
        completion_tokens: usize,
    },
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ModelResponse {
    pub content: String,
    pub thinking: Option<String>,
    pub tool_calls: Vec<ModelToolCall>,
    pub usage: TokenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

impl TokenUsage {
    pub fn new(prompt_tokens: usize, completion_tokens: usize) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_request_builder() {
        let req = ModelRequest::new("claude-3-7-sonnet-20250219")
            .with_system("You are an autonomous engineering agent.")
            .with_message(ProviderRole::User, "Fix compiler error")
            .with_thinking(2048);

        assert_eq!(req.model, "claude-3-7-sonnet-20250219");
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.thinking_budget, Some(2048));
    }
}
